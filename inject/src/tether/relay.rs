//! Command lists for the gnirehtet relay and Quest-side client.

use super::adb::tools_dir;

pub const RELAY_PORT: u16 = 31416;
pub const CLIENT_PACKAGE: &str = "com.genymobile.gnirehtet";

pub fn relay_path(home: &str) -> String {
    format!("{}/gnirehtet", tools_dir(home))
}

pub fn apk_path(home: &str) -> String {
    format!("{}/gnirehtet.apk", tools_dir(home))
}

pub fn relay_command(relay: &str, serial: &str) -> Vec<String> {
    vec![relay.to_string(), "run".to_string(), serial.to_string()]
}

pub fn client_start_commands() -> Vec<Vec<String>> {
    vec![
        vec![
            "adb".to_string(),
            "reverse".to_string(),
            "localabstract:gnirehtet".to_string(),
            format!("tcp:{RELAY_PORT}"),
        ],
        vec![
            "adb".to_string(),
            "shell".to_string(),
            "am".to_string(),
            "start".to_string(),
            "-n".to_string(),
            format!("{CLIENT_PACKAGE}/.GnirehtetActivity"),
            "-a".to_string(),
            format!("{CLIENT_PACKAGE}.START"),
        ],
    ]
}

pub fn client_stop_command() -> Vec<String> {
    vec![
        "adb".to_string(),
        "shell".to_string(),
        "am".to_string(),
        "start".to_string(),
        "-n".to_string(),
        format!("{CLIENT_PACKAGE}/.GnirehtetActivity"),
        "-a".to_string(),
        format!("{CLIENT_PACKAGE}.STOP"),
    ]
}

pub fn force_stop_command() -> Vec<String> {
    vec![
        "adb".to_string(),
        "shell".to_string(),
        "am".to_string(),
        "force-stop".to_string(),
        CLIENT_PACKAGE.to_string(),
    ]
}

pub fn remove_reverse_command() -> Vec<String> {
    vec![
        "adb".to_string(),
        "reverse".to_string(),
        "--remove".to_string(),
        "localabstract:gnirehtet".to_string(),
    ]
}

/// tun0 has an IPv4 address in `ip addr show tun0` output.
pub fn tun0_present(ip_addr_output: &str) -> bool {
    ip_addr_output
        .lines()
        .any(|line| line.trim_start().starts_with("inet "))
}

/// The output is empty or the device is missing.
pub fn tun0_gone(output: &str) -> bool {
    output.trim().is_empty() || output.contains("does not exist")
}

/// Receive and transmit bytes for tun0 from `/proc/net/dev` text.
pub fn parse_tun0_bytes(proc_net_dev: &str) -> Option<(u64, u64)> {
    for line in proc_net_dev.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("tun0:") {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            let receive = fields.first()?.parse::<u64>().ok()?;
            let transmit = fields.get(8)?.parse::<u64>().ok()?;
            return Some((receive, transmit));
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelState {
    Off,
    Starting,
    Connected,
}

pub fn tunnel_state(relay_alive: bool, tun0: bool) -> TunnelState {
    if !relay_alive {
        TunnelState::Off
    } else if !tun0 {
        TunnelState::Starting
    } else {
        TunnelState::Connected
    }
}
