//! Locked tests for the Stop 3D app fix (refs #305, #312): Stop 3D and the start roll-back
//! force-stop the Quest app the service detected, not always WiVRn's default app. Fake
//! runners only; nothing touches adb, WiVRn or the network.

use flat3d_inject::api::{AdbSource, LinkState};
use flat3d_inject::link::adb::AdbChoice;
use flat3d_inject::link::{
    stop_3d, stop_3d_commands, stop_3d_commands_with, stop_3d_with, CommandOutput, CommandRunner,
    WIVRN_APP,
};
use flat3d_inject::state::Request;
use flat3d_inject::switches::Settings;
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::watch::{Device, Feed, Paths, Service};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;

const ADB: &str = "/opt/flat3d-test/adb";
const SERIAL: &str = "SER";
const OTHER_APP: &str = "org.meumeu.wivrn";

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

struct Recorder {
    ran: Vec<Vec<String>>,
}

impl CommandRunner for Recorder {
    fn run(&mut self, command: &[String]) -> CommandOutput {
        self.ran.push(command.to_vec());
        CommandOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

#[test]
fn stop_commands_with_an_app_name_that_app() {
    let expected = vec![
        strings(&["adb", "shell", "am", "force-stop", OTHER_APP]),
        strings(&["adb", "reverse", "--remove", "tcp:9757"]),
        strings(&["flatpak", "kill", "io.github.wivrn.wivrn"]),
        strings(&["systemctl", "--user", "stop", "flat3d-wivrn.service"]),
    ];
    assert_eq!(stop_3d_commands_with(OTHER_APP), expected);
}

#[test]
fn the_no_arg_stop_commands_are_the_default_app() {
    assert_eq!(WIVRN_APP, "org.meumeu.wivrn.github");
    assert_eq!(stop_3d_commands(), stop_3d_commands_with(WIVRN_APP));
    assert_eq!(
        stop_3d_commands()[0],
        strings(&[
            "adb",
            "shell",
            "am",
            "force-stop",
            "org.meumeu.wivrn.github"
        ])
    );
}

#[test]
fn stop_3d_with_runs_the_commands_for_that_app() {
    let mut r = Recorder { ran: Vec::new() };
    assert_eq!(stop_3d_with(&mut r, OTHER_APP), Ok(LinkState::Stopped));
    assert_eq!(r.ran, stop_3d_commands_with(OTHER_APP));

    let mut d = Recorder { ran: Vec::new() };
    assert_eq!(stop_3d(&mut d), Ok(LinkState::Stopped));
    assert_eq!(d.ran, stop_3d_commands());
}

struct Frames(VecDeque<Option<Vec<Device>>>);

impl Feed for Frames {
    fn poll(&mut self) -> Option<Vec<Device>> {
        self.0.pop_front().flatten()
    }
}

fn device() -> Option<Vec<Device>> {
    Some(vec![Device {
        serial: SERIAL.to_string(),
        state: "device".to_string(),
    }])
}

fn service(
    name: &str,
    fake: FakeRunner,
    frames: Vec<Option<Vec<Device>>>,
) -> (Rc<RefCell<FakeRunner>>, Service<FakeRunner, Frames>) {
    let dir = format!("{}/stop-app-{name}", env!("CARGO_TARGET_TMPDIR"));
    let _ = fs::remove_dir_all(&dir);
    let paths = Paths {
        home: dir.clone(),
        state_dir: format!("{dir}/state"),
    };
    let fake = Rc::new(RefCell::new(fake));
    let choice = AdbChoice {
        path: ADB.to_string(),
        source: AdbSource::Running,
    };
    let service = Service::new(paths, Rc::clone(&fake), Frames(frames.into()), choice);
    (fake, service)
}

fn lines(fake: &Rc<RefCell<FakeRunner>>) -> Vec<String> {
    fake.borrow().ran.iter().map(|a| a.join(" ")).collect()
}

fn auto_3d(on: bool) -> Settings {
    Settings {
        auto_3d: on,
        ..Settings::default()
    }
}

/// The headset is ready, the default app is missing and `org.meumeu.wivrn` is installed.
fn detected_runner() -> FakeRunner {
    FakeRunner::new()
        .ok(
            &[ADB, "-s", SERIAL, "devices"],
            "List of devices attached\nSER\tdevice\n",
        )
        .ok(
            &[ADB, "-s", SERIAL, "shell", "dumpsys", "package", WIVRN_APP],
            "Unable to find package: org.meumeu.wivrn.github\n",
        )
        .ok(
            &[ADB, "-s", SERIAL, "shell", "dumpsys", "package", OTHER_APP],
            "Packages:\n  Package [org.meumeu.wivrn] (1a2b3c):\n    versionName=26.9\n",
        )
}

fn force_stop(app: &str) -> String {
    format!("{ADB} -s {SERIAL} shell am force-stop {app}")
}

#[test]
fn stop_3d_request_force_stops_the_detected_app() {
    let (fake, mut service) = service("detected-stop", detected_runner(), vec![device()]);
    let s = auto_3d(true);

    let up = service.tick(100, &s, None, (false, false));
    assert_eq!(up.state, LinkState::Ready);
    let stopped = service.tick(101, &s, Some(Request::Stop3d), (false, false));
    assert_eq!(stopped.state, LinkState::Stopped);

    let ran = lines(&fake);
    assert_eq!(
        ran.iter()
            .filter(|l| l.contains("am force-stop"))
            .collect::<Vec<_>>(),
        vec![&force_stop(OTHER_APP)],
        "{ran:?}"
    );
}

#[test]
fn stop_3d_with_nothing_detected_uses_the_default_and_never_probes() {
    let fake = FakeRunner::new().ok(
        &[ADB, "-s", SERIAL, "devices"],
        "List of devices attached\nSER\tdevice\n",
    );
    let (fake, mut service) = service("default-stop", fake, vec![device()]);
    let s = auto_3d(true);

    let up = service.tick(200, &s, None, (false, false));
    assert_eq!(up.state, LinkState::Ready);
    let probes_before = lines(&fake)
        .iter()
        .filter(|l| l.contains("dumpsys package"))
        .count();
    let stopped = service.tick(201, &s, Some(Request::Stop3d), (false, false));
    assert_eq!(stopped.state, LinkState::Stopped);

    let ran = lines(&fake);
    let probes_after = ran.iter().filter(|l| l.contains("dumpsys package")).count();
    assert_eq!(probes_after, probes_before, "{ran:?}");
    assert!(ran.contains(&force_stop(WIVRN_APP)), "{ran:?}");
    assert!(!ran.contains(&force_stop(OTHER_APP)), "{ran:?}");
}

#[test]
fn a_failed_start_rolls_back_with_the_detected_app() {
    let fake = detected_runner().fail(
        &[ADB, "-s", SERIAL, "shell", "am", "start"],
        "Error: Activity not started",
    );
    let (fake, mut service) = service("rollback", fake, vec![device()]);
    let s = auto_3d(false);

    let plugged = service.tick(300, &s, None, (false, false));
    assert_eq!(plugged.state, LinkState::Stopped);
    let failed = service.tick(301, &s, Some(Request::Start3d), (false, false));
    assert_eq!(failed.state, LinkState::Stopped);

    let ran = lines(&fake);
    assert_eq!(
        ran.iter()
            .filter(|l| l.contains("am force-stop"))
            .collect::<Vec<_>>(),
        vec![&force_stop(OTHER_APP)],
        "{ran:?}"
    );
}
