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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TetherMode {
    Off,
    Own,
    QuestTether,
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
    TetherError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub battery: Option<u8>,
    pub status: String,
    pub temp_c: Option<f64>,
    pub zone: Zone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub rx_bytes_s: u64,
    pub tx_bytes_s: u64,
    pub uptime_s: u64,
    pub reconnects: u32,
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
    pub tether: TetherMode,
    pub tunnel: bool,
    pub auto_3d: bool,
    pub adb: String,
    pub adb_source: AdbSource,
    pub health: Option<Health>,
    pub stats: Option<Stats>,
    pub events: Vec<LinkEvent>,
    pub updated: u64,
}

impl LinkSnapshot {
    /// The snapshot for a link service that is not running.
    pub fn stopped(now: u64) -> LinkSnapshot {
        LinkSnapshot {
            state: LinkState::Stopped,
            message: "Link service is not running.".into(),
            tether: TetherMode::Off,
            tunnel: false,
            auto_3d: false,
            adb: String::new(),
            adb_source: AdbSource::Missing,
            health: None,
            stats: None,
            events: Vec::new(),
            updated: now,
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
