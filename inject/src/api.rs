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

pub const ERROR_CODES: [&str; 8] = [
    "usage",
    "not-built",
    "not-found",
    "bad-recipe",
    "refused",
    "link-not-ready",
    "command-failed",
    "io",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        ApiError {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

impl From<crate::NotBuilt> for ApiError {
    fn from(e: crate::NotBuilt) -> Self {
        ApiError::new("not-built", &e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response<T> {
    pub ok: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

impl<T> Response<T> {
    pub fn success(command: &str, data: T) -> Self {
        Response {
            ok: true,
            command: command.to_string(),
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(command: &str, error: ApiError) -> Self {
        Response {
            ok: false,
            command: command.to_string(),
            data: None,
            error: Some(error),
        }
    }
}
