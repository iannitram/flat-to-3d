//! Locked tests for issue #24: `steam::installed_games`. Builds its sample Steam
//! folders in a temp dir, so no fixture files are needed. Linux paths.

use flat3d_inject::steam::{Game, installed_games};
use std::fs;
use std::path::{Path, PathBuf};

fn fresh_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("flat3d-steam-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn manifest(appid: u32, name: &str, installdir: &str, flags: u32) -> String {
    format!(
        "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"Universe\"\t\t\"1\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\t\"{flags}\"\n\t\"installdir\"\t\t\"{installdir}\"\n}}\n"
    )
}

fn add_manifest(library: &Path, appid: u32, name: &str, installdir: &str, flags: u32) {
    let apps = library.join("steamapps");
    fs::create_dir_all(&apps).unwrap();
    fs::write(apps.join(format!("appmanifest_{appid}.acf")), manifest(appid, name, installdir, flags)).unwrap();
}

fn write_vdf(root: &Path, header_comment: bool, libraries: &[&Path]) {
    let mut text = String::new();
    if header_comment {
        text.push_str("// libraryfolders written by the test\n");
    }
    text.push_str("\"libraryfolders\"\n{\n");
    for (i, lib) in libraries.iter().enumerate() {
        text.push_str(&format!("\t\"{i}\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t\t\"label\"\t\t\"\"\n\t}}\n", lib.display()));
    }
    text.push_str("}\n");
    fs::create_dir_all(root.join("steamapps")).unwrap();
    fs::write(root.join("steamapps").join("libraryfolders.vdf"), text).unwrap();
}

#[test]
fn two_libraries_give_two_games_sorted_by_name_ignoring_case() {
    let root = fresh_dir("two-root");
    let sd = fresh_dir("two-sd");
    write_vdf(&root, false, &[&root, &sd]);
    add_manifest(&root, 100, "Beta Game", "Beta Game", 4);
    add_manifest(&sd, 200, "alpha game", "alpha dir", 4);
    let games = installed_games(&root).unwrap();
    assert_eq!(
        games,
        vec![
            Game { appid: 200, name: "alpha game".into(), install_dir: sd.join("steamapps/common/alpha dir") },
            Game { appid: 100, name: "Beta Game".into(), install_dir: root.join("steamapps/common/Beta Game") },
        ]
    );
}

#[test]
fn a_manifest_with_state_flags_2_is_left_out() {
    let root = fresh_dir("flags-root");
    write_vdf(&root, false, &[&root]);
    add_manifest(&root, 1, "Installed", "Installed", 4);
    add_manifest(&root, 2, "Downloading", "Downloading", 2);
    let games = installed_games(&root).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].appid, 1);
}

#[test]
fn a_missing_library_path_is_skipped() {
    let root = fresh_dir("gone-root");
    let gone = root.join("no-such-library");
    write_vdf(&root, false, &[&root, &gone]);
    add_manifest(&root, 7, "Only Game", "Only Game", 4);
    let games = installed_games(&root).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].appid, 7);
}

#[test]
fn a_missing_vdf_gives_an_empty_list() {
    let root = fresh_dir("novdf-root");
    assert_eq!(installed_games(&root).unwrap(), vec![]);
}

#[test]
fn an_apostrophe_in_installdir_is_kept() {
    let root = fresh_dir("apos-root");
    write_vdf(&root, false, &[&root]);
    add_manifest(&root, 2651280, "Marvel's Spider-Man 2", "Marvel's Spider-Man 2", 4);
    let games = installed_games(&root).unwrap();
    assert_eq!(
        games,
        vec![Game {
            appid: 2651280,
            name: "Marvel's Spider-Man 2".into(),
            install_dir: root.join("steamapps/common/Marvel's Spider-Man 2"),
        }]
    );
}

#[test]
fn a_comment_line_in_the_vdf_is_ignored() {
    let root = fresh_dir("comment-root");
    write_vdf(&root, true, &[&root]);
    add_manifest(&root, 9, "Commented", "Commented", 4);
    let games = installed_games(&root).unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].name, "Commented");
}
