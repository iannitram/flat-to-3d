//! The undo record: how to put a game back to flat. docs/code-map.md, Parts.

use crate::api::{ApiError, UndoResult};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A file an injector replaced, and the backup it made first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replaced {
    /// The file in the game folder that we replaced.
    pub path: PathBuf,
    /// Where the original file was saved.
    pub backup: PathBuf,
}

/// What an injector did, as written to `<data_dir>/undo/<appid>.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoRecord {
    /// The Steam app id of the game.
    pub appid: u32,
    /// The game's launch options before we touched them, verbatim.
    pub launch_options_before: String,
    /// Files we created, in order.
    pub created: Vec<PathBuf>,
    /// Files we replaced, with the backup we made.
    pub replaced: Vec<Replaced>,
}

/// One ordered step for undoing an injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoAction {
    RemoveFile(PathBuf),
    RestoreFile { backup: PathBuf, path: PathBuf },
    SetLaunchOptions(String),
}

/// List the steps that restore the game, without changing the record.
pub fn undo_actions(record: &UndoRecord) -> Vec<UndoAction> {
    let mut actions = Vec::with_capacity(record.created.len() + record.replaced.len() + 1);
    actions.extend(
        record
            .created
            .iter()
            .rev()
            .cloned()
            .map(UndoAction::RemoveFile),
    );
    actions.extend(
        record
            .replaced
            .iter()
            .rev()
            .map(|replaced| UndoAction::RestoreFile {
                backup: replaced.backup.clone(),
                path: replaced.path.clone(),
            }),
    );
    actions.push(UndoAction::SetLaunchOptions(
        record.launch_options_before.clone(),
    ));
    actions
}

/// Escape `\`, newline and tab so one value stays on one line.
pub(crate) fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// Reverse [`escape`]: `\n` and `\t` come back as themselves, and a backslash followed by
/// any other character gives that character.
pub(crate) fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(match next {
                    'n' => '\n',
                    't' => '\t',
                    other => other,
                });
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl UndoRecord {
    /// The exact text of the undo record file.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        text.push_str("flat3d-undo 1\n");
        text.push_str(&format!("appid\t{}\n", self.appid));
        text.push_str(&format!(
            "launch-options\t{}\n",
            escape(&self.launch_options_before)
        ));
        for path in &self.created {
            text.push_str(&format!("created\t{}\n", escape(&path.to_string_lossy())));
        }
        for r in &self.replaced {
            text.push_str(&format!(
                "replaced\t{}\t{}\n",
                escape(&r.path.to_string_lossy()),
                escape(&r.backup.to_string_lossy())
            ));
        }
        text
    }

    /// Read an undo record, or say why the text is not one.
    pub fn from_text(text: &str) -> Result<UndoRecord, String> {
        let mut lines = text.split('\n');
        if lines.next() != Some("flat3d-undo 1") {
            return Err("not an undo record".to_string());
        }

        let mut appid = None;
        let mut launch_options = None;
        let mut created = Vec::new();
        let mut replaced = Vec::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            let mut fields = line.split('\t');
            let word = fields.next().unwrap_or("");
            match word {
                "appid" => {
                    let value = fields.next().unwrap_or("");
                    appid = Some(value.parse::<u32>().map_err(|_| "bad appid".to_string())?);
                }
                "launch-options" => {
                    launch_options = Some(unescape(fields.next().unwrap_or("")));
                }
                "created" => {
                    created.push(PathBuf::from(unescape(fields.next().unwrap_or(""))));
                }
                "replaced" => {
                    let (Some(path), Some(backup)) = (fields.next(), fields.next()) else {
                        return Err("bad replaced line".to_string());
                    };
                    replaced.push(Replaced {
                        path: PathBuf::from(unescape(path)),
                        backup: PathBuf::from(unescape(backup)),
                    });
                }
                other => return Err(format!("unknown line: {}", other)),
            }
        }

        let appid = appid.ok_or_else(|| "missing appid".to_string())?;
        let launch_options = launch_options.ok_or_else(|| "missing launch-options".to_string())?;

        Ok(UndoRecord {
            appid,
            launch_options_before: launch_options,
            created,
            replaced,
        })
    }
}

/// Where one game's undo record lives, `<data_dir>/undo/<appid>.txt`.
pub fn record_path(data_dir: &Path, appid: u32) -> PathBuf {
    data_dir.join("undo").join(format!("{}.txt", appid))
}

/// The engine's data directory: `$XDG_DATA_HOME/flat3d`, or `~/.local/share/flat3d` when
/// `XDG_DATA_HOME` is unset or empty.
pub fn data_dir(xdg_data_home: Option<&str>, home: &str) -> PathBuf {
    match xdg_data_home.filter(|s| !s.is_empty()) {
        Some(xdg) => PathBuf::from(xdg).join("flat3d"),
        None => PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("flat3d"),
    }
}

fn io_error(what: &str, path: &Path, e: &io::Error) -> ApiError {
    ApiError::new("io", &format!("{} {}: {}", what, path.display(), e))
}

/// Persist an undo record atomically, refusing to replace an existing recovery point.
pub fn save_record(data_dir: &Path, rec: &UndoRecord) -> Result<PathBuf, ApiError> {
    let path = record_path(data_dir, rec.appid);
    fs::create_dir_all(path.parent().expect("record path has parent"))
        .map_err(|e| io_error("cannot create", path.parent().unwrap(), &e))?;
    if path.exists() {
        return Err(ApiError::new(
            "refused",
            &format!(
                "an undo record for {} already exists; run flat3d-inject undo {} first",
                rec.appid, rec.appid
            ),
        ));
    }
    crate::fsx::write_atomic(&path, rec.to_text().as_bytes())
        .map_err(|e| io_error("cannot write", &path, &e))?;
    Ok(path)
}

/// Attempt every file restoration step and collect a line describing each outcome.
pub fn put_back(rec: &UndoRecord) -> Vec<String> {
    let mut restored = Vec::new();
    for action in undo_actions(rec) {
        match action {
            UndoAction::RemoveFile(file) => match fs::remove_file(&file) {
                Ok(()) => restored.push(format!("removed {}", file.display())),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    restored.push(format!("already gone {}", file.display()))
                }
                Err(e) => restored.push(format!("failed to remove {}: {}", file.display(), e)),
            },
            UndoAction::RestoreFile { backup, path } => match fs::copy(&backup, &path) {
                Ok(_) => match fs::remove_file(&backup) {
                    Ok(()) => restored.push(format!(
                        "restored {} from {}",
                        path.display(),
                        backup.display()
                    )),
                    Err(e) => {
                        restored.push(format!("failed to remove {}: {}", backup.display(), e))
                    }
                },
                Err(e) if e.kind() == io::ErrorKind::NotFound && !backup.exists() => {
                    restored.push(format!("already restored {}", path.display()))
                }
                Err(e) => restored.push(format!(
                    "failed to restore {} from {}: {}",
                    path.display(),
                    backup.display(),
                    e
                )),
            },
            UndoAction::SetLaunchOptions(_) => {}
        }
    }
    restored
}

/// Put a game back to flat from its undo record (`undo <appid>`).
///
/// Runs the file steps of [`undo_actions`] on disk: created files are removed, replaced files
/// get their backups copied back. The launch option step is not run here: the Decky plugin
/// removes only our wrapper word from the line (code map). The record is deleted only after
/// every file step has worked, so a failed undo can be run again; a file already gone or a
/// backup already restored counts as done.
pub fn undo_game(data_dir: &Path, appid: u32) -> Result<UndoResult, ApiError> {
    let path = record_path(data_dir, appid);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(ApiError::new(
                "not-found",
                &format!("no undo record for {}", appid),
            ));
        }
        Err(e) => return Err(io_error("cannot read", &path, &e)),
    };
    let record = UndoRecord::from_text(&text).map_err(|m| {
        ApiError::new(
            "command-failed",
            &format!("undo record for {}: {}", appid, m),
        )
    })?;
    if record.appid != appid {
        return Err(ApiError::new(
            "command-failed",
            &format!("undo record is for {}, not {}", record.appid, appid),
        ));
    }

    let mut restored = Vec::new();
    for action in undo_actions(&record) {
        match action {
            UndoAction::RemoveFile(file) => match fs::remove_file(&file) {
                Ok(()) => restored.push(format!("removed {}", file.display())),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    restored.push(format!("already gone {}", file.display()))
                }
                Err(e) => return Err(io_error("cannot remove", &file, &e)),
            },
            UndoAction::RestoreFile { backup, path } => match fs::copy(&backup, &path) {
                Ok(_) => {
                    fs::remove_file(&backup).map_err(|e| io_error("cannot remove", &backup, &e))?;
                    restored.push(format!(
                        "restored {} from {}",
                        path.display(),
                        backup.display()
                    ));
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound && !backup.exists() => {
                    restored.push(format!("already restored {}", path.display()))
                }
                Err(e) => return Err(io_error("cannot restore", &path, &e)),
            },
            UndoAction::SetLaunchOptions(_) => {}
        }
    }

    fs::remove_file(&path).map_err(|e| io_error("cannot remove", &path, &e))?;
    let summary = format!(
        "{} change(s) put back for {}; the launch option is left to the Decky plugin",
        restored.len(),
        appid
    );
    Ok(UndoResult {
        appid,
        restored,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh folder under the system temp dir for one test; the name keeps tests apart.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("flat3d-undo-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A fake game folder after an inject: one file created, one replaced with a backup, and
    /// the undo record that says so.
    fn injected(root: &Path, appid: u32) -> PathBuf {
        let game = root.join("game");
        fs::create_dir_all(&game).unwrap();
        let data = root.join("data");
        fs::create_dir_all(data.join("undo")).unwrap();
        fs::write(game.join("a.dll"), "injected").unwrap();
        fs::write(game.join("b.ini"), "stereo").unwrap();
        fs::write(game.join("b.ini.bak"), "original").unwrap();
        let record = UndoRecord {
            appid,
            launch_options_before: "-novid".to_string(),
            created: vec![game.join("a.dll")],
            replaced: vec![Replaced {
                path: game.join("b.ini"),
                backup: game.join("b.ini.bak"),
            }],
        };
        fs::write(record_path(&data, appid), record.to_text()).unwrap();
        data
    }

    #[test]
    fn undo_puts_the_files_back_and_deletes_the_record() {
        let root = scratch("clean");
        let data = injected(&root, 2651280);
        let game = root.join("game");

        let got = undo_game(&data, 2651280).unwrap();

        assert_eq!(got.appid, 2651280);
        assert_eq!(got.restored.len(), 2);
        assert!(got.restored[0].starts_with("removed "));
        assert!(got.restored[1].starts_with("restored "));
        assert!(got.summary.contains("2 change(s)"));
        assert!(!game.join("a.dll").exists());
        assert_eq!(fs::read_to_string(game.join("b.ini")).unwrap(), "original");
        assert!(!game.join("b.ini.bak").exists());
        assert!(!record_path(&data, 2651280).exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_second_undo_has_no_record() {
        let root = scratch("second");
        let data = injected(&root, 7);
        undo_game(&data, 7).unwrap();
        let err = undo_game(&data, 7).unwrap_err();
        assert_eq!(err.code, "not-found");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_already_gone_counts_as_done() {
        let root = scratch("gone");
        let data = injected(&root, 8);
        fs::remove_file(root.join("game").join("a.dll")).unwrap();
        let got = undo_game(&data, 8).unwrap();
        assert!(got.restored[0].starts_with("already gone "));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_record_for_another_game_is_refused() {
        let root = scratch("other");
        let data = injected(&root, 9);
        fs::copy(record_path(&data, 9), record_path(&data, 10)).unwrap();
        let err = undo_game(&data, 10).unwrap_err();
        assert_eq!(err.code, "command-failed");
        assert!(record_path(&data, 10).exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_record_is_not_found() {
        let root = scratch("missing");
        let err = undo_game(&root, 3).unwrap_err();
        assert_eq!(err.code, "not-found");
        assert_eq!(err.message, "no undo record for 3");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn data_dir_uses_xdg_then_home() {
        assert_eq!(data_dir(Some("/x"), "/h"), PathBuf::from("/x/flat3d"));
        assert_eq!(
            data_dir(Some(""), "/h"),
            PathBuf::from("/h/.local/share/flat3d")
        );
        assert_eq!(
            data_dir(None, "/h"),
            PathBuf::from("/h/.local/share/flat3d")
        );
    }
}
