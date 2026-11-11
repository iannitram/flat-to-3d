use flat3d_inject::{
    api::TetherMode,
    exec::{ProcessRunner, RelayRunner},
    testing::FakeRunner,
    tether::{
        adb::{choose_adb, CommandRunner},
        plan::{self, PlanCtx, PlanRun, TetherStep},
        relay, wifi, Tether,
    },
};

fn w(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_string()).collect()
}

#[test]
fn runs_each_plan_variant_in_order_and_records_argv() {
    let mut r = FakeRunner::new()
        .ok(&["adb", "shell", "ip"], "    inet 10.0.0.2/24\n")
        .ok(&["adb", "shell", "ip"], "")
        .ok(&["adb", "shell", "ip"], "")
        .ok(&["adb", "shell", "ip"], "    inet 10.0.0.2/24\n")
        .ok(
            &["adb", "shell", "cmd", "wifi"],
            "Wifi is connected to \"home\"\n",
        )
        .ok(&["ps"], "1 relay-helper run S1\n2 relay run S1\n")
        .ok(&["ps"], "3 relay run S1\n");
    let relay = w(&["relay", "run", "S1"]);
    let steps = vec![
        TetherStep::Run(w(&["first"])),
        TetherStep::SleepMs(4),
        TetherStep::IfVpnUp(w(&["vpn-up"])),
        TetherStep::IfNoTun0(vec![w(&["absent-1"]), w(&["absent-2"])]),
        TetherStep::WaitTun0 {
            tries: 2,
            every_ms: 9,
        },
        TetherStep::WaitWifi {
            tries: 1,
            every_ms: 3,
        },
        TetherStep::KillStaleRelay,
        TetherStep::SpawnRelay,
        TetherStep::CheckRelayAlive,
        TetherStep::KillRelay,
    ];
    let mut sleeps = Vec::new();
    let mut out = PlanRun::default();
    plan::run_plan_in(
        &steps,
        &PlanCtx {
            relay: relay.clone(),
        },
        &mut r,
        &mut |ms| sleeps.push(ms),
        &mut out,
    )
    .unwrap();
    assert_eq!(sleeps, [4, 9]);
    assert!(out.tun0);
    assert_eq!(out.ran[0], w(&["first"]));
    assert!(out.ran.contains(&w(&["kill", "2"])));
    assert_eq!(r.ran[0], w(&["first"]));
    assert!(r.ran.contains(&vec![
        "relay".into(),
        "relay".into(),
        "run".into(),
        "S1".into(),
        "".into()
    ]));
    assert_eq!(r.ran.last().unwrap(), &w(&["relay", "stop"]));
}

#[test]
fn wait_tun0_retries_and_sleeps_only_between_probes() {
    let mut r = FakeRunner::new()
        .ok(&["adb", "shell", "ip"], "")
        .ok(&["adb", "shell", "ip"], "")
        .ok(&["adb", "shell", "ip"], "    inet 10.0.0.2/24\n");
    let mut sleeps = Vec::new();
    let mut out = PlanRun::default();
    plan::run_plan_in(
        &[TetherStep::WaitTun0 {
            tries: 5,
            every_ms: 1500,
        }],
        &PlanCtx::default(),
        &mut r,
        &mut |ms| sleeps.push(ms),
        &mut out,
    )
    .unwrap();
    assert_eq!(r.ran.len(), 3);
    assert_eq!(sleeps, [1500, 1500]);
    assert!(out.tun0);

    let mut r = FakeRunner::new().ok(&["adb", "shell", "ip"], "");
    let mut sleeps = Vec::new();
    let mut out = PlanRun::default();
    plan::run_plan_in(
        &[TetherStep::WaitTun0 {
            tries: 5,
            every_ms: 1500,
        }],
        &PlanCtx::default(),
        &mut r,
        &mut |ms| sleeps.push(ms),
        &mut out,
    )
    .unwrap();
    assert_eq!(r.ran.len(), 5);
    assert_eq!(sleeps, [1500, 1500, 1500, 1500]);
    assert!(!out.tun0);
}

#[test]
fn if_no_tun0_runs_only_when_probe_is_absent() {
    let commands = vec![w(&["run-me"])];
    let mut present = FakeRunner::new().ok(&["adb", "shell", "ip"], "inet 10.0.0.2/24");
    plan::run_plan(
        &[TetherStep::IfNoTun0(commands.clone())],
        &mut present,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(present.ran.len(), 1);
    let mut absent = FakeRunner::new().ok(&["adb", "shell", "ip"], "");
    plan::run_plan(&[TetherStep::IfNoTun0(commands)], &mut absent, &mut |_| {}).unwrap();
    assert_eq!(absent.ran.len(), 2);
}

#[test]
fn detects_missing_relay_and_kills_only_matching_stale_processes() {
    let mut r = FakeRunner::new().ok(&["ps"], "4 relay other\n");
    let ctx = PlanCtx {
        relay: w(&["relay", "run", "S1"]),
    };
    assert_eq!(
        plan::run_plan_in(
            &[TetherStep::CheckRelayAlive],
            &ctx,
            &mut r,
            &mut |_| {},
            &mut PlanRun::default()
        ),
        Err("Tether relay exited during startup".into())
    );
    let mut r = FakeRunner::new().ok(
        &["ps"],
        "1 relay-helper run S1\n2 relay run S1\n3 relay run S2\n",
    );
    plan::run_plan_in(
        &[TetherStep::KillStaleRelay],
        &ctx,
        &mut r,
        &mut |_| {},
        &mut PlanRun::default(),
    )
    .unwrap();
    assert!(r.ran.contains(&w(&["kill", "2"])));
    assert!(!r.ran.contains(&w(&["kill", "1"])));
}

fn tether(runner: FakeRunner) -> Tether<FakeRunner> {
    let mut t = Tether::new(
        "/fixture",
        "S1",
        choose_adb(Some("/tools/adb"), None),
        runner,
    );
    t.set_sleep(Box::new(|_| {}));
    t
}

#[test]
fn connect_installs_apk_only_when_package_is_missing() {
    let base = FakeRunner::new()
        .ok(
            &["/tools/adb", "-s", "S1", "devices"],
            "List of devices attached\nS1\tdevice\n",
        )
        .ok(
            &["/tools/adb", "-s", "S1", "shell", "pm"],
            "package:com.genymobile.gnirehtet\n",
        )
        .ok(&["ps"], "")
        .ok(
            &["ps"],
            "4242 /fixture/.local/share/flat3d/bin/gnirehtet run S1\n",
        )
        .ok(
            &["/tools/adb", "-s", "S1", "shell", "ip"],
            "inet 10.0.0.2/24",
        );
    let mut t = tether(base);
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
    assert!(!t
        .adb
        .runner
        .ran
        .iter()
        .any(|argv| argv.get(3).map(String::as_str) == Some("install")));

    let base = FakeRunner::new()
        .ok(
            &["/tools/adb", "-s", "S1", "devices"],
            "List of devices attached\nS1\tdevice\n",
        )
        .ok(&["/tools/adb", "-s", "S1", "shell", "pm"], "")
        .ok(&["ps"], "")
        .ok(
            &["ps"],
            "4242 /fixture/.local/share/flat3d/bin/gnirehtet run S1\n",
        )
        .ok(
            &["/tools/adb", "-s", "S1", "shell", "ip"],
            "inet 10.0.0.2/24",
        );
    let mut t = tether(base);
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
    let install = t
        .adb
        .runner
        .ran
        .iter()
        .find(|argv| argv.get(3).map(String::as_str) == Some("install"))
        .unwrap();
    assert_eq!(install.last().unwrap(), &relay::apk_path("/fixture"));
}

#[test]
fn process_runner_errors_include_exit_and_stderr_tail() {
    let mut runner = ProcessRunner::default();
    assert_eq!(
        runner.run(&w(&[
            "sh",
            "-c",
            "printf 'x\\ny\\na\\nb\\nc\\nd\\n' >&2; exit 1"
        ])),
        Err("Tether command failed: exit 1; a / b / c / d".into())
    );
    assert_eq!(
        runner.run(&w(&["sh", "-c", "exit 1"])),
        Err("Tether command failed: exit 1".into())
    );
    assert!(runner
        .run(&w(&["program-that-does-not-exist-wp10"]))
        .unwrap_err()
        .starts_with("Could not run tether command: "));
}

#[test]
fn sync_wifi_redacts_psk_and_gives_up_after_three_failed_attempts() {
    let mut runner = FakeRunner::new();
    for _ in 0..4 {
        runner = runner
            .ok(
                &["nmcli", "-t", "-f", "active,ssid", "dev", "wifi"],
                "yes:home\n",
            )
            .ok(
                &["/tools/adb", "-s", "S1", "shell", "cmd", "wifi", "status"],
                "Wifi is connected to \"other\"\n",
            );
    }
    for _ in 0..3 {
        runner = runner
            .ok(
                &[
                    "nmcli",
                    "-s",
                    "-g",
                    "802-11-wireless-security.key-mgmt,802-11-wireless-security.psk",
                    "connection",
                    "show",
                ],
                "wpa-psk\nsecret pass\n",
            )
            .fail(
                &[
                    "/tools/adb",
                    "-s",
                    "S1",
                    "shell",
                    "cmd wifi connect-network 'home' wpa2 'secret pass'",
                ],
                "no network",
            );
    }
    let mut t = tether(runner);
    for _ in 0..3 {
        assert_eq!(
            t.sync_wifi(),
            wifi::SyncDecision::Switch,
            "{:?}",
            t.adb.runner.ran
        );
    }
    assert_eq!(t.sync_wifi(), wifi::SyncDecision::GiveUp);
    assert_eq!(t.last_wifi(), Some(&wifi::SyncDecision::GiveUp));
    let argv = t
        .adb
        .runner
        .ran
        .iter()
        .find(|argv| argv.iter().any(|arg| arg.contains("connect-network")))
        .unwrap();
    let redacted = wifi::redact_command(argv);
    assert!(redacted.contains("home"));
    assert!(!redacted.contains("secret pass"));
    assert_eq!(
        t.adb
            .runner
            .ran
            .iter()
            .filter(|argv| argv.iter().any(|arg| arg.contains("connect-network")))
            .count(),
        3
    );
}

#[test]
fn bare_plan_requires_relay_context_for_relay_steps() {
    let mut r = FakeRunner::new();
    assert_eq!(
        plan::run_plan(&[TetherStep::SpawnRelay], &mut r, &mut |_| {}),
        Err("No relay command".into())
    );
    assert_eq!(
        plan::run_plan(&[TetherStep::CheckRelayAlive], &mut r, &mut |_| {}),
        Err("No relay command".into())
    );
}

#[test]
fn tether_runner_trait_alias_drives_connect() {
    #[derive(Default)]
    struct Fake {
        tun0: bool,
    }
    impl CommandRunner for Fake {
        fn run(&mut self, argv: &[String]) -> Result<String, String> {
            if argv.ends_with(&w(&["devices"])) {
                return Ok("List of devices attached\nS1\tdevice\n".into());
            }
            if argv.ends_with(&w(&["reverse", "--list"])) {
                return Ok(String::new());
            }
            if argv.first().map(String::as_str) == Some("ps") {
                return Ok(if self.tun0 {
                    "4242 /fixture/.local/share/flat3d/bin/gnirehtet run S1\n".into()
                } else {
                    String::new()
                });
            }
            if argv.ends_with(&w(&[
                "shell",
                "pm",
                "list",
                "packages",
                "com.genymobile.gnirehtet",
            ])) {
                return Ok("package:com.genymobile.gnirehtet\n".into());
            }
            if argv.ends_with(&w(&["shell", "ip", "addr", "show", "tun0"])) {
                return Ok("inet 10.0.0.2/24\n".into());
            }
            Ok(String::new())
        }
        fn start_relay(&mut self, _: &[String], adb: &str) -> Result<(), String> {
            assert_eq!(adb, "/tools/adb");
            self.tun0 = true;
            Ok(())
        }
        fn stop_relay(&mut self) -> Result<(), String> {
            self.tun0 = false;
            Ok(())
        }
    }
    let mut t = Tether::new(
        "/fixture",
        "S1",
        choose_adb(Some("/tools/adb"), None),
        Fake::default(),
    );
    t.set_sleep(Box::new(|_| {}));
    assert_eq!(t.connect(false), Ok(TetherMode::Own));
}

#[test]
fn set_wivrn_uses_the_frozen_command_builder() {
    let mut t = tether(FakeRunner::new().ok(
        &["/tools/adb", "-s", "S1", "devices"],
        "List of devices attached\nS1\tdevice\n",
    ));
    t.set_wivrn("app.id".into(), vec!["--flag".into()]);
    let expected = flat3d_inject::link::start_3d_commands_with(&flat3d_inject::link::WivrnStart {
        scope: None,
        app: "app.id".into(),
        flags: vec!["--flag".into()],
    });
    assert_eq!(t.connect(true), Ok(TetherMode::QuestTether));
    let mut rewritten = vec![w(&["/tools/adb", "-s", "S1", "reverse", "--list"])];
    for command in expected {
        if command.first().map(String::as_str) == Some("adb") {
            let mut argv = w(&["/tools/adb", "-s", "S1"]);
            argv.extend_from_slice(&command[1..]);
            rewritten.push(argv);
        } else {
            rewritten.push(command);
        }
    }
    assert_eq!(t.adb.runner.ran, rewritten);
}
