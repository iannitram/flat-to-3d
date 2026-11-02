//! Locked tests for E4d: the steps that put a game back to flat, from its undo record.
//! Pure: this lists the steps in order; nothing is touched on disk.

use flat3d_inject::undo::{undo_actions, Replaced, UndoAction, UndoRecord};
use std::path::PathBuf;

fn record() -> UndoRecord {
    UndoRecord {
        appid: 2651280,
        launch_options_before: "-novid".into(),
        created: vec![PathBuf::from("/g/a.dll"), PathBuf::from("/g/b.ini")],
        replaced: vec![
            Replaced { path: PathBuf::from("/g/x.dll"), backup: PathBuf::from("/g/x.dll.bak") },
            Replaced { path: PathBuf::from("/g/y.dll"), backup: PathBuf::from("/g/y.dll.bak") },
        ],
    }
}

#[test]
fn created_files_go_first_newest_first() {
    let got = undo_actions(&record());
    assert_eq!(got[0], UndoAction::RemoveFile(PathBuf::from("/g/b.ini")));
    assert_eq!(got[1], UndoAction::RemoveFile(PathBuf::from("/g/a.dll")));
}

#[test]
fn replaced_files_are_restored_from_their_backups_newest_first() {
    let got = undo_actions(&record());
    assert_eq!(
        got[2],
        UndoAction::RestoreFile { backup: PathBuf::from("/g/y.dll.bak"), path: PathBuf::from("/g/y.dll") }
    );
    assert_eq!(
        got[3],
        UndoAction::RestoreFile { backup: PathBuf::from("/g/x.dll.bak"), path: PathBuf::from("/g/x.dll") }
    );
}

#[test]
fn the_launch_options_are_put_back_last() {
    let got = undo_actions(&record());
    assert_eq!(got.len(), 5);
    assert_eq!(got[4], UndoAction::SetLaunchOptions("-novid".to_string()));
}

#[test]
fn an_empty_record_only_resets_the_launch_options() {
    let r = UndoRecord { appid: 1, launch_options_before: String::new(), created: vec![], replaced: vec![] };
    assert_eq!(undo_actions(&r), vec![UndoAction::SetLaunchOptions(String::new())]);
}

#[test]
fn the_record_itself_is_not_changed() {
    let r = record();
    let before = r.clone();
    let _ = undo_actions(&r);
    assert_eq!(r, before);
}
