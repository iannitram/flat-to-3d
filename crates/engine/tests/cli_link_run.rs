use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::cli::{self, AnyCommand, Command, SideEffect};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::state::{self, Request};
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::{
    api::ERROR_CODES,
    cli::link::{self as cli_link, LinkCommand, SettingKey},
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn setup(name: &str) -> (PathBuf, EnginePaths) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cli_link_run")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    let mut env = BTreeMap::new();
    env.insert(
        "HOME".into(),
        root.join("home").to_string_lossy().into_owned(),
    );
    env.insert(
        "XDG_RUNTIME_DIR".into(),
        root.join("run").to_string_lossy().into_owned(),
    );
    let paths = EnginePaths::from_env(&env, 1000).unwrap();
    std::fs::create_dir_all(&paths.home).unwrap();
    (root, paths)
}

fn save_snapshot(paths: &EnginePaths, updated: u64) {
    std::fs::create_dir_all(&paths.state_dir).unwrap();
    let mut snapshot = LinkSnapshot::stopped(updated);
    snapshot.state = LinkState::Ready;
    std::fs::write(
        state::snapshot_path(&paths.state_dir.to_string_lossy()),
        serde_json::to_string(&snapshot).unwrap(),
    )
    .unwrap();
}

fn run(
    cmd: &LinkCommand,
    paths: &EnginePaths,
    runner: &mut FakeRunner,
    process: &mut FakeRunner,
) -> serde_json::Value {
    let mut ctx = cli::Ctx {
        paths,
        runner,
        process,
        json: false,
    };
    cli_link::run(cmd, &mut ctx).unwrap()
}

#[test]
fn request_3d_refuses_missing_and_stale_snapshots_without_writing() {
    let (_root, paths) = setup("missing-stale");
    let missing = cli_link::request_3d(Request::Start3d, &paths, 1005).unwrap_err();
    assert_eq!(missing.code, "link-not-ready");
    assert!(missing
        .message
        .contains("systemctl --user start flat3d-link.service"));
    assert!(!Path::new(&state::request_path(&paths.state_dir.to_string_lossy())).exists());

    save_snapshot(&paths, 1000);
    let stale = cli_link::request_3d(Request::Start3d, &paths, 1011).unwrap_err();
    assert_eq!(stale.code, "link-not-ready");
    assert!(stale.message.contains("11 s ago"));
    assert!(stale
        .message
        .contains("systemctl --user restart flat3d-link.service"));
    assert_ne!(missing.message, stale.message);
    assert!(!Path::new(&state::request_path(&paths.state_dir.to_string_lossy())).exists());
}

#[test]
fn request_3d_writes_start_then_stop_only_for_a_fresh_snapshot() {
    let (_root, paths) = setup("fresh-request");
    save_snapshot(&paths, 1000);
    assert_eq!(
        cli_link::request_3d(Request::Start3d, &paths, 1005).unwrap(),
        flat3d_inject::api::LinkReport {
            state: LinkState::Ready,
            message: "Requested start-3d.".into(),
        }
    );
    let request = state::request_path(&paths.state_dir.to_string_lossy());
    assert_eq!(
        std::fs::read_to_string(&request).unwrap().trim(),
        "start-3d"
    );
    cli_link::request_3d(Request::Stop3d, &paths, 1005).unwrap();
    assert_eq!(std::fs::read_to_string(&request).unwrap().trim(), "stop-3d");
}

#[test]
fn switches_and_settings_are_read_and_written_through_the_settings_file() {
    let (_root, paths) = setup("settings");
    let mut runner = FakeRunner::new();
    let mut process = FakeRunner::new();
    assert_eq!(
        run(
            &LinkCommand::SwitchSet {
                appid: 5,
                on: false
            },
            &paths,
            &mut runner,
            &mut process
        ),
        serde_json::json!({"appid":5,"on":false})
    );
    assert!(std::fs::read_to_string(paths.settings())
        .unwrap()
        .starts_with("flat3d-settings 1\n"));
    assert_eq!(
        run(
            &LinkCommand::SwitchGet { appid: 5 },
            &paths,
            &mut runner,
            &mut process
        ),
        serde_json::json!({"appid":5,"on":false})
    );
    assert_eq!(
        run(
            &LinkCommand::SettingSet {
                key: SettingKey::Auto3d,
                on: false
            },
            &paths,
            &mut runner,
            &mut process
        ),
        serde_json::json!({"key":"auto-3d","on":false})
    );
    // `tunnel` is retired (#223): setting it answers off and leaves the file alone.
    let before = std::fs::read_to_string(paths.settings()).unwrap();
    assert_eq!(
        run(
            &LinkCommand::SettingSet {
                key: SettingKey::Tunnel,
                on: true
            },
            &paths,
            &mut runner,
            &mut process
        ),
        serde_json::json!({"key":"tunnel","on":false})
    );
    assert_eq!(std::fs::read_to_string(paths.settings()).unwrap(), before);
    assert_eq!(
        run(
            &LinkCommand::SettingGet {
                key: SettingKey::Auto3d
            },
            &paths,
            &mut runner,
            &mut process
        ),
        serde_json::json!({"key":"auto-3d","on":false})
    );
    assert_eq!(
        run(
            &LinkCommand::SettingGet {
                key: SettingKey::Auto3d
            },
            &paths,
            &mut runner,
            &mut process
        ),
        serde_json::json!({"key":"auto-3d","on":false})
    );
}

#[test]
fn snapshot_returns_stopped_when_absent_and_the_saved_state_when_present() {
    let (_root, paths) = setup("snapshot");
    let mut runner = FakeRunner::new();
    let mut process = FakeRunner::new();
    assert_eq!(
        run(&LinkCommand::Snapshot, &paths, &mut runner, &mut process)["state"],
        "stopped"
    );
    save_snapshot(&paths, flat3d_inject::sys::unix_now());
    assert_eq!(
        run(&LinkCommand::Snapshot, &paths, &mut runner, &mut process)["state"],
        "ready"
    );
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let child = entry.path();
            if child.is_dir() {
                visit(root, &child, result);
            } else {
                result.insert(
                    child.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(child).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut result);
    }
    result
}

#[test]
fn read_verbs_leave_the_file_tree_and_runners_unchanged() {
    let (root, paths) = setup("read-only");
    std::fs::create_dir_all(&paths.config).unwrap();
    std::fs::write(
        paths.settings(),
        "flat3d-settings 1\nauto-3d\ton\ntunnel\ton\n",
    )
    .unwrap();
    save_snapshot(&paths, flat3d_inject::sys::unix_now());
    let before = tree(&root);
    let mut runner = FakeRunner::new();
    let mut process = FakeRunner::new();
    for cmd in [
        LinkCommand::Snapshot,
        LinkCommand::SwitchGet { appid: 5 },
        LinkCommand::SettingGet {
            key: SettingKey::Auto3d,
        },
    ] {
        run(&cmd, &paths, &mut runner, &mut process);
    }
    assert_eq!(tree(&root), before);
    assert!(runner.ran.is_empty());
    assert!(process.ran.is_empty());

    let (_root, empty_paths) = setup("read-only-empty");
    let before = tree(empty_paths.home.parent().unwrap());
    for cmd in [
        LinkCommand::Snapshot,
        LinkCommand::SwitchGet { appid: 5 },
        LinkCommand::SettingGet {
            key: SettingKey::Auto3d,
        },
    ] {
        run(&cmd, &empty_paths, &mut runner, &mut process);
    }
    assert_eq!(tree(empty_paths.home.parent().unwrap()), before);
    assert!(runner.ran.is_empty());
    assert!(process.ran.is_empty());
}

#[test]
fn install_error_is_a_known_api_error_and_does_not_enable_the_service() {
    let (_root, paths) = setup("install-fails");
    let mut runner = FakeRunner::new();
    let mut process = FakeRunner::new();
    {
        let mut ctx = cli::Ctx {
            paths: &paths,
            runner: &mut runner,
            process: &mut process,
            json: false,
        };
        let error = cli_link::run(&LinkCommand::Install, &mut ctx).unwrap_err();
        assert!(ERROR_CODES.contains(&error.code.as_str()));
    }
    assert!(!runner
        .ran
        .iter()
        .any(|argv| argv.windows(2).any(|w| w == ["enable", "--now"])));
}

#[test]
fn bad_link_arguments_keep_the_exact_usage_errors_and_exit_two() {
    for (args, message) in [
        (
            vec!["switch", "set", "5", "maybe", "--json"],
            "usage: flat3d-inject switch set <appid> on|off",
        ),
        (
            vec!["setting", "get", "volume", "--json"],
            "usage: flat3d-inject setting get <name>",
        ),
        (
            vec!["run", "5", "game", "--json"],
            "usage: flat3d-inject run <appid> -- <command words...>",
        ),
    ] {
        let args: Vec<String> = args.into_iter().map(str::to_string).collect();
        let output = cli::execute(&args);
        assert_eq!(output.code, 2);
        let response: serde_json::Value = serde_json::from_str(output.stdout.trim()).unwrap();
        assert_eq!(response["error"]["code"], "usage");
        assert_eq!(response["error"]["message"], message);
    }
}

#[test]
fn link_command_registry_matches_parsing_variants_and_effects_in_both_directions() {
    let cases: &[(&str, &[&str], &[SideEffect])] = &[
        (
            "watch",
            &[],
            &[SideEffect::WritesFiles, SideEffect::RunsCommands],
        ),
        ("snapshot", &[], &[SideEffect::ReadOnly]),
        (
            "install",
            &[],
            &[SideEffect::WritesFiles, SideEffect::RunsCommands],
        ),
        (
            "uninstall",
            &[],
            &[SideEffect::WritesFiles, SideEffect::RunsCommands],
        ),
        ("switch get", &["5"], &[SideEffect::ReadOnly]),
        ("switch set", &["5", "off"], &[SideEffect::WritesFiles]),
        ("setting get", &["auto-3d"], &[SideEffect::ReadOnly]),
        (
            "setting set",
            &["auto-3d", "off"],
            &[SideEffect::WritesFiles],
        ),
        (
            "run",
            &["5", "--", "game"],
            &[SideEffect::WritesFiles, SideEffect::RunsCommands],
        ),
        ("start-3d", &[], &[SideEffect::WritesFiles]),
        ("stop-3d", &[], &[SideEffect::WritesFiles]),
    ];
    let mut seen = BTreeSet::new();
    for (id, args, effects) in cases {
        let row = cli::COMMANDS.iter().find(|row| row.id == *id).unwrap();
        assert_eq!(row.effects, *effects, "{id}");
        let words: Vec<String> = id
            .split_whitespace()
            .chain(args.iter().copied())
            .map(str::to_string)
            .collect();
        let parsed = cli::parse_any(&words).unwrap().command;
        let matching = match (*id, parsed) {
            ("watch", AnyCommand::Link(LinkCommand::Watch)) => "watch",
            ("snapshot", AnyCommand::Link(LinkCommand::Snapshot)) => "snapshot",
            ("install", AnyCommand::Link(LinkCommand::Install)) => "install",
            ("uninstall", AnyCommand::Link(LinkCommand::Uninstall)) => "uninstall",
            ("switch get", AnyCommand::Link(LinkCommand::SwitchGet { .. })) => "switch get",
            ("switch set", AnyCommand::Link(LinkCommand::SwitchSet { .. })) => "switch set",
            ("setting get", AnyCommand::Link(LinkCommand::SettingGet { .. })) => "setting get",
            ("setting set", AnyCommand::Link(LinkCommand::SettingSet { .. })) => "setting set",
            ("run", AnyCommand::Link(LinkCommand::Run { .. })) => "run",
            ("start-3d", AnyCommand::Engine(Command::Start3d)) => "start-3d",
            ("stop-3d", AnyCommand::Engine(Command::Stop3d)) => "stop-3d",
            (_, other) => panic!("{id} parsed as {other:?}"),
        };
        assert_eq!(matching, *id);
        assert!(seen.insert(matching));
    }
    let link_variants = [
        "watch",
        "snapshot",
        "install",
        "uninstall",
        "switch get",
        "switch set",
        "setting get",
        "setting set",
        "run",
    ];
    for variant in link_variants {
        assert_eq!(
            cases.iter().filter(|(id, _, _)| *id == variant).count(),
            1,
            "{variant}"
        );
    }
    assert_eq!(seen.len(), cases.len());
}
