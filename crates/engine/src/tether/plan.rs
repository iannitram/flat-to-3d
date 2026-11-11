//! The tether's connect and disconnect steps as data, so the order can be tested without a headset.

use super::relay::{
    client_start_commands, client_stop_command, force_stop_command, remove_reverse_command,
};
use super::{adb::RelayRunner, relay, stepaside, wifi};

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

/// Context used by the plan runner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanCtx {
    pub relay: Vec<String>,
}

/// Commands run and tunnel state observed while executing a plan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlanRun {
    pub ran: Vec<Vec<String>>,
    pub tun0: bool,
}

/// Run plan steps through an injectable runner, recording commands before they are rewritten.
///
/// Call it through `Tether`. A bare `ProcessRunner` would start the relay with `ADB` empty.
pub fn run_plan_in(
    steps: &[TetherStep],
    ctx: &PlanCtx,
    r: &mut dyn RelayRunner,
    sleep: &mut dyn FnMut(u64),
    out: &mut PlanRun,
) -> Result<(), String> {
    fn run(
        command: &[String],
        r: &mut dyn RelayRunner,
        out: &mut PlanRun,
    ) -> Result<String, String> {
        out.ran.push(command.to_vec());
        r.run(command)
    }
    fn tun0(r: &mut dyn RelayRunner, out: &mut PlanRun) -> bool {
        let command = words(&["adb", "shell", "ip", "addr", "show", "tun0"]);
        run(&command, r, out)
            .ok()
            .is_some_and(|output| relay::tun0_present(&output))
    }
    fn wait<F>(tries: u32, every_ms: u64, sleep: &mut dyn FnMut(u64), mut probe: F) -> bool
    where
        F: FnMut() -> bool,
    {
        for index in 0..tries {
            if probe() {
                return true;
            }
            if index + 1 < tries {
                sleep(every_ms);
            }
        }
        false
    }

    for step in steps {
        match step {
            TetherStep::Run(command) => {
                run(command, r, out)?;
            }
            TetherStep::SleepMs(ms) => sleep(*ms),
            TetherStep::IfVpnUp(command) => {
                if tun0(r, out) {
                    run(command, r, out)?;
                }
            }
            TetherStep::IfNoTun0(commands) => {
                if !tun0(r, out) {
                    for command in commands {
                        run(command, r, out)?;
                    }
                }
            }
            TetherStep::WaitTun0 { tries, every_ms } => {
                out.tun0 = wait(*tries, *every_ms, sleep, || tun0(r, out));
            }
            TetherStep::WaitWifi { tries, every_ms } => {
                wait(*tries, *every_ms, sleep, || {
                    run(&wifi::quest_ssid_command(), r, out)
                        .ok()
                        .and_then(|output| wifi::parse_quest_ssid(&output))
                        .is_some()
                });
            }
            TetherStep::KillStaleRelay => {
                if !ctx.relay.is_empty() {
                    let ps = run(&words(&["ps", "-eo", "pid=,args="]), r, out)?;
                    for (pid, args) in stepaside::parse_ps(&ps) {
                        if args.split_whitespace().next() == ctx.relay.first().map(String::as_str) {
                            let _ = run(&words(&["kill", &pid.to_string()]), r, out);
                        }
                    }
                }
            }
            TetherStep::SpawnRelay => {
                if ctx.relay.is_empty() {
                    return Err("No relay command".to_string());
                }
                r.start_relay(&ctx.relay, "")?;
            }
            TetherStep::CheckRelayAlive => {
                if ctx.relay.is_empty() {
                    return Err("No relay command".to_string());
                }
                let ps = run(&words(&["ps", "-eo", "pid=,args="]), r, out)?;
                if !stepaside::parse_ps(&ps)
                    .iter()
                    .any(|(_, args)| args == &ctx.relay.join(" "))
                {
                    return Err("Tether relay exited during startup".to_string());
                }
            }
            TetherStep::KillRelay => r.stop_relay()?,
        }
    }
    Ok(())
}

/// Convenience runner when the caller does not need a context or execution record.
pub fn run_plan(
    steps: &[TetherStep],
    r: &mut dyn RelayRunner,
    sleep: &mut dyn FnMut(u64),
) -> Result<(), String> {
    run_plan_in(
        steps,
        &PlanCtx::default(),
        r,
        sleep,
        &mut PlanRun::default(),
    )
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
