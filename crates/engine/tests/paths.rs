use flat3d_inject::paths::EnginePaths;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn env(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn from_env_uses_xdg_locations_and_source_recipe_first() {
    let p = EnginePaths::from_env(
        &env(&[
            ("HOME", "/home/u"),
            ("XDG_RUNTIME_DIR", "/run/user/1000"),
            ("XDG_DATA_HOME", "/data"),
            ("XDG_CACHE_HOME", "/cache"),
            ("FLAT3D_SRC_DIR", "/src"),
        ]),
        1000,
    )
    .unwrap();
    assert_eq!(p.home, PathBuf::from("/home/u"));
    assert_eq!(p.uid, 1000);
    assert_eq!(p.state_dir, PathBuf::from("/run/user/1000/flat3d"));
    assert_eq!(p.config, PathBuf::from("/home/u/.config/flat3d"));
    assert_eq!(p.data, PathBuf::from("/data/flat3d"));
    assert_eq!(p.cache_dir, PathBuf::from("/cache/flat3d"));
    assert_eq!(p.log_dir, PathBuf::from("/home/u/.local/state/flat3d"));
    assert_eq!(p.bin, PathBuf::from("/home/u/.local/bin"));
    assert_eq!(
        p.recipe_dirs,
        [
            PathBuf::from("/src/recipes"),
            PathBuf::from("/data/flat3d/recipes")
        ]
    );
}

#[test]
fn from_env_uses_home_defaults_when_xdg_is_unset_or_empty() {
    let p =
        EnginePaths::from_env(&env(&[("HOME", "/home/u"), ("XDG_CACHE_HOME", "")]), 55).unwrap();
    assert_eq!(p.state_dir, PathBuf::from("/tmp/flat3d-55"));
    assert_eq!(p.data, PathBuf::from("/home/u/.local/share/flat3d"));
    assert_eq!(p.cache_dir, PathBuf::from("/home/u/.cache/flat3d"));
    assert_eq!(
        p.recipe_dirs,
        [PathBuf::from("/home/u/.local/share/flat3d/recipes")]
    );
}

#[test]
fn missing_or_empty_home_is_not_found() {
    assert_eq!(
        EnginePaths::from_env(&BTreeMap::new(), 1).unwrap_err().code,
        "not-found"
    );
    assert_eq!(
        EnginePaths::from_env(&env(&[("HOME", "")]), 1)
            .unwrap_err()
            .code,
        "not-found"
    );
}

#[test]
fn accessors_use_the_frozen_paths() {
    let p = EnginePaths::from_env(
        &env(&[
            ("HOME", "/h"),
            ("XDG_DATA_HOME", "/d"),
            ("XDG_CACHE_HOME", "/c"),
        ]),
        9,
    )
    .unwrap();
    assert_eq!(p.settings(), PathBuf::from("/h/.config/flat3d/settings"));
    assert_eq!(
        p.link_log(),
        PathBuf::from("/h/.local/state/flat3d/link.log")
    );
    assert_eq!(p.run_log(), PathBuf::from("/h/.local/state/flat3d/run.log"));
    assert_eq!(p.cache(), PathBuf::from("/c/flat3d"));
    assert_eq!(p.uevr_cache(), PathBuf::from("/c/flat3d/uevr"));
    assert_eq!(p.env_file(5), PathBuf::from("/d/flat3d/env/5.env"));
    assert_eq!(p.wrapper_bin(), PathBuf::from("/h/.local/bin/flat3d-run"));
    assert_eq!(p.viewer_bin(), PathBuf::from("/h/.local/bin/flat3d-viewer"));
    assert_eq!(p.pins(), PathBuf::from("/d/flat3d/pins"));
    let run = p.run_paths();
    assert_eq!(run.state, p.state_dir.to_string_lossy());
    assert_eq!(run.settings, p.settings());
    assert_eq!(run.recipes, PathBuf::from("/d/flat3d/recipes"));
    assert_eq!(run.log, p.run_log());
    let watch = p.watch_paths();
    assert_eq!(watch.home, "/h");
    assert_eq!(watch.state_dir, p.state_dir.to_string_lossy());
}
