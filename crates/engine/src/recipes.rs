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

/// One game's stereo keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
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
            RecipeError::Io { path, message } => write!(f, "{path}: {message}"),
            RecipeError::InFile { file, error } => write!(f, "{file}: {error}"),
        }
    }
}

impl std::error::Error for RecipeError {}

/// Parse one recipe file's `key: value` text into its [`Recipe`]. Values are not checked here.
pub fn parse_recipe(appid: u32, text: &str) -> Result<Recipe, RecipeError> {
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
        let key = raw_key.trim();
        if !KEYS.contains(&key) {
            continue;
        }
        if values.contains_key(key) {
            return Err(RecipeError::DuplicateKey {
                key: key.to_string(),
                line,
            });
        }
        let mut value = raw_value.trim();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = &value[1..value.len() - 1];
        }
        values.insert(key, value.to_string());
    }
    let mut take = |key: &'static str| {
        values
            .remove(key)
            .ok_or_else(|| RecipeError::MissingKey(key.to_string()))
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

/// Every `<appid>-<slug>.recipe` file under `folder`, parsed and checked, keyed by Steam appid.
/// Other files are ignored. A file's own error comes back as [`RecipeError::InFile`] naming it.
pub fn load_recipes(folder: &Path) -> Result<BTreeMap<u32, Recipe>, RecipeError> {
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
    let mut recipes: BTreeMap<u32, Recipe> = BTreeMap::new();
    let mut files: BTreeMap<u32, String> = BTreeMap::new();
    for name in names {
        let appid = name
            .split_once('-')
            .and_then(|(id, _)| id.parse::<u32>().ok())
            .ok_or_else(|| RecipeError::BadFileName(name.clone()))?;
        let path = folder.join(&name);
        let text = std::fs::read_to_string(&path).map_err(|e| RecipeError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let in_file = |error: RecipeError| RecipeError::InFile {
            file: name.clone(),
            error: Box::new(error),
        };
        let recipe = parse_recipe(appid, &text).map_err(in_file)?;
        check_recipe(&recipe).map_err(in_file)?;
        if let Some(first) = files.get(&appid) {
            return Err(RecipeError::DuplicateAppid {
                appid,
                first: first.clone(),
                second: name,
            });
        }
        files.insert(appid, name.clone());
        recipes.insert(appid, recipe);
    }
    Ok(recipes)
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
