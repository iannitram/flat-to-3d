//! The Epic Games Launcher hook: a marked block in Proton's `user_settings.py`.
//!
//! The launcher has no per-game environment, so the one per-game hook is the Python file Proton
//! reads (`$HOME/.config/epic-proton/user_settings.py`). Our block adds the 3D launch
//! environment to its `user_settings` dict only when the game's exe is in `sys.argv`.
//!
//! Several Epic games share the one file, so each game's undo record names the file's origin
//! (a `replaced` line pointing at one shared backup of the original bytes, or a `created` line
//! with the folders we made), never a snapshot of the file. Undo removes only that game's block;
//! the original bytes come back, or the file and our folders go, only when the last of our
//! blocks leaves.

use crate::api::{ApiError, UndoResult};
use crate::injectors::{Route, RouteEnv};
use crate::link::WivrnDirs;
use crate::paths::EnginePaths;
use crate::recipes::StoreKey;
use crate::undo::{Replaced, UndoRecord};
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// The start and end every one of our begin markers shares; the app name sits between.
const BEGIN_PREFIX: &str = "# >>> flat-to-3d epic:";
const BEGIN_SUFFIX: &str = " >>>";

/// The text of a `user_settings.py` we create because there was none.
pub const CREATED_TEXT: &str = "# Written by flat-to-3d.\nuser_settings = {}\n";

/// The one variable whose value is merged into what is already set, never overwritten.
const FS_RW: &str = "PRESSURE_VESSEL_FILESYSTEMS_RW";

/// Where the launcher's Proton reads per-game settings, under `home`.
pub fn user_settings_path(home: &Path) -> PathBuf {
    home.join(".config/epic-proton/user_settings.py")
}

/// The one backup of `user_settings.py` as it was before our first block. Every game's record
/// names it; it cannot clash with a record (`epic-<app name>.txt`, `<appid>.txt`).
pub fn shared_backup_path(data: &Path) -> PathBuf {
    data.join("undo").join("user_settings.py.flat3d-epic.bak")
}

/// The line that opens our block for `app_name`.
pub fn begin_marker(app_name: &str) -> String {
    format!("{BEGIN_PREFIX}{app_name}{BEGIN_SUFFIX}")
}

/// The line that closes our block for `app_name`.
pub fn end_marker(app_name: &str) -> String {
    format!("# <<< flat-to-3d epic:{app_name} <<<")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    Added,
    /// The same block is in the file and this game has an undo record for it.
    AlreadyThere,
}

/// Refuse an exe name that is not a plain file name made of safe characters. The name goes
/// inside a Python string literal, so quotes, backslashes and control characters are refused.
pub fn check_exe(exe: &str) -> Result<(), ApiError> {
    let plain = !exe.is_empty()
        && !exe.starts_with('.')
        && exe
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ' '));
    if plain {
        Ok(())
    } else {
        Err(ApiError::new(
            "refused",
            &format!("bad Epic game exe name: {exe:?}"),
        ))
    }
}

/// The 3D launch environment the Steam wrapper sets for `route`, built by the same function
/// (`wrapper::exec_spec_with`) from the same inputs: the injector env `ensure_applied` returned,
/// the `detect_server` folders and the inherited `PRESSURE_VESSEL_FILESYSTEMS_RW`.
pub fn three_d_env(
    route: Route,
    injector: &RouteEnv,
    wivrn: &WivrnDirs,
    existing_fs_rw: Option<&str>,
) -> RouteEnv {
    let spec = crate::wrapper::exec_spec_with(
        &crate::wrapper::Plan::ThreeD { route },
        &[],
        injector,
        wivrn,
        existing_fs_rw,
    );
    RouteEnv {
        env: spec.env,
        unset: spec.unset,
        sidecar: spec.sidecar,
    }
}

fn py_str(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// The marked block, ending in a newline. `PRESSURE_VESSEL_FILESYSTEMS_RW` is merged at launch
/// time into the value the file or the launcher already set, the way the Steam wrapper merges
/// the inherited value: kept as it is, with each of our folders added when missing.
pub fn block(app_name: &str, exe: &str, env: &RouteEnv) -> String {
    let mut text = format!("{}\n", begin_marker(app_name));
    text.push_str("import os as _f3d_os\nimport sys as _f3d_sys\n");
    text.push_str(&format!(
        "if any(_f3d_os.path.basename(a.replace('\\\\', '/')).lower() == {} for a in _f3d_sys.argv):\n",
        py_str(&exe.to_lowercase())
    ));
    for (key, value) in &env.env {
        if key == FS_RW {
            text.push_str(&format!(
                "    _f3d_rw = user_settings.get({k}) or _f3d_os.environ.get({k}, '')\n",
                k = py_str(key)
            ));
            text.push_str(&format!(
                "    for _f3d_dir in {}.split(':'):\n",
                py_str(value)
            ));
            text.push_str("        if _f3d_dir and _f3d_dir not in _f3d_rw.split(':'):\n");
            text.push_str(
                "            _f3d_rw = _f3d_rw + ':' + _f3d_dir if _f3d_rw else _f3d_dir\n",
            );
            text.push_str(&format!("    user_settings[{}] = _f3d_rw\n", py_str(key)));
        } else {
            text.push_str(&format!(
                "    user_settings[{}] = {}\n",
                py_str(key),
                py_str(value)
            ));
        }
    }
    if env.env.is_empty() {
        text.push_str("    pass\n");
    }
    text.push_str(&format!("{}\n", end_marker(app_name)));
    text
}

fn invalid(path: &Path, message: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {message}", path.display()),
    )
}

fn line_text(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

/// The byte range of our block for `app_name` in `text`, marker lines included. A begin
/// marker with no end marker, or a block that appears twice, is an error.
fn block_span(path: &Path, text: &str, app_name: &str) -> io::Result<Option<Range<usize>>> {
    let (begin, end) = (begin_marker(app_name), end_marker(app_name));
    let mut found = None;
    let mut start = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let here = offset;
        offset += line.len();
        let bare = line_text(line);
        match start {
            None if bare == begin => {
                if found.is_some() {
                    return Err(invalid(path, "our begin marker appears more than once"));
                }
                start = Some(here);
            }
            Some(_) if bare == begin => {
                return Err(invalid(path, "our begin marker appears more than once"));
            }
            Some(s) if bare == end => {
                found = Some(s..offset);
                start = None;
            }
            _ => {}
        }
    }
    if start.is_some() {
        return Err(invalid(path, "our begin marker has no end marker"));
    }
    Ok(found)
}

/// The app names of all our blocks in `text`, in file order.
pub fn block_names(text: &str) -> Vec<String> {
    text.split_inclusive('\n')
        .filter_map(|line| {
            line_text(line)
                .strip_prefix(BEGIN_PREFIX)?
                .strip_suffix(BEGIN_SUFFIX)
                .map(str::to_string)
        })
        .collect()
}

/// `original` with our block for `app_name` replaced in place, or appended when absent. Lines
/// outside the block are never touched. A begin marker with no end marker is an error.
pub fn splice(path: &Path, original: &str, app_name: &str, new_block: &str) -> io::Result<String> {
    match block_span(path, original, app_name)? {
        Some(span) => Ok(format!(
            "{}{new_block}{}",
            &original[..span.start],
            &original[span.end..]
        )),
        None => {
            let mut text = original.to_string();
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(new_block);
            Ok(text)
        }
    }
}

/// `text` without our block for `app_name`; every other byte is kept. No block: unchanged.
pub fn remove_block(path: &Path, text: &str, app_name: &str) -> io::Result<String> {
    Ok(match block_span(path, text, app_name)? {
        Some(span) => format!("{}{}", &text[..span.start], &text[span.end..]),
        None => text.to_string(),
    })
}

fn io_error(error: io::Error) -> ApiError {
    ApiError::new("command-failed", &error.to_string())
}

/// The file's bytes and text, or `None` when it does not exist.
fn read_file(file: &Path) -> Result<Option<(Vec<u8>, String)>, ApiError> {
    match fs::read(file) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes.clone())
                .map_err(|_| io_error(invalid(file, "not UTF-8 text")))?;
            Ok(Some((bytes, text)))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io_error(e)),
    }
}

/// Whether `record` is a `user_settings.py` record (and not, say, a Heroic config one).
fn is_ours(record: &UndoRecord, file: &Path) -> bool {
    record.replaced.iter().any(|r| r.path == file) || record.created.iter().any(|p| p == file)
}

/// Every other Epic game's `user_settings.py` undo record, sorted by app name.
fn peer_records(
    paths: &EnginePaths,
    file: &Path,
    except: &str,
) -> Result<Vec<(String, UndoRecord)>, ApiError> {
    let dir = paths.data.join("undo");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io_error(e)),
    };
    let mut peers = Vec::new();
    for entry in entries {
        let path = entry.map_err(io_error)?.path();
        let Some(app) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_prefix("epic-"))
            .and_then(|name| name.strip_suffix(".txt"))
            .map(str::to_string)
        else {
            continue;
        };
        let key = StoreKey::Epic(app.clone());
        if app == except || crate::undo::record_path_keyed(&paths.data, &key) != path {
            continue;
        }
        let record = crate::undo::read_record_keyed(&paths.data, &key)?;
        if record.key() == key && is_ours(&record, file) {
            peers.push((app, record));
        }
    }
    peers.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(peers)
}

/// Write our block for an Epic game into `user_settings.py`.
///
/// A game with no undo record gets one before the file changes. Its record names the file's
/// origin: the shared backup of the original bytes, or the file and folders we created. When
/// another game's block is already there, that game's record gives the origin. `AlreadyThere`
/// is returned only for a block this game has a record for; a block of ours with no record is
/// reported, never adopted silently.
pub fn apply(
    paths: &EnginePaths,
    app_name: &str,
    exe: &str,
    env: &RouteEnv,
) -> Result<Applied, ApiError> {
    crate::heroic_wrapper::check_app_name(app_name)?;
    check_exe(exe)?;
    if !env.unset.is_empty() || env.sidecar.is_some() {
        return Err(ApiError::new(
            "not-built",
            "this route unsets variables or starts a sidecar, which user_settings.py cannot do",
        ));
    }
    let file = user_settings_path(&paths.home);
    let existing = read_file(&file)?;
    let base = match &existing {
        Some((_, text)) => text.clone(),
        None => CREATED_TEXT.to_string(),
    };
    let new_block = block(app_name, exe, env);
    let span = block_span(&file, &base, app_name).map_err(io_error)?;
    let key = StoreKey::Epic(app_name.to_string());
    let record_path = crate::undo::record_path_keyed(&paths.data, &key);

    if record_path.exists() {
        let own = crate::undo::read_record_keyed(&paths.data, &key)?;
        if !is_ours(&own, &file) {
            return Err(ApiError::new(
                "refused",
                &format!(
                    "an undo record for epic:{app_name} already exists for another change; \
                     undo it first"
                ),
            ));
        }
        if existing.is_some() && span.as_ref().is_some_and(|s| base[s.clone()] == new_block) {
            return Ok(Applied::AlreadyThere);
        }
        let updated = splice(&file, &base, app_name, &new_block).map_err(io_error)?;
        crate::fsx::write_atomic(&file, updated.as_bytes()).map_err(io_error)?;
        return Ok(Applied::Added);
    }

    if span.is_some() {
        return Err(ApiError::new(
            "refused",
            &format!(
                "{} already holds a flat-to-3d block for epic:{app_name} with no undo record; \
                 delete the lines from `{}` to `{}`, then turn 3D on again",
                file.display(),
                begin_marker(app_name),
                end_marker(app_name)
            ),
        ));
    }
    let peers = peer_records(paths, &file, app_name)?;
    let others = block_names(&base);
    let mut record = UndoRecord {
        appid: 0,
        launch_options_before: format!("epic:{app_name}"),
        created: vec![],
        replaced: vec![],
        created_dirs: vec![],
    };
    let mut backup_made = None;
    if let Some((_, peer)) = peers.first() {
        if existing.is_none() {
            return Err(ApiError::new(
                "refused",
                &format!(
                    "{} is gone but undo records for it remain (epic:{}); undo those first",
                    file.display(),
                    peers
                        .iter()
                        .map(|(app, _)| app.as_str())
                        .collect::<Vec<_>>()
                        .join(", epic:")
                ),
            ));
        }
        // Every record for the file names the same origin; take it from the first.
        record.replaced = peer
            .replaced
            .iter()
            .filter(|r| r.path == file)
            .cloned()
            .collect();
        record.created = peer
            .created
            .iter()
            .filter(|p| **p == file)
            .cloned()
            .collect();
        record.created_dirs = peer.created_dirs.clone();
    } else if !others.is_empty() {
        return Err(ApiError::new(
            "refused",
            &format!(
                "{} holds flat-to-3d blocks with no undo record (epic:{}); delete them first",
                file.display(),
                others.join(", epic:")
            ),
        ));
    } else {
        match &existing {
            Some((bytes, _)) => {
                let backup = shared_backup_path(&paths.data);
                crate::fsx::write_atomic(&backup, bytes).map_err(io_error)?;
                record.replaced.push(Replaced {
                    path: file.clone(),
                    backup: backup.clone(),
                });
                backup_made = Some(backup);
            }
            None => {
                if let Some(parent) = file.parent() {
                    crate::undo::create_dir_all_recorded(parent, &mut record).map_err(io_error)?;
                }
                record.created.push(file.clone());
            }
        }
    }
    // Rollback removes only what this call made; copied peer folders are not ours to remove.
    let rollback = |record: &UndoRecord| {
        if let Some(backup) = &backup_made {
            let _ = fs::remove_file(backup);
        }
        if peers.is_empty() {
            crate::undo::remove_created_dirs(record);
        }
    };
    if let Err(e) = crate::undo::save_record_keyed(&paths.data, &key, &record) {
        rollback(&record);
        return Err(e);
    }
    let updated = splice(&file, &base, app_name, &new_block).map_err(io_error)?;
    if let Err(e) = crate::fsx::write_atomic(&file, updated.as_bytes()) {
        let _ = fs::remove_file(&record_path);
        rollback(&record);
        return Err(io_error(e));
    }
    Ok(Applied::Added)
}

/// Undo an Epic game's change from its undo record. For a `user_settings.py` record only this
/// game's block is removed; when it was the last of our blocks, the file goes back to its
/// original bytes, or is deleted with the folders we made. A record for another change (a
/// Heroic config) is undone the usual way.
pub fn undo(paths: &EnginePaths, app_name: &str) -> Result<UndoResult, ApiError> {
    crate::heroic_wrapper::check_app_name(app_name)?;
    let key = StoreKey::Epic(app_name.to_string());
    let record = crate::undo::read_record_keyed(&paths.data, &key)?;
    let file = user_settings_path(&paths.home);
    if !is_ours(&record, &file) {
        return crate::undo::undo_game_keyed(&paths.data, &key);
    }
    let peers = peer_records(paths, &file, app_name)?;
    let mut restored = Vec::new();
    let mut last = true;
    match read_file(&file)? {
        None => restored.push(format!("already gone {}", file.display())),
        Some((_, text)) => {
            let left = remove_block(&file, &text, app_name).map_err(io_error)?;
            if left == text {
                restored.push(format!(
                    "already gone: the epic:{app_name} block in {}",
                    file.display()
                ));
            }
            if block_names(&left).is_empty() {
                restored.extend(put_back_original(&file, &record, &left)?);
            } else {
                last = false;
                if left != text {
                    crate::fsx::write_atomic(&file, left.as_bytes()).map_err(io_error)?;
                    restored.push(format!(
                        "removed the epic:{app_name} block from {}",
                        file.display()
                    ));
                }
            }
        }
    }
    if last && peers.is_empty() {
        if !file.exists() {
            restored.extend(crate::undo::remove_created_dirs_reported(&record).map_err(io_error)?);
        }
        for r in record.replaced.iter().filter(|r| r.path == file) {
            match fs::remove_file(&r.backup) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(io_error(e)),
            }
        }
    }
    let record_path = crate::undo::record_path_keyed(&paths.data, &key);
    fs::remove_file(&record_path).map_err(io_error)?;
    let changes = restored
        .iter()
        .filter(|l| !l.starts_with("kept ") && !l.starts_with("already "))
        .count();
    Ok(UndoResult {
        appid: 0,
        restored,
        summary: format!("{changes} change(s) put back for epic:{app_name}"),
    })
}

/// The last of our blocks has left: `left` is the file without it. Put back the original
/// bytes, or delete the file we created and our folders. A file edited outside our blocks
/// since then keeps the edits (`left` is written) and is named as kept.
fn put_back_original(
    file: &Path,
    record: &UndoRecord,
    left: &str,
) -> Result<Vec<String>, ApiError> {
    let mut restored = Vec::new();
    if let Some(r) = record.replaced.iter().find(|r| r.path == file) {
        let original = match fs::read(&r.backup) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(io_error(e)),
        };
        // Our first block was appended after a newline we added when the file lacked one.
        let unedited = original.as_ref().filter(|original| {
            let mut joined = original.to_vec();
            joined.push(b'\n');
            left.as_bytes() == original.as_slice()
                || (!original.is_empty()
                    && !original.ends_with(b"\n")
                    && left.as_bytes() == joined.as_slice())
        });
        match unedited {
            Some(original) => {
                crate::fsx::write_atomic(file, original).map_err(io_error)?;
                restored.push(format!(
                    "restored {} from {}",
                    file.display(),
                    r.backup.display()
                ));
            }
            None => {
                crate::fsx::write_atomic(file, left.as_bytes()).map_err(io_error)?;
                restored.push(format!(
                    "removed our last block from {}; kept the rest (edited since 3D was turned on)",
                    file.display()
                ));
            }
        }
    } else if left == CREATED_TEXT {
        match fs::remove_file(file) {
            Ok(()) => restored.push(format!("removed {}", file.display())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                restored.push(format!("already gone {}", file.display()))
            }
            Err(e) => return Err(io_error(e)),
        }
    } else {
        crate::fsx::write_atomic(file, left.as_bytes()).map_err(io_error)?;
        restored.push(format!(
            "removed our last block from {}; kept the rest (edited since 3D was turned on)",
            file.display()
        ));
    }
    Ok(restored)
}
