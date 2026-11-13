use std::cell::RefCell;
#[cfg(test)]
use std::fs;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use super::files::{append_log, write_snapshot, Paths};
use super::frames::{parse_track_frames, Device};
use super::health_loop::{HealthLoop, HealthTick};
#[cfg(test)]
use super::machine::note_kind;
use super::machine::{
    link_commands_with, note_event, note_text, step, Effect, Event, Inputs, Note, Phase, Watch,
};
use crate::api::{LinkEvent, LinkSnapshot, LinkState};
use crate::exec::LinkRunner as CommandRunner;
use crate::link;
use crate::link::adb::{AdbChoice, AdbRunner};
use crate::link::quest_tether::quest_tether_status;
use crate::link::Headset;
use crate::state::{EventRing, Request};
use crate::switches::Settings;
use crate::wivrn::{detect_quest_app, WivrnStart, SERVER_FLAGS};

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
        // architecture-allowlist: follow-up (track-devices remains outside the runner seam).
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

/// Runs the health loop's `adb` words through an [`AdbRunner`] over the shared runner, so they
/// run as `<adb> -s <serial> ...`. With no adb or no headset serial nothing runs.
struct HealthAdb<R: link::CommandRunner>(Result<AdbRunner<R>, &'static str>);

impl<R: link::CommandRunner> CommandRunner for HealthAdb<R> {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        let adb = self.0.as_mut().map_err(|message| (*message).to_string())?;
        let output = link::CommandRunner::run(adb, command);
        if output.success {
            Ok(output.stdout)
        } else {
            Err(output.stderr.trim().to_string())
        }
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

/// The one runner the link service and its health loop share, so both act through the same
/// process runner.
struct Shared<R>(Rc<RefCell<R>>);

impl<R: CommandRunner> CommandRunner for Shared<R> {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        self.0.borrow_mut().run(command)
    }
}

/// The same runner in the link code's shape, so `link::stop_3d`, `detect_quest_app` and an
/// [`AdbRunner`] can act through it. A failure's message becomes the stderr the link code reads.
impl<R: CommandRunner> link::CommandRunner for Shared<R> {
    fn run(&mut self, command: &[String]) -> link::CommandOutput {
        match self.0.borrow_mut().run(command) {
            Ok(stdout) => link::CommandOutput {
                success: true,
                stdout,
                stderr: String::new(),
            },
            Err(message) => link::CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: stderr_of(&message).to_string(),
            },
        }
    }
}

/// The stderr inside a runner's failure message. `ProcessRunner` reports a failed command as
/// `Command failed: exit N; <stderr>`, and the link code matches on the stderr alone (Stop 3D
/// reads a server unit that is already gone as stopped); any other message is the stderr.
fn stderr_of(message: &str) -> &str {
    message
        .strip_prefix("Command failed: ")
        .and_then(|rest| rest.split_once("; "))
        .map_or(message, |(_, stderr)| stderr)
}

/// The link service's state: the watch state machine, the headset as last seen and the
/// event ring. Every effect runs through `runner` (adb commands get the adb in use).
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
    ring: EventRing,
    health: HealthLoop,
    /// The WiVRn app found on a headset (serial, package), so the dumpsys probes run once per
    /// headset.
    quest_app: Option<(String, String)>,
    /// Quest Tether's state as last read and when it was asked (read-only; nothing here acts on
    /// it).
    quest_tether: Option<String>,
    quest_tether_asked: Option<u64>,
}

/// Quest Tether is asked for its state at most this often (seconds).
const QUEST_TETHER_EVERY_SECS: u64 = 10;

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
                game_3d_running: false,
                game_flat_running: false,
            },
            ready: false,
            serial: String::new(),
            unauthorised: false,
            starting: false,
            ring: EventRing::new(),
            health: HealthLoop::new(None),
            quest_app: None,
            quest_tether: None,
            quest_tether_asked: None,
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
            game_3d_running: modes.0,
            game_flat_running: modes.1,
        };
        self.read_quest_tether(now);
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
                    if !self.unauthorised {
                        self.ring.push(
                            note_event(Note::AllowUsbDebugging),
                            note_text(Note::AllowUsbDebugging),
                        );
                    }
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

    /// Ask Quest Tether for its state, at most every ten seconds, and keep the `state` text for
    /// the snapshot. A missing program, a failed run or junk reads as not installed, quietly.
    /// Nothing reads the result except the faces.
    fn read_quest_tether(&mut self, now: u64) {
        let due = self
            .quest_tether_asked
            .is_none_or(|asked| now.saturating_sub(asked) >= QUEST_TETHER_EVERY_SECS);
        if !due {
            return;
        }
        self.quest_tether_asked = Some(now);
        self.quest_tether =
            quest_tether_status(&mut Shared(Rc::clone(&self.runner))).map(|status| status.state);
    }

    fn unplug(&mut self, now: u64) {
        if self.ready {
            self.ready = false;
            self.fire(now, Event::Unplugged);
        }
    }

    fn fire(&mut self, now: u64, event: Event) {
        let before = self.watch;
        let (next, effects) = step(&self.watch, &event, now, &self.inputs);
        self.watch = next;
        self.execute(now, before, effects);
    }

    fn execute(&mut self, now: u64, before: Watch, effects: Vec<Effect>) {
        let mut link_failed = false;
        for effect in effects {
            match effect {
                Effect::StartLink | Effect::Reattach => {
                    self.starting = true;
                    self.publish(now);
                    let w = self.wivrn_start();
                    let commands = link_commands_with(&effect, &w);
                    if self.run_steps(now, &commands, true).is_err() {
                        link_failed = true;
                        self.roll_back(now, before);
                    }
                    self.starting = false;
                }
                Effect::StopLink => {
                    // A server unit that is already gone reads as stopped, so a second Stop 3D
                    // logs nothing.
                    if let Err(e) = link::stop_3d(&mut self.own_adb()) {
                        self.log(now, &format!("stop: {}", e.message));
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

    /// The shared runner with the adb in use and the headset's serial.
    fn own_adb(&self) -> AdbRunner<Shared<R>> {
        AdbRunner::new(
            Shared(Rc::clone(&self.runner)),
            &self.adb.path,
            Some(&self.serial),
        )
    }

    /// The WiVRn start for this headset: the app `detect_quest_app` finds on it (WiVRn's
    /// default app when none answers), with the server flags. A found app is kept per serial;
    /// a miss is not, so the next start probes again.
    fn wivrn_start(&mut self) -> WivrnStart {
        let cached = self
            .quest_app
            .as_ref()
            .filter(|(serial, _)| *serial == self.serial)
            .map(|(_, package)| package.clone());
        let app = match cached {
            Some(package) => package,
            None => {
                let serial = self.serial.clone();
                match detect_quest_app(&mut self.own_adb(), &serial).map(|q| q.package) {
                    Some(package) => {
                        self.quest_app = Some((serial, package.clone()));
                        package
                    }
                    None => link::WIVRN_APP.to_string(),
                }
            }
        };
        WivrnStart {
            scope: None,
            app,
            flags: SERVER_FLAGS.map(String::from).to_vec(),
        }
    }

    fn adb_run(&self, command: &[String]) -> Result<String, String> {
        crate::link::adb::run(
            &mut Shared(Rc::clone(&self.runner)),
            &self.adb.path,
            &self.serial,
            command,
        )
    }

    /// A link start failed: it must not read as Ready. With the headset here, run Stop 3D's
    /// list (best effort, so a half-started server unit does not block a retry) and go back to
    /// Plugged, where Start 3D tries again. With the headset gone (a reattach in the grace),
    /// keep the phase the event found.
    fn roll_back(&mut self, now: u64, before: Watch) {
        if self.ready {
            self.log(
                now,
                "3D link did not start; stopping it so Start 3D can retry",
            );
            let commands = crate::link::stop_3d_commands();
            let _ = self.run_steps(now, &commands, false);
            self.watch = Watch {
                phase: Phase::Plugged,
            };
        } else {
            self.log(now, "3D link did not reattach");
            self.watch = before;
        }
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

    /// The snapshot as it stands, with the health the loop last read.
    fn current(&self, now: u64, tick: &HealthTick) -> LinkSnapshot {
        let (state, message) = self.state_text();
        LinkSnapshot {
            state,
            message,
            auto_3d: self.inputs.auto_3d,
            adb: self.adb.path.clone(),
            adb_source: self.adb.source,
            health: tick.health.clone(),
            events: self.ring.events().to_vec(),
            updated: now,
            quest_tether: self.quest_tether.clone(),
        }
    }

    fn publish(&mut self, now: u64) -> LinkSnapshot {
        let playing = self.inputs.game_3d_running;
        let mut adb = HealthAdb(if self.adb.path.is_empty() {
            Err("No adb is available")
        } else if self.serial.is_empty() {
            Err("No headset")
        } else {
            Ok(self.own_adb())
        });
        let tick = self.health.tick(now, playing, &mut adb);
        if let Some((kind, text)) = tick.toast.clone() {
            self.ring.push(kind, &text);
        }
        let snapshot = self.current(now, &tick);
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
#[cfg(test)]
mod service_tests {
    use super::*;
    use crate::api::{AdbSource, EventKind};
    use std::collections::VecDeque;

    /// Records every command as one line; reports the headset as ready to adb. With `gone`,
    /// every adb command fails as it does once the device is unplugged; a command whose line
    /// contains `fail` fails too.
    #[derive(Default)]
    struct Fake {
        ran: Vec<String>,
        gone: bool,
        fail: Option<String>,
        /// What `dumpsys battery` prints; empty by default.
        battery: String,
    }

    impl CommandRunner for Fake {
        fn run(&mut self, command: &[String]) -> Result<String, String> {
            let line = command.join(" ");
            self.ran.push(line.clone());
            if self.gone && command.first().map(String::as_str) == Some("adb") {
                return Err("Command failed".to_string());
            }
            if self
                .fail
                .as_ref()
                .is_some_and(|word| line.contains(word.as_str()))
            {
                return Err("Command failed".to_string());
            }
            if command.iter().any(|word| word == "devices") {
                return Ok("List of devices attached\nSER\tdevice\n".to_string());
            }
            if line.ends_with("dumpsys battery") {
                return Ok(self.battery.clone());
            }
            Ok(String::new())
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

    fn settings() -> Settings {
        Settings::default()
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
        let s = settings();

        let first = service.tick(100, &s, None, (false, false));
        assert_eq!(first.state, LinkState::Unauthorised);
        assert_eq!(first.events.len(), 1);
        assert_eq!(first.events[0].kind, EventKind::AllowDebugging);
        assert_eq!(first.events[0].seq, 1);
        assert!(fake
            .borrow()
            .ran
            .iter()
            .all(|c| !c.contains("dumpsys battery")));

        let second = service.tick(101, &s, None, (false, false));
        assert_eq!(second.state, LinkState::Ready);
        assert_eq!(second.events.len(), 2);
        assert_eq!(second.events[1].kind, EventKind::Ready);
        assert_eq!(second.events[1].seq, 2);
        assert!(fake
            .borrow()
            .ran
            .iter()
            .any(|line| line.contains("reverse tcp:9757 tcp:9757")));

        let third = service.tick(102, &s, None, (false, false));
        assert_eq!(third.state, LinkState::Ready);
        assert_eq!(third.events.len(), 2);
        let before = fake.borrow().ran.len();

        let fourth = service.tick(103, &s, None, (false, false));
        assert_eq!(fourth.state, LinkState::NoHeadset);
        assert_eq!(fourth.events.len(), 2);
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
        assert_eq!(written.events.len(), 2);
    }

    #[test]
    fn a_plug_runs_the_wivrn_commands_once() {
        let (fake, mut service) = service(
            "plug-once",
            vec![Some(device("SER", "device")), Some(Vec::new())],
        );
        let s = settings();

        let up = service.tick(200, &s, None, (false, false));
        assert_eq!(up.state, LinkState::Ready);
        assert_eq!(count(&fake, "systemd-run"), 1);

        let down = service.tick(201, &s, None, (false, false));
        assert_eq!(down.state, LinkState::NoHeadset);
        assert_eq!(count(&fake, "systemd-run"), 1);
    }

    #[test]
    fn a_3d_game_keeps_the_link_through_the_grace_then_it_closes() {
        let (fake, mut service) = service(
            "grace",
            vec![Some(device("SER", "device")), Some(Vec::new()), None],
        );
        let s = settings();

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

    fn count(fake: &Rc<RefCell<Fake>>, word: &str) -> usize {
        fake.borrow()
            .ran
            .iter()
            .filter(|line| line.contains(word))
            .count()
    }

    #[test]
    fn an_unplug_stops_the_link_when_adb_cannot_reach_the_headset() {
        let (fake, mut service) = service(
            "unplug-gone",
            vec![
                Some(device("SER", "device")),
                Some(Vec::new()),
                Some(device("SER", "device")),
            ],
        );
        let s = settings();

        let up = service.tick(400, &s, None, (false, false));
        assert_eq!(up.state, LinkState::Ready);

        fake.borrow_mut().gone = true;
        let before = fake.borrow().ran.len();
        let down = service.tick(401, &s, None, (false, false));
        assert_eq!(down.state, LinkState::NoHeadset);
        assert!(fake.borrow().ran[before..]
            .iter()
            .any(|line| line.ends_with("systemctl --user stop flat3d-wivrn.service")));

        fake.borrow_mut().gone = false;
        let back = service.tick(410, &s, None, (false, false));
        assert_eq!(back.state, LinkState::Ready);
    }

    #[test]
    fn with_auto_3d_off_a_plug_runs_nothing_and_start_runs_wivrn_once() {
        let (fake, mut service) = service("manual-plug", vec![Some(device("SER", "device"))]);
        let s = Settings {
            auto_3d: false,
            ..Settings::default()
        };

        let plugged = service.tick(500, &s, None, (false, false));
        assert_eq!(plugged.state, LinkState::Stopped);
        assert_eq!(count(&fake, "systemd-run"), 0);
        assert_eq!(count(&fake, "wivrn+tcp"), 0);
        let before = fake.borrow().ran.len();

        let started = service.tick(501, &s, Some(Request::Start3d), (false, false));
        assert_eq!(started.state, LinkState::Ready);
        assert_eq!(count(&fake, "systemd-run"), 1);
        assert!(fake.borrow().ran[before..]
            .iter()
            .any(|line| line.contains("systemd-run")));
        assert!(started
            .events
            .iter()
            .any(|event| event.kind == EventKind::Ready));
    }

    #[test]
    fn battery_and_temperature_toasts_fire_while_a_3d_game_plays() {
        let (fake, mut service) = service("toasts", vec![Some(device("SER", "device"))]);
        let s = settings();
        fake.borrow_mut().battery = "level: 80\nstatus: 3\ntemperature: 390\n".into();

        let warm = service.tick(700, &s, None, (true, false));
        let health = warm.health.expect("battery read through adb");
        assert_eq!(health.battery, Some(80));
        assert_eq!(health.temp_c, Some(39.0));
        assert!(warm.events.iter().any(|e| e.kind == EventKind::Warm));
        assert!(fake
            .borrow()
            .ran
            .iter()
            .any(|line| line == "adb -s SER shell dumpsys battery"));

        fake.borrow_mut().battery = "level: 79\nstatus: 3\ntemperature: 430\n".into();
        let hot = service.tick(731, &s, None, (true, false));
        assert!(hot.events.iter().any(|e| e.kind == EventKind::Hot));
    }

    #[test]
    fn a_failed_link_start_does_not_read_as_ready() {
        let (fake, mut service) = service("failed-start", vec![Some(device("SER", "device"))]);
        fake.borrow_mut().fail = Some("reverse tcp:9757".to_string());
        let s = settings();

        let after = service.tick(600, &s, None, (false, false));
        assert_ne!(after.state, LinkState::Ready);
        assert_eq!(after.state, LinkState::Stopped);
        assert!(after.events.is_empty());
        let log = fs::read_to_string(service.paths.log()).unwrap();
        assert!(log.contains("reverse tcp:9757"));

        fake.borrow_mut().fail = None;
        let retried = service.tick(601, &s, Some(Request::Start3d), (false, false));
        assert_eq!(retried.state, LinkState::Ready);
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
