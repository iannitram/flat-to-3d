//! Locked tests for E4c: the undo record every injector writes, so a game can be put back to
//! flat. A plain text file, one fact per line, tab-separated, `\`, newline and tab escaped.

use flat3d_inject::undo::{record_path, Replaced, UndoRecord};
use std::path::{Path, PathBuf};

fn sample() -> UndoRecord {
    UndoRecord {
        appid: 2651280,
        launch_options_before: "-novid".into(),
        created: vec![PathBuf::from("/g/a.dll"), PathBuf::from("/g/b.ini")],
        replaced: vec![Replaced {
            path: PathBuf::from("/g/d3d11.dll"),
            backup: PathBuf::from("/g/d3d11.dll.flat3d-backup"),
        }],
        created_dirs: vec![],
    }
}

const SAMPLE_TEXT: &str = "flat3d-undo 1\nappid\t2651280\nlaunch-options\t-novid\n\
created\t/g/a.dll\ncreated\t/g/b.ini\n\
replaced\t/g/d3d11.dll\t/g/d3d11.dll.flat3d-backup\n";

#[test]
fn a_record_writes_exactly_this_text() {
    assert_eq!(sample().to_text(), SAMPLE_TEXT);
}

#[test]
fn that_text_reads_back_to_the_same_record() {
    assert_eq!(UndoRecord::from_text(SAMPLE_TEXT), Ok(sample()));
}

#[test]
fn an_empty_record_round_trips() {
    let r = UndoRecord {
        appid: 7,
        launch_options_before: String::new(),
        created: vec![],
        replaced: vec![],
        created_dirs: vec![],
    };
    assert_eq!(r.to_text(), "flat3d-undo 1\nappid\t7\nlaunch-options\t\n");
    assert_eq!(UndoRecord::from_text(&r.to_text()), Ok(r));
}

#[test]
fn tabs_newlines_and_backslashes_survive() {
    let mut r = sample();
    r.launch_options_before = "a\tb\nc\\d".into();
    r.created = vec![PathBuf::from("/g/we\tird\nname\\x")];
    let text = r.to_text();
    assert!(text.contains("launch-options\ta\\tb\\nc\\\\d\n"));
    assert_eq!(UndoRecord::from_text(&text), Ok(r));
}

#[test]
fn a_wrong_header_is_not_an_undo_record() {
    assert_eq!(
        UndoRecord::from_text("hello\n"),
        Err("not an undo record".to_string())
    );
    assert_eq!(
        UndoRecord::from_text(""),
        Err("not an undo record".to_string())
    );
}

#[test]
fn a_missing_or_bad_appid_is_an_error() {
    let missing = "flat3d-undo 1\nlaunch-options\t\n";
    assert_eq!(
        UndoRecord::from_text(missing),
        Err("missing appid".to_string())
    );
    let bad = "flat3d-undo 1\nappid\tabc\nlaunch-options\t\n";
    assert_eq!(UndoRecord::from_text(bad), Err("bad appid".to_string()));
}

#[test]
fn missing_launch_options_unknown_lines_and_bad_replaced_lines_are_errors() {
    let no_opts = "flat3d-undo 1\nappid\t1\n";
    assert_eq!(
        UndoRecord::from_text(no_opts),
        Err("missing launch-options".to_string())
    );
    let unknown = "flat3d-undo 1\nappid\t1\nlaunch-options\t\nwhat\tx\n";
    assert_eq!(
        UndoRecord::from_text(unknown),
        Err("unknown line: what".to_string())
    );
    let short = "flat3d-undo 1\nappid\t1\nlaunch-options\t\nreplaced\t/only-one\n";
    assert_eq!(
        UndoRecord::from_text(short),
        Err("bad replaced line".to_string())
    );
}

#[test]
fn blank_lines_are_ignored() {
    let text = "flat3d-undo 1\n\nappid\t7\n\nlaunch-options\t\n\n";
    assert_eq!(UndoRecord::from_text(text).unwrap().appid, 7);
}

#[test]
fn the_record_lives_at_undo_slash_appid_txt() {
    let p = record_path(Path::new("/home/deck/.local/share/flat3d"), 2651280);
    assert_eq!(
        p,
        PathBuf::from("/home/deck/.local/share/flat3d/undo/2651280.txt")
    );
}
