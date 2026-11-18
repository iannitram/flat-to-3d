//! Locked test for issue #128 (uninstall half): install then uninstall on a temp HOME
//! leaves the HOME tree exactly as it was, folders included.

use flat3d_inject::{install, testing::FakeRunner};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn setup(name: &str, existing: &[(&str, &str)]) -> (PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("install_leaves_nothing")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let home = root.join("home");
    let src = root.join("src");
    fs::create_dir_all(&home).unwrap();
    for (path, data) in [
        ("flat3d-inject", "inject"),
        ("flat3d-run", "run"),
        ("flat3d-viewer", "viewer"),
        ("tools/adb", "adb"),
        ("tools/lib/libusb.so", "lib"),
        ("recipes/1-a.recipe", "recipe"),
        ("libflat3d_ui_layer.so", "layer"),
        ("VkLayer_FLAT3D_ui.json", "manifest"),
    ] {
        let file = src.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, data).unwrap();
    }
    for (path, data) in existing {
        let file = home.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, data).unwrap();
    }
    (home, src)
}

/// Every folder and file under `dir`, relative, with file contents ("<dir>" for folders).
fn snapshot(dir: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(base).unwrap().display().to_string();
            if path.is_dir() {
                out.insert(rel, "<dir>".to_string());
                walk(base, &path, out);
            } else {
                out.insert(rel, fs::read_to_string(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

fn round_trip(home: &Path, src: &Path) {
    let mut p = install::install_plan(home.to_str().unwrap(), src.to_str().unwrap());
    let no_rule = home.parent().unwrap().join("etc/70-flat3d-quest.rules");
    p.udev_rule = no_rule.display().to_string();
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    assert!(home.join(".local/bin/flat3d-inject").exists());
    let mut u = install::uninstall_plan(home.to_str().unwrap());
    u.udev_rule = no_rule.display().to_string();
    install::execute_uninstall(&u, &mut FakeRunner::new()).unwrap();
}

#[test]
fn uninstall_on_a_fresh_home_leaves_an_empty_home() {
    let (home, src) = setup("fresh", &[]);
    round_trip(&home, &src);
    assert_eq!(snapshot(&home), BTreeMap::new());
}

#[test]
fn uninstall_leaves_the_users_other_files_and_folders_as_they_were() {
    let (home, src) = setup(
        "shared",
        &[
            (".config/unrelated.conf", "keep config"),
            (".config/systemd/user/other.service", "keep unit"),
            (".local/bin/mytool", "keep tool"),
            (".local/share/other/data.txt", "keep data"),
            (".local/share/flat3d/state.json", "keep state"),
        ],
    );
    let before = snapshot(&home);
    round_trip(&home, &src);
    assert_eq!(snapshot(&home), before);
}

#[test]
fn uninstall_keeps_a_shared_folder_that_still_holds_a_file_of_the_users() {
    let (home, src) = setup(
        "partial-share",
        &[(".config/systemd/user/other.service", "x")],
    );
    round_trip(&home, &src);
    let after = snapshot(&home);
    let expected: BTreeMap<String, String> = [
        (".config", "<dir>"),
        (".config/systemd", "<dir>"),
        (".config/systemd/user", "<dir>"),
        (".config/systemd/user/other.service", "x"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    assert_eq!(after, expected);
}
