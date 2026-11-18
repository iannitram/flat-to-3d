//! Headset profiles (#411): the shipped Quest 2 file, the unknown-headset defaults,
//! the side-by-side stream size, and the packaging and code-map wiring.

use flat3d_inject::headsets::{
    self, find, load_headsets, load_headsets_from_dirs, lookup, lookup_in_paths, parse_headset,
    stream_size, stream_size_in_paths, HeadsetError, Identification, Resolution, UsbId,
};
use flat3d_inject::paths::EnginePaths;
use std::fs;
use std::path::{Path, PathBuf};

const QUEST_2: &str = "\
name: Quest 2
usb-id: 2833:5013
adb-product: hollywood
adb-model: Quest_2
panel-width: 1832
panel-height: 1920
refresh-rates: 60, 72, 80, 90, 120
field-of-view-note: The maker publishes no field-of-view figure for the Quest 2
connections: usb
client-app: org.meumeu.wivrn.github
client-app-name: WiVRn
stream-width: 1440
stream-height: 1440
stream-fps: 60
frame-generation: 2
stream-connection: usb
";

fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("flat3d-headsets-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (file, text) in files {
        fs::write(dir.join(file), text).unwrap();
    }
    dir
}

fn repo_headsets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("headsets")
}

fn paths(name: &str) -> EnginePaths {
    let root = folder(&format!("engine-{name}"), &[]);
    EnginePaths {
        home: root.join("home"),
        uid: 0,
        state_dir: root.join("runtime"),
        config: root.join("config"),
        data: root.join("data"),
        cache_dir: root.join("cache"),
        log_dir: root.join("logs"),
        bin: root.join("bin"),
        recipe_dirs: Vec::new(),
        headset_dirs: vec![root.join("first"), root.join("second")],
    }
}

#[test]
fn the_repo_loads_the_quest_2_profile() {
    let headsets = load_headsets(&repo_headsets()).unwrap();
    assert_eq!(headsets.len(), 1);
    let quest = &headsets[0];
    assert_eq!(quest.name, "Quest 2");
    assert_eq!(quest.usb, UsbId::new(0x2833, 0x5013));
    assert_eq!(quest.adb_product, "hollywood");
    assert_eq!(quest.adb_model, "Quest_2");
    assert_eq!(quest.panel, Resolution::new(1832, 1920));
    assert_eq!(quest.refresh_rates, vec![60, 72, 80, 90, 120]);
    assert_eq!(quest.field_of_view, None);
    assert!(quest
        .field_of_view_note
        .as_deref()
        .unwrap_or_default()
        .contains("no field-of-view figure"));
    assert_eq!(quest.connections, vec!["usb"]);
    assert_eq!(quest.client_app, "org.meumeu.wivrn.github");
    assert_eq!(quest.client_app_name, "WiVRn");
    assert_eq!(quest.stream, Resolution::new(1440, 1440));
    assert_eq!(quest.stream_fps, 60);
    assert_eq!(quest.frame_generation, 2);
    assert_eq!(quest.stream_connection, "usb");
    assert!(quest.is_known());
}

#[test]
fn the_side_by_side_stream_size_doubles_the_eye_width() {
    let headsets = load_headsets(&repo_headsets()).unwrap();
    assert_eq!(stream_size(&headsets[0]), (2880, 1440));
    assert_eq!(headsets[0].stream_size(), (2880, 1440));
}

#[test]
fn a_malformed_file_is_an_error_naming_the_file() {
    let broken = QUEST_2.replace("refresh-rates: 60, 72, 80, 90, 120\n", "");
    let dir = folder("broken", &[("broken.headset", broken.as_str())]);
    match load_headsets(&dir) {
        Err(HeadsetError::InFile { file, error }) => {
            assert_eq!(file, "broken.headset");
            assert!(error.to_string().contains("refresh-rates"), "{error}");
        }
        other => panic!("expected InFile, got {other:?}"),
    }
}

#[test]
fn a_bad_value_names_the_file_and_the_value() {
    let broken = QUEST_2.replace("frame-generation: 2", "frame-generation: 0");
    let dir = folder("bad-value", &[("quest-2.headset", broken.as_str())]);
    match load_headsets(&dir) {
        Err(HeadsetError::InFile { file, error }) => {
            assert_eq!(file, "quest-2.headset");
            assert!(error.to_string().contains("frame-generation"), "{error}");
        }
        other => panic!("expected InFile, got {other:?}"),
    }
}

#[test]
fn a_refresh_rate_of_zero_is_refused() {
    let text = QUEST_2.replace("refresh-rates: 60, 72, 80, 90, 120", "refresh-rates: 0, 60");
    match parse_headset(&text) {
        Err(HeadsetError::BadValue { key, value }) => {
            assert_eq!(key, "refresh-rates");
            assert_eq!(value, "0, 60");
        }
        other => panic!("expected BadValue, got {other:?}"),
    }
}

#[test]
fn unknown_files_are_ignored() {
    let dir = folder(
        "readme",
        &[
            ("README.md", "# headsets\nnot a profile\n"),
            ("quest-2.headset", QUEST_2),
        ],
    );
    assert_eq!(load_headsets(&dir).unwrap().len(), 1);
}

#[test]
fn an_unknown_headset_gets_the_safe_defaults_marked_unknown() {
    let identification = Identification::new(
        Some(UsbId::new(0x1234, 0x5678)),
        Some("unknown"),
        Some("Unknown_1"),
    );
    let profile = lookup(&[], &identification);
    assert!(!profile.is_known());
    assert_eq!(profile.stream, Resolution::new(1440, 1440));
    assert_eq!(profile.stream_fps, 60);
    assert_eq!(profile.frame_generation, 1);
    assert_eq!(profile.stream_size(), (2880, 1440));
    assert_eq!(profile.usb, UsbId::new(0x1234, 0x5678));
    assert_eq!(profile.adb_product, "unknown");
    assert_eq!(profile.adb_model, "Unknown_1");
}

#[test]
fn a_headset_is_found_by_usb_adb_product_or_model() {
    let headsets = load_headsets(&repo_headsets()).unwrap();
    for identification in [
        Identification::new(Some(UsbId::new(0x2833, 0x5013)), None, None),
        Identification::new(None, Some("hollywood"), None),
        Identification::new(None, None, Some("Quest_2")),
        Identification::new(
            Some(UsbId::new(0x2833, 0x5013)),
            Some("hollywood"),
            Some("Quest_2"),
        ),
    ] {
        let found = find(&headsets, &identification);
        assert_eq!(found.map(|profile| profile.name.as_str()), Some("Quest 2"));
        assert!(lookup(&headsets, &identification).is_known());
    }
    let wrong = Identification::new(None, Some("hollywood"), Some("Quest_3"));
    assert!(find(&headsets, &wrong).is_none());
    assert!(!lookup(&headsets, &wrong).is_known());
    assert!(find(&headsets, &Identification::default()).is_none());
}

#[test]
fn profiles_load_from_the_first_folder_that_holds_them() {
    let renamed = QUEST_2
        .replace("name: Quest 2", "name: Another Quest 2")
        .replace("usb-id: 2833:5013", "usb-id: 2833:5014")
        .replace("adb-product: hollywood", "adb-product: other")
        .replace("adb-model: Quest_2", "adb-model: Other_2");
    let first = folder("paths-first", &[("quest-2.headset", QUEST_2)]);
    let second = folder("paths-second", &[("other.headset", renamed.as_str())]);
    let mut paths = paths("first-holds");
    paths.headset_dirs = vec![first, second];
    let loaded = load_headsets_from_dirs(&paths.headset_dirs).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "Quest 2");
}

#[test]
fn an_empty_folder_is_skipped_for_the_next_one() {
    let empty = folder("paths-empty", &[("README.md", "# no profiles here\n")]);
    let second = folder("paths-next", &[("quest-2.headset", QUEST_2)]);
    let mut paths = paths("skip-empty");
    paths.headset_dirs = vec![empty, second];
    let identification = Identification::new(Some(UsbId::new(0x2833, 0x5013)), None, None);
    let profile = lookup_in_paths(&paths, &identification).unwrap();
    assert!(profile.is_known());
    assert_eq!(profile.name, "Quest 2");
}

#[test]
fn a_known_profile_is_found_and_sized_through_the_paths() {
    let dir = folder("paths-known", &[("quest-2.headset", QUEST_2)]);
    let mut paths = paths("known");
    paths.headset_dirs = vec![dir];
    let identification = Identification::new(None, Some("hollywood"), None);
    let profile = lookup_in_paths(&paths, &identification).unwrap();
    assert!(profile.is_known());
    assert_eq!(profile.name, "Quest 2");
    assert_eq!(
        stream_size_in_paths(&paths, &identification).unwrap(),
        (2880, 1440)
    );
}

#[test]
fn no_folder_present_gives_the_unknown_defaults() {
    let paths = paths("none");
    let identification = Identification::new(Some(UsbId::new(0x1234, 0x5678)), None, None);
    let profile = lookup_in_paths(&paths, &identification).unwrap();
    assert!(!profile.is_known());
    assert_eq!(profile.usb, UsbId::new(0x1234, 0x5678));
    assert_eq!(profile.stream, Resolution::new(1440, 1440));
    assert_eq!(
        stream_size_in_paths(&paths, &identification).unwrap(),
        (2880, 1440)
    );
}

#[test]
fn a_malformed_file_in_the_paths_is_still_an_error() {
    let broken = QUEST_2.replace("refresh-rates: 60, 72, 80, 90, 120\n", "");
    let dir = folder("paths-broken", &[("broken.headset", broken.as_str())]);
    let mut paths = paths("broken");
    paths.headset_dirs = vec![dir];
    match lookup_in_paths(&paths, &Identification::default()) {
        Err(HeadsetError::InFile { file, error }) => {
            assert_eq!(file, "broken.headset");
            assert!(error.to_string().contains("refresh-rates"), "{error}");
        }
        other => panic!("expected InFile, got {other:?}"),
    }
}

#[test]
fn two_files_with_one_identification_are_refused() {
    let renamed = QUEST_2.replace("name: Quest 2", "name: Another Quest 2");
    let dir = folder(
        "duplicate",
        &[("a.headset", QUEST_2), ("b.headset", renamed.as_str())],
    );
    match load_headsets(&dir) {
        Err(HeadsetError::DuplicateIdentification { first, second, .. }) => {
            assert_eq!(first, "a.headset");
            assert_eq!(second, "b.headset");
        }
        other => panic!("expected DuplicateIdentification, got {other:?}"),
    }
}

#[test]
fn a_repeated_usb_id_adb_product_or_adb_model_is_refused_on_its_own() {
    let other_usb = QUEST_2
        .replace("name: Quest 2", "name: Another Quest 2")
        .replace("adb-product: hollywood", "adb-product: other")
        .replace("adb-model: Quest_2", "adb-model: Other_2");
    let other_product = QUEST_2
        .replace("name: Quest 2", "name: Another Quest 2")
        .replace("usb-id: 2833:5013", "usb-id: 2833:5014")
        .replace("adb-model: Quest_2", "adb-model: Other_2");
    let other_model = QUEST_2
        .replace("name: Quest 2", "name: Another Quest 2")
        .replace("usb-id: 2833:5013", "usb-id: 2833:5014")
        .replace("adb-product: hollywood", "adb-product: other");
    for (case, second, field) in [
        ("duplicate-usb", other_usb, "usb-id"),
        ("duplicate-product", other_product, "adb-product"),
        ("duplicate-model", other_model, "adb-model"),
    ] {
        let dir = folder(
            case,
            &[("a.headset", QUEST_2), ("b.headset", second.as_str())],
        );
        match load_headsets(&dir) {
            Err(HeadsetError::DuplicateIdentification {
                identification,
                first,
                second,
            }) => {
                assert_eq!(first, "a.headset");
                assert_eq!(second, "b.headset");
                assert!(identification.starts_with(field), "{identification}");
            }
            other => panic!("expected DuplicateIdentification for {case}, got {other:?}"),
        }
    }
}

#[test]
fn the_readme_documents_the_format_and_the_code_map_says_present() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let readme = fs::read_to_string(root.join("headsets/README.md")).unwrap();
    assert!(readme.contains("key: value") && readme.contains("usb-id"));
    let map = fs::read_to_string(root.join("docs/code-map.md")).unwrap();
    assert!(map.contains("`crates/engine/src/headsets.rs`"));
    assert!(map.contains("| `headsets/`, `crates/engine/src/headsets.rs` | present |"));
    assert!(headsets::KEYS.contains(&"stream-fps"));
}
