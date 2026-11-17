//! WP-38: the Quest Tether line on both faces. One text, read-only; nothing in the engine reads
//! the snapshot field, which `nothing_depends_on_it` shows by running one watch script twice.

use flat3d_inject::api::{AdbSource, LinkSnapshot, LinkState};
use flat3d_inject::face::link_model::link_view;
use flat3d_inject::link::adb::AdbChoice;
use flat3d_inject::state::Request;
use flat3d_inject::switches::Settings;
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::watch::{Device, Feed, Paths, Service};
use serde_json::json;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;

fn fresh(quest_tether: Option<&str>) -> LinkSnapshot {
    let mut snapshot = LinkSnapshot::stopped(100);
    snapshot.state = LinkState::Ready;
    snapshot.quest_tether = quest_tether.map(String::from);
    snapshot
}

#[test]
fn connected_reads_connected() {
    let view = link_view(Some(fresh(Some("connected"))), 100);
    assert_eq!(
        view.quest_tether.as_deref(),
        Some("Quest Tether: connected")
    );
}

#[test]
fn every_other_state_reads_not_connected() {
    for state in [
        "idle",
        "starting",
        "reconnecting",
        "stopping",
        "error",
        "something-new",
    ] {
        let view = link_view(Some(fresh(Some(state))), 100);
        assert_eq!(
            view.quest_tether.as_deref(),
            Some("Quest Tether: not connected"),
            "{state}"
        );
    }
}

#[test]
fn none_is_hidden() {
    let view = link_view(Some(fresh(None)), 100);
    assert_eq!(view.quest_tether, None);
}

#[test]
fn no_snapshot_is_hidden() {
    assert_eq!(link_view(None, 100).quest_tether, None);
}

#[test]
fn a_stale_snapshot_is_hidden() {
    let mut stale = fresh(Some("connected"));
    stale.updated = 80;
    assert_eq!(link_view(Some(stale), 100).quest_tether, None);
}

#[test]
fn both_faces_get_the_same_text_from_the_view_json() {
    // The app draws `quest_tether` as it is; the Decky panel reads the same key from this JSON.
    let value = serde_json::to_value(link_view(Some(fresh(Some("connected"))), 100)).unwrap();
    assert_eq!(value["quest_tether"], json!("Quest Tether: connected"));
}

#[test]
fn the_snapshot_file_keeps_its_keys_without_quest_tether() {
    let value = serde_json::to_value(fresh(None)).unwrap();
    assert!(value.get("quest_tether").is_none(), "{value}");
    let value = serde_json::to_value(fresh(Some("idle"))).unwrap();
    assert_eq!(value["quest_tether"], json!("idle"));
    let old = r#"{"state":"ready","message":"m","auto_3d":false,"adb":"","adb_source":"missing","health":null,"events":[],"updated":1}"#;
    let read: LinkSnapshot = serde_json::from_str(old).unwrap();
    assert_eq!(read.quest_tether, None);
}

const ADB: &str = "/opt/flat3d-test/adb";
const SERIAL: &str = "SER";

struct Frames(VecDeque<Option<Vec<Device>>>);

impl Feed for Frames {
    fn poll(&mut self) -> Option<Vec<Device>> {
        self.0.pop_front().flatten()
    }
}

fn paths(name: &str) -> Paths {
    let dir = format!("{}/quest-tether-{name}", env!("CARGO_TARGET_TMPDIR"));
    let _ = fs::remove_dir_all(&dir);
    Paths {
        home: dir.clone(),
        state_dir: format!("{dir}/state"),
    }
}

fn choice() -> AdbChoice {
    AdbChoice {
        path: ADB.to_string(),
        source: AdbSource::Running,
    }
}

fn quiet_frames(first: Option<Vec<Device>>) -> Frames {
    let mut frames: VecDeque<Option<Vec<Device>>> = VecDeque::new();
    frames.push_back(first);
    frames.extend(std::iter::repeat_with(|| None).take(40));
    Frames(frames)
}

/// What one run of the watch script left: the commands that are not Quest Tether's, the events,
/// the state after each tick, and the last snapshot's Quest Tether field.
struct Run {
    commands: Vec<String>,
    events: Vec<String>,
    states: Vec<LinkState>,
    quest_tether: Option<String>,
}

/// Plug in, Start 3D, many ticks, Stop 3D.
fn run_script(name: &str, fake: FakeRunner) -> Run {
    let fake = Rc::new(RefCell::new(fake));
    let plugged = Some(vec![Device {
        serial: SERIAL.to_string(),
        state: "device".to_string(),
    }]);
    let mut service = Service::new(
        paths(name),
        Rc::clone(&fake),
        quiet_frames(plugged),
        choice(),
    );
    let settings = Settings::default();
    let script = [(1000, None), (1001, Some(Request::Start3d))]
        .into_iter()
        .chain((1002..1030).map(|now| (now, None)))
        .chain([(1030, Some(Request::Stop3d))]);
    let mut states = Vec::new();
    let mut last = None;
    for (now, request) in script {
        let snapshot = service.tick(now, &settings, request, (false, false));
        states.push(snapshot.state);
        last = Some(snapshot);
    }
    let last = last.unwrap();
    let commands = fake
        .borrow()
        .ran
        .iter()
        .filter(|argv| argv.first().map(String::as_str) != Some("quest-tether"))
        .map(|argv| argv.join(" "))
        .collect();
    let events = last
        .events
        .iter()
        .map(|event| format!("{} {:?} {}", event.seq, event.kind, event.text))
        .collect();
    Run {
        commands,
        events,
        states,
        quest_tether: last.quest_tether,
    }
}

fn ready_runner() -> FakeRunner {
    FakeRunner::new().ok(
        &[ADB, "-s", SERIAL, "devices"],
        "List of devices attached\nSER\tdevice\n",
    )
}

#[test]
fn nothing_depends_on_it() {
    let absent = ready_runner().fail(&["quest-tether"], "quest-tether: not found");
    let mut present = ready_runner();
    for _ in 0..8 {
        present = present.ok(&["quest-tether"], r#"{"state":"connected"}"#);
    }
    let none = run_script("none", absent);
    let up = run_script("connected", present);

    assert_eq!(none.quest_tether, None);
    assert_eq!(up.quest_tether, None);
    assert_eq!(none.states.last(), Some(&LinkState::Stopped));
    assert!(none.states.contains(&LinkState::Ready));
    assert_eq!(none.commands, up.commands);
    assert_eq!(none.events, up.events);
    assert_eq!(none.states, up.states);
}

#[test]
fn it_is_asked_at_most_every_ten_seconds() {
    let fake = Rc::new(RefCell::new(FakeRunner::new()));
    let mut service = Service::new(
        paths("rate"),
        Rc::clone(&fake),
        quiet_frames(Some(vec![Device {
            serial: SERIAL.into(),
            state: "unauthorized".into(),
        }])),
        choice(),
    );
    let settings = Settings::default();
    for now in 2000..2025 {
        service.tick(now, &settings, None, (false, false));
    }
    let asked = fake
        .borrow()
        .ran
        .iter()
        .filter(|argv| argv.first().map(String::as_str) == Some("quest-tether"))
        .count();
    // Ticks 2000..2025 ask at 2000, 2010 and 2020.
    assert_eq!(asked, 3);
}

#[test]
fn only_quest_link_states_identify_a_quest_display() {
    use flat3d_inject::link::quest_tether::is_quest_display;
    for state in [
        LinkState::Starting,
        LinkState::Ready,
        LinkState::Unauthorised,
    ] {
        assert!(is_quest_display(state), "{state:?}");
    }
    for state in [LinkState::Stopped, LinkState::NoHeadset] {
        assert!(!is_quest_display(state), "{state:?}");
        let mut snapshot = fresh(Some("connected"));
        snapshot.state = state;
        assert_eq!(link_view(Some(snapshot), 100).quest_tether, None);
    }
}

#[test]
fn quest_runner_status_controls_the_optional_line() {
    use flat3d_inject::link::quest_tether::status_command;
    let mut replies: Vec<_> = [
        "connected",
        "idle",
        "starting",
        "reconnecting",
        "stopping",
        "error",
    ]
    .into_iter()
    .map(|state| {
        (
            FakeRunner::new().ok(&["quest-tether"], &format!(r#"{{"state":"{state}"}}"#)),
            Some(if state == "connected" {
                "Quest Tether: connected"
            } else {
                "Quest Tether: not connected"
            }),
        )
    })
    .collect();
    replies.extend([
        (
            FakeRunner::new().fail(&["quest-tether"], "No such file or directory (os error 2)"),
            None,
        ),
        (FakeRunner::new().fail(&["quest-tether"], "exit 1"), None),
        (FakeRunner::new().ok(&["quest-tether"], "not json"), None),
    ]);
    for (index, (runner, expected)) in replies.into_iter().enumerate() {
        let fake = Rc::new(RefCell::new(runner));
        let mut service = Service::new(
            paths(&format!("reply-{index}")),
            Rc::clone(&fake),
            quiet_frames(Some(vec![Device {
                serial: SERIAL.into(),
                state: "unauthorized".into(),
            }])),
            choice(),
        );
        let snapshot = service.tick(100, &Settings::default(), None, (false, false));
        assert_eq!(snapshot.state, LinkState::Unauthorised);
        assert_eq!(
            link_view(Some(snapshot), 100).quest_tether.as_deref(),
            expected,
            "reply {index}"
        );
        assert_eq!(fake.borrow().ran, vec![status_command()]);
    }
}

#[test]
fn stopped_and_no_headset_never_run_quest_tether() {
    for (name, devices, expected) in [
        (
            "stopped",
            vec![Device {
                serial: SERIAL.into(),
                state: "device".into(),
            }],
            LinkState::Stopped,
        ),
        ("no-headset", vec![], LinkState::NoHeadset),
    ] {
        let fake = Rc::new(RefCell::new(FakeRunner::new()));
        let mut service = Service::new(
            paths(name),
            Rc::clone(&fake),
            quiet_frames(Some(devices)),
            AdbChoice {
                path: String::new(),
                source: AdbSource::Missing,
            },
        );
        let settings = Settings {
            auto_3d: false,
            ..Settings::default()
        };
        for now in 100..125 {
            let snapshot = service.tick(now, &settings, None, (false, false));
            assert_eq!(snapshot.state, expected);
            assert_eq!(link_view(Some(snapshot), now).quest_tether, None);
        }
        assert!(
            fake.borrow().ran.is_empty(),
            "{name}: {:?}",
            fake.borrow().ran
        );
    }
}

#[test]
fn leaving_quest_clears_status_and_returning_refreshes_immediately() {
    use flat3d_inject::link::quest_tether::status_command;
    let fake = Rc::new(RefCell::new(
        FakeRunner::new()
            .ok(&["quest-tether"], r#"{"state":"connected"}"#)
            .ok(&["quest-tether"], r#"{"state":"idle"}"#),
    ));
    let quest = vec![Device {
        serial: SERIAL.into(),
        state: "unauthorized".into(),
    }];
    let mut service = Service::new(
        paths("return"),
        Rc::clone(&fake),
        Frames(VecDeque::from([
            Some(quest.clone()),
            Some(vec![]),
            Some(quest),
        ])),
        choice(),
    );
    let settings = Settings::default();
    let first = service.tick(100, &settings, None, (false, false));
    assert_eq!(
        link_view(Some(first), 100).quest_tether.as_deref(),
        Some("Quest Tether: connected")
    );
    let gone = service.tick(101, &settings, None, (false, false));
    assert_eq!(link_view(Some(gone), 101).quest_tether, None);
    assert_eq!(fake.borrow().ran, vec![status_command()]);
    let back = service.tick(102, &settings, None, (false, false));
    assert_eq!(
        link_view(Some(back), 102).quest_tether.as_deref(),
        Some("Quest Tether: not connected")
    );
    assert_eq!(fake.borrow().ran, vec![status_command(), status_command()]);
}
