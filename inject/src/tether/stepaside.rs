//! Detect a foreign gnirehtet relay and choose whether our tether should step aside.

/// Parse `ps -eo pid=,args=` output into process ids and command lines.
pub fn parse_ps(out: &str) -> Vec<(u32, String)> {
    out.lines()
        .filter_map(|line| {
            let line = line.trim();
            let split = line.find(char::is_whitespace)?;
            let pid = line[..split].parse::<u32>().ok()?;
            let command = line[split..].trim();
            if command.is_empty() {
                return None;
            }
            Some((pid, command.to_string()))
        })
        .collect()
}

/// Return whether another process is running a gnirehtet relay.
pub fn foreign_relay(procs: &[(u32, String)], own_pid: Option<u32>) -> bool {
    procs.iter().any(|(pid, command)| {
        if Some(*pid) == own_pid || !command.contains("gnirehtet") {
            return false;
        }

        let mut words = command.split_whitespace();
        let Some(executable) = words.next() else {
            return false;
        };
        let name = executable.rsplit('/').next().unwrap_or(executable);
        name != "adb"
            && words.clone().any(|word| word == "run" || word == "relay")
    })
}

/// Choose Quest Tether when another relay already owns the tunnel.
pub fn tether_mode(tunnel_on: bool, foreign: bool) -> crate::api::TetherMode {
    if foreign {
        crate::api::TetherMode::QuestTether
    } else if tunnel_on {
        crate::api::TetherMode::Own
    } else {
        crate::api::TetherMode::Off
    }
}
