//! The UI layer ships beside the engine binaries and install/uninstall place it in the
//! user's Vulkan implicit-layer folder.

use flat3d_inject::{install, testing::FakeRunner};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixture(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("ui_layer_install")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    let home = root.join("home");
    let src = root.join("src");
    fs::create_dir_all(&home).unwrap();
    for (path, data) in [
        ("flat3d-inject", "inject"),
        ("flat3d-run", "run"),
        ("tools/adb", "adb"),
        (install::UI_LAYER_LIBRARY, "layer"),
        (install::UI_LAYER_MANIFEST, "manifest"),
    ] {
        let file = src.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, data).unwrap();
    }
    (root, home, src)
}

fn plan(home: &Path, src: &Path) -> install::InstallPlan {
    let mut plan = install::install_plan(home.to_str().unwrap(), src.to_str().unwrap());
    plan.udev_rule = home
        .parent()
        .unwrap()
        .join("etc/70-flat3d-quest.rules")
        .display()
        .to_string();
    plan
}

fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn the_plan_copies_the_library_and_manifest_beside_each_other() {
    let (_, home, src) = fixture("plan");
    let layer = install::implicit_layer_dir(home.to_str().unwrap());
    assert_eq!(
        plan(&home, &src).layer_files,
        vec![
            (
                src.join(install::UI_LAYER_LIBRARY).display().to_string(),
                format!("{layer}/{}", install::UI_LAYER_LIBRARY),
            ),
            (
                src.join(install::UI_LAYER_MANIFEST).display().to_string(),
                format!("{layer}/{}", install::UI_LAYER_MANIFEST),
            ),
        ]
    );
}

#[test]
fn install_writes_the_layer_and_uninstall_removes_it() {
    let (_, home, src) = fixture("round-trip");
    let p = plan(&home, &src);
    install::execute(&p, &mut FakeRunner::new()).unwrap();
    let library = home
        .join(".local/share/vulkan/implicit_layer.d")
        .join(install::UI_LAYER_LIBRARY);
    let manifest = home
        .join(".local/share/vulkan/implicit_layer.d")
        .join(install::UI_LAYER_MANIFEST);
    assert_eq!(fs::read_to_string(&library).unwrap(), "layer");
    assert_eq!(fs::read_to_string(&manifest).unwrap(), "manifest");
    assert_eq!(mode(&library), 0o755);
    assert_eq!(mode(&manifest), 0o644);

    let u = install::uninstall_plan(home.to_str().unwrap());
    assert!(u.layer_removes.contains(&library.display().to_string()));
    let mut u = u;
    u.udev_rule = p.udev_rule.clone();
    install::execute_uninstall(&u, &mut FakeRunner::new()).unwrap();
    assert!(!library.exists());
    assert!(!manifest.exists());
    assert!(!home.join(".local/share/vulkan").exists());
}
