//! The Heroic per-game wrapper: Steam's launch-option wrapper rule for an Epic game.

use crate::api::ApiError;
use crate::recipes::StoreKey;
use crate::undo::{Replaced, UndoRecord};
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

/// Locate the Heroic config root for the game under `home`, checking Flatpak then native config.
pub fn find_config_root(home: &Path, app_name: &str) -> PathBuf {
    let flatpak = home.join(crate::heroic::FLATPAK_CONFIG);
    let native = home.join(crate::heroic::NATIVE_CONFIG);
    if games_config(&flatpak, app_name).exists() {
        flatpak
    } else if games_config(&native, app_name).exists() {
        native
    } else if flatpak.exists() {
        flatpak
    } else {
        native
    }
}

/// Apply our wrapper to an Epic game's Heroic config, saving an undo record with the backup.
pub fn apply_epic(paths: &crate::paths::EnginePaths, app_name: &str) -> Result<Applied, ApiError> {
    check_app_name(app_name)?;
    let config_root = find_config_root(&paths.home, app_name);
    let config = games_config(&config_root, app_name);
    if !config.exists() {
        return Err(ApiError::new(
            "not-found",
            &format!("no Heroic config for {app_name} at {}", config.display()),
        ));
    }
    let wrapper_abs = paths.wrapper_bin().to_string_lossy().into_owned();
    let backup = paths
        .data
        .join("undo")
        .join(format!("epic-{app_name}.backup.json"));
    let applied = apply(&config, app_name, &wrapper_abs, &backup)
        .map_err(|e| ApiError::new("command-failed", &e.to_string()))?;

    let key = StoreKey::Epic(app_name.to_string());
    let record_path = crate::undo::record_path_keyed(&paths.data, &key);
    if !record_path.exists() {
        let record = UndoRecord {
            appid: 0,
            launch_options_before: format!("epic:{app_name}"),
            created: vec![],
            replaced: vec![Replaced {
                path: config,
                backup,
            }],
            created_dirs: vec![],
        };
        crate::undo::save_record_keyed(&paths.data, &key, &record)?;
    }
    Ok(applied)
}

/// Refuse an app name that could not be a plain file name: empty, a path separator, `..`, a control character (NUL, newline).
pub fn check_app_name(app_name: &str) -> Result<(), ApiError> {
    if app_name.is_empty()
        || app_name.contains(['/', '\\'])
        || app_name.chars().any(char::is_control)
        || app_name.contains("..")
    {
        return Err(ApiError::new(
            "refused",
            &format!("bad Epic app name: {app_name:?}"),
        ));
    }
    Ok(())
}

/// Turn 3D on for one store key: a Steam key gets the launch-option wrapper (the new text is
/// returned); an `epic:` key goes through `ensure_applied` to the Epic Games Launcher's
/// `user_settings.py` hook, with an undo record. The Heroic code above stays for the catalog.
pub fn apply_for_key(
    paths: &crate::paths::EnginePaths,
    key: &StoreKey,
    options: &str,
) -> Result<String, ApiError> {
    apply_for_key_with(paths, key, options, &mut crate::exec::SystemRunner)
}

/// [`apply_for_key`] with the command runner given (WiVRn detection for an `epic:` key).
pub fn apply_for_key_with(
    paths: &crate::paths::EnginePaths,
    key: &StoreKey,
    options: &str,
    runner: &mut dyn crate::exec::CommandRunner,
) -> Result<String, ApiError> {
    match key {
        StoreKey::Epic(app_name) => {
            check_app_name(app_name)?;
            let text = crate::recipes::find_recipe_text_keyed(&paths.recipe_dirs, key)
                .map_err(|e| ApiError::new("bad-recipe", &e))?
                .ok_or_else(|| ApiError::new("not-found", &format!("no recipe for {key}")))?;
            let recipe = crate::recipes::parse_recipe_keyed(key.clone(), &text)
                .map_err(|e| ApiError::new("bad-recipe", &e.to_string()))?;
            let settings = crate::recipes::route_settings(&text);
            let game = crate::steam::Game {
                appid: 0,
                name: key.to_string(),
                install_dir: PathBuf::new(),
            };
            crate::injectors::ensure_applied(&game, &recipe, &settings, paths, runner)
                .map_err(|e| crate::injectors::string_api_error(&e))?;
            Ok(key.to_string())
        }
        StoreKey::Steam(_) => Ok(crate::launch_options::add_wrapper_at(
            options,
            &paths.wrapper_bin().to_string_lossy(),
        )),
    }
}

/// Undo an Epic game's change from its undo record.
pub fn undo_epic(
    paths: &crate::paths::EnginePaths,
    app_name: &str,
) -> Result<crate::api::UndoResult, ApiError> {
    crate::epic_user_settings::undo(paths, app_name)
}
