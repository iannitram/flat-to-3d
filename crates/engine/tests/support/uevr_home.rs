#![allow(dead_code)]

use flat3d_inject::{paths::EnginePaths, steam::Game};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub fn home(case: &str) -> (PathBuf, EnginePaths, Game) {
    let temp = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = temp.join(case);
    let _ = fs::remove_dir_all(&root);
    let home = root.join("home");
    let game_dir = root.join("lib/steamapps/common/G");
    fs::create_dir_all(&game_dir).unwrap();
    let home_string = home.to_string_lossy().into_owned();
    let p = EnginePaths::from_env(&BTreeMap::from([("HOME".into(), home_string)]), 1000).unwrap();
    let game = Game {
        appid: 2075800,
        name: "Zero Company".into(),
        install_dir: game_dir,
    };
    (root, p, game)
}

pub fn uevr_ready(case: &str) -> (PathBuf, EnginePaths, Game) {
    let (root, p, game) = home(case);
    let dir = flat3d_inject::injectors::uevr::uevr_dir(&p);
    fs::create_dir_all(&dir).unwrap();
    for dll in flat3d_inject::injectors::uevr::UEVR_DLLS {
        fs::write(dir.join(dll), "").unwrap();
    }
    let injector = flat3d_inject::injectors::uevr::injector_path(&p);
    fs::create_dir_all(injector.parent().unwrap()).unwrap();
    fs::write(injector, "").unwrap();
    (root, p, game)
}

pub fn uevr_unfetched(case: &str) -> (PathBuf, EnginePaths, Game) {
    let (root, p, game) = home(case);
    let injector = flat3d_inject::injectors::uevr::injector_path(&p);
    fs::create_dir_all(injector.parent().unwrap()).unwrap();
    fs::write(injector, "").unwrap();
    (root, p, game)
}

pub fn uevr_no_injector(case: &str) -> (PathBuf, EnginePaths, Game) {
    let (root, p, game) = home(case);
    let dir = flat3d_inject::injectors::uevr::uevr_dir(&p);
    fs::create_dir_all(&dir).unwrap();
    for dll in flat3d_inject::injectors::uevr::UEVR_DLLS {
        fs::write(dir.join(dll), "").unwrap();
    }
    (root, p, game)
}
