//! The two runner seams shared by the engine.

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

/// Process operations used by the tether; tests supply a recording runner.
pub trait RelayRunner {
    fn run(&mut self, command: &[String]) -> Result<String, String>;
    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String>;
    fn stop_relay(&mut self) -> Result<(), String>;
}

/// The real runner owns only the relay it starts.
#[derive(Default)]
pub struct ProcessRunner {
    relay: Option<std::process::Child>,
}

impl RelayRunner for ProcessRunner {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        let Some((program, args)) = command.split_first() else {
            return Err("Could not run tether command: empty command".to_string());
        };
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .map_err(|error| format!("Could not run tether command: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail = tail_lines(&stderr, 4);
            let mut message = format!("Tether command failed: {}", exit_text(&output.status));
            if !tail.is_empty() {
                message.push_str("; ");
                message.push_str(&tail);
            }
            return Err(message);
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String> {
        let Some((program, args)) = command.split_first() else {
            return Err("Could not start tether relay".to_string());
        };
        let mut child = std::process::Command::new(program)
            .args(args)
            .env("ADB", adb)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| "Could not start tether relay".to_string())?;
        match child.try_wait() {
            Ok(None) => {
                self.relay = Some(child);
                Ok(())
            }
            Ok(Some(_)) => Err("Tether relay exited during startup".to_string()),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                Err("Could not check tether relay".to_string())
            }
        }
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        if let Some(child) = self.relay.as_mut() {
            if child
                .try_wait()
                .map_err(|_| "Could not check tether relay")?
                .is_none()
            {
                child.kill().map_err(|_| "Could not stop tether relay")?;
            }
            child.wait().map_err(|_| "Could not reap tether relay")?;
            self.relay = None;
        }
        Ok(())
    }
}

fn tail_lines(stderr: &str, n: usize) -> String {
    let lines: Vec<_> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    lines[lines.len().saturating_sub(n)..].join(" / ")
}

fn exit_text(status: &std::process::ExitStatus) -> String {
    if let Some(code) = status.code() {
        return format!("exit {code}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("exit signal {signal}");
        }
    }
    "exit signal".to_string()
}
