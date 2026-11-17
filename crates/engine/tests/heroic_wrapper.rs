use flat3d_inject::heroic_wrapper::{apply, games_config, has_wrapper, undo, Applied};
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::PathBuf;

const APP_NAME: &str = "dc9d2e595d0e4650b35d659f90d41059";
const WRAPPER: &str = "/opt/flat3d/bin/flat3d-run";
const FIXTURE: &str = r#"{
  "dc9d2e595d0e4650b35d659f90d41059": {
    "winePrefix": "/games/prefixes/alan-wake-2",
    "wineVersion": {
      "bin": "/opt/proton/proton",
      "name": "Proton",
      "type": "proton"
    },
    "enviromentOptions": [
      {
        "key": "EXAMPLE",
        "value": "enabled"
      }
    ],
    "wrapperOptions": []
  },
  "app_name": "dc9d2e595d0e4650b35d659f90d41059",
  "version": "v0",
  "explicit": true
}"#;

fn fixture(name: &str, text: &str) -> (PathBuf, PathBuf) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("heroic-wrapper-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("GamesConfig")).unwrap();
    let config = games_config(&dir, APP_NAME);
    fs::write(&config, text).unwrap();
    (config, dir.join("backups/original.json"))
}

fn read_value(config: &std::path::Path) -> Value {
    serde_json::from_slice(&fs::read(config).unwrap()).unwrap()
}

#[test]
fn apply_adds_exactly_one_wrapper_entry_and_keeps_every_other_key() {
    let (config, backup) = fixture("adds", FIXTURE);
    assert!(!has_wrapper(&config, APP_NAME, WRAPPER).unwrap());

    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::Added,
    );

    assert!(has_wrapper(&config, APP_NAME, WRAPPER).unwrap());
    assert!(!has_wrapper(&config, APP_NAME, "/opt/other/flat3d-run").unwrap());
    let mut after = read_value(&config);
    let entries = after[APP_NAME]["wrapperOptions"].as_array_mut().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries.remove(0), json!({ "exe": WRAPPER, "args": "" }));
    assert_eq!(after, serde_json::from_str::<Value>(FIXTURE).unwrap());
    assert_eq!(fs::read(&backup).unwrap(), FIXTURE.as_bytes());
    assert!(fs::read(&config).unwrap().ends_with(b"\n"));
}

#[test]
fn undo_restores_the_file_byte_for_byte() {
    let (config, backup) = fixture("undo", FIXTURE);
    apply(&config, APP_NAME, WRAPPER, &backup).unwrap();

    assert!(undo(&config, &backup).unwrap());

    assert_eq!(fs::read(&config).unwrap(), FIXTURE.as_bytes());
    assert!(!backup.exists());
}

#[test]
fn applying_twice_does_not_stack() {
    let (config, backup) = fixture("twice", FIXTURE);
    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::Added,
    );
    let first_apply = fs::read(&config).unwrap();

    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::AlreadyThere,
    );

    assert_eq!(fs::read(&config).unwrap(), first_apply);
    assert_eq!(
        read_value(&config)[APP_NAME]["wrapperOptions"],
        json!([{ "exe": WRAPPER, "args": "" }]),
    );
    assert_eq!(fs::read(&backup).unwrap(), FIXTURE.as_bytes());
    assert!(undo(&config, &backup).unwrap());
    assert_eq!(fs::read(&config).unwrap(), FIXTURE.as_bytes());
}

#[test]
fn a_missing_wrapper_options_key_is_created() {
    let text = FIXTURE.replace(",\n    \"wrapperOptions\": []", "");
    let (config, backup) = fixture("missing-options", &text);
    assert!(!has_wrapper(&config, APP_NAME, WRAPPER).unwrap());

    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::Added,
    );

    assert_eq!(
        read_value(&config)[APP_NAME]["wrapperOptions"],
        json!([{ "exe": WRAPPER, "args": "" }]),
    );
    assert!(undo(&config, &backup).unwrap());
    assert_eq!(fs::read(&config).unwrap(), text.as_bytes());
}

#[test]
fn an_existing_other_wrapper_is_kept() {
    let text = FIXTURE.replace(
        "\"wrapperOptions\": []",
        r#""wrapperOptions": [{"exe":"/usr/bin/gamemoderun","args":""}]"#,
    );
    let (config, backup) = fixture("other-wrapper", &text);

    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::Added,
    );

    assert_eq!(
        read_value(&config)[APP_NAME]["wrapperOptions"],
        json!([
            { "exe": "/usr/bin/gamemoderun", "args": "" },
            { "exe": WRAPPER, "args": "" },
        ]),
    );
    assert!(undo(&config, &backup).unwrap());
    assert_eq!(fs::read(&config).unwrap(), text.as_bytes());
}

#[test]
fn a_config_without_the_game_key_is_an_error() {
    let text = r#"{"app_name":"missing","version":"v0","explicit":true}"#;
    let (config, backup) = fixture("missing-game", text);

    let error = apply(&config, APP_NAME, WRAPPER, &backup).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains(&config.display().to_string()));
    assert_eq!(
        has_wrapper(&config, APP_NAME, WRAPPER).unwrap_err().kind(),
        io::ErrorKind::InvalidData,
    );
    assert_eq!(fs::read(&config).unwrap(), text.as_bytes());
    assert!(!backup.exists());
}

#[test]
fn undo_without_a_backup_does_nothing() {
    let (config, backup) = fixture("no-backup", FIXTURE);

    assert!(!undo(&config, &backup).unwrap());

    assert_eq!(fs::read(&config).unwrap(), FIXTURE.as_bytes());
    assert!(!backup.parent().unwrap().exists());
    let missing_config = config.with_file_name("missing.json");
    assert!(!undo(&missing_config, &backup).unwrap());
    assert!(!missing_config.exists());
}

#[test]
fn an_existing_matching_wrapper_leaves_the_file_and_backup_untouched() {
    let text = FIXTURE.replace(
        "\"wrapperOptions\": []",
        &format!(r#""wrapperOptions": [{{"exe":"{WRAPPER}","args":"--existing"}}]"#),
    );
    let (config, backup) = fixture("already-there", &text);

    assert!(has_wrapper(&config, APP_NAME, WRAPPER).unwrap());
    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::AlreadyThere,
    );
    assert_eq!(fs::read(&config).unwrap(), text.as_bytes());
    assert!(!backup.parent().unwrap().exists());
}

#[test]
fn malformed_configs_are_rejected_without_writing() {
    let cases = [
        "not JSON".to_string(),
        "[]".to_string(),
        format!(r#"{{"{APP_NAME}":[]}}"#),
        format!(r#"{{"{APP_NAME}":{{"wrapperOptions":null}}}}"#),
        format!(r#"{{"{APP_NAME}":{{"wrapperOptions":{{}}}}}}"#),
    ];
    for (index, text) in cases.iter().enumerate() {
        let (config, backup) = fixture(&format!("malformed-{index}"), text);

        for error in [
            apply(&config, APP_NAME, WRAPPER, &backup).unwrap_err(),
            has_wrapper(&config, APP_NAME, WRAPPER).unwrap_err(),
        ] {
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert!(error.to_string().contains(&config.display().to_string()));
        }
        assert_eq!(fs::read(&config).unwrap(), text.as_bytes());
        assert!(!backup.exists());
    }
}

#[test]
fn an_existing_backup_is_preserved() {
    let (config, backup) = fixture("keep-backup", FIXTURE);
    let original = b"original recovery bytes\n";
    fs::create_dir_all(backup.parent().unwrap()).unwrap();
    fs::write(&backup, original).unwrap();

    assert_eq!(
        apply(&config, APP_NAME, WRAPPER, &backup).unwrap(),
        Applied::Added,
    );

    assert_eq!(fs::read(&backup).unwrap(), original);
    assert!(undo(&config, &backup).unwrap());
    assert_eq!(fs::read(&config).unwrap(), original);
    assert!(!backup.exists());
}
