use flat3d_inject::{
    api::ApiError,
    install::{self, UnitSpec},
    testing::FakeRunner,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixture(name: &str, omit: &[&str]) -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("install_execute")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let home = root.join("home");
    let src = root.join("src");
    fs::create_dir_all(&home).unwrap();
    for (path, data) in [
        ("flat3d-inject", "inject"),
        ("flat3d-run", "run"),
        ("flat3d-viewer", "viewer"),
        ("tools/adb", "adb"),
        ("tools/lib/libusb.so", "lib"),
        ("recipes/1-a.recipe", "recipe"),
        ("recipes/notes.txt", "notes"),
    ] {
        if omit.contains(&path) {
            continue;
        }
        let file = src.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, data).unwrap();
    }
    (root, home, src)
}
fn plan(home: &Path, src: &Path) -> install::InstallPlan {
    let mut plan = install::install_plan(home.to_str().unwrap(), src.to_str().unwrap());
    // The real rule path is /etc/udev/rules.d; point the skip check at the fixture
    // so the result does not depend on the machine running the tests.
    plan.udev_rule = home
        .parent()
        .unwrap()
        .join("etc/udev/rules.d/70-flat3d-quest.rules")
        .display()
        .to_string();
    plan
}
fn udev_rule_in_place(p: &install::InstallPlan) {
    let rule = PathBuf::from(&p.udev_rule);
    fs::create_dir_all(rule.parent().unwrap()).unwrap();
    fs::write(rule, install::UDEV_RULE_TEXT).unwrap();
}
fn err(result: Result<flat3d_inject::api::InstallReport, ApiError>, code: &str) -> ApiError {
    let error = result.unwrap_err();
    assert_eq!(error.code, code);
    error
}
fn mode(path: PathBuf) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn default_unit_text_is_exactly_this() {
    assert_eq!(install::unit_text_with(&UnitSpec::default()), "[Unit]\nDescription=Flat to 3D headset link\nStartLimitIntervalSec=60\nStartLimitBurst=10\n\n[Service]\nType=simple\nExecStart=%h/.local/bin/flat3d-inject watch\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n");
}
#[test]
fn packaging_unit_file_equals_unit_text_with_default() {
    assert_eq!(
        include_str!("../../../packaging/flat3d-link.service"),
        install::unit_text_with(&UnitSpec::default())
    );
}
#[test]
fn custom_exec_start() {
    let spec = UnitSpec {
        exec_start: "/x/y watch".into(),
        start_limit_burst: 5,
        ..UnitSpec::default()
    };
    let text = install::unit_text_with(&spec);
    assert!(text.contains("ExecStart=/x/y watch\n"));
    assert!(text.contains("StartLimitBurst=5\n"));
}
#[test]
fn unit_exec_start_parses() {
    let args = UnitSpec::default()
        .exec_start
        .split_whitespace()
        .skip(1)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let parsed = flat3d_inject::cli::parse_any(&args).unwrap();
    assert!(matches!(
        parsed.command,
        flat3d_inject::cli::AnyCommand::Link(flat3d_inject::cli_link::LinkCommand::Watch)
    ));
}

#[test]
fn execute_copies_chmods_writes_unit_then_reloads_and_enables() {
    let (_, home, src) = fixture("copies", &[]);
    let p = plan(&home, &src);
    let mut runner = FakeRunner::new();
    let report = install::execute(&p, &mut runner).unwrap();
    let mut expected = vec![
        vec!["systemctl", "--user", "daemon-reload"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>(),
        vec![
            "systemctl",
            "--user",
            "enable",
            "--now",
            "flat3d-link.service",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    ];
    expected.extend(p.udev_commands.clone());
    assert_eq!(runner.ran, expected);
    assert_eq!(report.summary, "");
    assert_eq!(mode(home.join(".local/bin/flat3d-run")), 0o755);
    assert_eq!(mode(home.join(".local/bin/flat3d-viewer")), 0o755);
    assert_eq!(mode(home.join(".local/share/flat3d/bin/adb")), 0o755);
    assert_eq!(
        fs::read_to_string(home.join(".local/share/flat3d/flat3d-quest.rules")).unwrap(),
        include_str!("../../../packaging/flat3d-quest.rules")
    );
    assert_eq!(mode(home.join(".local/share/flat3d/flat3d-quest.rules")), 0o644);
    assert_eq!(
        fs::read_to_string(home.join(".config/systemd/user/flat3d-link.service")).unwrap(),
        install::unit_text_with(&UnitSpec::default())
    );
    assert!(home.join(".local/share/flat3d/bin/lib/libusb.so").exists());
    assert!(home.join(".local/share/flat3d/recipes/1-a.recipe").exists());
    assert!(!home.join(".local/share/flat3d/recipes/notes.txt").exists());
    let manifest = fs::read_to_string(home.join(".local/share/flat3d/installed.txt")).unwrap();
    for path in report.written.iter().filter(|path| *path != &p.manifest) {
        assert!(manifest.lines().any(|line| line == path));
    }
}
#[test]
fn missing_tools_are_skipped_and_named() {
    let (_, home, src) = fixture("missing-tools", &["tools/adb"]);
    let report = install::execute(&plan(&home, &src), &mut FakeRunner::new()).unwrap();
    assert!(report.summary.contains("not bundled: adb"));
}
#[test]
fn missing_tools_are_named_in_plan_order() {
    let (_, home, src) = fixture("missing-order", &["tools/adb", "flat3d-viewer"]);
    let report = install::execute(&plan(&home, &src), &mut FakeRunner::new()).unwrap();
    assert_eq!(report.summary, "not bundled: adb, flat3d-viewer");
}
#[test]
fn udev_rule_already_in_place_is_skipped() {
    let (_, home, src) = fixture("udev-present", &[]);
    let p = plan(&home, &src);
    udev_rule_in_place(&p);
    let mut runner = FakeRunner::new();
    let report = install::execute(&p, &mut runner).unwrap();
    assert_eq!(runner.ran.len(), 2);
    assert!(runner.ran.iter().all(|argv| argv[0] == "systemctl"));
    assert_eq!(report.summary, "");
}
#[test]
fn refused_sudo_prompt_is_a_setup_warning_not_an_install_failure() {
    let (_, home, src) = fixture("udev-refused", &["tools/adb"]);
    let p = plan(&home, &src);
    let mut runner = FakeRunner::new().fail(&["sudo"], "sudo: a password is required
");
    let report = install::execute(&p, &mut runner).unwrap();
    assert_eq!(runner.ran.last().unwrap(), &p.udev_commands[0]);
    assert!(runner.ran.contains(&vec![
        "systemctl".to_string(),
        "--user".into(),
        "enable".into(),
        "--now".into(),
        "flat3d-link.service".into()
    ]));
    assert!(report.summary.starts_with("not bundled: adb; warning: "));
    assert!(report
        .summary
        .contains("the Quest udev rule is not installed: sudo: a password is required; fix: "));
    assert!(PathBuf::from(&p.unit_path).exists());
}
#[test]
fn uninstall_removes_the_udev_rule_only_when_it_is_there() {
    let (_, home, src) = fixture("udev-uninstall", &[]);
    let p = plan(&home, &src);
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    let mut u = install::uninstall_plan(home.to_str().unwrap());
    u.udev_rule = p.udev_rule.clone();
    let mut absent = FakeRunner::new();
    install::execute_uninstall(&u, &mut absent).unwrap();
    assert!(absent.ran.iter().all(|argv| argv[0] == "systemctl"));
    assert!(!home.join(".local/share/flat3d/flat3d-quest.rules").exists());

    udev_rule_in_place(&p);
    let mut present = FakeRunner::new();
    install::execute_uninstall(&u, &mut present).unwrap();
    assert_eq!(present.ran[1], u.udev_commands[0]);
    assert_eq!(
        present.ran.last().unwrap(),
        &vec!["systemctl", "--user", "daemon-reload"]
    );

    let mut refused = FakeRunner::new().fail(&["sudo"], "sudo: a password is required
");
    let report = install::execute_uninstall(&u, &mut refused).unwrap();
    assert!(report
        .summary
        .contains("tolerated sudo sh -c rm -f"));
    assert!(report.summary.contains(": sudo: a password is required; fix: "));
}
#[test]
fn reinstall_keeps_earlier_manifest_entries_and_leaves_no_tmp() {
    let (_, home, src) = fixture("manifest-union", &[]);
    let p = plan(&home, &src);
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    fs::remove_file(src.join("tools/adb")).unwrap();
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    let listed = fs::read_to_string(&p.manifest).unwrap();
    assert!(listed.contains("/adb\n"));
    let lines: Vec<&str> = listed.lines().collect();
    let mut dedup = lines.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(lines.len(), dedup.len());
    let bin = home.join(".local/bin");
    assert!(fs::read_dir(bin).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
}
#[test]
fn dirs_outside_the_roots_are_refused() {
    let (_, home, src) = fixture("bad-dir", &[]);
    let mut p = plan(&home, &src);
    p.dirs.push(format!("{}/.config/flat3d", home.display()));
    let mut runner = FakeRunner::new();
    err(install::execute(&p, &mut runner), "refused");
    assert!(runner.ran.is_empty());
}
#[test]
fn file_destination_equal_to_a_root_without_slash_is_refused() {
    for which in ["unit", "copy"] {
        let (_, home, src) = fixture(&format!("root-no-slash-{which}"), &[]);
        let mut p = plan(&home, &src);
        let h = home.display();
        if which == "unit" {
            p.unit_path = format!("{h}/.config/systemd/user");
        } else {
            p.copies[0].1 = format!("{h}/.local/bin");
        }
        let mut runner = FakeRunner::new();
        err(install::execute(&p, &mut runner), "refused");
        assert!(runner.ran.is_empty());
        assert!(!home.join(".config").exists());
        assert!(!home.join(".local").exists());
        assert!(!home.join(".config/systemd/user.tmp").exists());
        assert!(!home.join(".local/bin.tmp").exists());
    }
}
#[test]
fn missing_binary_is_io_and_runs_nothing() {
    let (_, home, src) = fixture("missing-bin", &["flat3d-run"]);
    let mut runner = FakeRunner::new();
    let e = err(install::execute(&plan(&home, &src), &mut runner), "io");
    assert!(e
        .message
        .contains("; fix: run flat3d-inject install from the release folder"));
    assert!(runner.ran.is_empty());
    assert!(!home
        .join(".config/systemd/user/flat3d-link.service")
        .exists());
}
#[test]
fn systemctl_failure_is_command_failed() {
    let (_, home, src) = fixture("command-failure", &[]);
    let p = plan(&home, &src);
    let mut runner = FakeRunner::new().fail(&["systemctl", "--user", "enable"], "boom\n");
    let e = err(install::execute(&p, &mut runner), "command-failed");
    assert!(e.message.contains(": boom; fix: "));
    assert!(PathBuf::from(&p.unit_path).exists());
    assert!(PathBuf::from(&p.manifest).exists());
}
#[test]
fn failed_install_still_writes_manifest_and_uninstall_cleans_up() {
    let (_, home, src) = fixture("partial", &[]);
    let p = plan(&home, &src);
    let bad = home.join(".local/share/flat3d/recipes/1-a.recipe");
    fs::create_dir_all(&bad).unwrap();
    err(install::execute(&p, &mut FakeRunner::new()), "io");
    let manifest = PathBuf::from(&p.manifest);
    assert!(manifest.exists());
    let listed = fs::read_to_string(&manifest).unwrap();
    assert!(listed.contains("flat3d-inject"));
    assert!(listed.contains("adb"));
    install::execute_uninstall(
        &install::uninstall_plan(home.to_str().unwrap()),
        &mut FakeRunner::new(),
    )
    .unwrap();
    for item in listed.lines() {
        assert!(!PathBuf::from(item).exists());
    }
    assert!(!manifest.exists());
}
#[test]
fn plan_outside_the_roots_is_refused() {
    let (_, home, src) = fixture("outside", &[]);
    let mut p = plan(&home, &src);
    p.unit_path = format!("{}/.config/flat3d/x.service", home.display());
    let mut runner = FakeRunner::new();
    err(install::execute(&p, &mut runner), "refused");
    assert!(runner.ran.is_empty());
    assert!(!home.join(".local/bin/flat3d-run").exists());
}
#[test]
fn execute_twice_is_fine() {
    let (_, home, src) = fixture("twice", &[]);
    let p = plan(&home, &src);
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    let first = fs::read_to_string(&p.manifest).unwrap();
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    assert_eq!(fs::read_to_string(&p.manifest).unwrap(), first);
}
#[test]
fn uninstall_tolerates_missing_unit_and_removes_manifest_files() {
    let (_, home, src) = fixture("uninstall", &[]);
    let p = plan(&home, &src);
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    let installed: Vec<String> = fs::read_to_string(&p.manifest)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert!(!installed.is_empty());
    let mut r = FakeRunner::new().fail(&["systemctl", "--user", "disable"], "boom\n");
    let report =
        install::execute_uninstall(&install::uninstall_plan(home.to_str().unwrap()), &mut r)
            .unwrap();
    assert!(report
        .summary
        .contains("tolerated systemctl --user disable --now flat3d-link.service: boom"));
    assert_eq!(
        r.ran.last().unwrap(),
        &vec!["systemctl", "--user", "daemon-reload"]
    );
    assert!(!PathBuf::from(&p.manifest).exists());
    assert!(!PathBuf::from(&p.unit_path).exists());
    for listed in &installed {
        assert!(!PathBuf::from(listed).exists(), "{listed}");
    }
}
#[test]
fn uninstall_keeps_config_and_state() {
    let (_, home, src) = fixture("preserve", &[]);
    let p = plan(&home, &src);
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    let config = home.join(".config/flat3d/settings");
    let state = home.join(".local/state/flat3d/x");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::create_dir_all(state.parent().unwrap()).unwrap();
    fs::write(&config, "x").unwrap();
    fs::write(&state, "x").unwrap();
    let mut manifest = fs::read_to_string(&p.manifest).unwrap();
    manifest.push_str(&format!(
        "{}\n{}\n{}/.local/share/flat3d/../../state/flat3d/x\n",
        config.display(),
        state.display(),
        home.display()
    ));
    fs::write(&p.manifest, manifest).unwrap();
    let report = install::execute_uninstall(
        &install::uninstall_plan(home.to_str().unwrap()),
        &mut FakeRunner::new(),
    )
    .unwrap();
    assert!(config.exists() && state.exists());
    assert!(report
        .summary
        .contains(&format!("refused {}", config.display())));
    assert!(report
        .summary
        .contains(&format!("refused {}", state.display())));
}
#[test]
fn nothing_mentions_linger() {
    let (_, home, src) = fixture("linger", &[]);
    let p = plan(&home, &src);
    let mut r = FakeRunner::new();
    install::execute(&p, &mut r).unwrap();
    install::execute_uninstall(&install::uninstall_plan(home.to_str().unwrap()), &mut r).unwrap();
    assert!(r
        .ran
        .iter()
        .flatten()
        .all(|a| a != "loginctl" && a != "enable-linger"));
    assert!(!install::unit_text_with(&UnitSpec::default()).contains("Linger"));
}
