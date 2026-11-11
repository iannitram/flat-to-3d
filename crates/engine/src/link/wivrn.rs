use std::path::{Path, PathBuf};

use crate::link::{CommandRunner, WIVRN_APP};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatpakScope {
    User,
    System,
}

impl FlatpakScope {
    pub fn flag(self) -> &'static str {
        match self {
            Self::User => "--user",
            Self::System => "--system",
        }
    }
}

pub const SERVER_FLAGS: [&str; 2] = ["--early-active-runtime", "--no-publish-service"];
pub const QUEST_APPS: [&str; 5] = [
    "org.meumeu.wivrn.github",
    "org.meumeu.wivrn",
    "org.meumeu.wivrn.github.nighly",
    "org.meumeu.wivrn.github.testing",
    "org.meumeu.wivrn.local",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WivrnDirs {
    pub scope: FlatpakScope,
    pub app_dir: PathBuf,
    pub deploy: PathBuf,
    pub version: Option<String>,
}

impl WivrnDirs {
    pub fn user_default(home: &Path) -> WivrnDirs {
        let app_dir = home.join(".local/share/flatpak/app/io.github.wivrn.wivrn");
        WivrnDirs {
            scope: FlatpakScope::User,
            deploy: app_dir.join("current/active"),
            app_dir,
            version: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestApp {
    pub package: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WivrnStart {
    pub scope: Option<FlatpakScope>,
    pub app: String,
    pub flags: Vec<String>,
}

impl Default for WivrnStart {
    fn default() -> Self {
        Self {
            scope: None,
            app: WIVRN_APP.to_string(),
            flags: SERVER_FLAGS
                .iter()
                .map(|flag| (*flag).to_string())
                .collect(),
        }
    }
}

pub fn app_dir_from_location(_out: &str) -> Option<PathBuf> {
    None
}
pub fn version_from_info(_out: &str) -> Option<String> {
    None
}
pub fn version_name(_dumpsys: &str) -> Option<String> {
    None
}
pub fn versions_match(_server: &str, _app: &str) -> bool {
    false
}
pub fn detect_server(_r: &mut dyn CommandRunner) -> Result<WivrnDirs, String> {
    Err("not-built: link::wivrn::detect_server (WP-12)".into())
}
pub fn detect_quest_app(_r: &mut dyn CommandRunner, _serial: &str) -> Option<QuestApp> {
    None
}
