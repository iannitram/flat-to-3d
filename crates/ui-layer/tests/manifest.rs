//! The shipped layer manifest: a plain implicit layer that only loads when a route
//! sets its enable variable.

use serde_json::Value;

fn manifest() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packaging/VkLayer_FLAT3D_ui.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn the_manifest_names_the_layer_and_loads_its_library_by_relative_path() {
    let manifest = manifest();
    let layer = &manifest["layer"];
    assert_eq!(layer["name"], "VK_LAYER_FLAT3D_ui");
    assert_eq!(layer["type"], "GLOBAL");
    assert_eq!(layer["description"], "Flat to 3D UI depth");
    let library = layer["library_path"].as_str().unwrap();
    assert!(library.starts_with("./"), "{library}");
    assert_eq!(library.trim_start_matches("./"), "libflat3d_ui_layer.so");
}

#[test]
fn the_manifest_requires_the_enable_variable_and_honours_the_disable_one() {
    let layer = &manifest()["layer"];
    assert_eq!(layer["enable_environment"]["FLAT3D_UI_LAYER"], "1");
    assert_eq!(layer["disable_environment"]["FLAT3D_UI_LAYER_DISABLE"], "1");
}
