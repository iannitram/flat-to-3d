//! Installed Heroic (Epic) games: read Heroic's legendary `installed.json`.

use crate::recipes::StoreKey;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const NATIVE_CONFIG: &str = ".config/heroic";
pub const FLATPAK_ID: &str = "com.heroicgameslauncher.hgl";
pub const FLATPAK_CONFIG: &str = ".var/app/com.heroicgameslauncher.hgl/config/heroic";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub app_name: String,
    pub title: String,
    pub install_dir: PathBuf,
    pub executable: PathBuf,
    pub config_root: PathBuf,
}

impl Game {
    pub fn key(&self) -> StoreKey {
        StoreKey::Epic(self.app_name.clone())
    }
}

/// Rewrite a path written inside a Flatpak sandbox to its host counterpart if the host copy exists.
pub fn host_path(home: &Path, written: &Path) -> PathBuf {
    if written.exists() {
        return written.to_path_buf();
    }
    if let Ok(rest) = written.strip_prefix(home) {
        let candidate = home.join(".var/app").join(FLATPAK_ID).join(rest);
        if candidate.exists() {
            return candidate;
        }
    }
    written.to_path_buf()
}

/// Read installed games under the native Heroic config root then the Flatpak config root.
///
/// Missing roots or missing `installed.json` files contribute nothing (not an error).
/// An existing `installed.json` that is not a JSON object is an error.
pub fn installed_games(home: &Path) -> io::Result<Vec<Game>> {
    let native_root = home.join(NATIVE_CONFIG);
    let flatpak_root = home.join(FLATPAK_CONFIG);

    let mut games = Vec::new();
    read_root(home, &native_root, false, &mut games)?;
    read_root(home, &flatpak_root, true, &mut games)?;

    let mut seen = HashSet::new();
    games.retain(|game| seen.insert(game.app_name.clone()));
    games.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.app_name.cmp(&b.app_name))
    });
    Ok(games)
}

fn read_root(home: &Path, root: &Path, is_flatpak: bool, games: &mut Vec<Game>) -> io::Result<()> {
    let installed_path = root
        .join("legendaryConfig")
        .join("legendary")
        .join("installed.json");
    let text = match fs::read_to_string(&installed_path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {err}", installed_path.display()),
        )
    })?;
    let Some(obj) = value.as_object() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: not a JSON object", installed_path.display()),
        ));
    };

    for (key, entry) in obj {
        let Some(entry_obj) = entry.as_object() else {
            continue;
        };
        if entry_obj
            .get("is_dlc")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            continue;
        }
        let Some(install_path_str) = entry_obj.get("install_path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(executable_str) = entry_obj.get("executable").and_then(|v| v.as_str()) else {
            continue;
        };
        if install_path_str.is_empty() || executable_str.is_empty() {
            continue;
        }
        let app_name = entry_obj
            .get("app_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(key)
            .to_string();
        if app_name.is_empty() {
            continue;
        }
        let title = entry_obj
            .get("title")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&app_name)
            .to_string();

        let raw_install_dir = Path::new(install_path_str);
        let install_dir = if is_flatpak {
            host_path(home, raw_install_dir)
        } else {
            raw_install_dir.to_path_buf()
        };
        let executable = install_dir.join(executable_str);

        games.push(Game {
            app_name,
            title,
            install_dir,
            executable,
            config_root: root.to_path_buf(),
        });
    }

    Ok(())
}
