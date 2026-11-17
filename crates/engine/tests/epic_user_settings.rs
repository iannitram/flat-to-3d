//! The Epic Games Launcher hook (#379): turning 3D on for an `epic:<app name>` recipe writes a
//! marked block into Proton's `user_settings.py`. Every test uses a temp HOME and enters through
//! `ensure_applied_in`, the real turn-on path. No device, no real HOME.

use flat3d_inject::cli::{view, Ctx, ViewCommand};
use flat3d_inject::epic_user_settings::{begin_marker, end_marker, user_settings_path};
use flat3d_inject::injectors::{ensure_applied_in, RouteEnv};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::recipes::{self, StoreKey};
use flat3d_inject::steam::Game;
use flat3d_inject::testing::FakeRunner;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const APP: &str = "dc9d2e595d0e4650b35d659f90d41059";
const OTHER_APP: &str = "0123456789abcdef0123456789abcdef";

const EXISTING: &str =
    "import os\n\n# my own settings\nuser_settings = {'WINEDEBUG': '-all'}\nif os.environ.get('X'):\n    user_settings['Y'] = '1'\n";

fn recipe_text(app: &str, exe_line: &str) -> String {
    format!(
        "name: Alan Wake 2\nstereo-status: build\nstereo-route: build: northlight view hook\nstereo-engine: northlight\nstereo-api: dx12\nstereo-mod: none\nstereo-free: yes\nstereo-safe: yes\nstereo-checked: 2026-11-17\nstore-key: epic:{app}\n{exe_line}"
    )
}

fn fresh(name: &str) -> EnginePaths {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("epic-us-{name}"));
    let _ = fs::remove_dir_all(&dir);
    let home = dir.join("home");
    let rt = dir.join("runtime");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&rt).unwrap();
    let env = BTreeMap::from([
        ("HOME".into(), home.to_string_lossy().into()),
        ("XDG_RUNTIME_DIR".into(), rt.to_string_lossy().into()),
    ]);
    EnginePaths::from_env(&env, 1000).unwrap()
}

fn seed_file(paths: &EnginePaths, text: &str) {
    let file = user_settings_path(&paths.home);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, text).unwrap();
}

fn read(paths: &EnginePaths) -> String {
    fs::read_to_string(user_settings_path(&paths.home)).unwrap()
}

/// Turn 3D on through `ensure_applied_in` for the Epic game `app` described by `text`.
fn turn_on(paths: &EnginePaths, app: &str, text: &str) -> Result<RouteEnv, String> {
    let key: StoreKey = format!("epic:{app}")
        .parse()
        .unwrap_or(StoreKey::Epic(app.into()));
    let recipe = recipes::parse_recipe_keyed(key.clone(), text).unwrap();
    let settings = recipes::route_settings(text);
    let game = Game {
        appid: 0,
        name: format!("epic:{app}"),
        install_dir: PathBuf::new(),
    };
    let mut runner = FakeRunner::new();
    ensure_applied_in(
        &game,
        &recipe,
        &settings,
        paths,
        &mut runner,
        &BTreeMap::new(),
    )
}

fn undo_via_cli(
    paths: &EnginePaths,
    app: &str,
) -> Result<serde_json::Value, flat3d_inject::api::ApiError> {
    let mut fake = FakeRunner::new();
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths,
        runner: &mut fake,
        process: &mut process,
        json: true,
    };
    view::run(
        &ViewCommand::Launch {
            add: false,
            options: format!("epic:{app}"),
        },
        &mut ctx,
    )
}

fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

// Catches: the epic: recipe not reaching the user_settings writer (no block), the block setting
// env without the exe guard (it would apply to every game the launcher starts), a different env
// than the Steam wrapper sets, or the block rewriting lines of the existing file.
#[test]
fn epic_recipe_writes_one_guarded_block_and_keeps_other_lines() {
    let paths = fresh("block");
    seed_file(&paths, EXISTING);
    let env = turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    assert!(!env.env.is_empty());

    let text = read(&paths);
    assert!(
        text.starts_with(EXISTING),
        "existing lines must be kept byte for byte"
    );
    assert_eq!(count(&text, &begin_marker(APP)), 1);
    assert_eq!(count(&text, &end_marker(APP)), 1);
    let block = &text[EXISTING.len()..];
    let guard = block.find("== 'alanwake2.exe'").expect("exe guard");
    let first_set = block.find("user_settings[").expect("env assignment");
    assert!(guard < first_set, "env is set only under the guard");
    assert!(block.contains("sys.argv"));

    // The env is the one the Steam launch-option wrapper sets for a 3D launch.
    let wivrn = flat3d_inject::link::WivrnDirs::user_default(&paths.home);
    let spec = flat3d_inject::wrapper::exec_spec_with(
        &flat3d_inject::wrapper::Plan::ThreeD {
            route: flat3d_inject::injectors::Route::Aw2,
        },
        &[],
        &RouteEnv::default(),
        &wivrn,
        None,
    );
    assert_eq!(spec.env, env.env);
    for (key, value) in &spec.env {
        assert!(
            block.contains(&format!("    user_settings['{key}'] = '{value}'\n")),
            "{key}"
        );
    }
    assert!(spec
        .env
        .iter()
        .any(|(k, _)| k == "STEAM_COMPAT_LAUNCHER_SERVICE"));
}

// Catches: the exe being guessed from the app name, or a recipe with no exe line still writing.
#[test]
fn exe_comes_from_the_recipe_and_is_never_guessed() {
    let paths = fresh("no-exe");
    seed_file(&paths, EXISTING);
    let err = turn_on(&paths, APP, &recipe_text(APP, "")).unwrap_err();
    assert!(err.starts_with("bad-recipe: "), "{err}");
    assert_eq!(read(&paths), EXISTING);
    assert!(!paths.data.join("undo").exists());

    // A different exe line gives a different guard.
    turn_on(&paths, APP, &recipe_text(APP, "exe: Other.exe\n")).unwrap();
    assert!(read(&paths).contains("== 'other.exe'"));

    // An exe name that could break out of the Python string is refused, nothing new written.
    let paths = fresh("bad-exe");
    seed_file(&paths, EXISTING);
    let err = turn_on(&paths, APP, &recipe_text(APP, "exe: a'b.exe\n")).unwrap_err();
    assert!(err.starts_with("refused: "), "{err}");
    assert_eq!(read(&paths), EXISTING);
}

// Catches: a second apply adding a second block, or changing bytes the first apply wrote.
#[test]
fn applying_twice_leaves_exactly_one_block() {
    let paths = fresh("twice");
    seed_file(&paths, EXISTING);
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    turn_on(&paths, APP, &text).unwrap();
    let first = read(&paths);
    turn_on(&paths, APP, &text).unwrap();
    assert_eq!(read(&paths), first);
    assert_eq!(count(&read(&paths), &begin_marker(APP)), 1);
    assert_eq!(count(&read(&paths), &end_marker(APP)), 1);

    // Same for a file we created.
    let paths = fresh("twice-new");
    turn_on(&paths, APP, &text).unwrap();
    let first = read(&paths);
    turn_on(&paths, APP, &text).unwrap();
    assert_eq!(read(&paths), first);
    assert_eq!(count(&first, &begin_marker(APP)), 1);
    assert!(first.contains("user_settings = {}"));
}

// Catches: undo (through the launch remove command) not restoring the original file bytes, or
// leaving the undo record behind.
#[test]
fn undo_restores_the_original_file_byte_for_byte() {
    let paths = fresh("undo");
    seed_file(&paths, EXISTING);
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    assert_ne!(read(&paths), EXISTING);
    let record = flat3d_inject::undo::record_path_keyed(&paths.data, &StoreKey::Epic(APP.into()));
    let record_text = fs::read_to_string(&record).unwrap();
    assert!(record_text.contains(&format!("store-key\tepic:{APP}\n")));

    undo_via_cli(&paths, APP).unwrap();
    assert_eq!(
        fs::read(user_settings_path(&paths.home)).unwrap(),
        EXISTING.as_bytes()
    );
    assert!(!record.exists());

    // A file without a final newline comes back exactly too.
    let paths = fresh("undo-no-newline");
    seed_file(&paths, "user_settings = {}");
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    undo_via_cli(&paths, APP).unwrap();
    assert_eq!(read(&paths), "user_settings = {}");
}

// Catches: undo leaving behind a file (or folder) we created, so the launcher's folder is not
// put back to how it was.
#[test]
fn undo_deletes_a_file_we_created() {
    let paths = fresh("undo-created");
    let file = user_settings_path(&paths.home);
    assert!(!file.exists());
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    let text = read(&paths);
    assert!(text.starts_with("# Written by flat-to-3d.\nuser_settings = {}\n"));
    assert_eq!(count(&text, &begin_marker(APP)), 1);

    undo_via_cli(&paths, APP).unwrap();
    assert!(!file.exists());
    assert!(
        !file.parent().unwrap().exists(),
        "the folder we made goes too"
    );
}

// Catches: a second game's apply or undo touching the first game's block or the user's lines.
#[test]
fn two_games_keep_separate_blocks() {
    let paths = fresh("two-games");
    seed_file(&paths, EXISTING);
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    let after_first = read(&paths);
    turn_on(
        &paths,
        OTHER_APP,
        &recipe_text(OTHER_APP, "exe: Game.exe\n"),
    )
    .unwrap();
    let both = read(&paths);
    assert!(both.starts_with(&after_first));
    assert_eq!(count(&both, &begin_marker(OTHER_APP)), 1);

    undo_via_cli(&paths, OTHER_APP).unwrap();
    assert_eq!(read(&paths), after_first);
}

// Catches: a Steam recipe through the same path writing the Epic file, or the Steam
// launch-option wrapper no longer being added by the launch command.
#[test]
fn steam_recipe_gets_the_launch_option_wrapper_and_leaves_the_file_alone() {
    let paths = fresh("steam");
    seed_file(&paths, EXISTING);
    let text = recipe_text(APP, "exe: AlanWake2.exe\n").replace("store-key: epic:", "# was: ");
    let recipe = recipes::parse_recipe(870780, &text).unwrap();
    let game = Game {
        appid: 870780,
        name: "Control".into(),
        install_dir: PathBuf::new(),
    };
    let mut runner = FakeRunner::new();
    // The Steam route (here a `build` recipe) is not built: an error, and no Epic write.
    let result = ensure_applied_in(
        &game,
        &recipe,
        &recipes::route_settings(&text),
        &paths,
        &mut runner,
        &BTreeMap::new(),
    );
    assert!(result.is_err());
    assert_eq!(read(&paths), EXISTING);
    assert!(!paths.data.join("undo").exists());

    let mut fake = FakeRunner::new();
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &paths,
        runner: &mut fake,
        process: &mut process,
        json: true,
    };
    let value = view::run(
        &ViewCommand::Launch {
            add: true,
            options: "-novid".into(),
        },
        &mut ctx,
    )
    .unwrap();
    let options = value["options"].as_str().unwrap();
    assert!(options.contains(&paths.wrapper_bin().to_string_lossy().into_owned()));
    assert!(options.contains("%command%"));
    assert_eq!(read(&paths), EXISTING);
}

// Catches: an unsafe app name reaching the writer: it must be refused with nothing written
// (no file, no undo record, nothing outside the data folder).
#[test]
fn unsafe_app_names_are_refused_and_write_nothing() {
    let paths = fresh("unsafe");
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    for name in ["../x", "/", "a\0b", "", "a\nb", "a\\b", ".."] {
        let err = turn_on(&paths, name, &text).unwrap_err();
        assert!(err.starts_with("refused: "), "{name:?}: {err}");
        assert!(undo_via_cli(&paths, name).is_err(), "{name:?}");
    }
    assert!(!user_settings_path(&paths.home).exists());
    assert!(!user_settings_path(&paths.home).parent().unwrap().exists());
    assert!(!paths.data.join("undo").exists());
    assert!(!paths.data.exists());
}

// Catches: the launch add command for an epic: key still going to Heroic (or finding no recipe)
// instead of the user_settings hook, and remove not undoing it.
#[test]
fn launch_add_and_remove_use_the_user_settings_hook() {
    let paths = fresh("cli");
    let dir = paths.data.join("recipes");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("epic-alan-wake-2.recipe"),
        recipe_text(APP, "exe: AlanWake2.exe\n"),
    )
    .unwrap();
    seed_file(&paths, EXISTING);

    let mut fake = FakeRunner::new();
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &paths,
        runner: &mut fake,
        process: &mut process,
        json: true,
    };
    let value = view::run(
        &ViewCommand::Launch {
            add: true,
            options: format!("epic:{APP}"),
        },
        &mut ctx,
    )
    .unwrap();
    assert_eq!(value["options"], format!("epic:{APP}"));
    assert_eq!(count(&read(&paths), &begin_marker(APP)), 1);

    undo_via_cli(&paths, APP).unwrap();
    assert_eq!(read(&paths), EXISTING);
}
