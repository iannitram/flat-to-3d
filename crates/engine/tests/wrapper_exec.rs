//! Locked tests for E8b: the command and environment the game starts with. Pure.

use flat3d_inject::injectors::Route;
use flat3d_inject::wrapper::{exec_spec, FlatReason, Plan};

fn s(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}
fn pair(k: &str, v: &str) -> (String, String) {
    (k.to_string(), v.to_string())
}
const FOLDER: &str = "/home/deck/.local/share/flatpak/app/io.github.wivrn.wivrn";
const ARGV: [&str; 3] = ["proton", "waitforexitandrun", "game.exe"];

#[test]
fn flat_is_the_original_command_and_nothing_else() {
    let p = Plan::Flat {
        reason: FlatReason::SwitchOff,
    };
    let spec = exec_spec(
        &p,
        &s(&ARGV),
        &[pair("X", "1")],
        "/home/deck",
        Some("/data"),
    );
    assert_eq!(spec.argv, s(&ARGV));
    assert!(spec.env.is_empty());
}

#[test]
fn three_d_adds_the_proton_variables() {
    let p = Plan::ThreeD { route: Route::Uevr };
    let spec = exec_spec(&p, &s(&ARGV), &[], "/home/deck", None);
    assert_eq!(spec.argv, s(&ARGV));
    assert_eq!(
        spec.env,
        vec![
            pair("PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES", "1"),
            pair("PRESSURE_VESSEL_FILESYSTEMS_RW", FOLDER),
            pair("STEAM_COMPAT_LAUNCHER_SERVICE", "proton"),
        ]
    );
}

#[test]
fn an_existing_value_is_kept_first() {
    let p = Plan::ThreeD { route: Route::Uevr };
    let spec = exec_spec(&p, &s(&ARGV), &[], "/home/deck", Some("/mnt/games"));
    assert_eq!(
        spec.env[1],
        pair(
            "PRESSURE_VESSEL_FILESYSTEMS_RW",
            &format!("/mnt/games:{FOLDER}")
        )
    );
}

#[test]
fn the_folder_is_not_added_twice() {
    let p = Plan::ThreeD { route: Route::Uevr };
    let have = format!("/a:{FOLDER}:/b");
    let spec = exec_spec(&p, &s(&ARGV), &[], "/home/deck", Some(&have));
    assert_eq!(spec.env[1], pair("PRESSURE_VESSEL_FILESYSTEMS_RW", &have));
}

#[test]
fn an_empty_existing_value_counts_as_none() {
    let p = Plan::ThreeD { route: Route::Port };
    let spec = exec_spec(&p, &s(&ARGV), &[], "/home/deck", Some(""));
    assert_eq!(spec.env[1], pair("PRESSURE_VESSEL_FILESYSTEMS_RW", FOLDER));
}

#[test]
fn the_routes_variables_come_last_in_order() {
    let p = Plan::ThreeD {
        route: Route::Geo11,
    };
    let spec = exec_spec(
        &p,
        &s(&ARGV),
        &[pair("A", "1"), pair("B", "2")],
        "/home/deck",
        None,
    );
    assert_eq!(spec.env.len(), 5);
    assert_eq!(&spec.env[3..], &[pair("A", "1"), pair("B", "2")]);
}
