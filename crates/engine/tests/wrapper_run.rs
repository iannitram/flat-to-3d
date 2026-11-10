use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::recipes::{parse_recipe, Recipe};
use flat3d_inject::runs::{parse_run_record, RunMode};
use flat3d_inject::wrapper::{prepare, run, Launch, RunPaths};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};

const SETTINGS: &str = "flat3d-settings 1\nauto-3d\ton\ntunnel\ton\n";
const RECIPE: &str = "name: Test game\nstereo-status: works-now\nstereo-route: UEVR\nstereo-engine: UE4\nstereo-api: dx11\nstereo-mod: none\nstereo-free: yes\nstereo-safe: yes\nstereo-checked: 2026-11-01\n";

fn env() -> BTreeMap<String, String> {
    [("HOME", "/test-home"), ("SteamAppId", "42")]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect()
}

fn argv() -> Vec<String> {
    [
        "mangohud",
        "proton",
        "game with spaces.exe",
        "",
        "--flag=$x",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn snapshot() -> String {
    let mut snapshot = LinkSnapshot::stopped(100);
    snapshot.state = LinkState::Ready;
    serde_json::to_string(&snapshot).unwrap()
}

fn recipe() -> Recipe {
    parse_recipe(42, RECIPE).unwrap()
}

fn flat(launch: &Launch) {
    assert_eq!(launch.command.argv, argv());
    assert!(launch.command.env.is_empty());
    assert_eq!(launch.record.mode, RunMode::Flat);
}

#[test]
fn flat_game_keeps_argv_and_adds_no_env() {
    let launch = prepare(
        &argv(),
        &env(),
        Ok(&snapshot()),
        Ok(SETTINGS),
        Ok(None),
        7,
        100,
    );
    flat(&launch);
    assert!(launch.message.contains("no recipe"));
}

#[test]
fn three_d_gets_proton_env_and_keeps_existing_mounts() {
    let mut env = env();
    env.insert("PRESSURE_VESSEL_FILESYSTEMS_RW".into(), "/games".into());
    let launch = prepare(
        &argv(),
        &env,
        Ok(&snapshot()),
        Ok(SETTINGS),
        Ok(Some(&recipe())),
        7,
        100,
    );
    assert_eq!(launch.command.argv, argv());
    assert_eq!(launch.record.mode, RunMode::ThreeD);
    let additions: BTreeMap<_, _> = launch.command.env.into_iter().collect();
    assert_eq!(additions.len(), 3);
    assert_eq!(additions["PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES"], "1");
    assert_eq!(additions["STEAM_COMPAT_LAUNCHER_SERVICE"], "proton");
    assert_eq!(
        additions["PRESSURE_VESSEL_FILESYSTEMS_RW"],
        "/games:/test-home/.local/share/flatpak/app/io.github.wivrn.wivrn"
    );
}

#[test]
fn missing_corrupt_and_stale_snapshots_launch_flat() {
    for (input, now, reason) in [
        (Err("snapshot: missing".into()), 100, "missing"),
        (Ok("invalid"), 100, "snapshot:"),
        (Ok(snapshot().as_str()), 111, "stale"),
    ] {
        let launch = prepare(
            &argv(),
            &env(),
            input,
            Ok(SETTINGS),
            Ok(Some(&recipe())),
            7,
            now,
        );
        flat(&launch);
        assert!(launch.message.contains(reason));
    }
}

#[test]
fn off_unreadable_and_invalid_settings_launch_flat() {
    for input in [
        Ok("flat3d-settings 1\ngame-off\t42\n"),
        Err("settings: denied".into()),
        Ok("bad header"),
        Ok("flat3d-settings 1\ngame-off\tnot-an-id\n"),
        Ok("flat3d-settings 1\nauto-3d\tbroken\n"),
    ] {
        flat(&prepare(
            &argv(),
            &env(),
            Ok(&snapshot()),
            input,
            Ok(Some(&recipe())),
            7,
            100,
        ));
    }
}

#[test]
fn appid_fallback_and_bad_appids() {
    let mut env = env();
    env.remove("SteamAppId");
    env.insert("SteamGameId".into(), "42".into());
    let launch = prepare(
        &argv(),
        &env,
        Ok(&snapshot()),
        Ok(SETTINGS),
        Ok(Some(&recipe())),
        7,
        100,
    );
    assert_eq!(launch.record.appid, 42);
    assert_eq!(launch.record.mode, RunMode::ThreeD);
    for bad in [None, Some("garbage"), Some("0"), Some("4294967296")] {
        env.remove("SteamGameId");
        if let Some(bad) = bad {
            env.insert("SteamAppId".into(), bad.into());
        }
        let launch = prepare(
            &argv(),
            &env,
            Ok(&snapshot()),
            Ok(SETTINGS),
            Ok(Some(&recipe())),
            7,
            100,
        );
        flat(&launch);
        assert_eq!(launch.record.appid, 0);
        assert!(launch.message.contains("appid"));
    }
}

#[test]
fn stopped_link_unsafe_recipe_and_missing_home_launch_flat() {
    let stopped = serde_json::to_string(&LinkSnapshot::stopped(100)).unwrap();
    flat(&prepare(
        &argv(),
        &env(),
        Ok(&stopped),
        Ok(SETTINGS),
        Ok(Some(&recipe())),
        7,
        100,
    ));
    let mut unsafe_recipe = recipe();
    unsafe_recipe.safe = "no".into();
    flat(&prepare(
        &argv(),
        &env(),
        Ok(&snapshot()),
        Ok(SETTINGS),
        Ok(Some(&unsafe_recipe)),
        7,
        100,
    ));
    let mut env = env();
    env.remove("HOME");
    flat(&prepare(
        &argv(),
        &env,
        Ok(&snapshot()),
        Ok(SETTINGS),
        Ok(Some(&recipe())),
        7,
        100,
    ));
}

struct Fixture(RunPaths);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/wrapper-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(root.join("recipes")).unwrap();
        let paths = RunPaths {
            state: root.to_str().unwrap().to_owned(),
            settings: root.join("settings"),
            recipes: root.join("recipes"),
            log: root.join("run.log"),
        };
        std::fs::write(root.join("link.json"), snapshot()).unwrap();
        std::fs::write(&paths.settings, SETTINGS).unwrap();
        std::fs::write(paths.recipes.join("42.recipe"), RECIPE).unwrap();
        Self(paths)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0.state);
    }
}

#[test]
fn record_and_log_exist_before_launcher_and_log_appends() {
    let fixture = Fixture::new();
    for _ in 0..2 {
        let (launch, result) = run(&argv(), &env(), &fixture.0, 7, 100, |launch| {
            let saved =
                std::fs::read_to_string(flat3d_inject::state::run_record_path(&fixture.0.state, 7))
                    .unwrap();
            assert_eq!(parse_run_record(&saved), Some(launch.record));
            assert_eq!(launch.record.mode, RunMode::ThreeD);
            assert!(std::fs::read_to_string(&fixture.0.log)
                .unwrap()
                .contains("appid=42 pid=7 3d: Uevr"));
            Ok(23)
        });
        assert_eq!(result.unwrap(), 23);
        assert_eq!(launch.record.pid, 7);
    }
    assert_eq!(
        std::fs::read_to_string(&fixture.0.log)
            .unwrap()
            .lines()
            .count(),
        2
    );
}

#[test]
fn filesystem_failures_never_skip_launcher() {
    let mut fixture = Fixture::new();
    std::fs::remove_file(&fixture.0.settings).unwrap();
    let (_, result) = run(&argv(), &env(), &fixture.0, 7, 100, |launch| {
        flat(launch);
        Ok(())
    });
    result.unwrap();
    assert!(std::fs::read_to_string(&fixture.0.log)
        .unwrap()
        .contains("settings:"));
    fixture.0.state = fixture.0.settings.to_str().unwrap().into();
    std::fs::write(&fixture.0.settings, "not a directory").unwrap();
    fixture.0.log = fixture.0.settings.join("run.log");
    let (_, result) = run(&argv(), &env(), &fixture.0, 7, 100, |launch| {
        flat(launch);
        Ok(())
    });
    result.unwrap();
    // Restore the root so the fixture cleans up its temporary folder.
    fixture.0.state = fixture
        .0
        .settings
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .into();
}

#[test]
fn bad_recipe_and_exec_errors_are_logged() {
    let fixture = Fixture::new();
    std::fs::write(fixture.0.recipes.join("42.recipe"), "broken").unwrap();
    let (_, result) = run(&argv(), &env(), &fixture.0, 7, 100, |launch| {
        flat(launch);
        Err::<(), _>(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    });
    assert_eq!(
        result.unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let log = std::fs::read_to_string(&fixture.0.log).unwrap();
    assert!(log.contains("flat:"));
    assert!(log.contains("exec:"));
}
