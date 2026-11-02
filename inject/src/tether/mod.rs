//! Run the Quest Tether connect and disconnect flow through an injectable runner.

pub mod adb;
pub mod heal;
pub mod relay;
pub mod stepaside;
pub mod wifi;

use crate::{api::TetherMode, link};
use adb::{AdbRunner, CommandRunner};

pub struct Tether<R> {
    pub adb: AdbRunner<R>,
    home: String,
    serial: String,
    mode: TetherMode,
    added_reverses: Vec<String>,
    client_started: bool,
}

impl<R: CommandRunner> Tether<R> {
    pub fn new(home: &str, serial: &str, choice: adb::AdbChoice, runner: R) -> Self {
        Self {
            adb: AdbRunner::new(choice, serial, runner),
            home: home.to_string(),
            serial: serial.to_string(),
            mode: TetherMode::Off,
            added_reverses: Vec::new(),
            client_started: false,
        }
    }

    pub fn mode(&self) -> TetherMode {
        self.mode
    }

    /// A foreign relay owns its client and reconnects. Only add the WiVRn link.
    pub fn connect(&mut self, foreign_relay: bool) -> Result<TetherMode, String> {
        if self.mode != TetherMode::Off || !self.added_reverses.is_empty() {
            return Err("Tether is already active or needs cleanup".to_string());
        }
        if !self.adb.has_adb() {
            return Err(format!(
                "No adb is available: none is running, and the bundled adb is not at {}",
                adb::bundled_adb(&self.home)
            ));
        }
        let result = self.connect_steps(foreign_relay);
        if let Err(mut error) = result {
            if self.client_started {
                if self.adb.run(&relay::client_stop_command()).is_err() {
                    let _ = self.adb.run(&relay::force_stop_command());
                }
                self.client_started = false;
            }
            if let Err(cleanup) = self.cleanup_reverses() {
                error.push_str(&format!("; cleanup failed: {cleanup}"));
            }
            return Err(error);
        }
        self.mode = stepaside::tether_mode(true, foreign_relay);
        Ok(self.mode)
    }

    fn connect_steps(&mut self, foreign_relay: bool) -> Result<(), String> {
        let existing = self.adb.run(&adb::reverse_list_command())?;
        for command in link::start_3d_commands() {
            self.run_connect_command(&command, &existing)?;
        }
        if !foreign_relay {
            let commands = relay::client_start_commands();
            self.run_connect_command(&commands[0], &existing)?;
            self.adb
                .run(&adb::install_command(&relay::apk_path(&self.home)))?;
            for command in &commands[1..] {
                self.adb.run(command)?;
            }
            self.client_started = true;
            self.adb.start_relay(&relay::relay_command(
                &relay::relay_path(&self.home),
                &self.serial,
            ))?;
        }
        Ok(())
    }

    fn run_connect_command(&mut self, command: &[String], existing: &str) -> Result<(), String> {
        let reverse = command.first().map(String::as_str) == Some("adb")
            && command.get(1).map(String::as_str) == Some("reverse");
        if reverse {
            for line in existing.lines() {
                // `adb reverse --list` prints `<device spec> <host spec>`; a serial may come first.
                let words: Vec<_> = line.split_whitespace().collect();
                if words.len() >= 2 && words[words.len() - 2] == command[2] {
                    return if words[words.len() - 1] == command[3] {
                        Ok(())
                    } else {
                        Err("A required reverse is already in use".to_string())
                    };
                }
            }
        }
        let output = self.adb.run(command)?;
        if command == ["adb", "devices"]
            && link::headset_from_adb(&output) != link::Headset::Present
        {
            return Err("Headset is missing or USB debugging is not authorised".to_string());
        }
        if reverse {
            self.added_reverses.push(command[2].clone());
        }
        Ok(())
    }

    fn cleanup_reverses(&mut self) -> Result<(), String> {
        let mut error = None;
        for spec in std::mem::take(&mut self.added_reverses).into_iter().rev() {
            if let Err(message) = self.adb.run(&adb::remove_reverse_command(&spec)) {
                self.added_reverses.push(spec);
                error.get_or_insert(message);
            }
        }
        error.map_or(Ok(()), Err)
    }

    /// Stop our client and relay, preserving WiVRn and any foreign tunnel.
    pub fn disconnect(&mut self) -> Result<(), String> {
        if self.mode == TetherMode::Off {
            return self.cleanup_reverses();
        }
        if self.mode == TetherMode::QuestTether {
            self.mode = TetherMode::Off;
            self.added_reverses.clear();
            return Ok(());
        }
        let mut error = self.adb.run(&relay::client_stop_command()).err();
        if let Err(message) = self.adb.run(&relay::remove_reverse_command()) {
            error.get_or_insert(message);
        }
        if let Err(message) = self.adb.stop_relay() {
            error.get_or_insert(message);
        }
        if let Some(error) = error {
            return Err(error);
        }
        self.mode = TetherMode::Off;
        self.client_started = false;
        self.added_reverses.clear();
        Ok(())
    }
}
