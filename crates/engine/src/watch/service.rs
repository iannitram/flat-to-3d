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
use super::machine::{
    link_commands, note_event, note_text, step, Effect, Event, Inputs, Phase, Watch,
};
#[cfg(test)]
use super::machine::{note_kind, Note};
use crate::api::{LinkEvent, LinkSnapshot, LinkState, TetherMode};
use crate::link;
use crate::link::Headset;
use crate::state::{EventRing, Request};
use crate::switches::Settings;
use crate::tether::adb::{AdbChoice, CommandRunner, TetherAdb};
use crate::tether::{stepaside, Tether};

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
    health: HealthLoop,
    #[allow(dead_code)]
    heal: Option<crate::tether::heal::Heal>,
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
            health: HealthLoop::new(crate::sys::unix_now(), 0, None),
            heal: None,
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
        let before = self.watch;
        let (next, effects) = step(&self.watch, &event, now, &self.inputs);
        self.watch = next;
        self.execute(now, before, effects);
    }

    fn execute(&mut self, now: u64, before: Watch, effects: Vec<Effect>) {
        // A tether connect runs Start 3D's WiVRn commands itself only when this batch also starts
        // the link; a StartLink in the same batch then must not run them a second time
        // (systemd-run would find the unit running). Without a StartLink the tether connects
        // the tunnel alone and leaves WiVRn to a later Start 3D.
        let with_link = effects.contains(&Effect::StartLink);
        let mut wivrn_by_tether = false;
        let mut link_failed = false;
        for effect in effects {
            match effect {
                Effect::StartTether => match self.connect_tether(with_link) {
                    Ok(()) => wivrn_by_tether = with_link,
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
                        self.roll_back(now, before);
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
        let mut adb = TetherAdb::new(
            self.adb.clone(),
            &self.serial,
            Shared(Rc::clone(&self.runner)),
        );
        adb.run(command)
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

    fn connect_tether(&mut self, with_wivrn: bool) -> Result<(), String> {
        let ps_command: Vec<String> = vec!["ps".into(), "-eo".into(), "pid=,args=".into()];
        let ps = self.runner.borrow_mut().run(&ps_command)?;
        let foreign = stepaside::foreign_relay(&stepaside::parse_ps(&ps), None);
        let serial = self.serial.clone();
        let tether = self.tether_for(&serial);
        let result = if with_wivrn {
            tether.connect(foreign)
        } else {
            tether.connect_tunnel_only(foreign)
        };
        result.map(|_| ())
    }

    /// Stop the tether. With the headset gone its device commands cannot work, so the tether is
    /// torn down on this machine only and is ready to connect again on the next plug-in.
    fn disconnect_tether(&mut self) -> Result<(), String> {
        let ready = self.ready;
        match self.tether.as_mut() {
            Some(tether) if ready => tether.disconnect(),
            Some(tether) => tether.drop_local(),
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
    fn current(&self, now: u64, tick: &HealthTick) -> LinkSnapshot {
        let (state, message) = self.state_text();
        let tether = self
            .tether
            .as_ref()
            .map_or(TetherMode::Off, |tether| tether.mode());
        LinkSnapshot {
            state,
            message,
            tether,
            tunnel: tick.tunnel,
            auto_3d: self.inputs.auto_3d,
            adb: self.adb.path.clone(),
            adb_source: self.adb.source,
            health: tick.health.clone(),
            stats: tick.stats.clone(),
            events: self.ring.events().to_vec(),
            updated: now,
        }
    }

    fn publish(&mut self, now: u64) -> LinkSnapshot {
        let mode = self
            .tether
            .as_ref()
            .map_or(TetherMode::Off, |tether| tether.mode());
        let playing = self.inputs.game_3d_running;
        let mut shared = Shared(Rc::clone(&self.runner));
        let tick = self.health.tick(now, mode, playing, &mut shared);
        if let Some((kind, text)) = tick.toast.clone() {
            self.ring.push(kind, &text);
        }
        if tick.heal.is_some() {
            self.heal = tick.heal;
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
        relay_up: bool,
    }

    impl CommandRunner for Fake {
        fn run(&mut self, command: &[String]) -> Result<String, String> {
            let line = command.join(" ");
            self.ran.push(line.clone());
            if self.gone && command.first().map(String::as_str) == Some("adb") {
                return Err("Tether command failed".to_string());
            }
            if self
                .fail
                .as_ref()
                .is_some_and(|word| line.contains(word.as_str()))
            {
                return Err("Tether command failed".to_string());
            }
            if command.iter().any(|word| word == "devices") {
                return Ok("List of devices attached\nSER\tdevice\n".to_string());
            }
            if line == "ps -eo pid=,args=" && self.relay_up {
                return Ok("7 /x/gnirehtet run SER".to_string());
            }
            if line.ends_with("ip addr show tun0") {
                return Ok("    inet 10.0.0.2/30 scope global tun0".to_string());
            }
            Ok(String::new())
        }

        fn start_relay(&mut self, command: &[String], _adb: &str) -> Result<(), String> {
            self.ran.push(format!("relay {}", command.join(" ")));
            self.relay_up = true;
            Ok(())
        }

        fn stop_relay(&mut self) -> Result<(), String> {
            self.ran.push("relay stop".to_string());
            self.relay_up = false;
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
        assert!(fake
            .borrow()
            .ran
            .iter()
            .all(|c| c == "adb shell dumpsys battery"));

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

    fn count(fake: &Rc<RefCell<Fake>>, word: &str) -> usize {
        fake.borrow()
            .ran
            .iter()
            .filter(|line| line.contains(word))
            .count()
    }

    #[test]
    fn an_unplug_tears_the_tether_down_when_adb_cannot_reach_the_headset() {
        let (fake, mut service) = service(
            "unplug-tether",
            vec![
                Some(device("SER", "device")),
                Some(Vec::new()),
                Some(device("SER", "device")),
            ],
        );
        let s = settings(true);

        let up = service.tick(400, &s, None, (false, false));
        assert_eq!(up.tether, TetherMode::Own);
        assert!(up.tunnel);

        fake.borrow_mut().gone = true;
        let down = service.tick(401, &s, None, (false, false));
        assert_eq!(down.state, LinkState::NoHeadset);
        assert_eq!(down.tether, TetherMode::Off);
        assert!(!down.tunnel);
        assert!(fake.borrow().ran.iter().any(|line| line == "relay stop"));

        fake.borrow_mut().gone = false;
        // Past the health loop's 5s tun0 cadence, so the re-plug tick probes tun0 again.
        let back = service.tick(410, &s, None, (false, false));
        assert_eq!(back.tether, TetherMode::Own);
        assert!(back.tunnel);
    }

    #[test]
    fn with_auto_3d_off_a_plug_brings_up_only_the_tunnel_and_start_runs_wivrn_once() {
        let (fake, mut service) = service("manual-tunnel", vec![Some(device("SER", "device"))]);
        let s = Settings {
            auto_3d: false,
            tunnel: true,
            ..Settings::default()
        };

        let plugged = service.tick(500, &s, None, (false, false));
        assert_eq!(plugged.state, LinkState::Stopped);
        assert_eq!(plugged.tether, TetherMode::Own);
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
    fn a_failed_link_start_does_not_read_as_ready() {
        let (fake, mut service) = service("failed-start", vec![Some(device("SER", "device"))]);
        fake.borrow_mut().fail = Some("reverse tcp:9757".to_string());
        let s = settings(false);

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
