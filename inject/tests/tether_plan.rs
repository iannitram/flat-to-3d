//! Locked tests for E6e: the tether's connect and disconnect plans. Pure.

use flat3d_inject::tether::plan::{connect_plan, disconnect_plan, TetherStep};
use flat3d_inject::tether::relay::{client_start_commands, client_stop_command, force_stop_command, remove_reverse_command};

fn c(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn connect_goes_in_this_order() {
    let want = vec![
        TetherStep::KillStaleRelay,
        TetherStep::SpawnRelay,
        TetherStep::SleepMs(1500),
        TetherStep::CheckRelayAlive,
        TetherStep::WaitTun0 { tries: 15, every_ms: 1500 },
        TetherStep::IfNoTun0(client_start_commands()),
        TetherStep::WaitTun0 { tries: 8, every_ms: 1500 },
    ];
    assert_eq!(connect_plan(), want);
}

#[test]
fn disconnect_goes_in_this_order() {
    let want = vec![
        TetherStep::Run(client_stop_command()),
        TetherStep::SleepMs(1500),
        TetherStep::IfVpnUp(force_stop_command()),
        TetherStep::Run(c(&["adb", "shell", "svc", "wifi", "disable"])),
        TetherStep::SleepMs(4000),
        TetherStep::Run(c(&["adb", "shell", "svc", "wifi", "enable"])),
        TetherStep::WaitWifi { tries: 6, every_ms: 1500 },
        TetherStep::KillRelay,
        TetherStep::Run(remove_reverse_command()),
    ];
    assert_eq!(disconnect_plan(), want);
}

#[test]
fn the_relay_is_killed_only_after_the_wifi_wait() {
    let p = disconnect_plan();
    let wifi = p.iter().position(|s| matches!(s, TetherStep::WaitWifi { .. })).unwrap();
    let kill = p.iter().position(|s| *s == TetherStep::KillRelay).unwrap();
    let stop = p.iter().position(|s| *s == TetherStep::Run(client_stop_command())).unwrap();
    assert!(stop < wifi && wifi < kill);
}

#[test]
fn wivrns_reverse_survives_the_disconnect() {
    for step in disconnect_plan() {
        if let TetherStep::Run(cmd) = step {
            assert!(!cmd.iter().any(|w| w == "--remove-all"), "{cmd:?}");
            assert!(!cmd.iter().any(|w| w.contains("9757")), "{cmd:?}");
        }
    }
}
