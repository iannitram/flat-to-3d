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

/// What `undo <appid>` put back: one line per change, from the undo record (#81).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoResult {
    pub appid: u32,
    pub restored: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinkView {
    pub state: LinkState,
    pub label: String,
    pub hint: String,
    pub start: bool,
    pub stop: bool,
    pub health: crate::face::view::HealthView,
    pub auto_3d: bool,
    pub adb_source: AdbSource,
    /// The one Quest Tether line, read-only: "Quest Tether: connected", "... not connected" or
    /// "... not installed".
    pub quest_tether: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaunchText {
    pub options: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionPlan {
    pub name: String,
    pub wanted: Vec<u32>,
    pub add: Vec<u32>,
    pub remove: Vec<u32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogLines {
    pub lines: Vec<String>,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub fix: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupReport {
    pub steps: Vec<String>,
    pub summary: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Toast {
    pub title: String,
    pub body: String,
}

pub const TOAST_TITLE: &str = "Flat to 3D";

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

pub fn error_help(code: &str) -> (bool, &'static str) {
    match code {
        "usage" => (
            false,
            "check the command against the usage line in the message",
        ),
        "not-built" => (
            false,
            "this part is not built yet; the message names the work package that builds it",
        ),
        "not-found" => (
            false,
            "check the name or path in the message; flat3d-inject list shows the installed games",
        ),
        "bad-recipe" => (false, "fix the recipe file named in the message"),
        "refused" => (
            false,
            "do what the message names first, then run the command again",
        ),
        "link-not-ready" => (true, "systemctl --user restart flat3d-link.service"),
        "command-failed" => (
            true,
            "check the command named in the message, then run it again",
        ),
        "io" => (
            true,
            "check the path in the message, then run the command again",
        ),
        _ => (false, ""),
    }
}

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

#[cfg(test)]
mod error_help_tests {
    use super::{error_help, ERROR_CODES};

    #[test]
    fn every_error_code_has_help() {
        for code in ERROR_CODES {
            assert!(!error_help(code).1.is_empty(), "{code}");
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdbSource {
    Running,
    Bundled,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Zone {
    Green,
    Yellow,
    Red,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    Ready,
    LatePlug,
    GraceEnded,
    AllowDebugging,
    Warm,
    Hot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub battery: Option<u8>,
    pub status: String,
    pub temp_c: Option<f64>,
    pub zone: Zone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkEvent {
    pub seq: u64,
    pub kind: EventKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkSnapshot {
    pub state: LinkState,
    pub message: String,
    pub auto_3d: bool,
    pub adb: String,
    pub adb_source: AdbSource,
    pub health: Option<Health>,
    pub events: Vec<LinkEvent>,
    pub updated: u64,
    /// Quest Tether's `state` text as last read, read-only (`None` = not installed). Nothing in
    /// the engine acts on it. Left out of the file when `None`, so old files and readers agree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quest_tether: Option<String>,
}

impl LinkSnapshot {
    /// The snapshot for a link service that is not running.
    pub fn stopped(now: u64) -> LinkSnapshot {
        LinkSnapshot {
            state: LinkState::Stopped,
            message: "Link service is not running.".into(),
            auto_3d: false,
            adb: String::new(),
            adb_source: AdbSource::Missing,
            health: None,
            events: Vec::new(),
            updated: now,
            quest_tether: None,
        }
    }

    /// A snapshot older than the stale limit counts as stopped; a fresh one is kept as it is.
    pub fn effective(self, now: u64) -> LinkSnapshot {
        if crate::state::is_stale(self.updated, now) {
            LinkSnapshot::stopped(now)
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchState {
    pub appid: u32,
    pub on: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingState {
    pub key: String,
    pub on: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReport {
    pub written: Vec<String>,
    pub ran: Vec<String>,
    pub summary: String,
}
pub use crate::face::rows::GameRow;
