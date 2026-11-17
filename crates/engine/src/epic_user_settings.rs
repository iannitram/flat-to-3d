//! The Epic Games Launcher hook: a marked block in Proton's `user_settings.py`.
//!
//! The launcher has no per-game environment, so the one per-game hook is the Python file Proton
//! reads (`$HOME/.config/epic-proton/user_settings.py`). Our block adds the 3D launch
//! environment to its `user_settings` dict only when the game's exe is in `sys.argv`.

use crate::api::ApiError;
use crate::injectors::{Route, RouteEnv};
use crate::paths::EnginePaths;
use crate::recipes::StoreKey;
use crate::undo::{Replaced, UndoRecord};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Where the launcher's Proton reads per-game settings, under `home`.
pub fn user_settings_path(home: &Path) -> PathBuf {
    home.join(".config/epic-proton/user_settings.py")
}

/// The line that opens our block for `app_name`.
pub fn begin_marker(app_name: &str) -> String {
    format!("# >>> flat-to-3d epic:{app_name} >>>")
}

/// The line that closes our block for `app_name`.
pub fn end_marker(app_name: &str) -> String {
    format!("# <<< flat-to-3d epic:{app_name} <<<")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    Added,
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

/// The 3D launch environment the Steam wrapper sets for a 3D launch, as it would for this
/// route with no inherited value (`wrapper::exec_spec_with`). One source for both stores.
pub fn three_d_env(home: &Path) -> RouteEnv {
    let wivrn = crate::link::WivrnDirs::user_default(home);
    let spec = crate::wrapper::exec_spec_with(
        &crate::wrapper::Plan::ThreeD { route: Route::Aw2 },
        &[],
        &RouteEnv::default(),
        &wivrn,
        None,
    );
    RouteEnv {
        env: spec.env,
        unset: spec.unset,
        sidecar: None,
    }
}

fn py_str(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// The marked block, ending in a newline.
pub fn block(app_name: &str, exe: &str, env: &RouteEnv) -> String {
    let mut text = format!("{}\n", begin_marker(app_name));
    text.push_str("import os as _f3d_os\nimport sys as _f3d_sys\n");
    text.push_str(&format!(
        "if any(_f3d_os.path.basename(a.replace('\\\\', '/')).lower() == {} for a in _f3d_sys.argv):\n",
        py_str(&exe.to_lowercase())
    ));
    for (key, value) in &env.env {
        text.push_str(&format!(
            "    user_settings[{}] = {}\n",
            py_str(key),
            py_str(value)
        ));
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

/// `original` with our block for `app_name` replaced in place, or appended when absent. Lines
/// outside the block are never touched. A begin marker with no end marker is an error.
pub fn splice(path: &Path, original: &str, app_name: &str, new_block: &str) -> io::Result<String> {
    let (begin, end) = (begin_marker(app_name), end_marker(app_name));
    let lines: Vec<&str> = original.split_inclusive('\n').collect();
    let line_is = |line: &&str, marker: &str| line.trim_end_matches(['\n', '\r']) == marker;
    let begins: Vec<usize> = (0..lines.len())
        .filter(|i| line_is(&lines[*i], &begin))
        .collect();
    match begins.as_slice() {
        [] => {
            let mut text = original.to_string();
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(new_block);
            Ok(text)
        }
        [start] => {
            let stop = (*start..lines.len())
                .find(|i| line_is(&lines[*i], &end))
                .ok_or_else(|| invalid(path, "our begin marker has no end marker"))?;
            let mut text: String = lines[..*start].concat();
            text.push_str(new_block);
            text.push_str(&lines[stop + 1..].concat());
            Ok(text)
        }
        _ => Err(invalid(path, "our begin marker appears more than once")),
    }
}

fn io_error(error: io::Error) -> ApiError {
    ApiError::new("command-failed", &error.to_string())
}

/// Write our block for an Epic game into `user_settings.py`, saving an undo record on the
/// first apply. A repeated apply leaves one block and the first record in place.
pub fn apply(
    paths: &EnginePaths,
    app_name: &str,
    exe: &str,
    env: &RouteEnv,
) -> Result<Applied, ApiError> {
    crate::heroic_wrapper::check_app_name(app_name)?;
    check_exe(exe)?;
    let file = user_settings_path(&paths.home);
    let existing = match fs::read(&file) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(io_error(e)),
    };
    let original = match &existing {
        Some(bytes) => std::str::from_utf8(bytes)
            .map_err(|_| io_error(invalid(&file, "not UTF-8 text")))?
            .to_string(),
        None => "# Written by flat-to-3d.\nuser_settings = {}\n".to_string(),
    };
    let updated =
        splice(&file, &original, app_name, &block(app_name, exe, env)).map_err(io_error)?;
    if existing.is_some() && updated == original {
        return Ok(Applied::AlreadyThere);
    }

    let key = StoreKey::Epic(app_name.to_string());
    let record_path = crate::undo::record_path_keyed(&paths.data, &key);
    let first = !record_path.exists();
    let mut record = UndoRecord {
        appid: 0,
        launch_options_before: format!("epic:{app_name}"),
        created: vec![],
        replaced: vec![],
        created_dirs: vec![],
    };
    let mut backup_made = None;
    if first {
        match &existing {
            Some(bytes) => {
                let backup = paths
                    .data
                    .join("undo")
                    .join(format!("epic-{app_name}.user_settings.bak"));
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
        if let Err(e) = crate::undo::save_record_keyed(&paths.data, &key, &record) {
            if let Some(backup) = &backup_made {
                let _ = fs::remove_file(backup);
            }
            crate::undo::remove_created_dirs(&record);
            return Err(e);
        }
    }
    if let Err(e) = crate::fsx::write_atomic(&file, updated.as_bytes()) {
        if first {
            let _ = fs::remove_file(&record_path);
            if let Some(backup) = &backup_made {
                let _ = fs::remove_file(backup);
            }
            crate::undo::remove_created_dirs(&record);
        }
        return Err(io_error(e));
    }
    Ok(Applied::Added)
}

/// Undo an Epic game's `user_settings.py` change from its undo record: the file goes back to
/// its original bytes, or is deleted when we created it.
pub fn undo(paths: &EnginePaths, app_name: &str) -> Result<crate::api::UndoResult, ApiError> {
    crate::heroic_wrapper::check_app_name(app_name)?;
    crate::undo::undo_game_keyed(&paths.data, &StoreKey::Epic(app_name.to_string()))
}
