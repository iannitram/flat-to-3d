//! The Heroic per-game wrapper: Steam's launch-option wrapper rule for an Epic game.

use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Where Heroic stores one game's settings under its config root.
pub fn games_config(config_root: &Path, app_name: &str) -> PathBuf {
    config_root
        .join("GamesConfig")
        .join(format!("{app_name}.json"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    Added,
    AlreadyThere,
}

fn invalid_data(config: &Path, message: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {message}", config.display()),
    )
}

fn parse_config(config: &Path, bytes: &[u8]) -> io::Result<Value> {
    serde_json::from_slice(bytes).map_err(|error| invalid_data(config, &error.to_string()))
}

fn contains_wrapper(entries: &[Value], wrapper_abs: &str) -> bool {
    entries
        .iter()
        .any(|entry| entry.get("exe").and_then(Value::as_str) == Some(wrapper_abs))
}

/// Append our wrapper once, saving the original bytes before changing the config.
pub fn apply(
    config: &Path,
    app_name: &str,
    wrapper_abs: &str,
    backup: &Path,
) -> io::Result<Applied> {
    let original = fs::read(config)?;
    let mut value = parse_config(config, &original)?;
    let settings = value
        .as_object_mut()
        .and_then(|root| root.get_mut(app_name))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            invalid_data(
                config,
                "expected an object holding the game's settings object",
            )
        })?;
    let entries = settings
        .entry("wrapperOptions")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| invalid_data(config, "wrapperOptions must be an array"))?;
    if contains_wrapper(entries, wrapper_abs) {
        return Ok(Applied::AlreadyThere);
    }

    if !backup.try_exists()? {
        if let Some(parent) = backup.parent().filter(|path| !path.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        crate::fsx::write_atomic(backup, &original)?;
    }
    entries.push(json!({ "exe": wrapper_abs, "args": "" }));
    let mut text = serde_json::to_string_pretty(&value)
        .map_err(|error| invalid_data(config, &error.to_string()))?;
    text.push('\n');
    crate::fsx::write_atomic(config, text.as_bytes())?;
    Ok(Applied::Added)
}

/// Restore the exact original bytes, deleting the backup only after restoration succeeds.
pub fn undo(config: &Path, backup: &Path) -> io::Result<bool> {
    let original = match fs::read(backup) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    crate::fsx::write_atomic(config, &original)?;
    fs::remove_file(backup)?;
    Ok(true)
}

/// Whether the per-game settings already contain this exact wrapper executable.
pub fn has_wrapper(config: &Path, app_name: &str, wrapper_abs: &str) -> io::Result<bool> {
    let value = parse_config(config, &fs::read(config)?)?;
    let settings = value
        .as_object()
        .and_then(|root| root.get(app_name))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            invalid_data(
                config,
                "expected an object holding the game's settings object",
            )
        })?;
    match settings.get("wrapperOptions") {
        None => Ok(false),
        Some(value) => {
            let entries = value
                .as_array()
                .ok_or_else(|| invalid_data(config, "wrapperOptions must be an array"))?;
            Ok(contains_wrapper(entries, wrapper_abs))
        }
    }
}
