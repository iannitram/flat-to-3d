//! The link service: reads adb track-devices, decides, and runs the link. docs/code-map.md, Parts.

use std::cell::RefCell;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api::{EventKind, LinkEvent, LinkSnapshot, LinkState, TetherMode};
use crate::link::{self, Headset};
use crate::runs::{parse_run_record, running_modes, RunRecord};
use crate::state::{self, EventRing, Request};
use crate::switches::{parse_settings, Settings};
use crate::tether::adb::{self as tether_adb, AdbChoice, AdbRunner, CommandRunner, ProcessRunner};
use crate::tether::{stepaside, Tether};

/// The adb server's port: where a running server listens.
const ADB_SERVER_PORT: u16 = 5037;

/// One device listed in an `adb track-devices` frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// The device's serial.
    pub serial: String,
    /// adb's state word, e.g. `device` or `unauthorized`.
    pub state: String,
}

/// Read `adb track-devices` output into the devices of every complete frame, in order, plus the
/// unconsumed rest of the buffer.
///
/// A frame is four hex digits holding the payload length in bytes, then that many bytes of
/// `serial<TAB>state` lines. A frame that is not complete yet is left in the rest for the next
/// read; a bad header drops the whole buffer.
pub fn parse_track_frames(buf: &str) -> (Vec<Vec<Device>>, String) {
    let mut frames = Vec::new();
    let mut pos = 0;
    loop {
        let header_start = pos;
        let header_end = header_start + 4;
        let present = header_end.min(buf.len());
        // `get` is None when the header cut lands inside a multi-byte character: that is a
        // non-hex char, so the buffer is bad too.
        let Some(header) = buf.get(header_start..present) else {
            return (frames, String::new());
        };
        if !header.bytes().all(|b| b.is_ascii_hexdigit()) {
            return (frames, String::new());
        }
        if present - header_start < 4 {
            return (frames, buf[header_start..].to_string());
        }
        let Ok(len) = usize::from_str_radix(header, 16) else {
            return (frames, String::new());
        };
        let frame_end = header_end + len;
        if buf.len() < frame_end {
            return (frames, buf[header_start..].to_string());
        }
        let Some(payload) = buf.get(header_end..frame_end) else {
            return (frames, buf[header_start..].to_string());
        };
        frames.push(parse_payload(payload));
        pos = frame_end;
    }
}

/// The devices in one frame's payload, skipping blank lines and lines without a tab.
fn parse_payload(payload: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in payload.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((serial, state)) = line.split_once('\t') else {
            continue;
        };
        devices.push(Device {
            serial: serial.to_string(),
            state: state.to_string(),
        });
    }
    devices
}

pub const GRACE_SECS: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Absent,
    Plugged,
    Up,
    Grace { since: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watch {
    pub phase: Phase,
}

impl Watch {
    pub fn new() -> Watch {
        Watch {
            phase: Phase::Absent,
        }
    }
}

impl Default for Watch {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Plugged,
    Unplugged,
    Tick,
    StartRequested,
    StopRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inputs {
    pub auto_3d: bool,
    pub tunnel_on: bool,
    pub game_3d_running: bool,
    pub game_flat_running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    Ready,
    LateRestartFor3d,
    StoppedRestartForFlat,
    AllowUsbDebugging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    StartTether,
    StopTether,
    StartLink,
    Reattach,
    StopLink,
    Notify(Note),
}

/// Apply one event (plug, unplug, tick, or a hand-pressed Start/Stop) without running its effects.
pub fn step(w: &Watch, e: &Event, now: u64, i: &Inputs) -> (Watch, Vec<Effect>) {
    match (w.phase, e) {
        (Phase::Absent, Event::Plugged) if i.auto_3d => {
            let mut effects = Vec::new();
            if i.tunnel_on {
                effects.push(Effect::StartTether);
            }
            effects.push(Effect::StartLink);
            effects.push(Effect::Notify(if i.game_flat_running {
                Note::LateRestartFor3d
            } else {
                Note::Ready
            }));
            (Watch { phase: Phase::Up }, effects)
        }
        (Phase::Absent, Event::Plugged) => {
            let effects = if i.tunnel_on {
                vec![Effect::StartTether]
            } else {
                vec![]
            };
            (
                Watch {
                    phase: Phase::Plugged,
                },
                effects,
            )
        }
        (Phase::Grace { .. }, Event::Plugged) => {
            (Watch { phase: Phase::Up }, vec![Effect::Reattach])
        }
        (Phase::Up, Event::Unplugged) if i.game_3d_running => (
            Watch {
                phase: Phase::Grace { since: now },
            },
            vec![],
        ),
        (Phase::Up, Event::Unplugged) => {
            let mut effects = vec![Effect::StopLink];
            if i.tunnel_on {
                effects.push(Effect::StopTether);
            }
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Plugged, Event::Unplugged) => {
            let effects = if i.tunnel_on {
                vec![Effect::StopTether]
            } else {
                vec![]
            };
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Grace { .. }, Event::Tick) if !i.game_3d_running => {
            let mut effects = vec![Effect::StopLink];
            if i.tunnel_on {
                effects.push(Effect::StopTether);
            }
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Grace { since }, Event::Tick) if now.saturating_sub(since) >= GRACE_SECS => {
            let mut effects = vec![Effect::StopLink];
            if i.tunnel_on {
                effects.push(Effect::StopTether);
            }
            effects.push(Effect::Notify(Note::StoppedRestartForFlat));
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Plugged, Event::StartRequested) => (
            Watch { phase: Phase::Up },
            vec![Effect::StartLink, Effect::Notify(Note::Ready)],
        ),
        (Phase::Grace { .. }, Event::StartRequested) => {
            (Watch { phase: Phase::Up }, vec![Effect::Reattach])
        }
        (Phase::Up | Phase::Grace { .. }, Event::StopRequested) => (
            Watch {
                phase: Phase::Plugged,
            },
            vec![Effect::StopLink],
        ),
        _ => (*w, vec![]),
    }
}

/// The link commands an effect runs: Start 3D's list, the reattach part of it (from `adb
/// reverse` on; the server unit is already running), or Stop 3D's list. The tether and toast
/// effects run no link commands.
pub fn link_commands(effect: &Effect) -> Vec<Vec<String>> {
    match effect {
        Effect::StartLink => crate::link::start_3d_commands(),
        Effect::Reattach => crate::link::start_3d_commands()[2..].to_vec(),
        Effect::StopLink => crate::link::stop_3d_commands(),
        Effect::StartTether | Effect::StopTether | Effect::Notify(_) => Vec::new(),
    }
}

/// The toast text for a note, the same on every face.
pub fn note_text(n: Note) -> &'static str {
    match n {
        Note::Ready => "3D is ready. Press Play on a 3D Ready game.",
        Note::LateRestartFor3d => "Headset connected. Restart the game to play it in 3D.",
        Note::StoppedRestartForFlat => "Headset link closed. Restart the game to play it flat.",
        Note::AllowUsbDebugging => "Put the headset on and allow USB debugging.",
    }
}

/// The note's kind word, for the state file.
pub fn note_kind(n: Note) -> &'static str {
    match n {
        Note::Ready => "ready",
        Note::LateRestartFor3d => "late-plug",
        Note::StoppedRestartForFlat => "grace-ended",
        Note::AllowUsbDebugging => "allow-debugging",
    }
}

/// The files the link service reads and writes, under the user's home and the state directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    /// The user's home directory.
    pub home: String,
    /// The shared state directory (`state::state_dir`).
    pub state_dir: String,
}

impl Paths {
    /// The state file, `link.json`.
    pub fn snapshot(&self) -> String {
        state::snapshot_path(&self.state_dir)
    }

    /// The one-line request file.
    pub fn request(&self) -> String {
        state::request_path(&self.state_dir)
    }

    /// The directory of run records.
    pub fn runs(&self) -> String {
        state::runs_dir(&self.state_dir)
    }

    /// The settings file (`setting set`, `switch set`).
    pub fn settings(&self) -> String {
        format!("{}/.config/flat3d/settings", self.home)
    }

    /// The service's own log, `link.log`.
    pub fn log(&self) -> String {
        format!("{}/.local/state/flat3d/link.log", self.home)
    }
}

/// Where the device list comes from: the newest `adb track-devices` frame, when one arrived since
/// the last call.
pub trait Feed {
    fn poll(&mut self) -> Option<Vec<Device>>;
}

/// The real feed: a child `adb track-devices` whose output a thread reads into a channel.
pub struct TrackDevices {
    adb: String,
    child: Option<Child>,
    rx: Option<Receiver<Vec<u8>>>,
    rest: String,
}

impl TrackDevices {
    /// Start the feed with the adb in use. An empty path means no adb: the feed stays empty.
    pub fn new(adb: &str) -> Self {
        let mut feed = TrackDevices {
            adb: adb.to_string(),
            child: None,
            rx: None,
            rest: String::new(),
        };
        feed.spawn();
        feed
    }

    fn spawn(&mut self) {
        if self.adb.is_empty() {
            return;
        }
        let Ok(mut child) = Command::new(&self.adb)
            .arg("track-devices")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let Some(mut out) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return;
        };
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = out.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        self.child = Some(child);
        self.rx = Some(rx);
        self.rest.clear();
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.rx = None;
    }
}

impl Drop for TrackDevices {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Feed for TrackDevices {
    fn poll(&mut self) -> Option<Vec<Device>> {
        if self.child.is_none() {
            self.spawn();
        }
        let rx = self.rx.as_ref()?;
        let mut text = String::new();
        let mut closed = false;
        loop {
            match rx.try_recv() {
                // adb's frames are ASCII; a lossy read cannot split a frame's meaning.
                Ok(chunk) => text.push_str(&String::from_utf8_lossy(&chunk)),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    closed = true;
                    break;
                }
            }
        }
        self.rest.push_str(&text);
        let (mut frames, rest) = parse_track_frames(&self.rest);
        self.rest = rest;
        if closed {
            self.stop();
        }
        frames.pop()
    }
}

/// What the device list says about the headset.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Seen {
    /// A device in the `device` state (usable), with its serial.
    Ready(String),
    /// A device that has not authorised this machine yet.
    Unauthorised,
    /// No usable device.
    Absent,
}

fn classify(devices: &[Device]) -> Seen {
    if let Some(device) = devices.iter().find(|device| device.state == "device") {
        return Seen::Ready(device.serial.clone());
    }
    if devices.iter().any(|device| device.state == "unauthorized") {
        return Seen::Unauthorised;
    }
    Seen::Absent
}

/// The event kind a note is written under in the state file and the event ring.
pub fn note_event(n: Note) -> EventKind {
    match n {
        Note::Ready => EventKind::Ready,
        Note::LateRestartFor3d => EventKind::LatePlug,
        Note::StoppedRestartForFlat => EventKind::GraceEnded,
        Note::AllowUsbDebugging => EventKind::AllowDebugging,
    }
}

/// The one runner the link service and its tether share, so both act through the same process
/// runner and the relay it starts is owned in one place.
struct Shared<R>(Rc<RefCell<R>>);

impl<R: CommandRunner> CommandRunner for Shared<R> {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        self.0.borrow_mut().run(command)
    }

    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String> {
        self.0.borrow_mut().start_relay(command, adb)
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        self.0.borrow_mut().stop_relay()
    }
}

/// The link service's state: the watch state machine, the headset as last seen, the tether and
/// the event ring. Every effect runs through `runner` (adb commands get the adb in use).
pub struct Service<R: CommandRunner, F: Feed> {
    paths: Paths,
    runner: Rc<RefCell<R>>,
    feed: F,
    adb: AdbChoice,
    watch: Watch,
    inputs: Inputs,
    ready: bool,
    serial: String,
    unauthorised: bool,
    starting: bool,
    tether: Option<Tether<Shared<R>>>,
    tether_serial: String,
    ring: EventRing,
}

impl<R: CommandRunner, F: Feed> Service<R, F> {
    pub fn new(paths: Paths, runner: Rc<RefCell<R>>, feed: F, adb: AdbChoice) -> Self {
        Service {
            paths,
            runner,
            feed,
            adb,
            watch: Watch::new(),
            inputs: Inputs {
                auto_3d: false,
                tunnel_on: false,
                game_3d_running: false,
                game_flat_running: false,
            },
            ready: false,
            serial: String::new(),
            unauthorised: false,
            starting: false,
            tether: None,
            tether_serial: String::new(),
            ring: EventRing::new(),
        }
    }

    /// Continue numbering the event ring from events saved by an earlier run.
    pub fn restore(&mut self, events: Vec<LinkEvent>) {
        self.ring = EventRing::from_events(events);
    }

    /// One pass of the loop: read the device list, apply a request from the request file, then
    /// tick; write the state file and return the snapshot it wrote.
    pub fn tick(
        &mut self,
        now: u64,
        settings: &Settings,
        request: Option<Request>,
        modes: (bool, bool),
    ) -> LinkSnapshot {
        self.inputs = Inputs {
            auto_3d: settings.auto_3d,
            tunnel_on: settings.tunnel,
            game_3d_running: modes.0,
            game_flat_running: modes.1,
        };
        if let Some(devices) = self.feed.poll() {
            match classify(&devices) {
                Seen::Ready(serial) => {
                    self.unauthorised = false;
                    self.serial = serial;
                    if !self.ready {
                        self.ready = true;
                        self.fire(now, Event::Plugged);
                    }
                }
                Seen::Unauthorised => {
                    self.unauthorised = true;
                    self.unplug(now);
                }
                Seen::Absent => {
                    self.unauthorised = false;
                    self.unplug(now);
                }
            }
        }
        match request {
            Some(Request::Start3d) => self.fire(now, Event::StartRequested),
            Some(Request::Stop3d) => self.fire(now, Event::StopRequested),
            None => {}
        }
        self.fire(now, Event::Tick);
        self.publish(now)
    }

    fn unplug(&mut self, now: u64) {
        if self.ready {
            self.ready = false;
            self.fire(now, Event::Unplugged);
        }
    }

    fn fire(&mut self, now: u64, event: Event) {
        let (next, effects) = step(&self.watch, &event, now, &self.inputs);
        self.watch = next;
        self.execute(now, effects);
    }

    fn execute(&mut self, now: u64, effects: Vec<Effect>) {
        // A tether connect runs Start 3D's WiVRn commands itself, so a StartLink in the same
        // batch must not run them a second time (systemd-run would find the unit running).
        let mut wivrn_by_tether = false;
        let mut link_failed = false;
        for effect in effects {
            match effect {
                Effect::StartTether => match self.connect_tether() {
                    Ok(()) => wivrn_by_tether = true,
                    Err(message) => self.log(now, &format!("tether: {message}")),
                },
                Effect::StartLink | Effect::Reattach => {
                    if effect == Effect::StartLink && wivrn_by_tether {
                        continue;
                    }
                    self.starting = true;
                    self.publish(now);
                    let commands = link_commands(&effect);
                    if self.run_steps(now, &commands, true).is_err() {
                        link_failed = true;
                    }
                    self.starting = false;
                }
                Effect::StopLink => {
                    let commands = link_commands(&effect);
                    let _ = self.run_steps(now, &commands, false);
                }
                Effect::StopTether => {
                    if let Err(message) = self.disconnect_tether() {
                        self.log(now, &format!("tether stop: {message}"));
                    }
                }
                Effect::Notify(note) => {
                    if !link_failed {
                        self.ring.push(note_event(note), note_text(note));
                    }
                }
            }
        }
    }

    /// Run commands in order through the adb in use. A failed command is logged; with
    /// `stop_on_error` the rest are skipped. `adb devices` must list the headset as ready.
    fn run_steps(
        &mut self,
        now: u64,
        commands: &[Vec<String>],
        stop_on_error: bool,
    ) -> Result<(), String> {
        let mut first = None;
        for command in commands {
            let result = self.adb_run(command).and_then(|output| {
                if command.as_slice() == ["adb", "devices"]
                    && Headset::Present != link::headset_from_adb(&output)
                {
                    Err("the headset is not ready".to_string())
                } else {
                    Ok(())
                }
            });
            if let Err(message) = result {
                let line = format!("{}: {message}", command.join(" "));
                self.log(now, &line);
                first.get_or_insert(line);
                if stop_on_error {
                    break;
                }
            }
        }
        first.map_or(Ok(()), Err)
    }

    fn adb_run(&self, command: &[String]) -> Result<String, String> {
        let mut adb = AdbRunner::new(
            self.adb.clone(),
            &self.serial,
            Shared(Rc::clone(&self.runner)),
        );
        adb.run(command)
    }

    fn connect_tether(&mut self) -> Result<(), String> {
        let ps_command: Vec<String> = vec!["ps".into(), "-eo".into(), "pid=,args=".into()];
        let ps = self.runner.borrow_mut().run(&ps_command)?;
        let foreign = stepaside::foreign_relay(&stepaside::parse_ps(&ps), None);
        let serial = self.serial.clone();
        self.tether_for(&serial).connect(foreign).map(|_| ())
    }

    fn disconnect_tether(&mut self) -> Result<(), String> {
        match self.tether.as_mut() {
            Some(tether) => tether.disconnect(),
            None => Ok(()),
        }
    }

    /// The tether for this headset. A new one replaces an idle tether that was made for another
    /// serial; a tether that is still connected is kept.
    fn tether_for(&mut self, serial: &str) -> &mut Tether<Shared<R>> {
        let keep = match &self.tether {
            Some(tether) => self.tether_serial == serial || tether.mode() != TetherMode::Off,
            None => false,
        };
        if !keep {
            self.tether = None;
            self.tether_serial = serial.to_string();
        }
        self.tether.get_or_insert_with(|| {
            Tether::new(
                &self.paths.home,
                serial,
                self.adb.clone(),
                Shared(Rc::clone(&self.runner)),
            )
        })
    }

    fn state_text(&self) -> (LinkState, String) {
        if self.starting {
            return (LinkState::Starting, "Starting 3D.".into());
        }
        match self.watch.phase {
            Phase::Absent if self.unauthorised => (
                LinkState::Unauthorised,
                "Put the headset on and allow USB debugging.".into(),
            ),
            Phase::Absent => (
                LinkState::NoHeadset,
                "No headset. Plug it in with USB debugging on.".into(),
            ),
            Phase::Plugged => (
                LinkState::Stopped,
                "Headset connected. Start 3D to bring the link up.".into(),
            ),
            Phase::Up => (LinkState::Ready, "3D link is ready.".into()),
            Phase::Grace { .. } => (
                LinkState::Ready,
                "Headset gone. A 3D game keeps running; the link closes if the headset does not come back.".into(),
            ),
        }
    }

    /// The snapshot as it stands. Health and stats wait for the health loop (#125); `tunnel`
    /// is the tether being up until that loop checks tun0.
    fn current(&self, now: u64) -> LinkSnapshot {
        let (state, message) = self.state_text();
        let tether = self
            .tether
            .as_ref()
            .map_or(TetherMode::Off, |tether| tether.mode());
        LinkSnapshot {
            state,
            message,
            tether,
            tunnel: tether != TetherMode::Off,
            auto_3d: self.inputs.auto_3d,
            adb: self.adb.path.clone(),
            adb_source: self.adb.source,
            health: None,
            stats: None,
            events: self.ring.events().to_vec(),
            updated: now,
        }
    }

    fn publish(&mut self, now: u64) -> LinkSnapshot {
        let snapshot = self.current(now);
        if let Err(message) = write_snapshot(&self.paths.snapshot(), &snapshot) {
            self.log(now, &format!("state file: {message}"));
        }
        snapshot
    }

    fn log(&self, now: u64, line: &str) {
        append_log(&self.paths.log(), now, line);
    }
}

/// Write the snapshot to `path` atomically: a temp file, then a rename.
pub fn write_snapshot(path: &str, snapshot: &LinkSnapshot) -> Result<(), String> {
    let text = serde_json::to_string(snapshot).map_err(|error| error.to_string())?;
    let target = Path::new(path);
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    }
    let tmp = format!("{path}.tmp");
    fs::write(&tmp, text.as_bytes()).map_err(|error| format!("{tmp}: {error}"))?;
    fs::rename(&tmp, target).map_err(|error| format!("{path}: {error}"))
}

fn append_log(path: &str, now: u64, line: &str) {
    if let Some(dir) = Path::new(path).parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{now} {line}");
    }
}

/// Read the request file and remove it. Anything but `start-3d` or `stop-3d` is dropped.
fn take_request(path: &str) -> Option<Request> {
    let text = fs::read_to_string(path).ok()?;
    let _ = fs::remove_file(path);
    state::parse_request(&text)
}

/// The settings, or the defaults when the file is missing.
fn read_settings(path: &str) -> Settings {
    fs::read_to_string(path)
        .map(|text| parse_settings(&text))
        .unwrap_or_default()
}

/// Whether a live game runs in 3D and whether one runs flat, from the run records.
fn game_modes(runs_dir: &str) -> (bool, bool) {
    let records: Vec<RunRecord> = fs::read_dir(runs_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|text| parse_run_record(&text))
        .collect();
    let alive = |pid: u32| Path::new(&format!("/proc/{pid}")).exists();
    running_modes(&records, &alive)
}

/// The adb server's binary, when one is running (the one the tether and link should use).
fn running_adb_exe() -> Option<String> {
    let output = Command::new("ss").arg("-tlnp").output().ok()?;
    let pid = tether_adb::listener_pid(&String::from_utf8_lossy(&output.stdout), ADB_SERVER_PORT)?;
    fs::read_link(tether_adb::proc_exe_path(pid))
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default()
}

/// The link service's loop: every second, read the settings, the request file, the run records
/// and the device list, and carry out what they decide. Runs until it is stopped.
pub fn run(paths: &Paths) -> ! {
    let bundled_path = tether_adb::bundled_adb(&paths.home);
    let bundled = if Path::new(&bundled_path).is_file() {
        Some(bundled_path)
    } else {
        None
    };
    let choice = tether_adb::choose_adb(running_adb_exe().as_deref(), bundled.as_deref());
    let feed = TrackDevices::new(&choice.path);
    let previous = fs::read_to_string(paths.snapshot())
        .ok()
        .and_then(|text| serde_json::from_str::<LinkSnapshot>(&text).ok());
    let mut service = Service::new(
        paths.clone(),
        Rc::new(RefCell::new(ProcessRunner::default())),
        feed,
        choice,
    );
    if let Some(previous) = previous {
        service.restore(previous.events);
    }
    loop {
        let now = unix_now();
        let settings = read_settings(&paths.settings());
        let request = take_request(&paths.request());
        let modes = game_modes(&paths.runs());
        service.tick(now, &settings, request, modes);
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(test)]
mod service_tests {
    use super::*;
    use crate::api::AdbSource;
    use std::collections::VecDeque;

    /// Records every command as one line; reports the headset as ready to adb.
    #[derive(Default)]
    struct Fake {
        ran: Vec<String>,
    }

    impl CommandRunner for Fake {
        fn run(&mut self, command: &[String]) -> Result<String, String> {
            self.ran.push(command.join(" "));
            if command.iter().any(|word| word == "devices") {
                return Ok("List of devices attached\nSER\tdevice\n".to_string());
            }
            Ok(String::new())
        }

        fn start_relay(&mut self, command: &[String], _adb: &str) -> Result<(), String> {
            self.ran.push(format!("relay {}", command.join(" ")));
            Ok(())
        }

        fn stop_relay(&mut self) -> Result<(), String> {
            self.ran.push("relay stop".to_string());
            Ok(())
        }
    }

    /// One entry per tick: a new device list, or `None` when no new frame came.
    struct Frames(VecDeque<Option<Vec<Device>>>);

    impl Feed for Frames {
        fn poll(&mut self) -> Option<Vec<Device>> {
            self.0.pop_front().flatten()
        }
    }

    fn device(serial: &str, state: &str) -> Vec<Device> {
        vec![Device {
            serial: serial.to_string(),
            state: state.to_string(),
        }]
    }

    fn settings(tunnel: bool) -> Settings {
        Settings {
            tunnel,
            ..Settings::default()
        }
    }

    fn service(
        name: &str,
        frames: Vec<Option<Vec<Device>>>,
    ) -> (Rc<RefCell<Fake>>, Service<Fake, Frames>) {
        let dir = std::env::temp_dir().join(format!("flat3d-watch-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let paths = Paths {
            home: dir.to_string_lossy().into_owned(),
            state_dir: format!("{}/state", dir.display()),
        };
        let fake = Rc::new(RefCell::new(Fake::default()));
        let choice = AdbChoice {
            path: "adb".into(),
            source: AdbSource::Running,
        };
        (
            Rc::clone(&fake),
            Service::new(paths, fake, Frames(frames.into()), choice),
        )
    }

    #[test]
    fn plug_authorise_ready_unplug_sequence_sees_the_right_snapshots() {
        let (fake, mut service) = service(
            "sequence",
            vec![
                Some(device("SER", "unauthorized")),
                Some(device("SER", "device")),
                None,
                Some(Vec::new()),
            ],
        );
        let s = settings(false);

        let first = service.tick(100, &s, None, (false, false));
        assert_eq!(first.state, LinkState::Unauthorised);
        assert!(first.events.is_empty());
        assert!(fake.borrow().ran.is_empty());

        let second = service.tick(101, &s, None, (false, false));
        assert_eq!(second.state, LinkState::Ready);
        assert_eq!(second.events.len(), 1);
        assert_eq!(second.events[0].kind, EventKind::Ready);
        assert_eq!(second.events[0].seq, 1);
        assert!(fake
            .borrow()
            .ran
            .iter()
            .any(|line| line.contains("reverse tcp:9757 tcp:9757")));

        let third = service.tick(102, &s, None, (false, false));
        assert_eq!(third.state, LinkState::Ready);
        assert_eq!(third.events.len(), 1);
        let before = fake.borrow().ran.len();

        let fourth = service.tick(103, &s, None, (false, false));
        assert_eq!(fourth.state, LinkState::NoHeadset);
        assert_eq!(fourth.events.len(), 1);
        let ran = fake.borrow().ran.clone();
        assert!(ran[before..]
            .iter()
            .any(|line| line.ends_with("am force-stop org.meumeu.wivrn.github")));
        assert!(ran[before..]
            .iter()
            .any(|line| line.ends_with("systemctl --user stop flat3d-wivrn.service")));

        let written: LinkSnapshot =
            serde_json::from_str(&fs::read_to_string(service.paths.snapshot()).unwrap()).unwrap();
        assert_eq!(written.state, LinkState::NoHeadset);
        assert_eq!(written.events.len(), 1);
    }

    #[test]
    fn a_tether_connect_runs_the_wivrn_commands_once() {
        let (fake, mut service) = service(
            "tunnel",
            vec![Some(device("SER", "device")), Some(Vec::new())],
        );
        let s = settings(true);

        let up = service.tick(200, &s, None, (false, false));
        assert_eq!(up.state, LinkState::Ready);
        assert_eq!(up.tether, TetherMode::Own);
        assert!(up.tunnel);
        {
            let ran = &fake.borrow().ran;
            assert_eq!(
                ran.iter()
                    .filter(|line| line.contains("systemd-run"))
                    .count(),
                1
            );
            assert!(ran.iter().any(|line| line.starts_with("relay ")));
        }

        let down = service.tick(201, &s, None, (false, false));
        assert_eq!(down.state, LinkState::NoHeadset);
        assert_eq!(down.tether, TetherMode::Off);
        assert!(fake.borrow().ran.iter().any(|line| line == "relay stop"));
    }

    #[test]
    fn a_3d_game_keeps_the_link_through_the_grace_then_it_closes() {
        let (fake, mut service) = service(
            "grace",
            vec![Some(device("SER", "device")), Some(Vec::new()), None],
        );
        let s = settings(false);

        service.tick(300, &s, None, (false, false));
        let gone = service.tick(301, &s, None, (true, false));
        assert_eq!(gone.state, LinkState::Ready);
        assert!(!fake
            .borrow()
            .ran
            .iter()
            .any(|line| line.contains("force-stop")));

        let later = service.tick(302, &s, None, (false, false));
        assert_eq!(later.state, LinkState::NoHeadset);
        assert!(fake
            .borrow()
            .ran
            .iter()
            .any(|line| line.contains("force-stop")));
    }

    #[test]
    fn the_event_kinds_match_the_note_kinds() {
        for note in [
            Note::Ready,
            Note::LateRestartFor3d,
            Note::StoppedRestartForFlat,
            Note::AllowUsbDebugging,
        ] {
            let kind = serde_json::to_value(note_event(note)).unwrap();
            assert_eq!(kind, serde_json::Value::String(note_kind(note).to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_track_frames;

    #[test]
    fn a_non_ascii_char_in_the_header_drops_the_buffer() {
        assert_eq!(parse_track_frames("zzz\u{e9}rest"), (vec![], String::new()));
        assert_eq!(parse_track_frames("abc\u{e9}"), (vec![], String::new()));
        assert_eq!(parse_track_frames("0\u{e9}"), (vec![], String::new()));
    }
}
