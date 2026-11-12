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

/// Process operations used by the link service; tests supply a recording runner.
pub trait LinkRunner {
    fn run(&mut self, command: &[String]) -> Result<String, String>;
}

/// The real runner: one process per command, waiting for the exit.
#[derive(Default)]
pub struct ProcessRunner;

impl LinkRunner for ProcessRunner {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        let Some((program, args)) = command.split_first() else {
            return Err("Could not run command: empty command".to_string());
        };
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .map_err(|error| format!("Could not run command: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail = tail_lines(&stderr, 4);
            let mut message = format!("Command failed: {}", exit_text(&output.status));
            if !tail.is_empty() {
                message.push_str("; ");
                message.push_str(&tail);
            }
            return Err(message);
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
