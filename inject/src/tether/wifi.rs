//! Wi-Fi sync readers (SPEC.md M25), ported from Quest Tether's `net_sync.py`: the PC's
//! active network from `nmcli`, the Quest's network from `cmd wifi status`, and the key type
//! and passphrase of a saved network. Pure parsing; no process is run here.

/// The WPA key type of a saved network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    /// `wpa-psk` in NetworkManager.
    Wpa2,
    /// `sae` in NetworkManager.
    Wpa3,
}

impl Security {
    /// The name the sync uses for this key type: `"wpa2"` or `"wpa3"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Security::Wpa2 => "wpa2",
            Security::Wpa3 => "wpa3",
        }
    }
}

/// Undoes terse-mode escaping: `\:` is `:` and `\\` is `\`.
fn unescape_terse(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The active network's name from `nmcli -t -f active,ssid dev wifi`: the name on the first
/// `yes:` line, with escapes undone. `None` when no line is `yes` or its name is empty.
pub fn parse_active_ssid(nmcli_out: &str) -> Option<String> {
    let name = nmcli_out
        .lines()
        .find_map(|line| line.strip_prefix("yes:"))?;
    let name = unescape_terse(name);
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// The Quest's network from `cmd wifi status`: the text between the quotes in
/// `Wifi is connected to "NAME"`. `None` when that phrase is absent or the name is empty.
pub fn parse_quest_ssid(out: &str) -> Option<String> {
    const MARK: &str = "Wifi is connected to \"";
    let start = out.find(MARK)? + MARK.len();
    let rest = &out[start..];
    let end = rest.find('"')?;
    let name = &rest[..end];
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// A saved network's key type and passphrase from
/// `nmcli -s -g 802-11-wireless-security.key-mgmt,802-11-wireless-security.psk connection show NAME`.
/// The first line is the key type, the second the passphrase. `None` for any other key type,
/// a missing line or an empty passphrase.
pub fn parse_credentials(out: &str) -> Option<(Security, String)> {
    let mut lines = out.lines();
    let security = match lines.next()? {
        "wpa-psk" => Security::Wpa2,
        "sae" => Security::Wpa3,
        _ => return None,
    };
    let passphrase = lines.next()?;
    if passphrase.is_empty() {
        None
    } else {
        Some((security, passphrase.to_string()))
    }
}

/// Maximum number of failed attempts to sync the Quest's Wi-Fi network.
pub const MAX_WIFI_ATTEMPTS: u32 = 3;

/// What the Wi-Fi sync should do with the PC and Quest network names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDecision {
    NoPcWifi,
    NoQuestWifi,
    Same,
    GiveUp,
    Switch,
}

/// Decide whether the Quest should join the PC's Wi-Fi network.
pub fn sync_decision(pc: Option<&str>, quest: Option<&str>, failed_attempts: u32) -> SyncDecision {
    let Some(pc) = pc else {
        return SyncDecision::NoPcWifi;
    };
    let Some(quest) = quest else {
        return SyncDecision::NoQuestWifi;
    };
    if pc == quest {
        SyncDecision::Same
    } else if failed_attempts >= MAX_WIFI_ATTEMPTS {
        SyncDecision::GiveUp
    } else {
        SyncDecision::Switch
    }
}

/// Reset the failure counter after joining, or count this failed attempt.
pub fn next_attempts(prev: u32, joined: bool) -> u32 {
    if joined {
        0
    } else {
        prev + 1
    }
}
