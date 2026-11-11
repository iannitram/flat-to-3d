//! Periodic headset health, tunnel and throughput probes.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::api::{self, EventKind, TetherMode};
use crate::exec::RelayRunner;
use crate::health::{health_command, parse_dumpsys_battery, toast_text, ToastGate};
use crate::stats::{RateMeter, Soak, SoakSummary};
use crate::tether::heal::{Heal, Healer};
use crate::tether::relay::{parse_tun0_bytes, tun0_present, tunnel_state, TunnelState};
use crate::tether::stepaside::{foreign_relay, parse_ps};

pub const TUN0_EVERY_SECS: u64 = 5;
pub const HEALTH_EVERY_SECS: u64 = 30;

pub struct HealthTick {
    pub health: Option<api::Health>,
    pub stats: Option<api::Stats>,
    pub tunnel: bool,
    pub heal: Option<Heal>,
    pub toast: Option<(EventKind, String)>,
}

pub struct HealthLoop {
    healer: Healer,
    gate: ToastGate,
    meter: RateMeter,
    soak: Soak,
    soak_file: Option<PathBuf>,
    last_tun0: Option<u64>,
    last_health: Option<u64>,
    health: Option<api::Health>,
    up_since: Option<u64>,
    gave_up: bool,
    in_own: bool,
    reconnects_base: u32,
    tunnel: bool,
    rates: (u64, u64),
}

impl HealthLoop {
    pub fn new(start_unix: u64, mem_start_kb: u64, soak_file: Option<PathBuf>) -> Self {
        Self {
            healer: Healer::new(),
            gate: ToastGate::new(),
            meter: RateMeter::new(),
            soak: Soak::new(start_unix, mem_start_kb),
            soak_file,
            last_tun0: None,
            last_health: None,
            health: None,
            up_since: None,
            gave_up: false,
            in_own: false,
            reconnects_base: 0,
            tunnel: false,
            rates: (0, 0),
        }
    }

    pub fn tick(
        &mut self,
        now: u64,
        mode: TetherMode,
        playing: bool,
        r: &mut dyn RelayRunner,
    ) -> HealthTick {
        let mut heal = None;
        let mut tunnel = self.tunnel;
        let mut rates = self.rates;

        if mode == TetherMode::Own && due(now, self.last_tun0, TUN0_EVERY_SECS) {
            self.last_tun0 = Some(now);
            let ps = r.run(&["ps", "-eo", "pid=,args="].map(str::to_owned));
            let relay_alive = ps
                .map(|out| parse_ps(&out))
                .is_ok_and(|procs| foreign_relay(&procs, None));

            let ip = r.run(&["adb", "shell", "ip", "addr", "show", "tun0"].map(str::to_owned));
            let tun0 = ip.as_deref().is_ok_and(tun0_present);

            let counters = if tun0 {
                r.run(&["adb", "shell", "cat", "/proc/net/dev"].map(str::to_owned))
                    .ok()
                    .and_then(|out| parse_tun0_bytes(&out))
            } else {
                None
            };
            rates = self.meter.sample(now.saturating_mul(1000), counters);

            tunnel = tunnel_state(relay_alive, tun0) == TunnelState::Connected;
            self.tunnel = tunnel;
            self.rates = rates;
            if tunnel {
                self.up_since.get_or_insert(now);
                self.soak.sample(rates.0.saturating_add(rates.1));
            } else {
                self.up_since = None;
            }

            let action = self.healer.observe(now, relay_alive, tun0);
            if action == Heal::GiveUp {
                if !self.gave_up {
                    heal = Some(action);
                    self.gave_up = true;
                }
            } else if action != Heal::None {
                heal = Some(action);
            }

            let line = format!(
                "sample\t{now}\t{}\t{}\t{}\t{}\t{}\n",
                rates.0,
                rates.1,
                u8::from(tun0),
                u8::from(relay_alive),
                action.word()
            );
            append_best_effort(self.soak_file.as_ref(), &line);
        } else if mode != TetherMode::Own {
            if self.in_own {
                self.in_own = false;
                self.reconnects_base = self.reconnects();
                self.healer = Healer::new();
                self.gave_up = false;
                self.last_tun0 = None;
            }
            self.up_since = None;
            self.tunnel = false;
            tunnel = false;
            self.rates = (0, 0);
            rates = (0, 0);
            self.meter.sample(now.saturating_mul(1000), None);
        }

        if mode == TetherMode::Own {
            self.in_own = true;
        }

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
                    self.soak_file.as_ref(),
                    &format!("health\t{now}\t{battery}\t{temp}\t{zone}\n"),
                );
            } else {
                append_best_effort(
                    self.soak_file.as_ref(),
                    &format!("health\t{now}\t-\t-\tunknown\n"),
                );
            }
        }

        let stats = if tunnel {
            self.up_since.map(|start| api::Stats {
                rx_bytes_s: rates.0,
                tx_bytes_s: rates.1,
                uptime_s: now.saturating_sub(start),
                reconnects: self.reconnects(),
            })
        } else {
            None
        };

        HealthTick {
            health: self.health.clone(),
            stats,
            tunnel,
            heal,
            toast,
        }
    }

    pub fn reconnects(&self) -> u32 {
        self.reconnects_base
            .saturating_add(self.healer.reconnects())
    }

    pub fn finish(self, now: u64, mem_end_kb: u64, orphans: u32) -> SoakSummary {
        let reconnects = self.reconnects();
        let summary = self.soak.finish(now, mem_end_kb, reconnects, orphans);
        append_best_effort(self.soak_file.as_ref(), &summary.to_text());
        summary
    }
}

fn due(now: u64, last: Option<u64>, every_secs: u64) -> bool {
    last.is_none_or(|last| now.saturating_sub(last) >= every_secs)
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
