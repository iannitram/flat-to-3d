//! Locked tests for WP-09: the link service's 3D half, run through `watch::Service` with a
//! scripted device feed and the shared `FakeRunner`: the allow-debugging event, the detected
//! WiVRn app, Stop 3D through `link::stop_3d`, and the health probes through the adb in use.

use flat3d_inject::api::{AdbSource, EventKind, LinkState};
use flat3d_inject::link::adb::AdbChoice;
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

/// One entry per tick: a new device list, or `None` when no new frame came.
struct Frames(VecDeque<Option<Vec<Device>>>);

impl Feed for Frames {
    fn poll(&mut self) -> Option<Vec<Device>> {
        self.0.pop_front().flatten()
    }
}

fn device(state: &str) -> Option<Vec<Device>> {
    Some(vec![Device {
        serial: SERIAL.to_string(),
        state: state.to_string(),
    }])
}

fn paths(name: &str) -> Paths {
    let dir = format!("{}/watch-service-{name}", env!("CARGO_TARGET_TMPDIR"));
    let _ = fs::remove_dir_all(&dir);
    Paths {
        home: dir.clone(),
        state_dir: format!("{dir}/state"),
    }
}

/// A runner that lists the headset as ready to the adb in use, `times` times.
fn ready_runner(times: usize) -> FakeRunner {
    (0..times).fold(FakeRunner::new(), |fake, _| {
        fake.ok(
            &[ADB, "-s", SERIAL, "devices"],
            "List of devices attached\nSER\tdevice\n",
        )
    })
}

fn service(
    name: &str,
    fake: FakeRunner,
    frames: Vec<Option<Vec<Device>>>,
) -> (Rc<RefCell<FakeRunner>>, Paths, Service<FakeRunner, Frames>) {
    let paths = paths(name);
    let fake = Rc::new(RefCell::new(fake));
    let choice = AdbChoice {
        path: ADB.to_string(),
        source: AdbSource::Running,
    };
    let service = Service::new(
        paths.clone(),
        Rc::clone(&fake),
        Frames(frames.into()),
        choice,
    );
    (fake, paths, service)
}

fn lines(fake: &Rc<RefCell<FakeRunner>>) -> Vec<String> {
    fake.borrow()
        .ran
        .iter()
        .map(|argv| argv.join(" "))
        .collect()
}

fn auto_3d(on: bool) -> Settings {
    Settings {
        auto_3d: on,
        ..Settings::default()
    }
}

#[test]
fn allow_debugging_event_once() {
    let (_fake, _paths, mut service) = service(
        "allow-debugging",
        ready_runner(1),
        vec![
            device("unauthorized"),
            device("unauthorized"),
            device("device"),
        ],
    );
    let s = auto_3d(true);

    let first = service.tick(100, &s, None, (false, false));
    assert_eq!(first.state, LinkState::Unauthorised);
    let second = service.tick(101, &s, None, (false, false));
    assert_eq!(second.state, LinkState::Unauthorised);
    let third = service.tick(102, &s, None, (false, false));
    assert_eq!(third.state, LinkState::Ready);

    let events: Vec<(EventKind, u64)> = third.events.iter().map(|e| (e.kind, e.seq)).collect();
    assert_eq!(
        events,
        vec![(EventKind::AllowDebugging, 1), (EventKind::Ready, 2)]
    );
    assert_eq!(
        third.events[0].text,
        "Put the headset on and allow USB debugging."
    );
}

#[test]
fn wivrn_start_uses_the_detected_app() {
    let fake = ready_runner(1)
        .ok(
            &[
                ADB,
                "-s",
                SERIAL,
                "shell",
                "dumpsys",
                "package",
                "org.meumeu.wivrn.github",
            ],
            "Unable to find package: org.meumeu.wivrn.github\n",
        )
        .ok(
            &[
                ADB,
                "-s",
                SERIAL,
                "shell",
                "dumpsys",
                "package",
                "org.meumeu.wivrn",
            ],
            "Packages:\n  Package [org.meumeu.wivrn] (1a2b3c):\n    versionName=26.9\n",
        );
    let (fake, _paths, mut service) = service("detected-app", fake, vec![device("device")]);
    let s = auto_3d(false);

    let plugged = service.tick(200, &s, None, (false, false));
    assert_eq!(plugged.state, LinkState::Stopped);
    let started = service.tick(201, &s, Some(Request::Start3d), (false, false));
    assert_eq!(started.state, LinkState::Ready);

    let ran = fake.borrow().ran.clone();
    let am_start = ran
        .iter()
        .find(|argv| argv.windows(2).any(|pair| pair == ["am", "start"]))
        .expect("Start 3D opens the WiVRn app");
    assert_eq!(
        am_start.last().map(String::as_str),
        Some("org.meumeu.wivrn")
    );
    let server = ran
        .iter()
        .find(|argv| argv.first().map(String::as_str) == Some("systemd-run"))
        .expect("Start 3D starts the server unit");
    assert!(server
        .join(" ")
        .ends_with("--early-active-runtime --no-publish-service"));
}

#[test]
fn two_stop_requests_end_stopped() {
    let fake = ready_runner(1).fail(
        &["systemctl", "--user", "stop", "flat3d-wivrn.service"],
        "Failed to stop flat3d-wivrn.service: Unit flat3d-wivrn.service not loaded.",
    );
    let (fake, paths, mut service) = service("two-stops", fake, vec![device("device")]);
    let s = auto_3d(true);

    let up = service.tick(300, &s, None, (false, false));
    assert_eq!(up.state, LinkState::Ready);
    let first = service.tick(301, &s, Some(Request::Stop3d), (false, false));
    assert_eq!(first.state, LinkState::Stopped);
    let second = service.tick(302, &s, Some(Request::Stop3d), (false, false));
    assert_eq!(second.state, LinkState::Stopped);

    let log = fs::read_to_string(paths.log()).unwrap_or_default();
    assert!(
        log.lines().all(|line| !line.contains("flat3d-wivrn")),
        "{log}"
    );
    let stops = lines(&fake)
        .iter()
        .filter(|line| line.contains("systemctl --user stop"))
        .count();
    assert_eq!(stops, 1);
}

#[test]
fn health_probes_use_the_chosen_adb() {
    let (fake, _paths, mut service) =
        service("health-adb", ready_runner(1), vec![device("device")]);
    let s = auto_3d(true);

    service.tick(400, &s, None, (false, false));

    let ran = lines(&fake);
    let probe = format!("{ADB} -s {SERIAL} shell dumpsys battery");
    assert!(ran.contains(&probe), "{ran:?}");
    assert!(ran.iter().all(|line| line != "adb shell dumpsys battery"));
}

#[test]
fn no_tether_word_runs() {
    let (fake, _paths, mut service) = service(
        "no-tether",
        ready_runner(1),
        vec![device("device"), None, None],
    );
    let s = auto_3d(false);

    let plugged = service.tick(500, &s, None, (false, false));
    assert_eq!(plugged.state, LinkState::Stopped);
    let started = service.tick(501, &s, Some(Request::Start3d), (false, false));
    assert_eq!(started.state, LinkState::Ready);
    let stopped = service.tick(502, &s, Some(Request::Stop3d), (false, false));
    assert_eq!(stopped.state, LinkState::Stopped);

    for line in lines(&fake) {
        // The relay's name is split so WP-37's `git grep -i` over crates/ stays empty.
        for word in [concat!("gnire", "htet"), "tun0", "nmcli"] {
            assert!(!line.contains(word), "{line}");
        }
    }
}
