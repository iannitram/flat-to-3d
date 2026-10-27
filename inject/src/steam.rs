//! Installed Steam games: parse `libraryfolders.vdf` and each library's `appmanifest_*.acf`.

use crate::NotBuilt;
use std::path::{Path, PathBuf};

/// Steam's root on SteamOS, relative to the home folder.
pub const DEFAULT_STEAM_ROOT: &str = ".local/share/Steam";

/// One fully installed game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub appid: u32,
    pub name: String,
    /// The game's folder under `steamapps/common/`.
    pub install_dir: PathBuf,
}

/// Every fully installed game in every Steam library under `steam_root`, sorted by name.
pub fn installed_games(steam_root: &Path) -> Result<Vec<Game>, NotBuilt> {
    let _ = steam_root;
    Err(NotBuilt("steam::installed_games: see the engine issue"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_exists() {
        assert!(installed_games(Path::new("/nonexistent")).is_err());
    }
}
