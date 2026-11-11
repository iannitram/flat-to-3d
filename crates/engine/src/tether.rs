//! Run the Quest Tether connect and disconnect flow through an injectable runner.

pub mod adb;
pub mod heal;
pub mod plan;
pub mod relay;
pub mod stepaside;
pub mod wifi;

use crate::{api::TetherMode, link};
use adb::{CommandRunner, TetherAdb};
use plan::{PlanCtx, PlanRun};
use std::time::Duration;

pub struct Tether<R> {
    pub adb: TetherAdb<R>,
    home: String,
    serial: String,
    mode: TetherMode,
    added_reverses: Vec<String>,
    client_started: bool,
    sleep: Box<dyn FnMut(u64)>,
    wivrn: Option<Vec<Vec<String>>>,
    wifi_attempts: u32,
    last_wifi: Option<wifi::SyncDecision>,
}

impl<R: CommandRunner> Tether<R> {
    pub fn new(home: &str, serial: &str, choice: adb::AdbChoice, runner: R) -> Self {
        Self {
            adb: TetherAdb::new(choice, serial, runner),
            home: home.to_string(),
            serial: serial.to_string(),
            mode: TetherMode::Off,
            added_reverses: Vec::new(),
            client_started: false,
            sleep: Box::new(|ms| std::thread::sleep(Duration::from_millis(ms))),
            wivrn: None,
            wifi_attempts: 0,
            last_wifi: None,
        }
    }

    pub fn set_sleep(&mut self, sleep: Box<dyn FnMut(u64)>) {
        self.sleep = sleep;
    }

    pub fn set_wivrn(&mut self, app: String, flags: Vec<String>) {
        self.wivrn = Some(link::start_3d_commands_with(&link::WivrnStart {
            scope: None,
            app,
            flags,
        }));
    }

    pub fn sync_wifi(&mut self) -> wifi::SyncDecision {
        let pc = self
            .adb
            .run(&wifi::active_ssid_command())
            .ok()
            .and_then(|out| wifi::parse_active_ssid(&out));
        let quest = self
            .adb
            .run(&wifi::quest_ssid_command())
            .ok()
            .and_then(|out| wifi::parse_quest_ssid(&out));
        let decision = wifi::sync_decision(pc.as_deref(), quest.as_deref(), self.wifi_attempts);
        if decision == wifi::SyncDecision::Switch {
            let credentials = pc
                .as_deref()
                .map(wifi::credentials_command)
                .and_then(|command| self.adb.run(&command).ok())
                .and_then(|out| wifi::parse_credentials(&out));
            let joined = if let (Some(ssid), Some((sec, psk))) = (pc.as_deref(), credentials) {
                self.adb.run(&wifi::connect_argv(ssid, sec, &psk)).is_ok()
            } else {
                false
            };
            self.wifi_attempts = wifi::next_attempts(self.wifi_attempts, joined);
        }
        self.last_wifi = Some(decision);
        decision
    }

    pub fn last_wifi(&self) -> Option<&wifi::SyncDecision> {
        self.last_wifi.as_ref()
    }

    pub fn mode(&self) -> TetherMode {
        self.mode
    }

    /// A foreign relay owns its client and reconnects. Only add the WiVRn link.
    pub fn connect(&mut self, foreign_relay: bool) -> Result<TetherMode, String> {
        self.connect_with(foreign_relay, true)
    }

    /// Connect the tunnel alone: check the headset, then (without a foreign relay) the client
    /// and relay. WiVRn's server, reverse and app are left to Start 3D.
    pub fn connect_tunnel_only(&mut self, foreign_relay: bool) -> Result<TetherMode, String> {
        self.connect_with(foreign_relay, false)
    }

    fn connect_with(&mut self, foreign_relay: bool, wivrn: bool) -> Result<TetherMode, String> {
        if self.mode != TetherMode::Off || !self.added_reverses.is_empty() {
            return Err("Tether is already active or needs cleanup".to_string());
        }
        if !self.adb.has_adb() {
            return Err(format!(
                "No adb is available: none is running, and the bundled adb is not at {}; fix: flat3d-inject install",
                adb::bundled_adb(&self.home)
            ));
        }

        let mut spawned = false;
        let result = self.connect_steps(foreign_relay, wivrn, &mut spawned);
        if let Err(mut error) = result {
            if self.client_started {
                if self.adb.run(&relay::client_stop_command()).is_err() {
                    let _ = self.adb.run(&relay::force_stop_command());
                }
                self.client_started = false;
            }
            if spawned {
                let _ = self.adb.stop_relay();
            }
            if let Err(cleanup) = self.cleanup_reverses() {
                error.push_str(&format!("; cleanup failed: {cleanup}"));
            }
            return Err(error);
        }
        self.mode = stepaside::tether_mode(true, foreign_relay);
        Ok(self.mode)
    }

    fn connect_steps(
        &mut self,
        foreign_relay: bool,
        wivrn: bool,
        spawned: &mut bool,
    ) -> Result<(), String> {
        let existing = self.adb.run(&adb::reverse_list_command())?;
        let mut start = self.wivrn.clone().unwrap_or_else(link::start_3d_commands);
        if !wivrn {
            start.truncate(1);
        }
        for command in start {
            self.run_connect_command(&command, &existing)?;
        }
        if !foreign_relay {
            self.run_connect_command(&relay::client_start_commands()[0], &existing)?;
            let package_list = self.adb.run(&[
                "adb".into(),
                "shell".into(),
                "pm".into(),
                "list".into(),
                "packages".into(),
                relay::CLIENT_PACKAGE.into(),
            ])?;
            if !package_list
                .lines()
                .any(|line| line.trim() == format!("package:{}", relay::CLIENT_PACKAGE))
            {
                self.adb
                    .run(&adb::install_command(&relay::apk_path(&self.home)))?;
            }

            let ctx = PlanCtx {
                relay: relay::relay_command(&relay::relay_path(&self.home), &self.serial),
            };
            let mut out = PlanRun::default();
            let result = {
                let mut via = Via {
                    adb: &mut self.adb,
                    spawned: false,
                };
                let result = plan::run_plan_in(
                    &plan::connect_plan(),
                    &ctx,
                    &mut via,
                    &mut *self.sleep,
                    &mut out,
                );
                *spawned = via.spawned;
                result
            };
            self.client_started = out.ran.contains(&relay::client_start_commands()[1]);
            result?;
        }
        Ok(())
    }

    fn run_connect_command(&mut self, command: &[String], existing: &str) -> Result<(), String> {
        let reverse = command.first().map(String::as_str) == Some("adb")
            && command.get(1).map(String::as_str) == Some("reverse");
        if reverse {
            for line in existing.lines() {
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
        let ctx = PlanCtx {
            relay: relay::relay_command(&relay::relay_path(&self.home), &self.serial),
        };
        let mut out = PlanRun::default();
        let result = {
            let mut via = Via {
                adb: &mut self.adb,
                spawned: false,
            };
            plan::run_plan_in(
                &plan::disconnect_plan(),
                &ctx,
                &mut via,
                &mut *self.sleep,
                &mut out,
            )
        };
        match result {
            Ok(()) => {
                self.mode = TetherMode::Off;
                self.client_started = false;
                self.added_reverses.clear();
                Ok(())
            }
            Err(error) => {
                let remove = relay::remove_reverse_command();
                if !out.ran.contains(&remove) {
                    let _ = self.adb.stop_relay();
                    let _ = self.adb.run(&remove);
                }
                Err(error)
            }
        }
    }

    /// Tear the tether down on this machine only, for when the headset is already gone.
    pub fn drop_local(&mut self) -> Result<(), String> {
        let result = if self.mode == TetherMode::Own {
            self.adb.stop_relay()
        } else {
            Ok(())
        };
        self.mode = TetherMode::Off;
        self.client_started = false;
        self.added_reverses.clear();
        result
    }
}

struct Via<'a, R> {
    adb: &'a mut TetherAdb<R>,
    spawned: bool,
}

impl<R: CommandRunner> adb::RelayRunner for Via<'_, R> {
    fn run(&mut self, command: &[String]) -> Result<String, String> {
        self.adb.run(command)
    }

    fn start_relay(&mut self, command: &[String], _adb: &str) -> Result<(), String> {
        self.adb.start_relay(command)?;
        self.spawned = true;
        Ok(())
    }

    fn stop_relay(&mut self) -> Result<(), String> {
        self.adb.stop_relay()
    }
}
