use flat3d_inject::undo::{record_path, UndoRecord};
use std::{fs, path::PathBuf, process::Command};

fn root(case: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flat3d-cli-undo-{}-{case}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
fn invoke(home: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_flat3d-inject"))
        .args(args)
        .env("HOME", home)
        .env("XDG_DATA_HOME", home)
        .output()
        .unwrap()
}
fn record(home: &PathBuf, appid: u32, created: Vec<PathBuf>) {
    let data = home.join("flat3d");
    let path = record_path(&data, appid);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        UndoRecord {
            appid,
            launch_options_before: String::new(),
            created,
            replaced: vec![],
        }
        .to_text(),
    )
    .unwrap();
}

#[test]
fn undo_cli_handles_missing_success_repeat_usage_and_corruption() {
    let home = root("cases");
    let missing = invoke(&home, &["undo", "71", "--json"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stdout).contains("not-found"));
    let file = home.join("created.dll");
    fs::write(&file, "created").unwrap();
    record(&home, 72, vec![file.clone()]);
    let ok = invoke(&home, &["undo", "72", "--json"]);
    assert_eq!(ok.status.code(), Some(0));
    assert!(!file.exists());
    assert!(!record_path(&home.join("flat3d"), 72).exists());
    assert!(String::from_utf8_lossy(&ok.stdout).contains("\"ok\":true"));
    assert_eq!(invoke(&home, &["undo", "72"]).status.code(), Some(1));
    let usage = invoke(&home, &["undo", "abc"]);
    assert_eq!(usage.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&usage.stderr)
        .contains("usage: flat3d-inject undo <appid> [--json]"));
    let path = record_path(&home.join("flat3d"), 73);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "corrupt\n").unwrap();
    let corrupt = invoke(&home, &["undo", "73", "--json"]);
    assert_eq!(corrupt.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&corrupt.stdout).contains("command-failed"));
}
