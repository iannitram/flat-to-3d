//! Settings: Auto 3D, the tunnel and the per-game 3D switches. docs/code-map.md, API shape, State, IPC and files.

use std::collections::BTreeSet;

const HEADER: &str = "flat3d-settings 1";

/// The switches. Everything is on by default; only the games switched off are stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub auto_3d: bool,
    pub tunnel: bool,
    pub off: BTreeSet<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            auto_3d: true,
            tunnel: true,
            off: BTreeSet::new(),
        }
    }
}

impl Settings {
    /// A game is on unless its appid is in the off set.
    pub fn game_on(&self, appid: u32) -> bool {
        !self.off.contains(&appid)
    }

    /// Switch a game on (removes it from the off set) or off (adds it).
    pub fn set_game(&mut self, appid: u32, on: bool) {
        if on {
            self.off.remove(&appid);
        } else {
            self.off.insert(appid);
        }
    }
}

fn on_off(v: bool) -> &'static str {
    if v {
        "on"
    } else {
        "off"
    }
}

/// The text written to `~/.config/flat3d/settings`.
pub fn settings_text(s: &Settings) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');
    out.push_str(&format!("auto-3d\t{}\n", on_off(s.auto_3d)));
    out.push_str(&format!("tunnel\t{}\n", on_off(s.tunnel)));
    for appid in &s.off {
        out.push_str(&format!("game-off\t{appid}\n"));
    }
    out
}

/// Read the settings text. A missing or wrong header gives the defaults; inside a good file,
/// bad values and unknown lines are skipped, so the default stays.
pub fn parse_settings(text: &str) -> Settings {
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Settings::default();
    }
    let mut s = Settings::default();
    for line in lines {
        let mut parts = line.split('\t');
        match (parts.next(), parts.next()) {
            (Some("auto-3d"), Some("on")) => s.auto_3d = true,
            (Some("auto-3d"), Some("off")) => s.auto_3d = false,
            (Some("tunnel"), Some("on")) => s.tunnel = true,
            (Some("tunnel"), Some("off")) => s.tunnel = false,
            (Some("game-off"), Some(v)) => {
                if let Ok(appid) = v.parse::<u32>() {
                    s.off.insert(appid);
                }
            }
            _ => {}
        }
    }
    s
}
