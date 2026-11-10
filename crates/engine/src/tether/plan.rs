//! The tether's connect and disconnect steps as data, so the order can be tested without a headset.

use super::relay::{
    client_start_commands, client_stop_command, force_stop_command, remove_reverse_command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TetherStep {
    /// Run a command through the runner.
    Run(Vec<String>),
    SleepMs(u64),
    /// Run the command only if tun0 is still up.
    IfVpnUp(Vec<String>),
    /// Run the commands only if tun0 is still absent.
    IfNoTun0(Vec<Vec<String>>),
    WaitTun0 {
        tries: u32,
        every_ms: u64,
    },
    WaitWifi {
        tries: u32,
        every_ms: u64,
    },
    KillStaleRelay,
    SpawnRelay,
    CheckRelayAlive,
    KillRelay,
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| word.to_string()).collect()
}

/// Connect: clear a stale relay, start ours, wait for tun0, and start the client only if tun0 is missing.
pub fn connect_plan() -> Vec<TetherStep> {
    vec![
        TetherStep::KillStaleRelay,
        TetherStep::SpawnRelay,
        TetherStep::SleepMs(1500),
        TetherStep::CheckRelayAlive,
        TetherStep::WaitTun0 {
            tries: 15,
            every_ms: 1500,
        },
        TetherStep::IfNoTun0(client_start_commands()),
        TetherStep::WaitTun0 {
            tries: 8,
            every_ms: 1500,
        },
    ]
}

/// Disconnect, Wi-Fi first: stop the headset's VPN, wait for its Wi-Fi, and only then stop the relay.
/// A relay killed first strands the headset on a dead VPN.
pub fn disconnect_plan() -> Vec<TetherStep> {
    vec![
        TetherStep::Run(client_stop_command()),
        TetherStep::SleepMs(1500),
        TetherStep::IfVpnUp(force_stop_command()),
        TetherStep::Run(words(&["adb", "shell", "svc", "wifi", "disable"])),
        TetherStep::SleepMs(4000),
        TetherStep::Run(words(&["adb", "shell", "svc", "wifi", "enable"])),
        TetherStep::WaitWifi {
            tries: 6,
            every_ms: 1500,
        },
        TetherStep::KillRelay,
        TetherStep::Run(remove_reverse_command()),
    ]
}
