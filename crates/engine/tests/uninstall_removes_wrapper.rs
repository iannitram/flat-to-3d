//! Uninstall removes launch wrappers recorded by the Decky launch sync.

use flat3d_inject::{install, launch_options, testing::FakeRunner};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn setup(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("uninstall_removes_wrapper")
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
    ] {
        let file = src.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, data).unwrap();
    }
    (root, home, src)
}

fn config(options: &[(u32, &str)]) -> String {
    let mut text = String::from("\"UserLocalConfigStore\" {\n \"Software\" {\n  \"Valve\" {\n   \"Steam\" {\n    \"apps\" {\n");
    for (appid, line) in options {
        text.push_str(&format!(
            "     \"{appid}\" {{\n      \"LaunchOptions\" \"{line}\"\n     }}\n"
        ));
    }
    text.push_str("    }\n   }\n  }\n }\n}\n");
    text
}

fn apply(home: &std::path::Path, appid: u32, before: &str) -> String {
    let applied = launch_options::add_wrapper_at(
        before,
        &home.join(".local/bin/flat3d-run").to_string_lossy(),
    );
    launch_options::save_record(&home.join(".local/share/flat3d"), appid, before, &applied)
        .unwrap();
    applied
}

fn install_home(home: &std::path::Path, src: &std::path::Path) -> String {
    let home = home.to_string_lossy().into_owned();
    let src = src.to_string_lossy().into_owned();
    let mut install_plan = install::install_plan(&home, &src);
    let no_rule = format!("{home}/outside/70-flat3d-quest.rules");
    install_plan.udev_rule = no_rule.clone();
    install::execute(&install_plan, &mut FakeRunner::new()).unwrap();
    no_rule
}

fn uninstall_home(home: &std::path::Path, no_rule: &str) {
    let home = home.to_string_lossy().into_owned();
    let mut uninstall_plan = install::uninstall_plan(&home);
    uninstall_plan.udev_rule = no_rule.to_string();
    install::execute_uninstall(&uninstall_plan, &mut FakeRunner::new()).unwrap();
}

fn files(home: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(home: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(home, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(home).unwrap().display().to_string(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(home, home, &mut out);
    out
}

#[test]
fn uninstall_restores_each_launch_line_and_removes_its_records() {
    // Catches: uninstall leaves flat3d-run in one or more games' Steam launch options.
    let (_root, home, src) = setup("two-games");
    let no_rule = install_home(&home, &src);
    let config_path = home.join(".local/share/Steam/userdata/1/config/localconfig.vdf");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    let before = ["", "PROTON_LOG=1   %command%  -nolauncher"];
    let applied = [apply(&home, 11, before[0]), apply(&home, 22, before[1])];
    fs::write(
        &config_path,
        config(&[(11, &applied[0]), (22, &applied[1])]),
    )
    .unwrap();

    uninstall_home(&home, &no_rule);

    let after = fs::read_to_string(&config_path).unwrap();
    assert!(after.contains("\"LaunchOptions\" \"\""));
    assert!(after.contains(&format!("\"LaunchOptions\" \"{}\"", before[1])));
    assert!(!after.contains("flat3d-run"));
    assert!(!home.join(".local/share/flat3d/launch-options").exists());
    assert!(home
        .join(".local/share/Steam/userdata/1/config/localconfig.vdf")
        .exists());
    assert!(!home.join(".local/bin/flat3d-inject").exists());
    assert!(!home.join(".local/bin/flat3d-run").exists());
    assert!(!home.join(".local/share/flat3d/installed.txt").exists());
    assert_eq!(
        files(&home),
        BTreeMap::from([(
            ".local/share/Steam/userdata/1/config/localconfig.vdf".into(),
            fs::read(&config_path).unwrap(),
        )])
    );
}

#[test]
fn uninstall_without_recorded_games_still_cleans_install_files() {
    // Catches: uninstall assumes a launch record exists and fails on an unused install.
    let (_root, home, src) = setup("no-records");
    let no_rule = install_home(&home, &src);
    uninstall_home(&home, &no_rule);
    assert!(!home.join(".local/bin/flat3d-inject").exists());
    assert!(!home.join(".local/share/flat3d").exists());
}

#[test]
fn missing_game_entry_is_skipped_and_its_record_is_removed() {
    // Catches: uninstall panics or leaves stale records when a game disappears from localconfig.
    let (_root, home, src) = setup("removed-game");
    let no_rule = install_home(&home, &src);
    let config_path = home.join(".local/share/Steam/userdata/1/config/localconfig.vdf");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    let applied = apply(&home, 31, "%command% -safe");
    fs::write(&config_path, config(&[])).unwrap();

    uninstall_home(&home, &no_rule);

    assert!(!fs::read_to_string(&config_path).unwrap().contains(&applied));
    assert!(!home
        .join(".local/share/flat3d/launch-options/31.json")
        .exists());
}

#[test]
fn changed_launch_line_keeps_user_options_and_only_removes_the_wrapper() {
    // Catches: uninstall overwrites launch options edited after the wrapper was applied.
    let (_root, home, src) = setup("edited-line");
    let no_rule = install_home(&home, &src);
    let config_path = home.join(".local/share/Steam/userdata/1/config/localconfig.vdf");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    let applied = apply(&home, 41, "PROTON_LOG=1 %command%");
    let edited = format!("{applied} -new-option");
    fs::write(&config_path, config(&[(41, &edited)])).unwrap();

    uninstall_home(&home, &no_rule);

    let after = fs::read_to_string(&config_path).unwrap();
    assert!(after.contains("\"LaunchOptions\" \"PROTON_LOG=1 %command% -new-option\""));
    assert!(!after.contains("flat3d-run"));
}
