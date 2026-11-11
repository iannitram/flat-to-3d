//! Engine filesystem locations, derived from an explicit environment snapshot.

use crate::api::ApiError;
use crate::wrapper::RunPaths;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnginePaths {
    pub home: PathBuf,
    pub uid: u32,
    pub state_dir: PathBuf,
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub bin: PathBuf,
    pub recipe_dirs: Vec<PathBuf>,
}

impl EnginePaths {
    pub fn from_env(env: &BTreeMap<String, String>, uid: u32) -> Result<EnginePaths, ApiError> {
        let home = env
            .get("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| ApiError::new("not-found", "HOME is not set"))?;
        let config = home.join(".config/flat3d");
        let data = env
            .get("XDG_DATA_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("flat3d");
        let cache_dir = env
            .get("XDG_CACHE_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".cache"))
            .join("flat3d");
        let state = env.get("XDG_RUNTIME_DIR").map(String::as_str);
        let state_dir = PathBuf::from(crate::state::state_dir(state, uid));
        let log_dir = home.join(".local/state/flat3d");
        let bin = home.join(".local/bin");
        let mut recipe_dirs = Vec::new();
        if let Some(src) = env.get("FLAT3D_SRC_DIR").filter(|value| !value.is_empty()) {
            recipe_dirs.push(PathBuf::from(src).join("recipes"));
        }
        recipe_dirs.push(data.join("recipes"));
        Ok(Self {
            home,
            uid,
            state_dir,
            config,
            data,
            cache_dir,
            log_dir,
            bin,
            recipe_dirs,
        })
    }

    pub fn settings(&self) -> PathBuf {
        self.config.join("settings")
    }
    pub fn link_log(&self) -> PathBuf {
        self.log_dir.join("link.log")
    }
    pub fn run_log(&self) -> PathBuf {
        self.log_dir.join("run.log")
    }
    pub fn cache(&self) -> PathBuf {
        self.cache_dir.clone()
    }
    pub fn uevr_cache(&self) -> PathBuf {
        self.cache().join("uevr")
    }
    pub fn env_file(&self, appid: u32) -> PathBuf {
        self.data.join("env").join(format!("{appid}.env"))
    }
    pub fn wrapper_bin(&self) -> PathBuf {
        self.bin.join("flat3d-run")
    }
    pub fn viewer_bin(&self) -> PathBuf {
        self.bin.join("flat3d-viewer")
    }
    pub fn pins(&self) -> PathBuf {
        self.data.join("pins")
    }
    pub fn run_paths(&self) -> RunPaths {
        RunPaths {
            state: self.state_dir.to_string_lossy().into_owned(),
            settings: self.settings(),
            recipes: self.data.join("recipes"),
            log: self.run_log(),
        }
    }
    pub fn watch_paths(&self) -> crate::watch::Paths {
        crate::watch::Paths {
            home: self.home.to_string_lossy().into_owned(),
            state_dir: self.state_dir.to_string_lossy().into_owned(),
        }
    }
}
