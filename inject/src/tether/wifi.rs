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

/// The NetworkManager command that lists active Wi-Fi networks and their SSIDs.
pub fn active_ssid_command() -> Vec<String> {
    ["nmcli", "-t", "-f", "active,ssid", "dev", "wifi"]
        .iter()
        .map(|word| word.to_string())
        .collect()
}

/// The NetworkManager command that reads a saved network's key type and passphrase.
pub fn credentials_command(ssid: &str) -> Vec<String> {
    [
        "nmcli",
        "-s",
        "-g",
        "802-11-wireless-security.key-mgmt,802-11-wireless-security.psk",
        "connection",
        "show",
    ]
    .iter()
    .map(|word| word.to_string())
    .chain(std::iter::once(ssid.to_string()))
    .collect()
}

/// The adb command that reads the Quest's Wi-Fi status.
pub fn quest_ssid_command() -> Vec<String> {
    ["adb", "shell", "cmd", "wifi", "status"]
        .iter()
        .map(|word| word.to_string())
        .collect()
}

/// Quotes one word for the Quest shell, including embedded single quotes.
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Builds the adb command that connects the Quest to a Wi-Fi network.
pub fn connect_network_command(ssid: &str, security: Security, psk: &str) -> Vec<String> {
    vec![
        "adb".to_string(),
        "shell".to_string(),
        format!(
            "cmd wifi connect-network {} {} {}",
            shell_quote(ssid),
            security.as_str(),
            shell_quote(psk)
        ),
    ]
}

/// Joins command words for logging, replacing a connect-network passphrase with `***`.
pub fn redact_command(cmd: &[String]) -> String {
    let line = cmd.join(" ");
    if let Some(start) = line.find("connect-network ") {
        let suffix_start = start + "connect-network ".len();
        let suffix = &line[suffix_start..];
        if let Some(security_end) = [" wpa2 ", " wpa3 "]
            .iter()
            .filter_map(|marker| suffix.find(marker).map(|index| index + marker.len()))
            .min()
        {
            return format!("{}'***'", &line[..suffix_start + security_end]);
        }
    }
    line
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
