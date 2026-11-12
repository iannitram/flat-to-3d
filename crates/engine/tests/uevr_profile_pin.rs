//! The `uevr-profile` pin goes into the prefix (#275; layout rules ported from Kaon, see
//! `injectors/uevr_profile.rs`).

#[path = "support/uevr_home.rs"]
mod uevr_home;

use flat3d_inject::injectors::{fetch, uevr, uevr_profile};
use flat3d_inject::recipes::RouteSettings;
use flat3d_inject::undo;
use std::fs;
use std::path::{Path, PathBuf};

const STEM: &str = "Ace7Game";

fn settings(extra: &[(&str, &str)]) -> RouteSettings {
    let mut s: RouteSettings = [("uevr-exe", STEM), ("uevr-2d-screen", "yes")]
        .iter()
        .map(|(k, v)| ((*k).into(), (*v).into()))
        .collect();
    for (k, v) in extra {
        s.insert((*k).into(), (*v).into());
    }
    s
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A fetch cache folder as `fetch` leaves it: the unpacked zip plus the fetch marker.
fn cache(root: &Path, files: &[(&str, &str)]) -> PathBuf {
    let dir = root.join("profile-cache");
    for (rel, text) in files {
        write(&dir.join(rel), text);
    }
    write(&dir.join(fetch::MARKER), "sha\n");
    dir
}

fn profile_dir(root: &Path) -> PathBuf {
    uevr::profile_dir(&root.join("lib/steamapps/compatdata"), 2075800, STEM)
}

fn files_under(dir: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).unwrap();
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn a_wrapped_profile_is_installed_merged_and_undone() {
    let (root, p, game) = uevr_home::uevr_ready("profile-pin-wrapped");
    let fetched = cache(
        &root,
        &[
            (
                "Ace7Game/config.txt",
                "VR_RenderingMethod=2\r\nUI_Distance=2.0\r\n",
            ),
            ("Ace7Game/plugins/ace_combat_plugin.dll", "dll"),
        ],
    );
    let (applied, _env) =
        uevr::apply_with_profile(&game, &settings(&[]), &p, Some(&fetched)).unwrap();
    let dir = profile_dir(&root);
    assert_eq!(
        fs::read_to_string(dir.join("plugins/ace_combat_plugin.dll")).unwrap(),
        "dll"
    );
    let config = fs::read_to_string(dir.join(uevr::PROFILE_FILE)).unwrap();
    assert!(config.starts_with("VR_RenderingMethod=0\r\nUI_Distance=2.0\r\n"));
    assert!(config.contains("VR_2DScreenMode=true\r\n"));
    assert!(config.contains("Frontend_RequestedRuntime=openxr_loader.dll\r\n"));
    assert!(!config.contains("VR_RenderingMethod=2"));
    assert_eq!(
        applied.record.created,
        vec![
            dir.join(uevr::MARKER_FILE),
            dir.join("plugins/ace_combat_plugin.dll"),
            dir.join(uevr::PROFILE_FILE),
        ]
    );
    undo::put_back(&applied.record);
    assert!(files_under(&dir).is_empty());
}

#[test]
fn prefix_files_a_profile_replaces_are_backed_up_and_put_back() {
    let (root, p, game) = uevr_home::uevr_ready("profile-pin-replace");
    let dir = profile_dir(&root);
    write(&dir.join("config.txt"), "mine\n");
    write(&dir.join("plugins/ace_combat_plugin.dll"), "old dll");
    write(&dir.join("plugins/other.dll"), "other");
    let fetched = cache(
        &root,
        &[
            ("config.txt", "UI_Distance=3.0\n"),
            ("plugins/ace_combat_plugin.dll", "new dll"),
        ],
    );
    let (applied, _env) =
        uevr::apply_with_profile(&game, &settings(&[]), &p, Some(&fetched)).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join("plugins/ace_combat_plugin.dll")).unwrap(),
        "new dll"
    );
    assert!(fs::read_to_string(dir.join("config.txt"))
        .unwrap()
        .starts_with("UI_Distance=3.0\n"));
    assert_eq!(applied.record.replaced.len(), 2);
    undo::put_back(&applied.record);
    assert_eq!(
        files_under(&dir),
        vec![
            "config.txt",
            "plugins/ace_combat_plugin.dll",
            "plugins/other.dll"
        ]
    );
    assert_eq!(
        fs::read_to_string(dir.join("config.txt")).unwrap(),
        "mine\n"
    );
    assert_eq!(
        fs::read_to_string(dir.join("plugins/ace_combat_plugin.dll")).unwrap(),
        "old dll"
    );
}

#[test]
fn a_failed_copy_leaves_the_prefix_as_it_was() {
    let (root, p, game) = uevr_home::uevr_ready("profile-pin-fail");
    let dir = profile_dir(&root);
    write(&dir.join("plugins/a.dll"), "old a");
    // A folder where a profile file goes stops the copy after a.dll was replaced.
    fs::create_dir_all(dir.join("plugins/b.dll")).unwrap();
    let fetched = cache(
        &root,
        &[
            ("config.txt", "A=1\n"),
            ("plugins/a.dll", "new a"),
            ("plugins/b.dll", "new b"),
        ],
    );
    let err = uevr::apply_with_profile(&game, &settings(&[]), &p, Some(&fetched)).unwrap_err();
    assert!(err.starts_with("io: "), "{err}");
    assert_eq!(files_under(&dir), vec!["plugins/a.dll"]);
    assert_eq!(
        fs::read_to_string(dir.join("plugins/a.dll")).unwrap(),
        "old a"
    );
}

#[test]
fn a_folder_without_config_or_with_a_nested_zip_is_refused_before_any_write() {
    let (root, p, game) = uevr_home::uevr_ready("profile-pin-refused");
    let no_config = cache(&root, &[("Ace7Game/plugins/x.dll", "dll")]);
    let err = uevr::apply_with_profile(&game, &settings(&[]), &p, Some(&no_config)).unwrap_err();
    assert!(
        err.starts_with("refused: ") && err.contains("no config.txt"),
        "{err}"
    );
    assert!(!root.join("lib/steamapps/compatdata").exists());

    let nested = root.join("nested-cache");
    write(&nested.join("Ace7Game.zip"), "zip");
    write(&nested.join(fetch::MARKER), "sha\n");
    let err = uevr::apply_with_profile(&game, &settings(&[]), &p, Some(&nested)).unwrap_err();
    assert!(
        err.starts_with("refused: ") && err.contains("zip inside a zip"),
        "{err}"
    );
    assert!(!root.join("lib/steamapps/compatdata").exists());
}

#[cfg(unix)]
#[test]
fn backslash_names_from_a_windows_zip_become_folders() {
    let (root, _p, _game) = uevr_home::uevr_ready("profile-pin-backslash");
    let fetched = cache(
        &root,
        &[("config.txt", "A=1\n"), ("uobjecthook\\1_props.json", "{}")],
    );
    let profile = uevr_profile::read(&fetched).unwrap();
    assert_eq!(profile.files.len(), 1);
    assert_eq!(profile.files[0].rel, Path::new("uobjecthook/1_props.json"));
}

#[test]
fn a_profile_pin_that_is_unknown_or_not_fetched_is_an_error() {
    let (_root, p, game) = uevr_home::uevr_ready("profile-pin-missing");
    let err =
        uevr::apply_with(&game, &settings(&[("uevr-profile", "no-such-pin")]), &p).unwrap_err();
    assert!(err.starts_with("not-found: "), "{err}");
}
