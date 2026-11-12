//! Choosing the adb executable shared by the link service.

use crate::api::AdbSource;
use crate::link;
use std::fs;
use std::process::Command;

/// Find the executable backing the adb server listening on its default port.
pub fn running_adb_exe() -> Option<String> {
    // architecture-allowlist: follow-up (the `ss` probe remains outside the runner seam).
    let output = Command::new("ss").arg("-tlnp").output().ok()?;
    let pid = listener_pid(&String::from_utf8_lossy(&output.stdout), 5037)?;
    fs::read_link(proc_exe_path(pid))
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

pub fn listener_pid(ss_output: &str, port: u16) -> Option<u32> {
    let suffix = format!(":{port}");
    for line in ss_output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 || !fields[3].ends_with(&suffix) {
            continue;
        }
        if let Some(pid_start) = line.find("pid=") {
            let digits: String = line[pid_start + 4..]
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .collect();
            if let Ok(pid) = digits.parse() {
                return Some(pid);
            }
        }
    }
    None
}

pub fn proc_exe_path(pid: u32) -> String {
    format!("/proc/{pid}/exe")
}

pub fn tools_dir(home: &str) -> String {
    format!("{home}/.local/share/flat3d/bin")
}

pub fn bundled_adb(home: &str) -> String {
    format!("{}/adb", tools_dir(home))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdbChoice {
    pub path: String,
    pub source: AdbSource,
}

pub fn choose_adb(running_exe: Option<&str>, bundled: Option<&str>) -> AdbChoice {
    if let Some(path) = running_exe.filter(|path| !path.is_empty()) {
        return AdbChoice {
            path: path.to_string(),
            source: AdbSource::Running,
        };
    }
    if let Some(path) = bundled.filter(|path| !path.is_empty()) {
        return AdbChoice {
            path: path.to_string(),
            source: AdbSource::Bundled,
        };
    }
    AdbChoice {
        path: String::new(),
        source: AdbSource::Missing,
    }
}

/// Run one command, replacing the adb word with the selected executable and adding its serial.
pub fn run<R: crate::exec::RelayRunner>(
    runner: &mut R,
    adb: &str,
    serial: &str,
    argv: &[String],
) -> Result<String, String> {
    if argv.first().map(String::as_str) != Some("adb") {
        return runner.run(argv);
    }
    if adb.is_empty() {
        return Err("No adb is available".to_string());
    }
    let mut words = vec![adb.to_string(), "-s".to_string(), serial.to_string()];
    words.extend_from_slice(&argv[1..]);
    runner.run(&words)
}

/// E6b: wraps the link's `CommandRunner` so `adb ...` words name the adb in use and the
/// headset's serial; the link code never knows which adb it got.
pub struct AdbRunner<R: link::CommandRunner> {
    inner: R,
    adb: String,
    serial: Option<String>,
}

impl<R: link::CommandRunner> AdbRunner<R> {
    pub fn new(inner: R, adb: &str, serial: Option<&str>) -> Self {
        Self {
            inner,
            adb: adb.to_string(),
            serial: serial.map(str::to_string),
        }
    }

    pub fn inner(&self) -> &R {
        &self.inner
    }
}

impl<R: link::CommandRunner> link::CommandRunner for AdbRunner<R> {
    fn run(&mut self, command: &[String]) -> link::CommandOutput {
        if command.first().map(String::as_str) != Some("adb") {
            return self.inner.run(command);
        }
        let mut words = vec![self.adb.clone()];
        if let Some(serial) = self.serial.as_deref() {
            let server_command = matches!(
                command.get(1).map(String::as_str),
                Some("devices" | "start-server" | "kill-server" | "-s")
            );
            if !server_command {
                words.push("-s".to_string());
                words.push(serial.to_string());
            }
        }
        words.extend_from_slice(&command[1..]);
        self.inner.run(&words)
    }
}
