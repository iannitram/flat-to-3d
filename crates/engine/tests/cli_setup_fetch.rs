use flat3d_inject::{cli::setup_fetch::setup_fetch, testing::FakeRunner};
use std::fs;

#[path = "support/scenes.rs"]
mod scenes;

#[test]
fn cached_pin_needs_no_commands_and_unknown_or_manual_pins_map_cleanly() {
    let p = scenes::no_steam("setup-fetch-cached");
    let pin = flat3d_inject::injectors::fetch::pin("uevr").unwrap();
    let dir = flat3d_inject::injectors::fetch::cache_dir(&pin, &p);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(flat3d_inject::injectors::fetch::MARKER),
        format!("{}\n", pin.sha256),
    )
    .unwrap();
    let mut r = FakeRunner::new();
    let got = setup_fetch("uevr", &p, &mut r).unwrap();
    assert!(got.steps[0].starts_with("cached "));
    assert!(r.ran.is_empty());
    let err = setup_fetch("no-such-key", &p, &mut r).unwrap_err();
    assert_eq!(err.code, "not-found");
    let manual = flat3d_inject::injectors::fetch::parse_pins(flat3d_inject::injectors::fetch::PINS)
        ["ac7-prereqs"]
        .page
        .clone();
    let err = setup_fetch("ac7-prereqs", &p, &mut r).unwrap_err();
    assert_eq!(err.code, "refused");
    assert!(err.message.contains(&manual));
}

#[test]
fn failed_curl_cleans_the_partial_cache() {
    let p = scenes::no_steam("setup-fetch-fail");
    let pin = flat3d_inject::injectors::fetch::pin("uevr").unwrap();
    let mut r = FakeRunner::new().fail(&["curl"], "fixture failure");
    let err = setup_fetch("uevr", &p, &mut r).unwrap_err();
    assert_eq!(err.code, "command-failed");
    assert!(!flat3d_inject::injectors::fetch::cache_dir(&pin, &p).exists());
}
