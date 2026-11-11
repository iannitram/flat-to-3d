#![allow(dead_code)]

use flat3d_inject::api::{LinkSnapshot, LinkState};
use flat3d_inject::paths::EnginePaths;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "\
\"AppState\" {\n\
    \"appid\" \"{appid}\"\n\
    \"name\" \"{name}\"\n\
    \"StateFlags\" \"4\"\n\
    \"installdir\" \"{dir}\"\n\
}\n";

fn fresh(case: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("flat3d-scenes-{}-{case}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let home = root.join("home");
    let rt = root.join("rt");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&rt).unwrap();
    (root, home, rt)
}

fn paths(home: &Path, rt: &Path) -> EnginePaths {
    let env = BTreeMap::from([
        ("HOME".to_string(), home.to_string_lossy().into_owned()),
        (
            "XDG_RUNTIME_DIR".to_string(),
            rt.to_string_lossy().into_owned(),
        ),
    ]);
    EnginePaths::from_env(&env, 1000).unwrap()
}

pub fn no_steam(case: &str) -> EnginePaths {
    let (_root, home, rt) = fresh(case);
    paths(&home, &rt)
}

fn two_libs(case: &str, old_vdf: bool) -> EnginePaths {
    let (root, home, rt) = fresh(case);
    let steam = home.join(".local/share/Steam");
    let lib2 = root.join("lib2");
    for library in [&steam, &lib2] {
        fs::create_dir_all(library.join("steamapps/common")).unwrap();
    }
    add_game(&steam, 100, "Beta Game", "Beta Game");
    add_game(&lib2, 200, "alpha game", "alpha dir");
    let vdf = if old_vdf {
        format!(
            "\"libraryfolders\" {{\n\t\"0\" \"{}\"\n\t\"1\" \"{}\"\n}}\n",
            steam.display(),
            lib2.display()
        )
    } else {
        format!(
            "\"libraryfolders\" {{\n\t\"0\" {{ \"path\" \"{}\" }}\n\t\"1\" {{ \"path\" \"{}\" }}\n}}\n",
            steam.display(),
            lib2.display()
        )
    };
    fs::write(steam.join("steamapps/libraryfolders.vdf"), vdf).unwrap();
    paths(&home, &rt)
}

pub fn two_libs_new_vdf(case: &str) -> EnginePaths {
    two_libs(case, false)
}

pub fn two_libs_old_vdf(case: &str) -> EnginePaths {
    two_libs(case, true)
}

fn add_game(library: &Path, appid: u32, name: &str, dir: &str) {
    fs::write(
        library.join(format!("steamapps/appmanifest_{appid}.acf")),
        MANIFEST
            .replace("{appid}", &appid.to_string())
            .replace("{name}", name)
            .replace("{dir}", dir),
    )
    .unwrap();
}

pub fn link_ready_fresh(case: &str) -> EnginePaths {
    let paths = no_steam(case);
    let mut snapshot = LinkSnapshot::stopped(flat3d_inject::sys::unix_now());
    snapshot.state = LinkState::Ready;
    snapshot.message = "Link is ready.".into();
    snapshot.updated = flat3d_inject::sys::unix_now();
    write_snapshot(&paths, &snapshot);
    paths
}

pub fn link_stale(case: &str) -> EnginePaths {
    let paths = no_steam(case);
    let mut snapshot = LinkSnapshot::stopped(flat3d_inject::sys::unix_now().saturating_sub(11));
    snapshot.state = LinkState::Ready;
    snapshot.message = "Link is ready.".into();
    snapshot.updated = flat3d_inject::sys::unix_now().saturating_sub(11);
    write_snapshot(&paths, &snapshot);
    paths
}

pub fn write_recipe(paths: &EnginePaths, file_name: &str, text: &str) {
    let recipes = &paths.recipe_dirs[0];
    fs::create_dir_all(recipes).unwrap();
    fs::write(recipes.join(file_name), text).unwrap();
}

pub fn write_snapshot(paths: &EnginePaths, snapshot: &LinkSnapshot) {
    fs::create_dir_all(&paths.state_dir).unwrap();
    let path = PathBuf::from(flat3d_inject::state::snapshot_path(
        &paths.state_dir.to_string_lossy(),
    ));
    fs::write(path, serde_json::to_vec(snapshot).unwrap()).unwrap();
}
