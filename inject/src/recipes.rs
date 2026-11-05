//! Recipes: each game's machine-readable stereo keys (docs/notes/DECISIONS.md, "Recipe keys").
//! They live as a data file in this repo under `recipes/` (the owner's 2026-10-26 ruling, #15);
//! for 1.0 the engine needs Spider-Man 2's (SPEC M12).

use std::collections::BTreeMap;
use std::path::Path;

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
}
