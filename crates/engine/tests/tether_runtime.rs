use flat3d_inject::api::TetherMode;
use flat3d_inject::exec::RelayRunner as CommandRunner;
use flat3d_inject::link::adb::choose_adb;
use flat3d_inject::tether::Tether;
use std::collections::BTreeMap;

#[derive(Default)]
struct Fake {
    calls: Vec<String>,
    fail_at: Option<usize>,
    reverses: BTreeMap<String, String>,
    bare_list: bool,
    relay_started: bool,
    tun0: bool,
    package_installed: bool,
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
        if words[0] == "ps" {
            return Ok(if self.relay_started {
                "4242 /fixture/.local/share/flat3d/bin/gnirehtet run S1\n".into()
            } else {
                String::new()
            });
        }
        if words[0] == "/tools/adb" {
            assert_eq!(&words[1..3], ["-s", "S1"]);
            let args = &words[3..];
            if args == ["devices"] {
                return Ok("List of devices attached\nS1\tdevice\n".into());
            }
            if args == ["reverse", "--list"] {
                return Ok(self
                    .reverses
                    .iter()
                    .map(|(from, to)| {
                        if self.bare_list {
                            format!("{from} {to}\n")
                        } else {
                            format!("S1 {from} {to}\n")
                        }
                    })
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
            if args
                == [
                    "shell",
                    "pm",
                    "list",
                    "packages",
                    "com.genymobile.gnirehtet",
                ]
            {
                return Ok(if self.package_installed {
                    "package:com.genymobile.gnirehtet\n"
                } else {
                    ""
                }
                .into());
            }
            if args == ["shell", "ip", "addr", "show", "tun0"] {
                return Ok(if self.tun0 {
                    "    inet 10.0.0.2/24\n"
                } else {
                    ""
                }
                .into());
            }
            if args == ["shell", "cmd", "wifi", "status"] {
                return Ok("Wifi is connected to \"home\"\n".into());
            }
            if args.first().map(String::as_str) == Some("install") {
                return Ok("Success\n".into());
            }
            if args.last().map(String::as_str) == Some("com.genymobile.gnirehtet.STOP") {
                self.tun0 = false;
            }
        }
        Ok(String::new())
    }

    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String> {
        assert_eq!(adb, "/tools/adb");
        self.record(command)?;
        self.relay_started = true;
        self.tun0 = true;
        Ok(())
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        self.record(&["stop-owned-relay".into()])?;
        self.relay_started = false;
        Ok(())
    }
}

fn tether(mut fake: Fake) -> Tether<Fake> {
    fake.tun0 = true;
    let mut t = Tether::new("/fixture", "S1", choose_adb(Some("/tools/adb"), None), fake);
    t.set_sleep(Box::new(|_| {}));
    t
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
    "/tools/adb -s S1 shell pm list packages com.genymobile.gnirehtet",
    "/tools/adb -s S1 install -r /fixture/.local/share/flat3d/bin/gnirehtet.apk",
    "ps -eo pid=,args=",
    "/fixture/.local/share/flat3d/bin/gnirehtet run S1",
    "ps -eo pid=,args=",
    "/tools/adb -s S1 shell ip addr show tun0",
    "/tools/adb -s S1 shell ip addr show tun0",
    "/tools/adb -s S1 shell ip addr show tun0",
];
const REMOVE: &str = "/tools/adb -s S1 reverse --remove localabstract:gnirehtet";
const STOP: &str = "/tools/adb -s S1 shell am start -n com.genymobile.gnirehtet/.GnirehtetActivity -a com.genymobile.gnirehtet.STOP";

#[test]
fn connect_and_disconnect_run_plans_and_preserve_wivrn() {
    let mut t = tether(Fake::default());
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
    assert_eq!(t.adb.runner.calls, [WIVRN, OWN].concat());
    t.adb.runner.calls.clear();
    assert_eq!(t.disconnect(), Ok(()));
    assert_eq!(
        t.adb.runner.calls,
        [
            STOP,
            "/tools/adb -s S1 shell ip addr show tun0",
            "/tools/adb -s S1 shell svc wifi disable",
            "/tools/adb -s S1 shell svc wifi enable",
            "/tools/adb -s S1 shell cmd wifi status",
            "stop-owned-relay",
            REMOVE,
        ]
    );
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
fn existing_package_skips_apk_install() {
    let mut t = tether(Fake {
        package_installed: true,
        ..Fake::default()
    });
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
    assert!(!t
        .adb
        .runner
        .calls
        .iter()
        .any(|call| call.contains(" install -r ")));
}

#[test]
fn failed_install_cleans_only_reverses_added_by_connect() {
    let mut t = tether(Fake {
        fail_at: Some(8),
        ..Fake::default()
    });
    assert_eq!(
        t.connect(false),
        Err("The requested tether step failed".into())
    );
    assert_eq!(
        t.adb.runner.calls,
        [
            WIVRN,
            &OWN[..3],
            &[REMOVE, "/tools/adb -s S1 reverse --remove tcp:9757"],
        ]
        .concat()
    );
    assert!(t.adb.runner.reverses.is_empty());
    assert_eq!(t.mode(), TetherMode::Off);
}

#[test]
fn rollback_keeps_a_preexisting_wivrn_reverse() {
    let mut t = tether(Fake {
        fail_at: Some(8),
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
fn preexisting_wivrn_reverse_is_kept_when_adb_lists_without_serial() {
    let mut t = tether(Fake {
        bare_list: true,
        fail_at: Some(8),
        ..Fake::default()
    });
    t.adb
        .runner
        .reverses
        .insert("tcp:9757".into(), "tcp:9757".into());
    assert!(t.connect(false).is_err());
    assert_eq!(t.adb.runner.calls.last().unwrap(), REMOVE);
    assert_eq!(t.adb.runner.reverses["tcp:9757"], "tcp:9757");
}

#[test]
fn every_failing_connect_step_rolls_back() {
    for fail_at in 1..=11 {
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
fn missing_adb_has_an_install_hint() {
    let mut t = Tether::new("/fixture", "S1", choose_adb(None, None), Fake::default());
    let error = t.connect(false).unwrap_err();
    assert!(
        error.contains("the bundled adb is not at /fixture/.local/share/flat3d/bin/adb"),
        "{error}"
    );
    assert!(error.contains("fix: flat3d-inject install"));
    assert!(t.adb.runner.calls.is_empty());
    assert_eq!(t.mode(), TetherMode::Off);
}

#[test]
fn tunnel_probe_failure_is_tolerated_and_relay_start_failure_never_stops_client() {
    let mut t = tether(Fake {
        fail_at: Some(12),
        ..Fake::default()
    });
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
    let mut t = tether(Fake {
        fail_at: Some(10),
        ..Fake::default()
    });
    assert_eq!(
        t.connect(false),
        Err("The requested tether step failed".into())
    );
    assert_eq!(
        t.adb.runner.calls,
        [
            WIVRN,
            &OWN[..5],
            &[REMOVE, "/tools/adb -s S1 reverse --remove tcp:9757"],
        ]
        .concat()
    );
    assert!(!t.adb.runner.calls.iter().any(|call| call == STOP));
}
