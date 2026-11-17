//! Recipes: each game's machine-readable stereo keys (docs/notes/DECISIONS.md, "Recipe keys").
//! They live as a data file in this repo under `recipes/` (the owner's 2026-10-26 ruling, #15);
//! for 1.0 the engine needs Spider-Man 2's (SPEC M12).

use std::collections::BTreeMap;
use std::path::Path;

pub type RouteSettings = BTreeMap<String, String>;
pub const UEVR_KEYS: [&str; 6] = [
    "uevr-exe",
    "uevr-rendering",
    "uevr-2d-screen",
    "uevr-native-stereo-fix",
    "uevr-delay",
    "uevr-profile",
];
pub const GEO11_KEYS: [&str; 7] = [
    "geo11-fix",
    "geo11-exe",
    "geo11-layout",
    "geo11-separation",
    "geo11-convergence",
    "geo11-d3dx",
    "geo11-remove-first",
];
pub const PORT_KEYS: [&str; 0] = [];

/// Read only route-specific settings from a recipe text.
pub fn route_settings(text: &str) -> RouteSettings {
    let mut result = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, raw_value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if !key.starts_with("uevr-") && !key.starts_with("geo11-") && !key.starts_with("port-") {
            continue;
        }
        let value = raw_value.trim();
        let value = if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            &value[1..value.len() - 1]
        } else {
            value
        };
        result.insert(key.to_string(), value.to_string());
    }
    result
}

/// Validate route settings against the selected recipe route and its value rules.
pub fn check_route_settings(route: &str, settings: &RouteSettings) -> Result<(), String> {
    use crate::injectors::{route_of, Route};
    let allowed: &[&str] = match route_of(route) {
        Route::Uevr => &UEVR_KEYS,
        Route::Geo11 => &GEO11_KEYS,
        Route::Port | Route::NoRoute => PORT_KEYS.as_slice(),
    };
    if settings.is_empty() {
        if matches!(route_of(route), Route::Uevr | Route::Geo11) {
            return Err("route settings are missing required keys".into());
        }
        return Ok(());
    }
    if matches!(route_of(route), Route::Port | Route::NoRoute) {
        return Err("route does not accept route settings".into());
    }
    for (key, value) in settings {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("key `{key}` is not allowed for this route"));
        }
        let valid = match key.as_str() {
            "uevr-exe" | "geo11-exe" => !value.trim().is_empty(),
            "uevr-rendering" => matches!(value.as_str(), "native" | "synced"),
            "uevr-2d-screen" | "uevr-native-stereo-fix" => matches!(value.as_str(), "yes" | "no"),
            "uevr-delay" => value.parse::<u32>().is_ok_and(|n| n <= 120),
            "uevr-profile" | "geo11-fix" => is_pin_key(value),
            "geo11-layout" => matches!(
                value.as_str(),
                "sbs" | "tab" | "sbs_reversed" | "tab_reversed"
            ),
            "geo11-separation" => value.parse::<u32>().is_ok_and(|n| n <= 100),
            "geo11-convergence" => value.parse::<f32>().is_ok_and(|n| n > 0.0),
            "geo11-d3dx" => valid_pairs(value),
            "geo11-remove-first" => valid_relative_paths(value),
            _ => false,
        };
        if !valid {
            return Err(format!("bad value for `{key}`: `{value}`"));
        }
    }
    for required in match route_of(route) {
        Route::Uevr => &UEVR_KEYS[0..1],
        Route::Geo11 => &GEO11_KEYS[0..2],
        _ => &[],
    } {
        if !settings.contains_key(*required) {
            return Err(format!("missing required key `{required}`"));
        }
    }
    Ok(())
}

fn is_pin_key(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn valid_pairs(value: &str) -> bool {
    !value.is_empty()
        && value.split(';').all(|pair| {
            pair.split_once('=')
                .is_some_and(|(key, val)| !key.is_empty() && !val.is_empty() && !val.contains('='))
        })
}

fn valid_relative_paths(value: &str) -> bool {
    !value.is_empty()
        && value.split(';').all(|entry| {
            let path = Path::new(entry);
            !entry.is_empty()
                && !path.is_absolute()
                && path
                    .components()
                    .all(|component| matches!(component, std::path::Component::Normal(_)))
        })
}

/// Allowed `stereo-status` values.
pub const STATUSES: [&str; 6] = [
    "works-now",
    "likely",
    "build",
    "not-safe",
    "wont-work",
    "untested",
];
/// Allowed `stereo-api` values.
pub const APIS: [&str; 6] = ["dx9", "dx11", "dx12", "vulkan", "opengl", "unknown"];
/// Allowed `stereo-free` values.
pub const FREES: [&str; 3] = ["yes", "no", "unclear"];
/// Allowed `stereo-safe` values.
pub const SAFES: [&str; 2] = ["yes", "no"];
/// Known stores supported by the recipe loader.
pub const STORES: [&str; 2] = ["steam", "epic"];
/// The keys one recipe file must give, in the order a missing one is reported.
pub const KEYS: [&str; 9] = [
    "name",
    "stereo-status",
    "stereo-route",
    "stereo-engine",
    "stereo-api",
    "stereo-mod",
    "stereo-free",
    "stereo-safe",
    "stereo-checked",
];

/// A recipe's store-scoped identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StoreKey {
    Steam(u32),
    Epic(String),
}

impl std::fmt::Display for StoreKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreKey::Steam(appid) => write!(f, "steam:{appid}"),
            StoreKey::Epic(name) => write!(f, "epic:{name}"),
        }
    }
}

impl std::str::FromStr for StoreKey {
    type Err = RecipeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((prefix, id)) = s.split_once(':') else {
            return Err(RecipeError::BadStoreKey(s.to_string()));
        };
        if prefix.is_empty() {
            return Err(RecipeError::BadStoreKey(s.to_string()));
        }
        match prefix {
            "steam" => {
                let appid = id
                    .parse::<u32>()
                    .map_err(|_| RecipeError::BadStoreKey(s.to_string()))?;
                Ok(StoreKey::Steam(appid))
            }
            "epic" => {
                let id = id.trim();
                if id.is_empty() || id.contains('\n') || id.contains('\r') {
                    return Err(RecipeError::BadStoreKey(s.to_string()));
                }
                Ok(StoreKey::Epic(id.to_string()))
            }
            _ => Err(RecipeError::UnknownStore(s.to_string())),
        }
    }
}

/// One game's stereo keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    /// The Steam appid; 0 for a recipe from another store, whose [`StoreKey`] is its key in
    /// [`load_recipes_keyed`]. No `StoreKey` field here: the locked tests build `Recipe` literals.
    pub appid: u32,
    pub name: String,
    /// `stereo-status`, one of [`STATUSES`].
    pub status: String,
    /// `stereo-route`, e.g. "build: port spidy stereo screen".
    pub route: String,
    /// `stereo-engine`.
    pub engine: String,
    /// `stereo-api`, one of [`APIS`].
    pub api: String,
    /// `stereo-mod`: a URL or "none".
    pub mod_url: String,
    /// `stereo-free`: "yes" | "no" | "unclear".
    pub free: String,
    /// `stereo-safe`: "yes" | "no".
    pub safe: String,
    /// `stereo-checked`: YYYY-MM-DD.
    pub checked: String,
}

/// Why a recipe file did not parse or load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeError {
    /// A line with no `:`.
    BadLine { line: usize, text: String },
    /// A key given twice; `line` is the second one.
    DuplicateKey { key: String, line: usize },
    /// A key in [`KEYS`] that never appeared.
    MissingKey(String),
    /// A value the key does not allow; `key` is the file's key, e.g. "stereo-status".
    BadValue { key: String, value: String },
    /// A file whose name is not a recipe file name.
    BadFileName(String),
    /// Two files for one appid; file names, sorted.
    DuplicateAppid {
        appid: u32,
        first: String,
        second: String,
    },
    /// Two files for one store key; file names, sorted.
    DuplicateStoreKey {
        key: String,
        first: String,
        second: String,
    },
    /// An unknown store prefix in a store key or file name.
    UnknownStore(String),
    /// A key with no `:` or an empty/invalid id.
    BadStoreKey(String),
    /// A file could not be read.
    Io { path: String, message: String },
    /// A file's own error; `file` is the file name only.
    InFile {
        file: String,
        error: Box<RecipeError>,
    },
}

impl std::fmt::Display for RecipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecipeError::BadLine { line, text } => {
                write!(f, "line {line}: not `key: value`: {text}")
            }
            RecipeError::DuplicateKey { key, line } => {
                write!(f, "line {line}: duplicate key `{key}`")
            }
            RecipeError::MissingKey(key) => write!(f, "missing key `{key}`"),
            RecipeError::BadValue { key, value } => write!(f, "bad value for `{key}`: `{value}`"),
            RecipeError::BadFileName(name) => write!(f, "bad file name: {name}"),
            RecipeError::DuplicateAppid {
                appid,
                first,
                second,
            } => write!(f, "appid {appid} in both {first} and {second}"),
            RecipeError::DuplicateStoreKey { key, first, second } => {
                write!(f, "store key {key} in both {first} and {second}")
            }
            RecipeError::UnknownStore(key) => {
                let prefix = key.split_once(':').map_or(key.as_str(), |(p, _)| p);
                write!(
                    f,
                    "unknown store \"{prefix}\" in store key \"{key}\"; known stores: {}",
                    STORES.join(", ")
                )
            }
            RecipeError::BadStoreKey(key) => write!(f, "bad store key: {key}"),
            RecipeError::Io { path, message } => write!(f, "{path}: {message}"),
            RecipeError::InFile { file, error } => write!(f, "{file}: {error}"),
        }
    }
}

impl std::error::Error for RecipeError {}

/// Parse one recipe file's `key: value` text into its [`Recipe`]. Values are not checked here.
pub fn parse_recipe(appid: u32, text: &str) -> Result<Recipe, RecipeError> {
    parse_recipe_keyed(StoreKey::Steam(appid), text)
}

/// Parse one recipe file's `key: value` text into its [`Recipe`] for the given [`StoreKey`].
/// Values are not checked here.
pub fn parse_recipe_keyed(key: StoreKey, text: &str) -> Result<Recipe, RecipeError> {
    let mut values: BTreeMap<&str, String> = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            return Err(RecipeError::BadLine {
                line,
                text: trimmed.to_string(),
            });
        };
        let key_name = raw_key.trim();
        if !KEYS.contains(&key_name) {
            continue;
        }
        if values.contains_key(key_name) {
            return Err(RecipeError::DuplicateKey {
                key: key_name.to_string(),
                line,
            });
        }
        let mut value = raw_value.trim();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = &value[1..value.len() - 1];
        }
        values.insert(key_name, value.to_string());
    }
    let mut take = |k: &'static str| {
        values
            .remove(k)
            .ok_or_else(|| RecipeError::MissingKey(k.to_string()))
    };
    let appid = match &key {
        StoreKey::Steam(id) => *id,
        _ => 0,
    };
    Ok(Recipe {
        appid,
        name: take("name")?,
        status: take("stereo-status")?,
        route: take("stereo-route")?,
        engine: take("stereo-engine")?,
        api: take("stereo-api")?,
        mod_url: take("stereo-mod")?,
        free: take("stereo-free")?,
        safe: take("stereo-safe")?,
        checked: take("stereo-checked")?,
    })
}

/// Check a recipe's values against the allowed lists; reports the first bad key.
/// Only the five checked keys are looked at: free text is not checked.
pub fn check_recipe(recipe: &Recipe) -> Result<(), RecipeError> {
    let bad = |key: &str, value: &str| RecipeError::BadValue {
        key: key.to_string(),
        value: value.to_string(),
    };
    if !STATUSES.contains(&recipe.status.as_str()) {
        return Err(bad("stereo-status", &recipe.status));
    }
    if !APIS.contains(&recipe.api.as_str()) {
        return Err(bad("stereo-api", &recipe.api));
    }
    if !FREES.contains(&recipe.free.as_str()) {
        return Err(bad("stereo-free", &recipe.free));
    }
    if !SAFES.contains(&recipe.safe.as_str()) {
        return Err(bad("stereo-safe", &recipe.safe));
    }
    if !is_date_shape(&recipe.checked) {
        return Err(bad("stereo-checked", &recipe.checked));
    }
    Ok(())
}

/// `YYYY-MM-DD` by shape only: 10 bytes, `-` at 4 and 7, ASCII digits elsewhere.
fn is_date_shape(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
}

fn extract_store_key(text: &str) -> Result<Option<String>, RecipeError> {
    let mut found: Option<(String, usize)> = None;
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        if raw_key.trim() == "store-key" {
            if found.is_some() {
                return Err(RecipeError::DuplicateKey {
                    key: "store-key".to_string(),
                    line,
                });
            }
            let mut value = raw_value.trim();
            if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
                value = &value[1..value.len() - 1];
            }
            found = Some((value.to_string(), line));
        }
    }
    Ok(found.map(|(v, _)| v))
}

/// Every `.recipe` file under `folder`, parsed and checked, keyed by [`StoreKey`].
/// Other files are ignored. A file's own error comes back as [`RecipeError::InFile`] naming it.
pub fn load_recipes_keyed(folder: &Path) -> Result<BTreeMap<StoreKey, Recipe>, RecipeError> {
    let entries = std::fs::read_dir(folder).map_err(|e| RecipeError::Io {
        path: folder.display().to_string(),
        message: e.to_string(),
    })?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| RecipeError::Io {
            path: folder.display().to_string(),
            message: e.to_string(),
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".recipe") {
            names.push(name);
        }
    }
    names.sort();
    let mut recipes: BTreeMap<StoreKey, Recipe> = BTreeMap::new();
    let mut files: BTreeMap<StoreKey, String> = BTreeMap::new();
    for name in names {
        let Some(stem) = name.strip_suffix(".recipe") else {
            return Err(RecipeError::BadFileName(name));
        };
        let Some((prefix, slug)) = stem.split_once('-') else {
            return Err(RecipeError::BadFileName(name));
        };
        if prefix.is_empty() || slug.is_empty() {
            return Err(RecipeError::BadFileName(name));
        }
        let is_steam_file = prefix.bytes().all(|b| b.is_ascii_digit());
        let steam_appid = if is_steam_file {
            Some(
                prefix
                    .parse::<u32>()
                    .map_err(|_| RecipeError::BadFileName(name.clone()))?,
            )
        } else {
            if !STORES.contains(&prefix) {
                return Err(RecipeError::InFile {
                    file: name.clone(),
                    error: Box::new(RecipeError::UnknownStore(prefix.to_string())),
                });
            }
            None
        };
        let path = folder.join(&name);
        let text = std::fs::read_to_string(&path).map_err(|e| RecipeError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let in_file = |error: RecipeError| RecipeError::InFile {
            file: name.clone(),
            error: Box::new(error),
        };
        let raw_store_key = extract_store_key(&text).map_err(in_file)?;
        let store_key = if let Some(appid) = steam_appid {
            if let Some(raw) = raw_store_key {
                let parsed: Result<StoreKey, _> = raw.parse();
                if parsed != Ok(StoreKey::Steam(appid)) {
                    return Err(in_file(RecipeError::BadValue {
                        key: "store-key".to_string(),
                        value: raw,
                    }));
                }
                StoreKey::Steam(appid)
            } else {
                StoreKey::Steam(appid)
            }
        } else {
            let raw = raw_store_key
                .ok_or_else(|| in_file(RecipeError::MissingKey("store-key".to_string())))?;
            let parsed: StoreKey = raw.parse().map_err(in_file)?;
            let store_name = match &parsed {
                StoreKey::Steam(_) => "steam",
                StoreKey::Epic(_) => "epic",
            };
            if store_name != prefix {
                return Err(in_file(RecipeError::BadValue {
                    key: "store-key".to_string(),
                    value: raw,
                }));
            }
            parsed
        };
        let recipe = parse_recipe_keyed(store_key.clone(), &text).map_err(in_file)?;
        check_recipe(&recipe).map_err(in_file)?;
        if let Some(first) = files.get(&store_key) {
            if let StoreKey::Steam(appid) = store_key {
                return Err(RecipeError::DuplicateAppid {
                    appid,
                    first: first.clone(),
                    second: name,
                });
            } else {
                return Err(RecipeError::DuplicateStoreKey {
                    key: store_key.to_string(),
                    first: first.clone(),
                    second: name,
                });
            }
        }
        files.insert(store_key.clone(), name.clone());
        recipes.insert(store_key, recipe);
    }
    Ok(recipes)
}

/// Every `<appid>-<slug>.recipe` file under `folder`, parsed and checked, keyed by Steam appid.
/// Other files are ignored. A file's own error comes back as [`RecipeError::InFile`] naming it.
/// A `.recipe` name that starts with neither an appid nor a known store stays
/// [`RecipeError::BadFileName`] here, as before store keys; [`load_recipes_keyed`] calls it
/// [`RecipeError::UnknownStore`].
pub fn load_recipes(folder: &Path) -> Result<BTreeMap<u32, Recipe>, RecipeError> {
    let keyed = load_recipes_keyed(folder).map_err(|e| match e {
        RecipeError::InFile { file, error } if matches!(*error, RecipeError::UnknownStore(_)) => {
            RecipeError::BadFileName(file)
        }
        other => other,
    })?;
    let mut steam_recipes = BTreeMap::new();
    for (key, recipe) in keyed {
        if let StoreKey::Steam(appid) = key {
            steam_recipes.insert(appid, recipe);
        }
    }
    Ok(steam_recipes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_include_works_now() {
        assert!(STATUSES.contains(&"works-now"));
    }

    #[test]
    fn an_unreadable_recipe_file_is_an_io_error_naming_the_path() {
        let dir =
            std::env::temp_dir().join(format!("flat3d-recipes-unreadable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("100-unreadable.recipe")).unwrap();
        match load_recipes(&dir) {
            Err(RecipeError::Io { path, .. }) => {
                assert!(path.contains("100-unreadable.recipe"), "{path}");
            }
            other => panic!("expected Io, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_name_without_a_dash_is_a_bad_file_name() {
        let dir =
            std::env::temp_dir().join(format!("flat3d-recipes-nodash-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("12345.recipe"), "name: Test\n").unwrap();
        assert_eq!(
            load_recipes(&dir),
            Err(RecipeError::BadFileName("12345.recipe".into()))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
