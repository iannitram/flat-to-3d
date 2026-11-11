use flat3d_inject::{api::ERROR_CODES, injectors, paths::EnginePaths, testing::FakeRunner, undo};
use std::{fs, path::PathBuf};

#[path = "support/uevr_home.rs"]
mod uevr_home;

const APPID: u32 = 2075800;
fn recipe(profile: Option<&str>, safe: &str) -> String {
    format!("name: Game\nstereo-status: works-now\nstereo-route: uevr\nstereo-engine: custom\nstereo-api: unknown\nstereo-mod: none\nstereo-free: yes\nstereo-safe: {safe}\nstereo-checked: 2026-11-11\nuevr-exe: Game\n{}", profile.map(|s| format!("uevr-profile: {s}\n")).unwrap_or_default())
}
fn setup(case: &str, cached: bool, profile: Option<&str>, safe: &str) -> (PathBuf, EnginePaths) {
    let (root, mut p, _game) = uevr_home::uevr_ready(case);
    let library = root.join("lib");
    let steam = p.home.join(flat3d_inject::steam::DEFAULT_STEAM_ROOT);
    fs::create_dir_all(steam.join("steamapps")).unwrap();
    fs::write(
        steam.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{\n \"0\" {{ \"path\" \"{}\" }}\n}}\n",
            library.display()
        ),
    )
    .unwrap();
    fs::write(library.join("steamapps/appmanifest_2075800.acf"), "\"AppState\" {\n \"appid\" \"2075800\"\n \"name\" \"Game\"\n \"StateFlags\" \"4\"\n \"installdir\" \"G\"\n}\n").unwrap();
    let recipes = root.join("recipes");
    fs::create_dir_all(&recipes).unwrap();
    fs::write(recipes.join("2075800-game.recipe"), recipe(profile, safe)).unwrap();
    p.recipe_dirs = vec![recipes];
    if cached {
        let pin = flat3d_inject::injectors::fetch::pin("uevr").unwrap();
        let dir = flat3d_inject::injectors::fetch::cache_dir(&pin, &p);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(flat3d_inject::injectors::fetch::MARKER),
            format!("{}\n", pin.sha256),
        )
        .unwrap();
    }
    (root, p)
}

#[test]
fn inject_is_idempotent_and_never_touches_steam_or_launches() {
    let (_root, p) = setup("inject-repeat", true, None, "yes");
    let steam_config = p
        .home
        .join(flat3d_inject::steam::DEFAULT_STEAM_ROOT)
        .join("userdata/1/config/localconfig.vdf");
    fs::create_dir_all(steam_config.parent().unwrap()).unwrap();
    fs::write(&steam_config, b"untouched\n").unwrap();
    let before = fs::read(&steam_config).unwrap();
    let mut runner = FakeRunner::new();
    let first = injectors::inject(&p, APPID, &mut runner).unwrap();
    assert!(!first.launched);
    let record_path = undo::record_path(&p.data, APPID);
    assert!(record_path.exists());
    let record = undo::UndoRecord::from_text(&fs::read_to_string(&record_path).unwrap()).unwrap();
    assert_eq!(record.launch_options_before, "");
    assert!(fs::read_to_string(p.env_file(APPID))
        .unwrap()
        .starts_with("flat3d-env 1\nroute\tuevr\n"));
    assert_eq!(fs::read(&steam_config).unwrap(), before);
    assert!(runner.ran.is_empty());
    let second = injectors::inject(&p, APPID, &mut runner).unwrap();
    assert!(second.summary.ends_with("already set up"));
    assert!(runner.ran.is_empty());
    assert!(runner.ran.iter().all(|argv| !argv
        .first()
        .is_some_and(|s| s.contains("steam") || s.contains("Game"))));
    assert_eq!(fs::read(&steam_config).unwrap(), before);
}

#[test]
fn missing_profile_pin_and_download_failure_leave_prefix_untouched() {
    let (_root, p) = setup(
        "inject-pin-missing",
        true,
        Some("missing-profile-key"),
        "yes",
    );
    let mut runner = FakeRunner::new();
    let err = injectors::inject(&p, APPID, &mut runner).unwrap_err();
    assert_eq!(err.code, "not-found");
    assert!(err.message.contains("missing-profile-key"));
    assert!(!undo::record_path(&p.data, APPID).exists());
    assert!(!p.env_file(APPID).exists());

    let (_root, p) = setup("inject-download-fail", false, None, "yes");
    let mut runner = FakeRunner::new().fail(&["curl"], "fixture failure");
    let err = injectors::inject(&p, APPID, &mut runner).unwrap_err();
    assert_eq!(err.code, "command-failed");
    assert!(!undo::record_path(&p.data, APPID).exists());
    assert!(!p.env_file(APPID).exists());
    let pin = flat3d_inject::injectors::fetch::pin("uevr").unwrap();
    assert!(!flat3d_inject::injectors::fetch::cache_dir(&pin, &p).exists());
}

#[test]
fn env_write_failure_rolls_back_and_bad_or_missing_recipe_is_clean() {
    let (_root, p) = setup("inject-env-fail", true, None, "yes");
    fs::create_dir_all(&p.data).unwrap();
    fs::write(p.data.join("env"), "file blocks directory").unwrap();
    let mut runner = FakeRunner::new();
    let err = injectors::inject(&p, APPID, &mut runner).unwrap_err();
    assert_eq!(err.code, "io");
    assert!(!undo::record_path(&p.data, APPID).exists());
    let marker = root
        .join("lib/steamapps/compatdata")
        .join(APPID.to_string())
        .join("pfx/drive_c/users/steamuser/AppData/Roaming/UnrealVRMod/Game")
        .join(flat3d_inject::injectors::uevr::MARKER_FILE);
    assert!(!marker.exists());

    let (_root, p) = setup("inject-unsafe", true, None, "no");
    assert_eq!(
        injectors::inject(&p, APPID, &mut FakeRunner::new())
            .unwrap_err()
            .code,
        "refused"
    );
    let (_root, mut p) = setup("inject-no-recipe", true, None, "yes");
    p.recipe_dirs.clear();
    let err = injectors::inject(&p, APPID, &mut FakeRunner::new()).unwrap_err();
    assert_eq!(err.code, "not-found");
    assert!(err.message.contains("no recipe"));
    let (root, p) = setup("inject-not-installed", true, None, "yes");
    fs::remove_file(root.join("lib/steamapps/appmanifest_2075800.acf")).unwrap();
    let err = injectors::inject(&p, APPID, &mut FakeRunner::new()).unwrap_err();
    assert_eq!(err.code, "not-found");
    assert!(ERROR_CODES.contains(&err.code.as_str()));
}
