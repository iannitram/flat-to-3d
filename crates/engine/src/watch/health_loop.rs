//! Periodic headset battery and temperature poll with the heat toasts.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::api::{self, EventKind};
use crate::exec::LinkRunner;
use crate::health::{health_command, parse_dumpsys_battery, toast_text, ToastGate};

pub const HEALTH_EVERY_SECS: u64 = 30;

pub struct HealthTick {
    pub health: Option<api::Health>,
    pub toast: Option<(EventKind, String)>,
}

/// Polls battery and temperature through Flat to 3D's own adb and raises the 38 C and 42 C toasts.
pub struct HealthLoop {
    gate: ToastGate,
    health_file: Option<PathBuf>,
    last_health: Option<u64>,
    health: Option<api::Health>,
}

impl HealthLoop {
    pub fn new(health_file: Option<PathBuf>) -> Self {
        Self {
            gate: ToastGate::new(),
            health_file,
            last_health: None,
            health: None,
        }
    }

    pub fn tick(&mut self, now: u64, playing: bool, r: &mut dyn LinkRunner) -> HealthTick {
        let mut toast = None;
        if due(now, self.last_health, HEALTH_EVERY_SECS) {
            self.last_health = Some(now);
            self.health = r
                .run(&health_command())
                .ok()
                .map(|out| parse_dumpsys_battery(&out));
            if let Some(health) = &self.health {
                if let (Some(kind), Some(temp)) =
                    (self.gate.observe(health.zone, playing), health.temp_c)
                {
                    toast = Some((kind, toast_text(kind, temp)));
                }
                let battery = health.battery.map_or_else(|| "-".into(), |v| v.to_string());
                let temp = health
                    .temp_c
                    .map_or_else(|| "-".into(), |v| format!("{v:.1}"));
                let zone = format!("{:?}", health.zone).to_lowercase();
                append_best_effort(
                    self.health_file.as_ref(),
                    &format!("health\t{now}\t{battery}\t{temp}\t{zone}\n"),
                );
            } else {
                append_best_effort(
                    self.health_file.as_ref(),
                    &format!("health\t{now}\t-\t-\tunknown\n"),
                );
            }
        }
        HealthTick {
            health: self.health.clone(),
            toast,
        }
    }
}

fn due(now: u64, last: Option<u64>, every_secs: u64) -> bool {
    // A clock that steps backwards must not stall the probes until it catches up.
    last.is_none_or(|last| now < last || now - last >= every_secs)
}

fn append_best_effort(path: Option<&PathBuf>, text: &str) {
    let Some(path) = path else { return };
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(text.as_bytes());
    }
}
