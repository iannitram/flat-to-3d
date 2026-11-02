use flat3d_inject::api::TetherMode;
use flat3d_inject::tether::adb::{choose_adb, CommandRunner};
use flat3d_inject::tether::Tether;
use std::collections::BTreeMap;

#[derive(Default)]
struct Fake {
    calls: Vec<String>,
    fail_at: Option<usize>,
    reverses: BTreeMap<String, String>,
}

impl Fake {
    fn record(&mut self, words: &[String]) -> Result<(), String> {
        self.calls.push(words.join(" "));
        if self.fail_at == Some(self.calls.len()) {
            return Err("The requested tether step failed".into());
        }
        Ok(())
    }
}

impl CommandRunner for Fake {
    fn run(&mut self, words: &[String]) -> Result<String, String> {
        self.record(words)?;
        if words[0] != "/tools/adb" {
            return Ok(String::new());
        }
        assert_eq!(&words[1..3], ["-s", "S1"]);
        let args = &words[3..];
        if args == ["devices"] {
            return Ok("List of devices attached\nS1\tdevice\n".into());
        }
        if args == ["reverse", "--list"] {
            return Ok(self
                .reverses
                .iter()
                .map(|(from, to)| format!("S1 {from} {to}\n"))
                .collect());
        }
        if args.first().map(String::as_str) == Some("reverse") {
            if args[1] == "--remove" {
                self.reverses.remove(&args[2]);
            } else {
                assert_ne!(args[1], "--remove-all");
                self.reverses.insert(args[1].clone(), args[2].clone());
            }
        }
        Ok(String::new())
    }

    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String> {
        assert_eq!(adb, "/tools/adb");
        self.record(command)
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        self.record(&["stop-owned-relay".into()])
    }
}

fn tether(fake: Fake) -> Tether<Fake> {
    Tether::new("/fixture", "S1", choose_adb(Some("/tools/adb"), None), fake)
}

const WIVRN: &[&str] = &[
    "/tools/adb -s S1 reverse --list",
    "/tools/adb -s S1 devices",
    "systemd-run --user --collect --unit=flat3d-wivrn flatpak run --command=wivrn-server io.github.wivrn.wivrn",
    "/tools/adb -s S1 reverse tcp:9757 tcp:9757",
    "/tools/adb -s S1 shell am start -a android.intent.action.VIEW -d wivrn+tcp://localhost org.meumeu.wivrn.github",
];
const OWN: &[&str] = &[
    "/tools/adb -s S1 reverse localabstract:gnirehtet tcp:31416",
    "/tools/adb -s S1 install -r /fixture/.local/share/flat3d/bin/gnirehtet.apk",
    "/tools/adb -s S1 shell am start -n com.genymobile.gnirehtet/.GnirehtetActivity -a com.genymobile.gnirehtet.START",
    "/fixture/.local/share/flat3d/bin/gnirehtet run S1",
];
const REMOVE: &str = "/tools/adb -s S1 reverse --remove localabstract:gnirehtet";
const STOP: &str = "/tools/adb -s S1 shell am start -n com.genymobile.gnirehtet/.GnirehtetActivity -a com.genymobile.gnirehtet.STOP";

#[test]
fn connect_and_disconnect_exact_order_preserves_wivrn() {
    let mut t = tether(Fake::default());
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
    assert_eq!(t.adb.runner.calls, [WIVRN, OWN].concat());
    t.adb.runner.calls.clear();
    assert_eq!(t.disconnect(), Ok(()));
    assert_eq!(t.adb.runner.calls, [STOP, REMOVE, "stop-owned-relay"]);
    assert_eq!(t.mode(), TetherMode::Off);
    assert_eq!(t.adb.runner.reverses.len(), 1);
    assert_eq!(t.adb.runner.reverses["tcp:9757"], "tcp:9757");
    t.adb.runner.calls.clear();
    assert_eq!(t.disconnect(), Ok(()));
    assert!(t.adb.runner.calls.is_empty());
}

#[test]
fn foreign_relay_only_adds_wivrn_and_disconnect_leaves_it_alone() {
    let mut t = tether(Fake::default());
    t.adb
        .runner
        .reverses
        .insert("localabstract:gnirehtet".into(), "tcp:31416".into());
    assert_eq!(t.connect(true), Ok(TetherMode::QuestTether));
    assert_eq!(t.adb.runner.calls, WIVRN);
    t.adb.runner.calls.clear();
    assert_eq!(t.disconnect(), Ok(()));
    assert!(t.adb.runner.calls.is_empty());
    assert_eq!(t.adb.runner.reverses.len(), 2);
}

#[test]
fn failed_install_stops_connect_and_removes_only_added_reverses() {
    let mut t = tether(Fake {
        fail_at: Some(7),
        ..Fake::default()
    });
    assert_eq!(
        t.connect(false),
        Err("The requested tether step failed".into())
    );
    let expected = [
        WIVRN,
        &OWN[..2],
        &[REMOVE, "/tools/adb -s S1 reverse --remove tcp:9757"],
    ]
    .concat();
    assert_eq!(t.adb.runner.calls, expected);
    assert!(t.adb.runner.reverses.is_empty());
    assert_eq!(t.mode(), TetherMode::Off);
}

#[test]
fn every_connect_step_failure_cleans_up() {
    for fail_at in 1..=9 {
        let mut t = tether(Fake {
            fail_at: Some(fail_at),
            ..Fake::default()
        });
        assert!(t.connect(false).is_err(), "step {fail_at}");
        assert!(t.adb.runner.reverses.is_empty(), "step {fail_at}");
        assert_eq!(t.mode(), TetherMode::Off);
    }
}

#[test]
fn rollback_keeps_a_preexisting_wivrn_reverse() {
    let mut t = tether(Fake {
        fail_at: Some(6),
        ..Fake::default()
    });
    t.adb
        .runner
        .reverses
        .insert("tcp:9757".into(), "tcp:9757".into());
    assert!(t.connect(false).is_err());
    assert_eq!(t.adb.runner.calls.last().unwrap(), REMOVE);
    assert_eq!(t.adb.runner.reverses.len(), 1);
    assert_eq!(t.adb.runner.reverses["tcp:9757"], "tcp:9757");
}

#[test]
fn relay_start_failure_stops_the_client_then_removes_reverses() {
    let mut t = tether(Fake {
        fail_at: Some(9),
        ..Fake::default()
    });
    assert!(t.connect(false).is_err());
    assert_eq!(
        t.adb.runner.calls,
        [
            WIVRN,
            OWN,
            &[STOP, REMOVE, "/tools/adb -s S1 reverse --remove tcp:9757"]
        ]
        .concat()
    );
}
