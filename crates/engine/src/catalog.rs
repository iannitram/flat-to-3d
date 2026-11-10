//! Installed Steam games joined to their configured recipes.

use crate::{api, paths::EnginePaths, recipes, steam};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// The games installed in Steam and their recipe records.
pub struct Catalog {
    pub installed: Vec<steam::Game>,
    pub recipes: BTreeMap<u32, recipes::Recipe>,
    pub games: Vec<api::Game>,
}

/// Read installed games and the matching recipes without requiring every recipe directory to exist.
pub fn catalog(paths: &EnginePaths) -> Result<Catalog, api::ApiError> {
    let steam_root = paths.home.join(steam::DEFAULT_STEAM_ROOT);
    let installed = steam::installed_games(&steam_root)
        .map_err(|error| api::ApiError::new("io", &format!("{}: {error}", steam_root.display())))?;

    let mut found = BTreeMap::new();
    for game in &installed {
        let Some((path, text)) = find_recipe(&paths.recipe_dirs, game.appid)
            .map_err(|error| api::ApiError::new("io", &error))?
        else {
            continue;
        };
        let recipe = recipes::parse_recipe(game.appid, &text)
            .and_then(|recipe| {
                recipes::check_recipe(&recipe)?;
                Ok(recipe)
            })
            .map_err(|error| {
                api::ApiError::new("bad-recipe", &format!("{}: {error}", path.display()))
            })?;
        found.insert(game.appid, recipe);
    }
    let games = join(&installed, &found);
    Ok(Catalog {
        installed,
        recipes: found,
        games,
    })
}

/// Join installed games to recipe status while preserving Steam's sorted order.
pub fn join(games: &[steam::Game], recipes: &BTreeMap<u32, recipes::Recipe>) -> Vec<api::Game> {
    games
        .iter()
        .map(|game| {
            let recipe = recipes.get(&game.appid);
            api::Game {
                appid: game.appid,
                name: game.name.clone(),
                install_dir: game.install_dir.display().to_string(),
                status: recipe.map_or_else(|| "untested".to_string(), |r| r.status.clone()),
                has_recipe: recipe.is_some(),
            }
        })
        .collect()
}

/// Find the exact appid recipe first, then the first sorted dashed recipe.
pub fn find_recipe(dirs: &[PathBuf], appid: u32) -> Result<Option<(PathBuf, String)>, String> {
    let exact_name = format!("{appid}.recipe");
    let prefix = format!("{appid}-");
    for dir in dirs {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("{}: {error}", dir.display())),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(&prefix) && name.ends_with(".recipe") && entry.path().is_file() {
                names.push(name);
            }
        }
        names.sort();
        let exact = dir.join(&exact_name);
        let found = if exact.is_file() {
            Some(exact)
        } else {
            names.first().map(|name| dir.join(name))
        };
        if let Some(path) = found {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            return Ok(Some((path, text)));
        }
    }
    Ok(None)
}
