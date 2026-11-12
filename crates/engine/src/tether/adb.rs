//! Tether-specific adb and relay execution.

pub use crate::exec::{ProcessRunner, RelayRunner, RelayRunner as CommandRunner};
pub use crate::link::adb::{
    bundled_adb, choose_adb, listener_pid, proc_exe_path, tools_dir, AdbChoice,
};

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
        crate::link::adb::run(&mut self.runner, &self.path, &self.serial, command)
    }

    pub fn start_relay(&mut self, command: &[String]) -> Result<(), String> {
        self.runner.start_relay(command, &self.path)
    }

    pub fn stop_relay(&mut self) -> Result<(), String> {
        self.runner.stop_relay()
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
