//! Wired tests for Heroic wrapper apply, undo, and launch commands (#379).
//! All tests use temp folders under CARGO_TARGET_TMPDIR; no device, no real HOME.

use flat3d_inject::api::UndoResult;
use flat3d_inject::cli::view::{self, Ctx};
use flat3d_inject::cli::ViewCommand;
use flat3d_inject::heroic;
use flat3d_inject::heroic_wrapper::{self, Applied};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::recipes::{self, StoreKey};
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::undo::{self, UndoRecord};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "dc9d2e595d0e4650b35d659f90d41059";

const HEROIC_GAME_CONFIG: &str = r#"{
  "dc9d2e595d0e4650b35d659f90d41059": {
    "winePrefix": "/games/prefixes/alan-wake-2",
    "wineVersion": {
      "bin": "/opt/proton/proton",
      "name": "Proton",
      "type": "proton"
    },
    "wrapperOptions": []
  },
  "app_name": "dc9d2e595d0e4650b35d659f90d41059",
  "version": "v0"
}
"#;

fn fresh_env(name: &str) -> (PathBuf, EnginePaths) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("heroic-wired-{name}"));
    let _ = fs::remove_dir_all(&dir);
    let home = dir.join("home");
    let rt = dir.join("runtime");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&rt).unwrap();

    let env = BTreeMap::from([
        ("HOME".into(), home.to_string_lossy().into()),
        ("XDG_RUNTIME_DIR".into(), rt.to_string_lossy().into()),
    ]);
    let paths = EnginePaths::from_env(&env, 1000).unwrap();
    (dir, paths)
}

fn seed_heroic_config(home: &Path, app_name: &str, content: &str) -> PathBuf {
    let config_dir = home.join(heroic::FLATPAK_CONFIG).join("GamesConfig");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join(format!("{app_name}.json"));
    fs::write(&config_path, content).unwrap();
    config_path
}

// Catches: apply for an epic: recipe failing to call heroic_wrapper::apply, not finding the config root inside a temp HOME, or omitting the backup path from the undo record.
#[test]
fn apply_wires_heroic_wrapper_and_records_backup_in_undo_record() {
    let (_dir, paths) = fresh_env("apply-wires");
    let config_path = seed_heroic_config(&paths.home, APP_NAME, HEROIC_GAME_CONFIG);
    let orig_bytes = fs::read(&config_path).unwrap();

    let recipe = recipes::parse_recipe_keyed(
        StoreKey::Epic(APP_NAME.to_string()),
        "name\tAlan Wake 2\nstereo-status\tworks-now\nstereo-route\tbuild: native\nstereo-engine\tNorthlight\nstereo-api\tdx12\nstereo-mod\tnone\nstereo-free\tyes\nstereo-safe\tyes\nstereo-checked\t2026-11-17\n",
    )
    .unwrap();

    let applied = heroic_wrapper::apply_recipe(&paths, &recipe, APP_NAME).unwrap();
    assert_eq!(applied, Applied::Added);

    let wrapper_str = paths.wrapper_bin().to_string_lossy().into_owned();
    assert!(heroic_wrapper::has_wrapper(&config_path, APP_NAME, &wrapper_str).unwrap());

    let record_path = undo::record_path_keyed(&paths.data, &StoreKey::Epic(APP_NAME.to_string()));
    assert!(
        record_path.exists(),
        "undo record must exist at {}",
        record_path.display()
    );

    let text = fs::read_to_string(&record_path).unwrap();
    let rec = UndoRecord::from_text(&text).unwrap();
    assert_eq!(rec.key(), StoreKey::Epic(APP_NAME.to_string()));
    assert_eq!(rec.replaced.len(), 1);
    assert_eq!(rec.replaced[0].path, config_path);
    assert!(
        rec.replaced[0].backup.exists(),
        "backup file must exist on disk"
    );
    assert_eq!(fs::read(&rec.replaced[0].backup).unwrap(), orig_bytes);
}

// Catches: running apply twice duplicating wrapper entries or mutating config bytes on idempotent re-application.
#[test]
fn apply_twice_leaves_one_wrapper_and_preserves_config_bytes() {
    let (_dir, paths) = fresh_env("apply-twice");
    let config_path = seed_heroic_config(&paths.home, APP_NAME, HEROIC_GAME_CONFIG);

    let first = heroic_wrapper::apply_epic(&paths, APP_NAME).unwrap();
    assert_eq!(first, Applied::Added);
    let first_bytes = fs::read(&config_path).unwrap();

    let second = heroic_wrapper::apply_epic(&paths, APP_NAME).unwrap();
    assert_eq!(second, Applied::AlreadyThere);
    let second_bytes = fs::read(&config_path).unwrap();

    assert_eq!(first_bytes, second_bytes);
    let wrapper_str = paths.wrapper_bin().to_string_lossy().into_owned();
    assert!(heroic_wrapper::has_wrapper(&config_path, APP_NAME, &wrapper_str).unwrap());
}

// Catches: undo failing to restore exact Heroic config bytes or leaving backup and record files on disk.
#[test]
fn undo_restores_heroic_config_byte_for_byte_and_deletes_backup_and_record() {
    let (_dir, paths) = fresh_env("undo-restores");
    let config_path = seed_heroic_config(&paths.home, APP_NAME, HEROIC_GAME_CONFIG);
    let orig_bytes = fs::read(&config_path).unwrap();

    heroic_wrapper::apply_epic(&paths, APP_NAME).unwrap();
    let record_path = undo::record_path_keyed(&paths.data, &StoreKey::Epic(APP_NAME.to_string()));
    let rec = UndoRecord::from_text(&fs::read_to_string(&record_path).unwrap()).unwrap();
    let backup_path = rec.replaced[0].backup.clone();

    assert!(record_path.exists());
    assert!(backup_path.exists());
    assert_ne!(fs::read(&config_path).unwrap(), orig_bytes);

    let result = heroic_wrapper::undo_epic(&paths, APP_NAME).unwrap();
    assert_eq!(result.appid, 0);
    assert_eq!(result.restored.len(), 1);
    assert!(result.restored[0].starts_with("restored "));

    assert_eq!(fs::read(&config_path).unwrap(), orig_bytes);
    assert!(!backup_path.exists(), "backup file must be deleted");
    assert!(!record_path.exists(), "record file must be deleted");
}

// Catches: store-keyed undo records breaking backward compatibility with existing on-disk Steam undo records.
#[test]
fn legacy_steam_undo_record_fixture_loads_and_undoes() {
    let (_dir, paths) = fresh_env("legacy-steam");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.join("tests/fixtures/steam-undo-870780.txt");
    let raw_fixture = fs::read_to_string(&fixture_path).expect("fixture must exist");

    let game_dir = paths.home.join("game");
    let sub_dir = game_dir.join("sub");
    fs::create_dir_all(&sub_dir).unwrap();
    let created_file = game_dir.join("created.dll");
    let replaced_file = game_dir.join("d3d11.dll");
    let backup_file = game_dir.join("d3d11.dll.flat3d-backup");

    fs::write(&created_file, "injected-dll").unwrap();
    fs::write(&replaced_file, "modified-d3d11").unwrap();
    fs::write(&backup_file, "original-d3d11").unwrap();

    let fixture_text = raw_fixture.replace("TARGET_DIR", &game_dir.to_string_lossy());
    let rec_path = undo::record_path(&paths.data, 870780);
    fs::create_dir_all(rec_path.parent().unwrap()).unwrap();
    fs::write(&rec_path, &fixture_text).unwrap();

    let parsed = UndoRecord::from_text(&fixture_text).unwrap();
    assert_eq!(parsed.appid, 870780);
    assert_eq!(parsed.key(), StoreKey::Steam(870780));

    let res: UndoResult = undo::undo_game(&paths.data, 870780).unwrap();
    assert_eq!(res.appid, 870780);
    assert!(!created_file.exists());
    assert_eq!(fs::read_to_string(&replaced_file).unwrap(), "original-d3d11");
    assert!(!backup_file.exists());
    assert!(!sub_dir.exists());
    assert!(!rec_path.exists());
}

// Catches: launch add and launch remove CLI commands ignoring Epic games or failing to call heroic_wrapper.
#[test]
fn launch_add_and_remove_handles_epic_games_via_cli() {
    let (_dir, paths) = fresh_env("launch-cli");
    let config_path = seed_heroic_config(&paths.home, APP_NAME, HEROIC_GAME_CONFIG);
    let orig_bytes = fs::read(&config_path).unwrap();

    let mut fake = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &paths,
        runner: &mut fake,
    };

    // Test launch add --epic
    let add_cmd = ViewCommand::Launch {
        add: true,
        options: format!("epic:{APP_NAME}"),
    };
    let add_val = view::run(&add_cmd, &mut ctx).unwrap();
    assert_eq!(add_val["options"], format!("epic:{APP_NAME}"));

    let wrapper_str = paths.wrapper_bin().to_string_lossy().into_owned();
    assert!(heroic_wrapper::has_wrapper(&config_path, APP_NAME, &wrapper_str).unwrap());
    let record_path = undo::record_path_keyed(&paths.data, &StoreKey::Epic(APP_NAME.to_string()));
    assert!(record_path.exists());

    // Test launch remove --epic
    let rem_cmd = ViewCommand::Launch {
        add: false,
        options: format!("epic:{APP_NAME}"),
    };
    let rem_val = view::run(&rem_cmd, &mut ctx).unwrap();
    assert_eq!(rem_val["options"], "");

    assert_eq!(fs::read(&config_path).unwrap(), orig_bytes);
    assert!(!record_path.exists());
}

// Catches: cli view parse rejecting launch add|remove with --epic flag.
#[test]
fn launch_view_parse_accepts_epic_flags() {
    let words = |slice: &[&str]| slice.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    let parsed_add = view::parse(&words(&["launch", "add", "--epic", APP_NAME]))
        .unwrap()
        .unwrap();
    assert_eq!(
        parsed_add,
        ViewCommand::Launch {
            add: true,
            options: format!("epic:{APP_NAME}"),
        }
    );

    let parsed_rem = view::parse(&words(&["launch", "remove", "--epic", APP_NAME]))
        .unwrap()
        .unwrap();
    assert_eq!(
        parsed_rem,
        ViewCommand::Launch {
            add: false,
            options: format!("epic:{APP_NAME}"),
        }
    );
}
