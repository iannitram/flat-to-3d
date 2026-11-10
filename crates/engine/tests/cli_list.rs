//! List and status commands, Steam library path helpers, and read-only behavior.

#[path = "support/scenes.rs"]
mod scenes;

use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::cli::{self, engine, Command, Ctx, SideEffect};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::steam;
use flat3d_inject::testing::FakeRunner;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const GOOD_RECIPE: &str = "\
name: Beta Game\n\
stereo-status: works-now\n\
stereo-route: build: test route\n\
stereo-engine: custom\n\
stereo-api: dx12\n\
stereo-mod: none\n\
stereo-free: yes\n\
stereo-safe: yes\n\
stereo-checked: 2026-10-31\n";

fn run(
    paths: &EnginePaths,
    command: Command,
) -> Result<serde_json::Value, flat3d_inject::api::ApiError> {
    let mut runner = FakeRunner::new();
    let mut relay = FakeRunner::new();
    let mut ctx = Ctx {
        paths,
        runner: &mut runner,
        relay: &mut relay,
        json: true,
    };
    engine::run(&command, &mut ctx)
}

#[test]
fn no_steam_gives_an_empty_list() {
    let paths = scenes::no_steam("list-empty");
    assert_eq!(run(&paths, Command::List).unwrap(), serde_json::json!([]));
}

#[test]
fn two_libraries_join_recipe_status_in_name_order_for_both_vdf_forms() {
    for (case, build) in [
        (
            "list-new-vdf",
            scenes::two_libs_new_vdf as fn(&str) -> EnginePaths,
        ),
        (
            "list-old-vdf",
            scenes::two_libs_old_vdf as fn(&str) -> EnginePaths,
        ),
    ] {
        let paths = build(case);
        scenes::write_recipe(&paths, "100-beta.recipe", GOOD_RECIPE);
        let actual = run(&paths, Command::List).unwrap();
        let games = actual.as_array().unwrap();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0]["name"], "alpha game");
        assert_eq!(games[0]["status"], "untested");
        assert_eq!(games[0]["has_recipe"], false);
        assert_eq!(games[1]["name"], "Beta Game");
        assert_eq!(games[1]["status"], "works-now");
        assert_eq!(games[1]["has_recipe"], true);
        assert!(games[1]["install_dir"]
            .as_str()
            .unwrap()
            .contains("steamapps"));
    }
}

#[test]
fn a_legacy_appid_recipe_is_found_and_joined() {
    let paths = scenes::two_libs_new_vdf("list-legacy-recipe");
    scenes::write_recipe(&paths, "100.recipe", GOOD_RECIPE);
    let games = run(&paths, Command::List).unwrap();
    assert_eq!(games[1]["appid"], 100);
    assert_eq!(games[1]["status"], "works-now");
    assert_eq!(games[1]["has_recipe"], true);
}

#[test]
fn a_malformed_recipe_is_a_bad_recipe_naming_its_file() {
    let paths = scenes::two_libs_new_vdf("list-bad-recipe");
    scenes::write_recipe(
        &paths,
        "100.recipe",
        &GOOD_RECIPE.replace("works-now", "maybe"),
    );
    let error = run(&paths, Command::List).unwrap_err();
    assert_eq!(error.code, "bad-recipe");
    assert!(error.message.contains("100.recipe"), "{}", error.message);
}

#[test]
fn library_of_and_compatdata_follow_the_expected_search_order() {
    let paths = scenes::two_libs_new_vdf("steam-compatdata");
    let steam_root = paths.home.join(steam::DEFAULT_STEAM_ROOT);
    let games = steam::installed_games(&steam_root).unwrap();
    let game = games.iter().find(|game| game.appid == 200).unwrap();
    let lib2 = game.install_dir.ancestors().nth(3).unwrap().to_path_buf();
    assert_eq!(steam::library_of(game), Some(lib2.clone()));
    assert_eq!(
        steam::library_of(&steam::Game {
            appid: 1,
            name: "bad path".into(),
            install_dir: paths.home.join("not/a/steam/game"),
        }),
        None
    );

    let own = lib2.join("steamapps/compatdata/200");
    let root = steam_root.join("steamapps/compatdata/200");
    fs::create_dir_all(&own).unwrap();
    fs::create_dir_all(&root).unwrap();
    assert_eq!(steam::compatdata(&steam_root, game), Some(own.clone()));
    fs::remove_dir_all(&own).unwrap();
    assert_eq!(steam::compatdata(&steam_root, game), Some(root.clone()));
    fs::remove_dir_all(&root).unwrap();
    assert_eq!(steam::compatdata(&steam_root, game), None);
}

#[test]
fn status_reports_stopped_fresh_ready_and_stale_snapshot_states() {
    let paths = scenes::no_steam("status-snapshots");
    let absent = run(&paths, Command::Status).unwrap();
    assert_eq!(absent["state"], "stopped");
    assert_eq!(absent["message"], "Link service is not running.");

    let now = flat3d_inject::sys::unix_now();
    let mut fresh = LinkSnapshot::stopped(now);
    fresh.state = LinkState::Ready;
    fresh.message = "Link is ready.".into();
    fresh.updated = now;
    scenes::write_snapshot(&paths, &fresh);
    assert_eq!(run(&paths, Command::Status).unwrap()["state"], "ready");

    let mut stale = fresh;
    stale.updated = now.saturating_sub(11);
    scenes::write_snapshot(&paths, &stale);
    assert_eq!(run(&paths, Command::Status).unwrap()["state"], "stopped");
}

#[test]
fn the_binary_prints_one_parseable_list_json_envelope() {
    let paths = scenes::no_steam("binary-list-json");
    let runtime = paths.state_dir.parent().unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_flat3d-inject"))
        .args(["list", "--json"])
        .env_clear()
        .env("HOME", &paths.home)
        .env("XDG_RUNTIME_DIR", runtime)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "{stdout:?}");
    let response: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["command"], "list");
    assert_eq!(response["data"], serde_json::json!([]));
}

fn tree_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut files);
    }
    files
}

#[test]
fn list_and_status_are_read_only_and_the_registry_says_read_only() {
    let paths = scenes::two_libs_new_vdf("effects-read-only");
    scenes::write_recipe(&paths, "100.recipe", GOOD_RECIPE);
    let mut snapshot = LinkSnapshot::stopped(flat3d_inject::sys::unix_now());
    snapshot.state = LinkState::Ready;
    scenes::write_snapshot(&paths, &snapshot);
    let before_home = tree_bytes(&paths.home);
    let runtime = paths.state_dir.parent().unwrap();
    let before_runtime = tree_bytes(runtime);

    let mut runner = FakeRunner::new();
    let mut relay = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &paths,
        runner: &mut runner,
        relay: &mut relay,
        json: true,
    };
    engine::run(&Command::List, &mut ctx).unwrap();
    engine::run(&Command::Status, &mut ctx).unwrap();
    assert!(runner.ran.is_empty());
    assert!(relay.ran.is_empty());
    assert_eq!(tree_bytes(&paths.home), before_home);
    assert_eq!(tree_bytes(runtime), before_runtime);

    for name in ["list", "status"] {
        let row = cli::COMMANDS.iter().find(|row| row.id == name).unwrap();
        assert_eq!(row.effects, &[SideEffect::ReadOnly]);
    }
}
