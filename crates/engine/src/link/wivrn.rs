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

pub fn app_dir_from_location(out: &str) -> Option<PathBuf> {
    let deploy = Path::new(out.trim());
    let mut app_dir = deploy.to_path_buf();
    for _ in 0..3 {
        app_dir.pop();
    }
    (app_dir.ends_with("app/io.github.wivrn.wivrn") && app_dir != deploy).then_some(app_dir)
}
pub fn version_from_info(out: &str) -> Option<String> {
    out.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Version:")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    })
}
pub fn version_name(dumpsys: &str) -> Option<String> {
    dumpsys.lines().find_map(|line| {
        line.trim()
            .strip_prefix("versionName=")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    })
}
pub fn versions_match(server: &str, app: &str) -> bool {
    server.split('-').next() == app.split('-').next()
}
pub fn detect_server(r: &mut dyn CommandRunner) -> Result<WivrnDirs, String> {
    for scope in [FlatpakScope::User, FlatpakScope::System] {
        let flag = scope.flag();
        let location = r.run(&[
            "flatpak".into(),
            "info".into(),
            "--show-location".into(),
            flag.into(),
            "io.github.wivrn.wivrn".into(),
        ]);
        if !location.success {
            continue;
        }
        let deploy = PathBuf::from(location.stdout.trim());
        let Some(app_dir) = app_dir_from_location(&location.stdout) else {
            continue;
        };
        let info = r.run(&[
            "flatpak".into(),
            "info".into(),
            flag.into(),
            "io.github.wivrn.wivrn".into(),
        ]);
        let version = info
            .success
            .then(|| version_from_info(&info.stdout))
            .flatten();
        return Ok(WivrnDirs {
            scope,
            app_dir,
            deploy,
            version,
        });
    }
    Err("WiVRn server is not installed as a Flatpak (tried --user and --system); fix: flatpak install --user flathub io.github.wivrn.wivrn".into())
}
pub fn detect_quest_app(r: &mut dyn CommandRunner, serial: &str) -> Option<QuestApp> {
    for package in QUEST_APPS {
        let output = r.run(&[
            "adb".into(),
            "-s".into(),
            serial.into(),
            "shell".into(),
            "dumpsys".into(),
            "package".into(),
            package.into(),
        ]);
        if !output.success {
            return None;
        }
        if let Some(version) = version_name(&output.stdout) {
            return Some(QuestApp {
                package: package.into(),
                version: Some(version),
            });
        }
    }
    None
}
