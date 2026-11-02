//! The undo record: how to put a game back to flat. docs/code-map.md, Parts.

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
fn escape(s: &str) -> String {
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
fn unescape(s: &str) -> String {
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
