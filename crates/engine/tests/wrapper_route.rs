//! WP-31: the wrapper route end to end. `run_with` applies the recipe's route, the record and
//! run.log carry a flat reason code, and `flat3d-run` keeps argv and env untouched on a flat launch.

use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::injectors::RouteEnv;
use flat3d_inject::link::{FlatpakScope, WivrnDirs};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::recipes::{parse_recipe, Recipe};
use flat3d_inject::runs::{parse_run_record, RunMode};
use flat3d_inject::state;
use flat3d_inject::switches::Settings;
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::wrapper::{
    exec_spec_with, flat_after_sidecar_error, prepare_with_route, record_text, run_with,
    sidecar_argv, FlatReason, Launch, Plan, FLAT_REASONS,
};
use flat3d_viewer::spec::{Layout, ViewerSpec};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

// Copied verbatim from wrapper_run.rs, where these consts are private.
const SETTINGS: &str = "flat3d-settings 1\nauto-3d\ton\ntunnel\ton\n";
const RECIPE: &str = "name: Test game\nstereo-status: works-now\nstereo-route: UEVR\nstereo-engine: UE4\nstereo-api: dx11\nstereo-mod: none\nstereo-free: yes\nstereo-safe: yes\nstereo-checked: 2026-11-01\n";

const NOW: u64 = 1000;

/// What the route closure returns.
type RouteResult = Result<(RouteEnv, WivrnDirs), String>;

const SYSTEM_WIVRN: &str = "/var/lib/flatpak/app/io.github.wivrn.wivrn";

struct Tmp(PathBuf);

impl Tmp {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("wrapper-route")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Tmp(root)
    }

    fn env(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "HOME".to_string(),
                self.0.join("home").to_string_lossy().into_owned(),
            ),
            (
                "XDG_RUNTIME_DIR".to_string(),
                self.0.join("run").to_string_lossy().into_owned(),
            ),
        ])
    }

    fn paths(&self) -> EnginePaths {
        let mut paths = EnginePaths::from_env(&self.env(), 1000).unwrap();
        paths.recipe_dirs = vec![self.0.join("recipes")];
        paths
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn s(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

fn argv() -> Vec<String> {
    s(&["proton", "waitforexitandrun", "game with spaces.exe"])
}

fn game_env() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HOME".to_string(), "/test-home".to_string()),
        ("SteamAppId".to_string(), "42".to_string()),
    ])
}

fn run_env(tmp: &Tmp) -> BTreeMap<String, String> {
    let mut env = tmp.env();
    env.insert("SteamAppId".into(), "42".into());
    env
}

fn ready(updated: u64) -> LinkSnapshot {
    let mut snapshot = LinkSnapshot::stopped(updated);
    snapshot.state = LinkState::Ready;
    snapshot
}

fn recipe() -> Recipe {
    parse_recipe(42, RECIPE).unwrap()
}

fn save_snapshot(paths: &EnginePaths, snapshot: &LinkSnapshot) {
    let path = state::snapshot_path(&paths.state_dir.to_string_lossy());
    std::fs::create_dir_all(Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_string(snapshot).unwrap()).unwrap();
}

/// A Ready snapshot, the settings file and `42-test-game.recipe` (the A5 name form).
fn seed(tmp: &Tmp) -> EnginePaths {
    let paths = tmp.paths();
    save_snapshot(&paths, &ready(NOW));
    std::fs::create_dir_all(paths.settings().parent().unwrap()).unwrap();
    std::fs::write(paths.settings(), SETTINGS).unwrap();
    std::fs::create_dir_all(&paths.recipe_dirs[0]).unwrap();
    std::fs::write(paths.recipe_dirs[0].join("42-test-game.recipe"), RECIPE).unwrap();
    paths
}

fn system_dirs() -> WivrnDirs {
    let app_dir = PathBuf::from(SYSTEM_WIVRN);
    WivrnDirs {
        scope: FlatpakScope::System,
        deploy: app_dir.join("x86_64/stable/abc"),
        app_dir,
        version: None,
    }
}

fn uevr_env() -> RouteEnv {
    RouteEnv {
        env: vec![(
            "PROTON_REMOTE_DEBUG_CMD".to_string(),
            "/cache/uevr/injector.exe".to_string(),
        )],
        unset: s(&["XR_RUNTIME_JSON"]),
        sidecar: None,
    }
}

/// `prepare_with_route` with a Ready snapshot, default settings and the test recipe.
fn prepare_ready(
    env: &BTreeMap<String, String>,
    route: &mut dyn FnMut(u32, &Recipe) -> RouteResult,
) -> Launch {
    let snapshot = ready(NOW);
    let settings = Settings::default();
    let recipe = recipe();
    prepare_with_route(
        &argv(),
        env,
        Ok(Some(&snapshot)),
        Ok(&settings),
        Ok(Some(&recipe)),
        route,
        7,
        NOW,
    )
}

fn reason(launch: &Launch) -> Option<String> {
    record_text(launch)
        .lines()
        .find_map(|line| line.strip_prefix("reason\t"))
        .map(str::to_string)
}

fn assert_flat(launch: &Launch, message: &str, code: &str) {
    assert_eq!(launch.record.mode, RunMode::Flat);
    assert_eq!(launch.message, message);
    assert_eq!(launch.command.argv, argv());
    assert!(launch.command.env.is_empty());
    assert!(launch.command.unset.is_empty());
    assert_eq!(launch.command.sidecar, None);
    assert_eq!(reason(launch).as_deref(), Some(code));
}

fn env_map(launch: &Launch) -> BTreeMap<String, String> {
    launch.command.env.iter().cloned().collect()
}

// 1
#[test]
fn a_slug_named_recipe_is_found_and_reaches_the_route() {
    let tmp = Tmp::new();
    let paths = seed(&tmp);
    let mut runner = FakeRunner::new();
    let (launch, result) = run_with(&argv(), &run_env(&tmp), &paths, 7, NOW, &mut runner, |_| {
        Ok(())
    });
    result.unwrap();
    assert_flat(&launch, "flat: route: game 42 is not installed", "route");
    assert!(!launch.message.contains("no recipe"));
    assert!(runner.ran.is_empty());
}

// 2
#[test]
fn a_missing_settings_file_is_not_invalid_settings() {
    let tmp = Tmp::new();
    let paths = seed(&tmp);
    std::fs::remove_file(paths.settings()).unwrap();
    let mut runner = FakeRunner::new();
    let (launch, result) = run_with(&argv(), &run_env(&tmp), &paths, 7, NOW, &mut runner, |_| {
        Ok(())
    });
    result.unwrap();
    assert!(!launch.message.contains("invalid settings"));
    assert!(!launch.message.contains("settings:"));
    assert_eq!(launch.message, "flat: route: game 42 is not installed");
}

// 3
#[test]
fn a_link_that_is_not_ready_never_calls_the_route() {
    for link in [
        LinkState::Starting,
        LinkState::Stopped,
        LinkState::NoHeadset,
        LinkState::Unauthorised,
    ] {
        let mut snapshot = ready(NOW);
        snapshot.state = link;
        let settings = Settings::default();
        let recipe = recipe();
        let mut calls = 0;
        let launch = prepare_with_route(
            &argv(),
            &game_env(),
            Ok(Some(&snapshot)),
            Ok(&settings),
            Ok(Some(&recipe)),
            &mut |_, _| {
                calls += 1;
                Ok((uevr_env(), system_dirs()))
            },
            7,
            NOW,
        );
        assert_eq!(calls, 0, "{link:?}");
        assert_flat(&launch, "flat: headset link is not ready", "link-not-ready");

        let tmp = Tmp::new();
        let paths = seed(&tmp);
        save_snapshot(&paths, &snapshot);
        let mut runner = FakeRunner::new();
        let (launch, _) = run_with(&argv(), &run_env(&tmp), &paths, 7, NOW, &mut runner, |_| {
            Ok(())
        });
        assert_eq!(launch.message, "flat: headset link is not ready");
        assert!(runner.ran.is_empty(), "{link:?}");
    }
}

// 4
#[test]
fn a_stale_snapshot_plays_flat() {
    let snapshot = ready(100);
    let settings = Settings::default();
    let recipe = recipe();
    let launch = prepare_with_route(
        &argv(),
        &game_env(),
        Ok(Some(&snapshot)),
        Ok(&settings),
        Ok(Some(&recipe)),
        &mut |_, _| panic!("route called for a stale snapshot"),
        7,
        111,
    );
    assert_flat(&launch, "flat: snapshot is stale", "stale");
}

// 5
#[test]
fn no_snapshot_plays_flat() {
    let settings = Settings::default();
    let recipe = recipe();
    let launch = prepare_with_route(
        &argv(),
        &game_env(),
        Ok(None),
        Ok(&settings),
        Ok(Some(&recipe)),
        &mut |_, _| panic!("route called with no snapshot"),
        7,
        NOW,
    );
    assert!(launch.message.contains("snapshot"));
    assert_flat(&launch, "flat: no link snapshot", "snapshot");
}

// 6
#[test]
fn three_d_takes_the_route_env_and_the_system_wivrn_folder() {
    let mut env = game_env();
    env.insert("PRESSURE_VESSEL_FILESYSTEMS_RW".into(), "/games".into());
    let mut calls = Vec::new();
    let launch = prepare_ready(&env, &mut |appid, recipe| {
        calls.push((appid, recipe.route.clone()));
        Ok((uevr_env(), system_dirs()))
    });
    assert_eq!(calls, [(42, "UEVR".to_string())]);
    assert_eq!(launch.record.mode, RunMode::ThreeD);
    assert_eq!(launch.message, "3d: Uevr");
    assert_eq!(launch.command.argv, argv());
    let env = env_map(&launch);
    assert_eq!(
        env["PRESSURE_VESSEL_FILESYSTEMS_RW"],
        format!("/games:{SYSTEM_WIVRN}")
    );
    assert_eq!(env["PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES"], "1");
    assert_eq!(env["STEAM_COMPAT_LAUNCHER_SERVICE"], "proton");
    assert!(env.contains_key("PROTON_REMOTE_DEBUG_CMD"));
    assert_eq!(launch.command.unset, ["XR_RUNTIME_JSON"]);
    assert_eq!(launch.command.sidecar, None);
}

// 7
#[test]
fn a_route_error_plays_flat() {
    let launch = prepare_ready(&game_env(), &mut |_, _| {
        Err("route: env file missing".into())
    });
    assert_flat(&launch, "flat: route: env file missing", "route");
}

// 8
#[test]
fn a_wivrn_error_plays_flat() {
    let launch = prepare_ready(&game_env(), &mut |_, _| Err("wivrn: not installed".into()));
    assert_flat(&launch, "flat: wivrn: not installed", "wivrn");
}

// 9
#[test]
fn an_empty_home_plays_flat_without_the_route() {
    let mut env = game_env();
    env.insert("HOME".into(), String::new());
    let mut calls = 0;
    let launch = prepare_ready(&env, &mut |_, _| {
        calls += 1;
        Ok((uevr_env(), system_dirs()))
    });
    assert_eq!(calls, 0);
    assert_flat(&launch, "flat: HOME is missing", "home");
}

// 10
#[test]
fn exec_spec_with_flat_drops_the_sidecar() {
    let route = RouteEnv {
        env: vec![("A".into(), "1".into())],
        unset: s(&["B"]),
        sidecar: Some(s(&["/b/flat3d-viewer", "--layout", "sbs"])),
    };
    let spec = exec_spec_with(
        &Plan::Flat {
            reason: FlatReason::SwitchOff,
        },
        &argv(),
        &route,
        &system_dirs(),
        Some("/games"),
    );
    assert_eq!(spec.argv, argv());
    assert!(spec.env.is_empty());
    assert!(spec.unset.is_empty());
    assert_eq!(spec.sidecar, None);
}

fn three_d() -> Plan {
    Plan::ThreeD {
        route: flat3d_inject::injectors::Route::Geo11,
    }
}

// 11
#[test]
fn a_sidecar_is_copied_verbatim() {
    let sidecar = s(&["/b/flat3d-viewer", "--layout", "sbs", "--source", "auto"]);
    let route = RouteEnv {
        env: vec![],
        unset: vec![],
        sidecar: Some(sidecar.clone()),
    };
    let spec = exec_spec_with(&three_d(), &argv(), &route, &system_dirs(), None);
    assert_eq!(spec.sidecar, Some(sidecar));
    assert!(!spec.sidecar.unwrap().iter().any(|arg| arg == "--until-pid"));
}

// 12
#[test]
fn a_route_fs_rw_replaces_the_base_entry() {
    let route = RouteEnv {
        env: vec![("PRESSURE_VESSEL_FILESYSTEMS_RW".into(), "/route".into())],
        unset: vec![],
        sidecar: None,
    };
    let spec = exec_spec_with(&three_d(), &argv(), &route, &system_dirs(), Some("/games"));
    let entries: Vec<_> = spec
        .env
        .iter()
        .filter(|(key, _)| key == "PRESSURE_VESSEL_FILESYSTEMS_RW")
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1, "/route");
    assert_eq!(spec.env.len(), 3);
}

// 13
#[test]
fn a_key_in_env_and_unset_stays_only_in_env() {
    let route = RouteEnv {
        env: vec![("WINEDLLOVERRIDES".into(), "d3d11=n,b".into())],
        unset: s(&["WINEDLLOVERRIDES", "XR_RUNTIME_JSON", "XR_RUNTIME_JSON"]),
        sidecar: None,
    };
    let spec = exec_spec_with(&three_d(), &argv(), &route, &system_dirs(), None);
    assert!(spec.env.iter().any(|(key, _)| key == "WINEDLLOVERRIDES"));
    assert_eq!(spec.unset, ["XR_RUNTIME_JSON"]);
}

// 14
#[test]
fn run_with_writes_the_reason_to_the_record_and_run_log() {
    let tmp = Tmp::new();
    let paths = seed(&tmp);
    let mut runner = FakeRunner::new();
    let state_dir = paths.state_dir.to_string_lossy().into_owned();
    let (launch, result) = run_with(
        &argv(),
        &run_env(&tmp),
        &paths,
        7,
        NOW,
        &mut runner,
        |launch| {
            let saved = std::fs::read_to_string(state::run_record_path(&state_dir, 7)).unwrap();
            assert_eq!(parse_run_record(&saved), Some(launch.record));
            Ok(5)
        },
    );
    assert_eq!(result.unwrap(), 5);
    let saved = std::fs::read_to_string(state::run_record_path(&state_dir, 7)).unwrap();
    assert_eq!(saved, record_text(&launch));
    assert!(saved.lines().any(|line| line == "reason\troute"));
    assert!(saved.lines().any(|line| line.starts_with("note\t")));
    let log = std::fs::read_to_string(paths.run_log()).unwrap();
    assert!(log.contains("appid=42 pid=7 flat: route: game 42 is not installed"));
}

// 15
#[test]
fn the_launcher_runs_once_when_the_log_dir_is_a_file() {
    let tmp = Tmp::new();
    let paths = seed(&tmp);
    std::fs::create_dir_all(paths.log_dir.parent().unwrap()).unwrap();
    std::fs::write(&paths.log_dir, "not a directory").unwrap();
    let mut runner = FakeRunner::new();
    let mut calls = 0;
    let (_, result) = run_with(&argv(), &run_env(&tmp), &paths, 7, NOW, &mut runner, |_| {
        calls += 1;
        Err::<(), _>(std::io::Error::other("exec failed"))
    });
    assert_eq!(calls, 1);
    assert_eq!(result.unwrap_err().to_string(), "exec failed");
}

// 16
#[test]
fn a_sidecar_error_rewrites_the_record_to_flat() {
    let tmp = Tmp::new();
    let paths = seed(&tmp);
    let launch = prepare_ready(&game_env(), &mut |_, _| {
        Ok((
            RouteEnv {
                env: vec![("WINEDLLOVERRIDES".into(), "d3d11,nvapi64=n,b".into())],
                unset: vec![],
                sidecar: Some(s(&["/b/flat3d-viewer", "--layout", "sbs"])),
            },
            system_dirs(),
        ))
    });
    assert_eq!(launch.record.mode, RunMode::ThreeD);
    let spec = flat_after_sidecar_error(&paths, &launch, NOW, "no such file");
    assert_eq!(spec.argv, argv());
    assert!(spec.env.is_empty());
    assert!(spec.unset.is_empty());
    assert_eq!(spec.sidecar, None);
    let saved = std::fs::read_to_string(state::run_record_path(
        &paths.state_dir.to_string_lossy(),
        7,
    ))
    .unwrap();
    assert_eq!(parse_run_record(&saved).unwrap().mode, RunMode::Flat);
    assert!(saved.lines().any(|line| line == "reason\tsidecar"));
    assert!(saved
        .lines()
        .any(|line| line == "note\tsidecar: no such file"));
    let log = std::fs::read_to_string(paths.run_log()).unwrap();
    assert!(log.contains("appid=42 pid=7 flat: sidecar: no such file"));
}

// 17
#[test]
fn reason_codes_cover_every_flat_launch() {
    assert_eq!(
        FLAT_REASONS,
        [
            "appid",
            "snapshot",
            "stale",
            "settings",
            "recipe",
            "no-recipe",
            "switch-off",
            "link-not-ready",
            "not-injectable",
            "home",
            "route",
            "wivrn",
            "sidecar"
        ]
    );
    let launch = prepare_ready(&game_env(), &mut |_, _| Ok((uevr_env(), system_dirs())));
    assert_eq!(launch.record.mode, RunMode::ThreeD);
    let text = record_text(&launch);
    assert_eq!(text, launch.record.to_text());
    assert!(!text.contains("reason\t"));
    assert!(!text.contains("note\t"));

    let snapshot = ready(NOW);
    let settings = Settings::default();
    let off = Settings {
        auto_3d: true,
        off: [42].into(),
    };
    let good = recipe();
    let mut unsafe_recipe = recipe();
    unsafe_recipe.safe = "no".into();
    let mut no_appid = game_env();
    no_appid.remove("SteamAppId");
    let mut never = |_: u32, _: &Recipe| -> RouteResult { panic!("route called for a flat plan") };
    let cases: Vec<(Launch, &str)> = vec![
        (
            prepare_with_route(
                &argv(),
                &no_appid,
                Ok(Some(&snapshot)),
                Ok(&settings),
                Ok(Some(&good)),
                &mut never,
                7,
                NOW,
            ),
            "appid",
        ),
        (
            prepare_with_route(
                &argv(),
                &game_env(),
                Err("snapshot: bad json".into()),
                Ok(&settings),
                Ok(Some(&good)),
                &mut never,
                7,
                NOW,
            ),
            "snapshot",
        ),
        (
            prepare_with_route(
                &argv(),
                &game_env(),
                Ok(Some(&snapshot)),
                Err("settings: denied".into()),
                Ok(Some(&good)),
                &mut never,
                7,
                NOW,
            ),
            "settings",
        ),
        (
            prepare_with_route(
                &argv(),
                &game_env(),
                Ok(Some(&snapshot)),
                Ok(&settings),
                Err("recipe: broken".into()),
                &mut never,
                7,
                NOW,
            ),
            "recipe",
        ),
        (
            prepare_with_route(
                &argv(),
                &game_env(),
                Ok(Some(&snapshot)),
                Ok(&settings),
                Ok(None),
                &mut never,
                7,
                NOW,
            ),
            "no-recipe",
        ),
        (
            prepare_with_route(
                &argv(),
                &game_env(),
                Ok(Some(&snapshot)),
                Ok(&off),
                Ok(Some(&good)),
                &mut never,
                7,
                NOW,
            ),
            "switch-off",
        ),
        (
            prepare_with_route(
                &argv(),
                &game_env(),
                Ok(Some(&snapshot)),
                Ok(&settings),
                Ok(Some(&unsafe_recipe)),
                &mut never,
                7,
                NOW,
            ),
            "not-injectable",
        ),
    ];
    for (launch, code) in &cases {
        assert_eq!(launch.record.mode, RunMode::Flat);
        assert_eq!(reason(launch).as_deref(), Some(*code), "{}", launch.message);
        assert!(FLAT_REASONS.contains(code));
        let note = launch.message.strip_prefix("flat: ").unwrap();
        assert!(record_text(launch)
            .lines()
            .any(|line| line == format!("note\t{note}")));
    }
}

// 18
#[test]
fn sidecar_argv_resolves_the_program_and_appends_until_pid() {
    let exe_dir = Path::new("/opt/flat3d");
    let bin = Path::new("/home-bin");
    let there = |path: &Path| path == Path::new("/opt/flat3d/flat3d-viewer");
    let nowhere = |_: &Path| false;

    assert_eq!(
        sidecar_argv(&s(&["/abs/viewer", "-x"]), Some(exe_dir), bin, 99, &there).unwrap(),
        s(&["/abs/viewer", "-x", "--until-pid", "99"])
    );
    assert_eq!(
        sidecar_argv(&s(&["flat3d-viewer", "-x"]), Some(exe_dir), bin, 99, &there).unwrap(),
        s(&["/opt/flat3d/flat3d-viewer", "-x", "--until-pid", "99"])
    );
    assert_eq!(
        sidecar_argv(&s(&["flat3d-viewer"]), Some(exe_dir), bin, 99, &nowhere).unwrap(),
        s(&["/home-bin/flat3d-viewer", "--until-pid", "99"])
    );
    assert_eq!(
        sidecar_argv(&s(&["flat3d-viewer"]), None, bin, 99, &there).unwrap(),
        s(&["/home-bin/flat3d-viewer", "--until-pid", "99"])
    );
    let last = sidecar_argv(&s(&["v", "a", "b"]), None, bin, 99, &nowhere).unwrap();
    assert_eq!(&last[last.len() - 2..], ["--until-pid", "99"]);
    assert!(sidecar_argv(&[], Some(exe_dir), bin, 99, &there).is_err());

    let mut geo11 = s(&["/b/flat3d-viewer"]);
    geo11.extend(
        ViewerSpec {
            layout: Layout::Sbs,
            ..ViewerSpec::default()
        }
        .to_args(),
    );
    let mut expected = geo11.clone();
    expected.extend(s(&["--until-pid", "99"]));
    let built = sidecar_argv(&geo11, Some(exe_dir), bin, 99, &nowhere).unwrap();
    assert_eq!(built, expected);
    assert_eq!(
        built.join(" "),
        "/b/flat3d-viewer --layout sbs --source auto --until-pid 99"
    );
}

#[cfg(unix)]
const STAND_IN: &str = "printf \"%s\\n\" \"$@\" > \"$OUT\"; exit 3";

// 19
#[cfg(unix)]
#[test]
fn the_bin_plays_flat_with_argv_and_env_untouched() {
    let tmp = Tmp::new();
    let out = tmp.0.join("out.txt");
    let env = tmp.env();
    let child = std::process::Command::new(env!("CARGO_BIN_EXE_flat3d-run"))
        .args(["/bin/sh", "-c", STAND_IN, "sh", "a", "b c"])
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", &env["HOME"])
        .env("XDG_RUNTIME_DIR", &env["XDG_RUNTIME_DIR"])
        .env("SteamAppId", "42")
        .env("OUT", &out)
        .spawn()
        .unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "a\nb c\n");

    let paths = tmp.paths();
    let record = std::fs::read_to_string(state::run_record_path(
        &paths.state_dir.to_string_lossy(),
        pid,
    ))
    .unwrap();
    let parsed = parse_run_record(&record).unwrap();
    assert_eq!(parsed.mode, RunMode::Flat);
    assert_eq!(parsed.appid, 42);
    assert!(record.lines().any(|line| line == "reason\tsnapshot"));
    let log = std::fs::read_to_string(paths.run_log()).unwrap();
    assert!(
        log.contains(&format!("appid=42 pid={pid} flat: no link snapshot")),
        "{log}"
    );
}

// 20
#[cfg(unix)]
#[test]
fn the_bin_without_home_still_runs_the_game() {
    let tmp = Tmp::new();
    let out = tmp.0.join("out.txt");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_flat3d-run"))
        .args(["/bin/sh", "-c", STAND_IN, "sh", "a", "b c"])
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("OUT", &out)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "a\nb c\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("flat3d-run: paths:"));
}

// 21
#[test]
fn an_inherited_winedlloverrides_is_merged_and_the_route_wins() {
    let geo11 = || {
        Ok((
            RouteEnv {
                env: vec![("WINEDLLOVERRIDES".into(), "d3d11,nvapi64=n,b".into())],
                unset: vec![],
                sidecar: None,
            },
            system_dirs(),
        ))
    };
    let mut env = game_env();
    env.insert("WINEDLLOVERRIDES".into(), "dinput8=n,b;d3d11=b".into());
    let launch = prepare_ready(&env, &mut |_, _| geo11());
    let entries: Vec<_> = launch
        .command
        .env
        .iter()
        .filter(|(key, _)| key == "WINEDLLOVERRIDES")
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1, "dinput8=n,b;d3d11,nvapi64=n,b");

    let launch = prepare_ready(&game_env(), &mut |_, _| geo11());
    assert_eq!(env_map(&launch)["WINEDLLOVERRIDES"], "d3d11,nvapi64=n,b");
}
