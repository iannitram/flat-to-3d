//! The one JSON shape both faces read: docs/code-map.md, API shape.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub appid: u32,
    pub name: String,
    pub install_dir: String,
    pub status: String,
    pub has_recipe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub appid: u32,
    pub name: String,
    pub status: String,
    pub route: String,
    pub engine: String,
    pub api: String,
    pub mod_url: String,
    pub free: String,
    pub safe: String,
    pub checked: String,
}

impl From<&crate::recipes::Recipe> for Recipe {
    fn from(r: &crate::recipes::Recipe) -> Self {
        Recipe {
            appid: r.appid,
            name: r.name.clone(),
            status: r.status.clone(),
            route: r.route.clone(),
            engine: r.engine.clone(),
            api: r.api.clone(),
            mod_url: r.mod_url.clone(),
            free: r.free.clone(),
            safe: r.safe.clone(),
            checked: r.checked.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkState {
    NoHeadset,
    Unauthorised,
    Starting,
    Ready,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkReport {
    pub state: LinkState,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectResult {
    pub appid: u32,
    pub route: String,
    pub launched: bool,
    pub summary: String,
}
