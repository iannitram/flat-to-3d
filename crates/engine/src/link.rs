//! Start 3D and Stop 3D over WiVRn's wired mode (`adb reverse tcp:9757`).

pub mod adb;
pub mod wivrn;
pub use wivrn::{
    detect_quest_app, detect_server, FlatpakScope, QuestApp, WivrnDirs, WivrnStart, QUEST_APPS,
    SERVER_FLAGS,
};

pub use crate::exec::{CommandOutput, CommandRunner, SystemRunner};

/// WiVRn's port, reversed over USB.
pub const WIVRN_PORT: u16 = 9757;

/// WiVRn's Flatpak package.
pub const WIVRN_FLATPAK: &str = "io.github.wivrn.wivrn";

/// The WiVRn app on the headset.
pub const WIVRN_APP: &str = "org.meumeu.wivrn.github";

/// The user unit that runs the WiVRn server.
pub const WIVRN_UNIT: &str = "flat3d-wivrn";

/// Where the headset link stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headset {
    /// adb does not list the headset as ready.
    Missing,
    /// adb lists the headset, but it has not authorised this machine.
    Unauthorised,
    /// adb lists the headset and it is ready.
    Present,
}

/// Read `adb devices` output into a [`Headset`].
pub fn headset_from_adb(stdout: &str) -> Headset {
    let mut seen_unauthorised = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
            continue;
        }
        match line.split_whitespace().nth(1) {
            Some("device") => return Headset::Present,
            Some("unauthorized") => seen_unauthorised = true,
            _ => {}
        }
    }
    if seen_unauthorised {
        Headset::Unauthorised
    } else {
        Headset::Missing
    }
}

/// Where the headset link stands (the API's enum).
pub use crate::api::LinkState;

/// The commands Start 3D runs, in order (pure: tests assert on this list).
pub fn start_3d_commands() -> Vec<Vec<String>> {
    let port = format!("tcp:{WIVRN_PORT}");
    let unit = format!("--unit={WIVRN_UNIT}");
    vec![
        vec!["adb".into(), "devices".into()],
        vec![
            "systemd-run".into(),
            "--user".into(),
            "--collect".into(),
            unit,
            "flatpak".into(),
            "run".into(),
            "--command=wivrn-server".into(),
            WIVRN_FLATPAK.into(),
        ],
        vec!["adb".into(), "reverse".into(), port.clone(), port],
        vec![
            "adb".into(),
            "shell".into(),
            "am".into(),
            "start".into(),
            "-a".into(),
            "android.intent.action.VIEW".into(),
            "-d".into(),
            "wivrn+tcp://localhost".into(),
            WIVRN_APP.into(),
        ],
    ]
}

pub fn start_3d_commands_with(s: &WivrnStart) -> Vec<Vec<String>> {
    let mut commands = start_3d_commands();
    let mut server = vec![
        "systemd-run".into(),
        "--user".into(),
        "--collect".into(),
        format!("--unit={WIVRN_UNIT}"),
        "flatpak".into(),
        "run".into(),
    ];
    if let Some(scope) = s.scope {
        server.push(scope.flag().into());
    }
    server.push("--command=wivrn-server".into());
    server.push(WIVRN_FLATPAK.into());
    server.extend(s.flags.iter().cloned());
    commands[1] = server;
    *commands[3].last_mut().expect("am start has an app") = s.app.clone();
    commands
}

/// The commands Stop 3D runs, in order (pure: tests assert on this list).
pub fn stop_3d_commands() -> Vec<Vec<String>> {
    let port = format!("tcp:{WIVRN_PORT}");
    let unit = format!("{WIVRN_UNIT}.service");
    vec![
        vec![
            "adb".into(),
            "shell".into(),
            "am".into(),
            "force-stop".into(),
            WIVRN_APP.into(),
        ],
        vec!["adb".into(), "reverse".into(), "--remove".into(), port],
        vec!["systemctl".into(), "--user".into(), "stop".into(), unit],
    ]
}

/// A Start 3D command that failed: its words, and its trimmed stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkError {
    /// The command's words joined with spaces.
    pub command: String,
    /// Its stderr, trimmed.
    pub message: String,
}

impl From<LinkError> for crate::api::ApiError {
    fn from(e: LinkError) -> Self {
        crate::api::ApiError::new("command-failed", &format!("{}: {}", e.command, e.message))
    }
}

/// Run one command; a failure becomes a [`LinkError`] naming it.
fn run_step(
    runner: &mut dyn CommandRunner,
    command: &[String],
) -> Result<CommandOutput, LinkError> {
    let output = runner.run(command);
    if output.success {
        Ok(output)
    } else {
        Err(LinkError {
            command: command.join(" "),
            message: output.stderr.trim().to_string(),
        })
    }
}

/// Run [`start_3d_commands`] in order and report where the link stands. A missing or
/// unauthorised headset stops after `adb devices`; the first failing command stops the rest.
/// It never returns `Starting`.
pub fn start_3d(runner: &mut dyn CommandRunner) -> Result<LinkState, LinkError> {
    let commands = start_3d_commands();
    let devices = run_step(runner, &commands[0])?;
    match headset_from_adb(&devices.stdout) {
        Headset::Missing => return Ok(LinkState::NoHeadset),
        Headset::Unauthorised => return Ok(LinkState::Unauthorised),
        Headset::Present => {}
    }
    for command in &commands[1..] {
        run_step(runner, command)?;
    }
    Ok(LinkState::Ready)
}

pub fn start_3d_with(
    runner: &mut dyn CommandRunner,
    s: &WivrnStart,
) -> Result<LinkState, LinkError> {
    let commands = start_3d_commands_with(s);
    let devices = run_step(runner, &commands[0])?;
    match headset_from_adb(&devices.stdout) {
        Headset::Missing => return Ok(LinkState::NoHeadset),
        Headset::Unauthorised => return Ok(LinkState::Unauthorised),
        Headset::Present => {}
    }
    for command in &commands[1..] {
        run_step(runner, command)?;
    }
    Ok(LinkState::Ready)
}

/// True when systemd says the stop target is not loaded or not found (nothing to stop).
fn unit_already_gone(stderr: &str) -> bool {
    let message = stderr.trim();
    message == "Unit flat3d-wivrn.service not loaded"
        || message == "Unit flat3d-wivrn.service not found"
        || message == "Failed to stop flat3d-wivrn.service: Unit flat3d-wivrn.service not loaded."
        || message == "Failed to stop flat3d-wivrn.service: Unit flat3d-wivrn.service not found."
}

/// Run all of [`stop_3d_commands`] in order, whatever each returns: with the headset
/// unplugged the two adb steps fail, and the server must still stop. The last command (the
/// server stop) decides: success or a narrowly recognized missing unit is `Stopped`;
/// other failures are a [`LinkError`] naming it.
pub fn stop_3d(runner: &mut dyn CommandRunner) -> Result<LinkState, LinkError> {
    let commands = stop_3d_commands();
    let (last, before) = commands
        .split_last()
        .expect("stop_3d_commands is not empty");
    for command in before {
        let _ = runner.run(command);
    }
    let output = runner.run(last);
    if !output.success && !unit_already_gone(&output.stderr) {
        return Err(LinkError {
            command: last.join(" "),
            message: output.stderr.trim().to_string(),
        });
    }
    Ok(LinkState::Stopped)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A runner that answers from a script and records every command it was asked to run.
    struct Scripted {
        answers: std::collections::VecDeque<CommandOutput>,
        ran: Vec<Vec<String>>,
    }

    impl CommandRunner for Scripted {
        fn run(&mut self, command: &[String]) -> CommandOutput {
            self.ran.push(command.to_vec());
            self.answers.pop_front().expect("unexpected extra command")
        }
    }

    fn scripted(answers: Vec<CommandOutput>) -> Scripted {
        Scripted {
            answers: answers.into(),
            ran: Vec::new(),
        }
    }

    fn out(success: bool, stderr: &str) -> CommandOutput {
        CommandOutput {
            success,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    #[test]
    fn stop_runs_all_three_in_order_and_is_stopped() {
        let mut f = scripted(vec![out(true, ""), out(true, ""), out(true, "")]);
        assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
        assert_eq!(f.ran, stop_3d_commands());
    }

    #[test]
    fn unplugged_headset_still_stops_the_server() {
        let no_dev = "error: no devices/emulators found";
        let mut f = scripted(vec![out(false, no_dev), out(false, no_dev), out(true, "")]);
        assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
        assert_eq!(f.ran, stop_3d_commands());
    }

    #[test]
    fn server_stop_failing_is_an_error_naming_it() {
        let mut f = scripted(vec![
            out(true, ""),
            out(true, ""),
            out(
                false,
                "Failed to stop flat3d-wivrn.service: Access denied\n",
            ),
        ]);
        let expected = LinkError {
            command: "systemctl --user stop flat3d-wivrn.service".into(),
            message: "Failed to stop flat3d-wivrn.service: Access denied".into(),
        };
        assert_eq!(stop_3d(&mut f), Err(expected));
        assert_eq!(f.ran.len(), 3);
    }

    #[test]
    fn start_stop_start_reports_ready_stopped_ready() {
        let device = "List of devices attached\n1WMHH0A1B2C3D4\tdevice\n\n";
        let dev = |s: &str| CommandOutput {
            success: true,
            stdout: s.into(),
            stderr: String::new(),
        };
        let mut f = scripted(vec![
            dev(device),
            out(true, ""),
            out(true, ""),
            out(true, ""),
            out(true, ""),
            out(true, ""),
            out(true, ""),
            dev(device),
            out(true, ""),
            out(true, ""),
            out(true, ""),
        ]);
        assert_eq!(start_3d(&mut f), Ok(LinkState::Ready));
        assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
        assert_eq!(start_3d(&mut f), Ok(LinkState::Ready));
        let mut expected = start_3d_commands();
        expected.extend(stop_3d_commands());
        expected.extend(start_3d_commands());
        assert_eq!(f.ran, expected);
    }

    #[test]
    fn system_runner_empty_command_stderr() {
        let out = SystemRunner.run(&[]);
        assert!(!out.success);
        assert!(out.stdout.is_empty());
        assert_eq!(out.stderr, "empty command");
    }

    #[test]
    fn system_runner_spawn_error_stdout_empty() {
        let out = SystemRunner.run(&["flat3d-no-such-program".into()]);
        assert!(!out.success);
        assert!(out.stdout.is_empty());
    }

    #[test]
    fn adb_failing_runs_nothing_more() {
        struct Fake(Vec<Vec<String>>);
        impl CommandRunner for Fake {
            fn run(&mut self, command: &[String]) -> CommandOutput {
                self.0.push(command.to_vec());
                CommandOutput {
                    success: false,
                    stdout: String::new(),
                    stderr: "adb: not found".into(),
                }
            }
        }
        let mut f = Fake(Vec::new());
        assert!(start_3d(&mut f).is_err());
        assert_eq!(f.0.len(), 1);
    }

    #[test]
    fn step_three_failing_stops_after_four_commands() {
        struct Fake {
            step: usize,
            ran: Vec<Vec<String>>,
        }
        impl CommandRunner for Fake {
            fn run(&mut self, command: &[String]) -> CommandOutput {
                self.ran.push(command.to_vec());
                let out = match self.step {
                    0 => CommandOutput {
                        success: true,
                        stdout: "List of devices attached\n123\tdevice\n".into(),
                        stderr: String::new(),
                    },
                    1 | 2 => CommandOutput {
                        success: true,
                        stdout: String::new(),
                        stderr: String::new(),
                    },
                    _ => CommandOutput {
                        success: false,
                        stdout: String::new(),
                        stderr: "Error: Activity not found\n".into(),
                    },
                };
                self.step += 1;
                out
            }
        }
        let mut f = Fake {
            step: 0,
            ran: Vec::new(),
        };
        let err = start_3d(&mut f).unwrap_err();
        assert!(err.command.starts_with("adb shell am start"));
        assert_eq!(err.message, "Error: Activity not found");
        assert_eq!(f.ran.len(), 4);
    }
}
