//! The Epic Games Launcher hook (#379, #385): turning 3D on for an `epic:<app name>` recipe
//! writes a marked block into Proton's `user_settings.py`. Every test uses a temp HOME and enters
//! through `ensure_applied_in`, the real turn-on path, or the launch command. No device, no real
//! HOME; WiVRn detection answers from a scripted runner.

use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::cli::{view, Ctx, ViewCommand};
use flat3d_inject::epic_user_settings::{
    begin_marker, end_marker, shared_backup_path, three_d_env, user_settings_path,
};
use flat3d_inject::injectors::{ensure_applied_in, epic_injector_env, Route, RouteEnv};
use flat3d_inject::link::{self, WivrnDirs};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::recipes::{self, StoreKey};
use flat3d_inject::steam::Game;
use flat3d_inject::switches::Settings;
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::wrapper::prepare_with_route;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const APP: &str = "dc9d2e595d0e4650b35d659f90d41059";
const OTHER_APP: &str = "0123456789abcdef0123456789abcdef";

const EXISTING: &str =
    "import os\n\n# my own settings\nuser_settings = {'WINEDEBUG': '-all'}\nif os.environ.get('X'):\n    user_settings['Y'] = '1'\n";

/// Where the scripted runner says the WiVRn Flatpak is deployed.
const WIVRN_DEPLOY: &str = "/test-flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc\n";
const WIVRN_APP_DIR: &str = "/test-flatpak/app/io.github.wivrn.wivrn";

const NOW: u64 = 1000;

/// An Epic recipe text with the given route and `exe:` line.
fn recipe_with(app: &str, status: &str, route: &str, exe_line: &str) -> String {
    format!(
        "name: Alan Wake 2\nstereo-status: {status}\nstereo-route: {route}\nstereo-engine: northlight\nstereo-api: dx12\nstereo-mod: none\nstereo-free: yes\nstereo-safe: yes\nstereo-checked: 2026-11-17\nstore-key: epic:{app}\n{exe_line}"
    )
}

/// A works-now AW2 recipe, the route the Epic hook carries.
fn recipe_text(app: &str, exe_line: &str) -> String {
    recipe_with(app, "works-now", "aw2 view hook", exe_line)
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

/// A runner whose `flatpak info` answers name a user WiVRn install.
fn wivrn_runner() -> FakeRunner {
    FakeRunner::new()
        .ok(
            &["flatpak", "info", "--show-location", "--user"],
            WIVRN_DEPLOY,
        )
        .ok(&["flatpak", "info", "--user"], "Version: 26.9\n")
}

/// What `detect_server` gives with [`wivrn_runner`]: the folders the Steam wrapper gets.
fn wivrn_dirs() -> WivrnDirs {
    link::detect_server(&mut wivrn_runner()).unwrap()
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
    let mut runner = wivrn_runner();
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

/// The text of `app`'s block in `text`, marker lines included.
fn block_of(text: &str, app: &str) -> String {
    let start = text.find(&begin_marker(app)).expect("begin marker");
    let end_line = end_marker(app);
    let end = text[start..].find(&end_line).expect("end marker") + start + end_line.len() + 1;
    text[start..end].to_string()
}

fn record_path(paths: &EnginePaths, app: &str) -> PathBuf {
    flat3d_inject::undo::record_path_keyed(&paths.data, &StoreKey::Epic(app.into()))
}

/// The 3D launch the Steam wrapper builds (`prepare_with_route`) for `text` as a Steam
/// recipe, with the route step answering `injector` and `dirs`.
fn steam_wrapper_launch(
    text: &str,
    inherited_fs_rw: &str,
    injector: &RouteEnv,
    dirs: &WivrnDirs,
) -> flat3d_inject::wrapper::Launch {
    let steam_text = text.replace("store-key: epic:", "# was: ");
    let recipe = recipes::parse_recipe(42, &steam_text).unwrap();
    let mut snapshot = LinkSnapshot::stopped(NOW);
    snapshot.state = LinkState::Ready;
    let env = BTreeMap::from([
        ("HOME".to_string(), "/test-home".to_string()),
        ("SteamAppId".to_string(), "42".to_string()),
        (
            "PRESSURE_VESSEL_FILESYSTEMS_RW".to_string(),
            inherited_fs_rw.to_string(),
        ),
    ]);
    prepare_with_route(
        &["proton".to_string(), "run".to_string()],
        &env,
        Ok(Some(&snapshot)),
        Ok(&Settings::default()),
        Ok(Some(&recipe)),
        &mut |_, _| Ok((injector.clone(), dirs.clone())),
        7,
        NOW,
    )
}

#[test]
fn epic_recipe_writes_one_guarded_block_and_keeps_other_lines() {
    let paths = fresh("block");
    seed_file(&paths, EXISTING);
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();

    let text = read(&paths);
    // Catches: the block rewriting or dropping lines of the user's own file.
    assert!(
        text.starts_with(EXISTING),
        "existing lines must be kept byte for byte"
    );
    // Catches: no block written, or a second copy of it.
    assert_eq!(count(&text, &begin_marker(APP)), 1);
    assert_eq!(count(&text, &end_marker(APP)), 1);
    let block = &text[EXISTING.len()..];
    let guard = block.find("== 'alanwake2.exe'").expect("exe guard");
    let first_set = block.find("user_settings[").expect("env assignment");
    // Catches: env set outside the exe guard, so it would apply to every launcher game.
    assert!(guard < first_set, "env is set only under the guard");
    assert!(block.contains("sys.argv"));

    // The env is the one the Steam wrapper's own launch builder sets for this recipe with the
    // same detect_server folders: every plain variable is assigned as is, and the WiVRn folder
    // is merged into PRESSURE_VESSEL_FILESYSTEMS_RW, never assigned over it.
    let text_recipe = recipe_text(APP, "exe: AlanWake2.exe\n");
    let launch = steam_wrapper_launch(&text_recipe, "", &RouteEnv::default(), &wivrn_dirs());
    assert!(matches!(
        launch.record.mode,
        flat3d_inject::runs::RunMode::ThreeD
    ));
    for (key, value) in &launch.command.env {
        if key == "PRESSURE_VESSEL_FILESYSTEMS_RW" {
            // Catches: the block assigning the folder over the inherited value instead of
            // merging into it.
            assert!(!block.contains(&format!("user_settings['{key}'] = '")));
            assert!(block.contains(&format!(
                "user_settings.get('{key}') or _f3d_os.environ.get('{key}', '')"
            )));
            // Catches: the block's folder taken from the user default WiVRn folder instead of
            // the detect_server folder the Steam wrapper gets.
            assert_eq!(value, WIVRN_APP_DIR);
            assert!(block.contains(&format!("for _f3d_dir in '{value}'.split(':'):")));
        } else {
            // Catches: a variable the Steam wrapper sets missing from, or different in, the
            // block.
            assert!(
                block.contains(&format!("    user_settings['{key}'] = '{value}'\n")),
                "{key}"
            );
        }
    }
}

#[test]
fn epic_env_matches_the_steam_wrapper_env_and_steam_refusals_hold() {
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    let dirs = wivrn_dirs();

    // The injector env the Epic route step returns for this recipe.
    let recipe = recipes::parse_recipe_keyed(StoreKey::Epic(APP.into()), &text).unwrap();
    let (route, injector) = epic_injector_env(&recipe, &recipes::route_settings(&text)).unwrap();
    // Catches: the route taken from anything but the recipe's stereo-route.
    assert_eq!(route, Route::Aw2);

    // Same recipe, same injector env, same detect_server folders, same inherited folders: the
    // Steam wrapper's launch env and the Epic env are equal. A non-empty injector env shows the
    // route env is carried, not replaced by a default.
    let routed = RouteEnv {
        env: vec![
            ("WINEDLLOVERRIDES".into(), "version=n,b".into()),
            ("F3D_TEST".into(), "1".into()),
        ],
        unset: vec![],
        sidecar: None,
    };
    for injector in [injector, routed] {
        let launch = steam_wrapper_launch(&text, "/games", &injector, &dirs);
        assert!(matches!(
            launch.record.mode,
            flat3d_inject::runs::RunMode::ThreeD
        ));
        let epic = three_d_env(route, &injector, &dirs, Some("/games"));
        // Catches: three_d_env building from RouteEnv::default(), the user default WiVRn
        // folder, or no inherited value, instead of the injector env, the detect_server
        // folders and the merged PRESSURE_VESSEL_FILESYSTEMS_RW the Steam wrapper uses.
        assert_eq!(epic.env, launch.command.env);
        assert_eq!(epic.unset, launch.command.unset);
    }
    // Catches: the inherited folder list overwritten instead of merged.
    let epic = three_d_env(route, &RouteEnv::default(), &dirs, Some("/games"));
    assert!(epic.env.contains(&(
        "PRESSURE_VESSEL_FILESYSTEMS_RW".to_string(),
        format!("/games:{WIVRN_APP_DIR}")
    )));

    // A Port route: the Steam path refuses it, and the Epic path gives the same refusal and
    // writes nothing.
    let port = recipe_with(
        APP,
        "works-now",
        "build: northlight view hook",
        "exe: AlanWake2.exe\n",
    );
    let steam_text = port.replace("store-key: epic:", "# was: ");
    let paths = fresh("port");
    seed_file(&paths, EXISTING);
    let steam_err = ensure_applied_in(
        &Game {
            appid: 42,
            name: "Alan Wake 2".into(),
            install_dir: PathBuf::new(),
        },
        &recipes::parse_recipe(42, &steam_text).unwrap(),
        &recipes::route_settings(&steam_text),
        &paths,
        &mut wivrn_runner(),
        &BTreeMap::new(),
    )
    .unwrap_err();
    let epic_err = turn_on(&paths, APP, &port).unwrap_err();
    // Catches: the Epic path writing a block for a route the Steam path refuses.
    assert_eq!(epic_err, steam_err);
    assert!(epic_err.starts_with("not-built: "), "{epic_err}");
    assert_eq!(read(&paths), EXISTING);
    assert!(!paths.data.join("undo").exists());

    // A recipe the Steam wrapper runs flat (not works-now) is refused here too.
    let build = recipe_with(APP, "build", "aw2 view hook", "exe: AlanWake2.exe\n");
    let launch = steam_wrapper_launch(&build, "", &RouteEnv::default(), &dirs);
    assert!(matches!(
        launch.record.mode,
        flat3d_inject::runs::RunMode::Flat
    ));
    let err = turn_on(&paths, APP, &build).unwrap_err();
    // Catches: the Epic path skipping the refusal rule the Steam wrapper applies.
    assert!(err.starts_with("refused: "), "{err}");
    assert_eq!(read(&paths), EXISTING);
}

#[test]
fn exe_comes_from_the_recipe_and_is_never_guessed() {
    let paths = fresh("no-exe");
    seed_file(&paths, EXISTING);
    let err = turn_on(&paths, APP, &recipe_text(APP, "")).unwrap_err();
    // Catches: the exe being guessed from the app name when the recipe has no exe line.
    assert!(err.starts_with("bad-recipe: "), "{err}");
    assert_eq!(read(&paths), EXISTING);
    assert!(!paths.data.join("undo").exists());

    // Catches: the guard naming a fixed exe instead of the recipe's.
    turn_on(&paths, APP, &recipe_text(APP, "exe: Other.exe\n")).unwrap();
    assert!(read(&paths).contains("== 'other.exe'"));

    let paths = fresh("bad-exe");
    seed_file(&paths, EXISTING);
    let err = turn_on(&paths, APP, &recipe_text(APP, "exe: a'b.exe\n")).unwrap_err();
    // Catches: an exe name that breaks out of the Python string being written.
    assert!(err.starts_with("refused: "), "{err}");
    assert_eq!(read(&paths), EXISTING);
}

#[test]
fn applying_twice_leaves_exactly_one_block() {
    let paths = fresh("twice");
    seed_file(&paths, EXISTING);
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    turn_on(&paths, APP, &text).unwrap();
    let first = read(&paths);
    turn_on(&paths, APP, &text).unwrap();
    // Catches: a second apply adding a second block or changing bytes the first wrote.
    assert_eq!(read(&paths), first);
    assert_eq!(count(&read(&paths), &begin_marker(APP)), 1);
    assert_eq!(count(&read(&paths), &end_marker(APP)), 1);

    let paths = fresh("twice-new");
    turn_on(&paths, APP, &text).unwrap();
    let first = read(&paths);
    turn_on(&paths, APP, &text).unwrap();
    // Catches: the same, for a file we created.
    assert_eq!(read(&paths), first);
    assert_eq!(count(&first, &begin_marker(APP)), 1);
    assert!(first.contains("user_settings = {}"));
}

#[test]
fn undo_restores_the_original_file_byte_for_byte() {
    let paths = fresh("undo");
    seed_file(&paths, EXISTING);
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    assert_ne!(read(&paths), EXISTING);
    let record = record_path(&paths, APP);
    let record_text = fs::read_to_string(&record).unwrap();
    // Catches: the record written without its epic store key.
    assert!(record_text.contains(&format!("store-key\tepic:{APP}\n")));

    undo_via_cli(&paths, APP).unwrap();
    // Catches: undo through the launch remove command not restoring the original bytes, or
    // leaving the record or the shared backup behind.
    assert_eq!(
        fs::read(user_settings_path(&paths.home)).unwrap(),
        EXISTING.as_bytes()
    );
    assert!(!record.exists());
    assert!(!shared_backup_path(&paths.data).exists());

    let paths = fresh("undo-no-newline");
    seed_file(&paths, "user_settings = {}");
    turn_on(&paths, APP, &recipe_text(APP, "exe: AlanWake2.exe\n")).unwrap();
    undo_via_cli(&paths, APP).unwrap();
    // Catches: the newline added before our block left behind after undo.
    assert_eq!(read(&paths), "user_settings = {}");
}

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
    // Catches: undo leaving behind the file or the folders we created.
    assert!(!file.exists());
    assert!(
        !paths.home.join(".config").exists(),
        "the folders we made go too"
    );
}

/// Apply `first` then the other game, undo `undo_first` then the other, from `origin` (the
/// user's file, or none), checking each step.
fn two_game_round(origin: Option<&str>, first: &str, undo_first: &str) {
    let other = |app: &str| if app == APP { OTHER_APP } else { APP };
    let label = format!(
        "{}-{}-{}",
        origin.map_or("none", |o| if o.ends_with('\n') { "file" } else { "nonl" }),
        &first[..4],
        &undo_first[..4]
    );
    let paths = fresh(&format!("two-{label}"));
    if let Some(text) = origin {
        seed_file(&paths, text);
    }
    let file = user_settings_path(&paths.home);
    turn_on(&paths, first, &recipe_text(first, "exe: First.exe\n")).unwrap();
    let after_first = read(&paths);
    let second = other(first);
    turn_on(&paths, second, &recipe_text(second, "exe: Second.exe\n")).unwrap();
    let both = read(&paths);
    assert!(both.starts_with(&after_first), "{label}");
    let block_undone = block_of(&both, undo_first);
    let kept = other(undo_first);
    let block_kept = block_of(&both, kept);

    undo_via_cli(&paths, undo_first).unwrap();
    let after_one = read(&paths);
    // Catches: undo restoring a whole-file snapshot, which drops the other game's block or
    // brings back one already removed, instead of removing only its own block.
    assert_eq!(after_one, both.replacen(&block_undone, "", 1), "{label}");
    assert_eq!(block_of(&after_one, kept), block_kept, "{label}");
    // Catches: the other game stranded without its undo record.
    assert!(record_path(&paths, kept).exists(), "{label}");
    assert!(!record_path(&paths, undo_first).exists(), "{label}");

    undo_via_cli(&paths, kept).unwrap();
    match origin {
        // Catches: the original bytes not coming back once the last block leaves.
        Some(text) => assert_eq!(fs::read(&file).unwrap(), text.as_bytes(), "{label}"),
        // Catches: the file we created, or its folders, left behind after the last undo.
        None => {
            assert!(!file.exists(), "{label}");
            assert!(!paths.home.join(".config").exists(), "{label}");
        }
    }
    // Catches: a record or the shared backup left behind after both undos.
    let left: Vec<_> = fs::read_dir(paths.data.join("undo"))
        .map(|dir| dir.map(|e| e.unwrap().file_name()).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "{label}: {left:?}");
}

#[test]
fn two_games_undo_in_every_order_removes_only_their_own_blocks() {
    for origin in [Some(EXISTING), Some("user_settings = {}"), None] {
        for first in [APP, OTHER_APP] {
            for undo_first in [APP, OTHER_APP] {
                two_game_round(origin, first, undo_first);
            }
        }
    }
}

#[test]
fn a_block_with_no_undo_record_is_never_already_there() {
    let paths = fresh("orphan");
    seed_file(&paths, EXISTING);
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    turn_on(&paths, APP, &text).unwrap();
    let with_block = read(&paths);
    // Same block, record gone: a later undo could not remove it.
    fs::remove_file(record_path(&paths, APP)).unwrap();
    let err = turn_on(&paths, APP, &text).unwrap_err();
    // Catches: apply answering AlreadyThere for a matching block without checking for an
    // undo record, leaving a block nothing can undo.
    assert!(err.starts_with("refused: "), "{err}");
    assert!(err.contains("no undo record"), "{err}");
    assert_eq!(read(&paths), with_block);
    assert!(!record_path(&paths, APP).exists());

    // Another game's block with no record: its origin is unknown, so the mismatch is
    // reported and nothing is written.
    let paths = fresh("orphan-other");
    seed_file(&paths, EXISTING);
    turn_on(
        &paths,
        OTHER_APP,
        &recipe_text(OTHER_APP, "exe: Game.exe\n"),
    )
    .unwrap();
    fs::remove_file(record_path(&paths, OTHER_APP)).unwrap();
    let before = read(&paths);
    let err = turn_on(&paths, APP, &text).unwrap_err();
    // Catches: a second game taking a fresh "original" from a file that already holds a
    // block of ours, so its undo would put that block back.
    assert!(err.starts_with("refused: "), "{err}");
    assert_eq!(read(&paths), before);
    assert!(!record_path(&paths, APP).exists());
}

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
    let mut runner = wivrn_runner();
    let result = ensure_applied_in(
        &game,
        &recipe,
        &recipes::route_settings(&text),
        &paths,
        &mut runner,
        &BTreeMap::new(),
    );
    // Catches: a Steam recipe through the same path writing the Epic file.
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
    // Catches: the Steam launch-option wrapper no longer added by the launch command.
    assert!(options.contains(&paths.wrapper_bin().to_string_lossy().into_owned()));
    assert!(options.contains("%command%"));
    assert_eq!(read(&paths), EXISTING);
}

#[test]
fn unsafe_app_names_are_refused_and_write_nothing() {
    let paths = fresh("unsafe");
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    for name in ["../x", "/", "a\0b", "", "a\nb", "a\\b", ".."] {
        let err = turn_on(&paths, name, &text).unwrap_err();
        // Catches: an unsafe app name reaching the writer or the undo path.
        assert!(err.starts_with("refused: "), "{name:?}: {err}");
        assert!(undo_via_cli(&paths, name).is_err(), "{name:?}");
    }
    // Catches: anything written (file, record, data folder) for a refused name.
    assert!(!user_settings_path(&paths.home).exists());
    assert!(!user_settings_path(&paths.home).parent().unwrap().exists());
    assert!(!paths.data.join("undo").exists());
    assert!(!paths.data.exists());
}

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

    let mut fake = wivrn_runner();
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
    // Catches: the launch add command for an epic: key going to Heroic or finding no recipe
    // instead of the user_settings hook.
    assert_eq!(value["options"], format!("epic:{APP}"));
    assert_eq!(count(&read(&paths), &begin_marker(APP)), 1);
    // Catches: WiVRn detection run on a runner other than the command's own.
    assert!(fake
        .ran
        .iter()
        .any(|argv| argv.iter().any(|a| a == "--show-location")));

    undo_via_cli(&paths, APP).unwrap();
    // Catches: launch remove not undoing the block.
    assert_eq!(read(&paths), EXISTING);
}

#[test]
fn wivrn_not_found_writes_nothing() {
    let paths = fresh("no-wivrn");
    seed_file(&paths, EXISTING);
    let text = recipe_text(APP, "exe: AlanWake2.exe\n");
    let recipe = recipes::parse_recipe_keyed(StoreKey::Epic(APP.into()), &text).unwrap();
    let game = Game {
        appid: 0,
        name: format!("epic:{APP}"),
        install_dir: PathBuf::new(),
    };
    let err = ensure_applied_in(
        &game,
        &recipe,
        &recipes::route_settings(&text),
        &paths,
        &mut FakeRunner::new(),
        &BTreeMap::new(),
    )
    .unwrap_err();
    // Catches: the block written with guessed WiVRn folders when detect_server finds none.
    assert!(err.starts_with("not-found: "), "{err}");
    assert_eq!(read(&paths), EXISTING);
    assert!(!paths.data.join("undo").exists());
}
