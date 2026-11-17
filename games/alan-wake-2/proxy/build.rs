// Links the export list in version.def (names and ordinals of the system version.dll)
// into the MSVC build. Other targets need nothing.
fn main() {
    println!("cargo:rerun-if-changed=version.def");
    let msvc = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if msvc {
        let def = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("version.def");
        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def.display());
    }
}
