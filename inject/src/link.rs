//! Start 3D and Stop 3D over WiVRn's wired mode (`adb reverse tcp:9757`).

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

/// What one command left behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// The command exited with success.
    pub success: bool,
    /// Its standard output, as lossy UTF-8.
    pub stdout: String,
    /// Its standard error, as lossy UTF-8.
    pub stderr: String,
}

/// Runs one command and waits for it (tests fake this).
pub trait CommandRunner {
    /// Run `command` (program, then its arguments) and report its output.
    fn run(&mut self, command: &[String]) -> CommandOutput;
}

/// The real runner: `std::process::Command`, waiting for the exit.
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&mut self, command: &[String]) -> CommandOutput {
        let Some((program, args)) = command.split_first() else {
            return CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: "empty command".into(),
            };
        };
        match std::process::Command::new(program).args(args).output() {
            Ok(output) => CommandOutput {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            Err(error) => CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: format!("{program}: {error}"),
            },
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
