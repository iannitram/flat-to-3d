//! Choosing the adb executable shared by the tether and link service.

use crate::api::AdbSource;
use crate::link;

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

/// Process operations used by the tether; tests supply a recording runner.
pub trait CommandRunner {
    fn run(&mut self, command: &[String]) -> Result<String, String>;
    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String>;
    fn stop_relay(&mut self) -> Result<(), String>;
}

/// The real runner owns only the relay it starts.
#[derive(Default)]
pub struct ProcessRunner {
    relay: Option<std::process::Child>,
}

impl CommandRunner for ProcessRunner {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        let output = std::process::Command::new(&command[0])
            .args(&command[1..])
            .output()
            .map_err(|_| "Could not run tether command".to_string())?;
        if !output.status.success() {
            return Err("Tether command failed".to_string());
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn start_relay(&mut self, command: &[String], adb: &str) -> Result<(), String> {
        let mut child = std::process::Command::new(&command[0])
            .args(&command[1..])
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

/// Use the chosen adb and headset for every tether word list (the tether's own runner).
pub struct TetherAdb<R> {
    pub runner: R,
    path: String,
    serial: String,
}

impl<R: CommandRunner> TetherAdb<R> {
    pub fn new(choice: AdbChoice, serial: &str, runner: R) -> Self {
        Self {
            runner,
            path: choice.path,
            serial: serial.to_string(),
        }
    }

    /// False when neither a running nor a bundled adb was chosen.
    pub fn has_adb(&self) -> bool {
        !self.path.is_empty()
    }

    pub fn run(&mut self, command: &[String]) -> Result<String, String> {
        if command.first().map(String::as_str) != Some("adb") {
            return self.runner.run(command);
        }
        if self.path.is_empty() {
            return Err("No adb is available".to_string());
        }
        let mut words = vec![self.path.clone(), "-s".to_string(), self.serial.clone()];
        words.extend_from_slice(&command[1..]);
        self.runner.run(&words)
    }

    pub fn start_relay(&mut self, command: &[String]) -> Result<(), String> {
        self.runner.start_relay(command, &self.path)
    }

    pub fn stop_relay(&mut self) -> Result<(), String> {
        self.runner.stop_relay()
    }
}

/// E6b: wraps the link's `CommandRunner` so `adb ...` words name the adb in use and the
/// headset's serial; the link and tether code never know which adb they got.
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

pub fn reverse_list_command() -> Vec<String> {
    vec!["adb".into(), "reverse".into(), "--list".into()]
}

pub fn install_command(apk: &str) -> Vec<String> {
    vec!["adb".into(), "install".into(), "-r".into(), apk.into()]
}

pub fn remove_reverse_command(spec: &str) -> Vec<String> {
    vec![
        "adb".into(),
        "reverse".into(),
        "--remove".into(),
        spec.into(),
    ]
}

#[cfg(test)]
mod adb_runner_tests {
    use super::*;
    use crate::link::CommandOutput;

    struct Fake {
        ran: Vec<Vec<String>>,
    }

    impl link::CommandRunner for Fake {
        fn run(&mut self, command: &[String]) -> CommandOutput {
            self.ran.push(command.to_vec());
            CommandOutput {
                success: true,
                stdout: "out".into(),
                stderr: String::new(),
            }
        }
    }

    fn c(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    fn runner(serial: Option<&str>) -> AdbRunner<Fake> {
        AdbRunner::new(Fake { ran: vec![] }, "/h/bin/adb", serial)
    }

    #[test]
    fn the_adb_word_is_swapped_and_the_serial_added() {
        let mut r = runner(Some("S1"));
        link::CommandRunner::run(&mut r, &c(&["adb", "reverse", "tcp:1", "tcp:2"]));
        assert_eq!(
            r.inner().ran,
            vec![c(&["/h/bin/adb", "-s", "S1", "reverse", "tcp:1", "tcp:2"])]
        );
    }

    #[test]
    fn without_a_serial_there_is_no_dash_s() {
        let mut r = runner(None);
        link::CommandRunner::run(&mut r, &c(&["adb", "shell", "ls"]));
        assert_eq!(r.inner().ran, vec![c(&["/h/bin/adb", "shell", "ls"])]);
    }

    #[test]
    fn server_commands_never_get_a_serial() {
        let mut r = runner(Some("S1"));
        link::CommandRunner::run(&mut r, &c(&["adb", "devices"]));
        link::CommandRunner::run(&mut r, &c(&["adb", "kill-server"]));
        link::CommandRunner::run(&mut r, &c(&["adb", "start-server"]));
        assert_eq!(
            r.inner().ran,
            vec![
                c(&["/h/bin/adb", "devices"]),
                c(&["/h/bin/adb", "kill-server"]),
                c(&["/h/bin/adb", "start-server"]),
            ]
        );
    }

    #[test]
    fn a_command_that_already_has_a_serial_is_not_doubled() {
        let mut r = runner(Some("S1"));
        link::CommandRunner::run(&mut r, &c(&["adb", "-s", "OTHER", "shell", "ls"]));
        assert_eq!(
            r.inner().ran,
            vec![c(&["/h/bin/adb", "-s", "OTHER", "shell", "ls"])]
        );
    }

    #[test]
    fn other_commands_pass_through_unchanged() {
        let mut r = runner(Some("S1"));
        link::CommandRunner::run(
            &mut r,
            &c(&["systemctl", "--user", "stop", "flat3d-wivrn.service"]),
        );
        link::CommandRunner::run(&mut r, &[]);
        assert_eq!(
            r.inner().ran,
            vec![
                c(&["systemctl", "--user", "stop", "flat3d-wivrn.service"]),
                vec![]
            ]
        );
    }

    #[test]
    fn the_inner_result_is_returned() {
        let mut r = runner(None);
        let out = link::CommandRunner::run(&mut r, &c(&["adb", "devices"]));
        assert!(out.success);
        assert_eq!(out.stdout, "out");
    }
}
