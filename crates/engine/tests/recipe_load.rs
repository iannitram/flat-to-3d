//! Locked tests for E1c: `recipes::load_recipes`, every `<appid>-<slug>.recipe` in a folder.
//! Builds its folders in a temp dir. Uses E1a's parser and E1b's checks.

use flat3d_inject::recipes::{load_recipes, RecipeError};
use std::fs;
use std::path::{Path, PathBuf};

fn recipe(name: &str, status: &str, api: &str) -> String {
    format!(
        "# test recipe\nname: {name}\nstereo-status: {status}\nstereo-route: none\n\
         stereo-engine: custom\nstereo-api: {api}\nstereo-mod: none\nstereo-free: yes\n\
         stereo-safe: yes\nstereo-checked: 2026-10-31\n"
    )
}

fn folder(name: &str, files: &[(&str, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("flat3d-recipes-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (file, text) in files {
        fs::write(dir.join(file), text).unwrap();
    }
    dir
}

/// Load a folder holding one file; return the InFile error's file name and inner error.
fn in_file(file: &str, text: String) -> (String, RecipeError) {
    match load_recipes(&folder(file, &[(file, text)])) {
        Err(RecipeError::InFile { file, error }) => (file, *error),
        other => panic!("expected InFile, got {other:?}"),
    }
}

fn bad_value(key: &str, value: &str) -> RecipeError {
    let (key, value) = (key.to_string(), value.to_string());
    RecipeError::BadValue { key, value }
}

#[test]
fn valid_recipes_load_keyed_by_the_appid_in_the_file_name() {
    let sm2 = recipe("Marvel's Spider-Man 2", "works-now", "dx12");
    let ac7 = recipe("ACE COMBAT 7", "likely", "dx11");
    let files = [
        ("2651280-spider-man-2.recipe", sm2),
        ("502500-ace-combat-7.recipe", ac7),
    ];
    let map = load_recipes(&folder("two", &files)).unwrap();
    assert_eq!(
        map.keys().copied().collect::<Vec<_>>(),
        vec![502500, 2651280]
    );
    assert_eq!(map[&2651280].appid, 2651280);
    assert_eq!(map[&2651280].name, "Marvel's Spider-Man 2");
    assert_eq!(map[&502500].status, "likely");
    assert_eq!(map[&502500].api, "dx11");
}

#[test]
fn files_not_ending_in_dot_recipe_are_ignored() {
    let files = [
        ("README.md", "# Recipes\nnot: a recipe".to_string()),
        ("1-notes.txt", "garbage".to_string()),
        ("10-ok.recipe", recipe("Ten", "untested", "unknown")),
    ];
    let map = load_recipes(&folder("others", &files)).unwrap();
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), vec![10]);
}

#[test]
fn an_empty_folder_gives_an_empty_map() {
    assert!(load_recipes(&folder("empty", &[])).unwrap().is_empty());
}

#[test]
fn a_missing_folder_is_an_io_error_naming_the_folder() {
    let gone = std::env::temp_dir().join("flat3d-recipes-no-such-folder");
    match load_recipes(&gone) {
        Err(RecipeError::Io { path, .. }) => assert!(path.contains("no-such-folder"), "{path}"),
        other => panic!("expected Io, got {other:?}"),
    }
}

#[test]
fn an_unknown_status_names_the_file() {
    let (file, error) = in_file("5-bad.recipe", recipe("Five", "maybe", "dx11"));
    assert_eq!(file, "5-bad.recipe");
    assert_eq!(error, bad_value("stereo-status", "maybe"));
}

#[test]
fn an_unknown_api_names_the_file() {
    let (file, error) = in_file("6-bad.recipe", recipe("Six", "build", "metal"));
    assert_eq!(file, "6-bad.recipe");
    assert_eq!(error, bad_value("stereo-api", "metal"));
}

#[test]
fn a_missing_key_names_the_file() {
    let text = recipe("Seven", "build", "dx9").replace("stereo-mod: none\n", "");
    let (file, error) = in_file("7-short.recipe", text);
    assert_eq!(file, "7-short.recipe");
    assert_eq!(error, RecipeError::MissingKey("stereo-mod".into()));
}

#[test]
fn a_duplicate_appid_names_both_files_in_sorted_order() {
    let b = ("8-b.recipe", recipe("Eight B", "build", "dx11"));
    let a = ("8-a.recipe", recipe("Eight A", "build", "dx11"));
    let (first, second) = ("8-a.recipe".to_string(), "8-b.recipe".to_string());
    let expected = RecipeError::DuplicateAppid {
        appid: 8,
        first,
        second,
    };
    assert_eq!(load_recipes(&folder("dup", &[b, a])), Err(expected));
}

#[test]
fn a_file_name_without_a_leading_appid_is_a_bad_file_name() {
    let dir = folder(
        "name",
        &[("spider-man.recipe", recipe("X", "build", "dx11"))],
    );
    let expected = RecipeError::BadFileName("spider-man.recipe".into());
    assert_eq!(load_recipes(&dir), Err(expected));
}

#[test]
fn the_error_text_names_the_file() {
    let dir = folder("text", &[("9-bad.recipe", recipe("Nine", "maybe", "dx11"))]);
    let shown = load_recipes(&dir).unwrap_err().to_string();
    assert!(shown.contains("9-bad.recipe"), "{shown}");
}

#[test]
fn the_repo_recipes_folder_loads() {
    let real = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("recipes");
    let loaded = load_recipes(&real);
    assert!(loaded.is_ok(), "{loaded:?}");
}

#[test]
fn the_readme_documents_the_format_and_the_code_map_says_present() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let readme = fs::read_to_string(root.join("recipes/README.md")).unwrap();
    assert!(readme.contains("<appid>-<slug>.recipe") && readme.contains("key: value"));
    let map = fs::read_to_string(root.join("docs/code-map.md")).unwrap();
    assert!(map.contains("| `crates/engine/src/recipes.rs` | present |"));
}
