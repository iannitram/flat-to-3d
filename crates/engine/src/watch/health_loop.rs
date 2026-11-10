use std::path::PathBuf;

use crate::api::{self, EventKind, TetherMode};
use crate::exec::RelayRunner;
use crate::tether::heal::Heal;

pub struct HealthTick {
    pub health: Option<api::Health>,
    pub stats: Option<api::Stats>,
    pub tunnel: bool,
    pub heal: Option<Heal>,
    pub toast: Option<(EventKind, String)>,
}

pub struct HealthLoop;

impl HealthLoop {
    pub fn new(_start_unix: u64, _mem_start_kb: u64, _soak_file: Option<PathBuf>) -> Self {
        Self
    }
    pub fn tick(
        &mut self,
        _now: u64,
        mode: TetherMode,
        _playing: bool,
        _r: &mut dyn RelayRunner,
    ) -> HealthTick {
        HealthTick {
            health: None,
            stats: None,
            tunnel: mode != TetherMode::Off,
            heal: None,
            toast: None,
        }
    }
}
