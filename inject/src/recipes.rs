//! Recipes: each game's machine-readable stereo keys (docs/notes/DECISIONS.md, "Recipe keys").
//! They live as a data file in this repo under `recipes/` (the owner's 2026-10-26 ruling, #15);
//! for 1.0 the engine needs Spider-Man 2's (SPEC M12).

use crate::NotBuilt;
use std::collections::BTreeMap;
use std::path::Path;

/// Allowed `stereo-status` values.
pub const STATUSES: [&str; 6] = ["works-now", "likely", "build", "not-safe", "wont-work", "untested"];
/// Allowed `stereo-api` values.
pub const APIS: [&str; 6] = ["dx9", "dx11", "dx12", "vulkan", "opengl", "unknown"];

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

/// Every recipe under `folder`, keyed by Steam appid. Rejects unknown status or api values.
pub fn load_recipes(folder: &Path) -> Result<BTreeMap<u32, Recipe>, NotBuilt> {
    let _ = folder;
    Err(NotBuilt("recipes::load_recipes: see the recipes issue"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_exists() {
        assert!(load_recipes(Path::new("recipes")).is_err());
        assert!(STATUSES.contains(&"works-now"));
    }
}
