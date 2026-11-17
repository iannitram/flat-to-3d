use flat3d_inject::catalog;
use flat3d_inject::heroic;
use flat3d_inject::recipes::StoreKey;
use flat3d_inject::steam;
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn control_seed() -> String {
    let path = manifest_dir().join("tests/fixtures/recipe-seeds/870780-control.recipe");
    fs::read_to_string(&path).expect("fixture recipe must exist")
}

fn fresh_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("heroic-scan-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_heroic_install_lists_alan_wake_2() {
    let dir = fresh_dir("a_heroic_install_lists_alan_wake_2");
    let home = dir.join("home");
    let config_root = home.join(heroic::FLATPAK_CONFIG);
    let legendary_dir = config_root.join("legendaryConfig/legendary");
    fs::create_dir_all(&legendary_dir).unwrap();

    let install_dir = dir.join("games/AlanWake2");
    fs::create_dir_all(&install_dir).unwrap();

    let json_text = format!(
        r#"{{
  "dc9d2e595d0e4650b35d659f90d41059": {{
    "app_name": "dc9d2e595d0e4650b35d659f90d41059",
    "title": "Alan Wake 2",
    "install_path": "{}",
    "executable": "AlanWake2.exe",
    "platform": "Windows",
    "is_dlc": false
  }},
  "AlanWake2DLC1": {{
    "app_name": "AlanWake2DLC1",
    "title": "Alan Wake 2 - Night Springs",
    "install_path": "{}",
    "executable": "AlanWake2.exe",
    "platform": "Windows",
    "is_dlc": true
  }}
}}"#,
        install_dir.display(),
        install_dir.display(),
    );
    fs::write(legendary_dir.join("installed.json"), json_text).unwrap();

    let games = heroic::installed_games(&home).unwrap();
    assert_eq!(games.len(), 1);
    let game = &games[0];
    assert_eq!(game.app_name, "dc9d2e595d0e4650b35d659f90d41059");
    assert_eq!(game.title, "Alan Wake 2");
    assert_eq!(game.install_dir, install_dir);
    assert_eq!(game.executable, install_dir.join("AlanWake2.exe"));
    assert_eq!(game.config_root, config_root);
    assert_eq!(
        game.key(),
        StoreKey::Epic("dc9d2e595d0e4650b35d659f90d41059".to_string()),
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_scan_matches_its_epic_recipe() {
    let dir = fresh_dir("the_scan_matches_its_epic_recipe");
    let home = dir.join("home");
    let config_root = home.join(heroic::FLATPAK_CONFIG);
    let legendary_dir = config_root.join("legendaryConfig/legendary");
    fs::create_dir_all(&legendary_dir).unwrap();

    let install_dir = dir.join("games/AlanWake2");
    fs::create_dir_all(&install_dir).unwrap();

    let json_text = format!(
        r#"{{
  "dc9d2e595d0e4650b35d659f90d41059": {{
    "app_name": "dc9d2e595d0e4650b35d659f90d41059",
    "title": "Alan Wake 2",
    "install_path": "{}",
    "executable": "AlanWake2.exe",
    "platform": "Windows",
    "is_dlc": false
  }}
}}"#,
        install_dir.display(),
    );
    fs::write(legendary_dir.join("installed.json"), json_text).unwrap();

    let recipes_dir = dir.join("recipes");
    fs::create_dir_all(&recipes_dir).unwrap();

    let alan_wake_recipe = "name: Alan Wake 2\n\
         stereo-status: untested\n\
         stereo-route: custom\n\
         stereo-engine: northlight\n\
         stereo-api: dx12\n\
         stereo-mod: none\n\
         stereo-free: yes\n\
         stereo-safe: yes\n\
         stereo-checked: 2026-11-17\n\
         store-key: epic:dc9d2e595d0e4650b35d659f90d41059\n";
    fs::write(
        recipes_dir.join("epic-alan-wake-2.recipe"),
        alan_wake_recipe,
    )
    .unwrap();
    fs::write(recipes_dir.join("870780-control.recipe"), control_seed()).unwrap();

    let library = catalog::scan_libraries(&home).unwrap();
    assert_eq!(library.epic.len(), 1);
    assert_eq!(library.epic[0].app_name, "dc9d2e595d0e4650b35d659f90d41059");

    let missing_dir = dir.join("nonexistent_recipes");
    let matched =
        catalog::epic_recipes(&library.epic, &[recipes_dir.clone(), missing_dir]).unwrap();
    assert_eq!(matched.len(), 1);
    let recipe = matched.get("dc9d2e595d0e4650b35d659f90d41059").unwrap();
    assert_eq!(recipe.name, "Alan Wake 2");
    assert_eq!(recipe.status, "untested");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn no_heroic_gives_steam_only() {
    let dir = fresh_dir("no_heroic_gives_steam_only");
    let home = dir.join("home");
    let steam_root = home.join(steam::DEFAULT_STEAM_ROOT);
    let steamapps = steam_root.join("steamapps");
    fs::create_dir_all(&steamapps).unwrap();

    let vdf = format!(
        "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t\t\"label\"\t\t\"\"\n\t}}\n}}\n",
        steam_root.display(),
    );
    fs::write(steamapps.join("libraryfolders.vdf"), vdf).unwrap();

    let manifest = "\"AppState\"\n{\n\t\"appid\"\t\t\"870780\"\n\t\"Universe\"\t\t\"1\"\n\t\"name\"\t\t\"Control\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Control\"\n}\n";
    fs::write(steamapps.join("appmanifest_870780.acf"), manifest).unwrap();

    let library = catalog::scan_libraries(&home).unwrap();
    assert!(library.epic.is_empty());
    assert_eq!(library.steam.len(), 1);
    assert_eq!(library.steam[0].appid, 870780);
    assert_eq!(library.steam[0].name, "Control");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_flatpak_path_is_rewritten_only_when_the_host_copy_exists() {
    let dir = fresh_dir("a_flatpak_path_is_rewritten_only_when_the_host_copy_exists");
    let home = dir.join("home");

    // Case 1: path exists as written -> kept as written
    let game1_dir = home.join("Games/game1");
    fs::create_dir_all(&game1_dir).unwrap();
    let result1 = heroic::host_path(&home, &game1_dir);
    assert_eq!(result1, game1_dir);

    // Case 2: path does not exist as written, but host copy exists under .var/app/<FLATPAK_ID>/<rest> -> rewritten
    let host_copy = home
        .join(".var/app")
        .join(heroic::FLATPAK_ID)
        .join("Games/game2");
    fs::create_dir_all(&host_copy).unwrap();
    let written2 = home.join("Games/game2");
    assert!(!written2.exists());
    let result2 = heroic::host_path(&home, &written2);
    assert_eq!(result2, host_copy);

    // Case 3: neither written path nor host copy exists -> kept as written
    let written3 = home.join("Games/nonexistent");
    assert!(!written3.exists());
    let result3 = heroic::host_path(&home, &written3);
    assert_eq!(result3, written3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_broken_installed_json_is_an_error() {
    let dir = fresh_dir("a_broken_installed_json_is_an_error");
    let home = dir.join("home");
    let config_root = home.join(heroic::FLATPAK_CONFIG);
    let legendary_dir = config_root.join("legendaryConfig/legendary");
    fs::create_dir_all(&legendary_dir).unwrap();

    fs::write(
        legendary_dir.join("installed.json"),
        "broken non-json content",
    )
    .unwrap();

    let err = heroic::installed_games(&home).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    let msg = err.to_string();
    assert!(
        msg.contains("installed.json"),
        "error message should name installed.json, got: {msg}",
    );

    let scan_err = catalog::scan_libraries(&home).unwrap_err();
    assert_eq!(scan_err.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        scan_err.to_string().contains("installed.json"),
        "scan_libraries error should name installed.json, got: {scan_err}",
    );

    let _ = fs::remove_dir_all(&dir);
}
