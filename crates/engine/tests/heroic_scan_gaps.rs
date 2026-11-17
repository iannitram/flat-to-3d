//! Locked test: the Heroic (Epic) scan behaviours the #366 review found untested. Each test names
//! the mutation it catches. Temp folders under CARGO_TARGET_TMPDIR; no Heroic, no device.

use flat3d_inject::catalog;
use flat3d_inject::heroic;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const RECIPE_KEY: &str = "dc9d2e595d0e4650b35d659f90d41059";

fn fresh_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("heroic-gaps-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Write `<config_root>/legendaryConfig/legendary/installed.json`.
fn write_installed(config_root: &Path, json: &str) {
    let legendary = config_root.join("legendaryConfig/legendary");
    fs::create_dir_all(&legendary).unwrap();
    fs::write(legendary.join("installed.json"), json).unwrap();
}

/// One non-DLC installed.json entry.
fn entry(app_name: &str, title: &str, install_path: &Path) -> String {
    format!(
        r#""{app_name}": {{"app_name": "{app_name}", "title": "{title}", "install_path": "{}", "executable": "Game.exe", "is_dlc": false}}"#,
        install_path.display()
    )
}

fn recipe(name: &str) -> String {
    format!(
        "name: {name}\n\
         stereo-status: untested\n\
         stereo-route: custom\n\
         stereo-engine: northlight\n\
         stereo-api: dx12\n\
         stereo-mod: none\n\
         stereo-free: yes\n\
         stereo-safe: yes\n\
         stereo-checked: 2026-11-17\n\
         store-key: epic:{RECIPE_KEY}\n"
    )
}

#[test]
fn the_earlier_recipe_dir_wins() {
    // Catches: found.entry(app_name).or_insert(recipe) -> found.insert(app_name, recipe)
    // in catalog::epic_recipes (the later dir would win and the name would be "Second").
    let dir = fresh_dir("the_earlier_recipe_dir_wins");
    let home = dir.join("home");
    let install = dir.join("games/AlanWake2");
    fs::create_dir_all(&install).unwrap();
    write_installed(
        &home.join(heroic::FLATPAK_CONFIG),
        &format!("{{{}}}", entry(RECIPE_KEY, "Alan Wake 2", &install)),
    );
    let first = dir.join("recipes-first");
    let second = dir.join("recipes-second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::write(first.join("epic-alan-wake-2.recipe"), recipe("First")).unwrap();
    fs::write(second.join("epic-alan-wake-2.recipe"), recipe("Second")).unwrap();

    let games = heroic::installed_games(&home).unwrap();
    let matched = catalog::epic_recipes(&games, &[first, second]).unwrap();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched.get(RECIPE_KEY).unwrap().name, "First");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_game_in_both_roots_is_listed_once_from_the_native_root() {
    // Catches: the seen.retain dedupe dropped in heroic::installed_games (two games), or the
    // Flatpak root read before the native one (the kept copy would carry the Flatpak config_root).
    let dir = fresh_dir("a_game_in_both_roots_is_listed_once_from_the_native_root");
    let home = dir.join("home");
    let install = dir.join("games/AlanWake2");
    fs::create_dir_all(&install).unwrap();
    let json = format!("{{{}}}", entry(RECIPE_KEY, "Alan Wake 2", &install));
    write_installed(&home.join(heroic::NATIVE_CONFIG), &json);
    write_installed(&home.join(heroic::FLATPAK_CONFIG), &json);

    let games = heroic::installed_games(&home).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].app_name, RECIPE_KEY);
    assert_eq!(games[0].config_root, home.join(heroic::NATIVE_CONFIG));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn games_are_sorted_by_title_ignoring_case() {
    // Catches: the sort_by dropped (installed.json key order puts "Zed Game" first), or a
    // case-sensitive title sort ("Zed Game" sorts before "alpha game" in byte order).
    let dir = fresh_dir("games_are_sorted_by_title_ignoring_case");
    let home = dir.join("home");
    let install = dir.join("games/any");
    fs::create_dir_all(&install).unwrap();
    write_installed(
        &home.join(heroic::FLATPAK_CONFIG),
        &format!(
            "{{{}, {}}}",
            entry("aaa111", "Zed Game", &install),
            entry("bbb222", "alpha game", &install)
        ),
    );

    let titles: Vec<String> = heroic::installed_games(&home)
        .unwrap()
        .into_iter()
        .map(|g| g.title)
        .collect();
    assert_eq!(
        titles,
        vec!["alpha game".to_string(), "Zed Game".to_string()]
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_native_root_is_read_and_its_paths_are_kept_as_written() {
    // Catches: NATIVE_CONFIG not read (no games), or the native root passed is_flatpak = true
    // (its missing install path would be rewritten to the existing Flatpak copy).
    let dir = fresh_dir("the_native_root_is_read_and_its_paths_are_kept_as_written");
    let home = dir.join("home");
    let written = home.join("Games/AlanWake2");
    let flatpak_copy = home
        .join(".var/app")
        .join(heroic::FLATPAK_ID)
        .join("Games/AlanWake2");
    fs::create_dir_all(&flatpak_copy).unwrap();
    assert!(!written.exists());
    write_installed(
        &home.join(heroic::NATIVE_CONFIG),
        &format!("{{{}}}", entry(RECIPE_KEY, "Alan Wake 2", &written)),
    );

    let games = heroic::installed_games(&home).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].config_root, home.join(heroic::NATIVE_CONFIG));
    assert_eq!(games[0].install_dir, written);
    assert_eq!(games[0].executable, written.join("Game.exe"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_flatpak_install_path_is_rewritten_to_the_host_copy_end_to_end() {
    // Catches: the Flatpak root passed is_flatpak = false, or host_path skipped in read_root
    // (install_dir and executable would keep the sandbox path, which does not exist).
    let dir = fresh_dir("a_flatpak_install_path_is_rewritten_to_the_host_copy_end_to_end");
    let home = dir.join("home");
    let written = home.join("Games/AlanWake2");
    let host_copy = home
        .join(".var/app")
        .join(heroic::FLATPAK_ID)
        .join("Games/AlanWake2");
    fs::create_dir_all(&host_copy).unwrap();
    assert!(!written.exists());
    write_installed(
        &home.join(heroic::FLATPAK_CONFIG),
        &format!("{{{}}}", entry(RECIPE_KEY, "Alan Wake 2", &written)),
    );

    let games = heroic::installed_games(&home).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].config_root, home.join(heroic::FLATPAK_CONFIG));
    assert_eq!(games[0].install_dir, host_copy);
    assert_eq!(games[0].executable, host_copy.join("Game.exe"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_installed_json_that_is_not_an_object_is_an_error() {
    // Catches: the as_object() else-branch returning Ok (an array would list no games), or a
    // message that does not end "not a JSON object" or does not name installed.json.
    let dir = fresh_dir("an_installed_json_that_is_not_an_object_is_an_error");
    let home = dir.join("home");
    write_installed(&home.join(heroic::NATIVE_CONFIG), "[]");

    let err = heroic::installed_games(&home).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidData);
    let msg = err.to_string();
    assert!(msg.contains("installed.json"), "got: {msg}");
    assert!(msg.ends_with(": not a JSON object"), "got: {msg}");

    let scan = catalog::scan_libraries(&home).unwrap_err();
    assert_eq!(scan.kind(), ErrorKind::InvalidData);

    let _ = fs::remove_dir_all(&dir);
}
