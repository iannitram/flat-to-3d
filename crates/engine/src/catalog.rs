//! Installed Steam games joined to their configured recipes.

use crate::{api, heroic, paths::EnginePaths, recipes, steam};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The games installed in Steam and their recipe records.
pub struct Catalog {
    pub installed: Vec<steam::Game>,
    pub recipes: BTreeMap<recipes::StoreKey, recipes::Recipe>,
    pub games: Vec<api::Game>,
}

/// Read installed games and the matching recipes without requiring every recipe directory to exist.
pub fn catalog(paths: &EnginePaths) -> Result<Catalog, api::ApiError> {
    let steam_root = paths.home.join(steam::DEFAULT_STEAM_ROOT);
    let installed = steam::installed_games(&steam_root).map_err(|error| {
        let vdf = steam_root.join("steamapps").join("libraryfolders.vdf");
        api::ApiError::new("io", &format!("{}: {error}", vdf.display()))
    })?;

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
        found.insert(recipes::StoreKey::Steam(game.appid), recipe);
    }
    let games = join(&installed, &found);
    Ok(Catalog {
        installed,
        recipes: found,
        games,
    })
}

/// Join installed games to recipe status while preserving Steam's sorted order.
pub fn join(
    games: &[steam::Game],
    recipes: &BTreeMap<recipes::StoreKey, recipes::Recipe>,
) -> Vec<api::Game> {
    games
        .iter()
        .map(|game| {
            let recipe = recipes.get(&recipes::StoreKey::Steam(game.appid));
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

/// The games installed across known game libraries (Steam and Heroic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub steam: Vec<steam::Game>,
    pub epic: Vec<heroic::Game>,
}

/// Scan for installed games across Steam and Heroic library roots.
pub fn scan_libraries(home: &Path) -> io::Result<Library> {
    let steam = steam::installed_games(&home.join(steam::DEFAULT_STEAM_ROOT))?;
    let epic = heroic::installed_games(home)?;
    Ok(Library { steam, epic })
}

/// Load Epic recipes for installed games from the given recipe directories.
///
/// For each existing directory, recipes are loaded; missing directories are skipped.
/// An earlier directory wins when both define a recipe for the same store key.
pub fn epic_recipes(
    games: &[heroic::Game],
    recipe_dirs: &[PathBuf],
) -> Result<BTreeMap<String, recipes::Recipe>, recipes::RecipeError> {
    let installed: HashSet<&str> = games.iter().map(|g| g.app_name.as_str()).collect();
    let mut found = BTreeMap::new();
    for dir in recipe_dirs {
        let keyed = match recipes::load_recipes_keyed(dir) {
            Ok(keyed) => keyed,
            Err(recipes::RecipeError::Io { .. }) if !dir.exists() => continue,
            Err(err) => return Err(err),
        };
        for (key, recipe) in keyed {
            if let recipes::StoreKey::Epic(app_name) = key {
                if installed.contains(app_name.as_str()) {
                    found.entry(app_name).or_insert(recipe);
                }
            }
        }
    }
    Ok(found)
}
