//! Heal the tunnel after repeated misses, with spacing and flap breakers.

use std::collections::VecDeque;

pub const MISS_LIMIT: u32 = 3;
pub const MIN_HEAL_GAP_SECS: u64 = 30;
pub const FLAP_WINDOW_SECS: u64 = 600;
pub const FLAP_MAX: usize = 6;
pub const MAX_RECONNECTS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heal {
    None,
    RestartClient,
    RestartAdbThenClient,
    FullReconnect,
    GiveUp,
}

impl Heal {
    pub fn word(self) -> &'static str {
        match self {
            Heal::None => "none",
            Heal::RestartClient => "restart-client",
            Heal::RestartAdbThenClient => "restart-adb-then-client",
            Heal::FullReconnect => "full-reconnect",
            Heal::GiveUp => "give-up",
        }
    }
}

pub struct Healer {
    misses: u32,
    stage: u8,
    last_heal: Option<u64>,
    heals: VecDeque<u64>,
    reconnects: u32,
    tripped: bool,
}

impl Default for Healer {
    fn default() -> Self {
        Self::new()
    }
}

impl Healer {
    pub fn new() -> Healer {
        Healer {
            misses: 0,
            stage: 0,
            last_heal: None,
            heals: VecDeque::new(),
            reconnects: 0,
            tripped: false,
        }
    }

    pub fn observe(&mut self, now: u64, relay_alive: bool, tun0: bool) -> Heal {
        if self.tripped {
            return Heal::GiveUp;
        }

        if relay_alive && tun0 {
            self.misses = 0;
            self.stage = 0;
            return Heal::None;
        }

        self.misses = self.misses.saturating_add(1);
        if self.misses < MISS_LIMIT
            || self
                .last_heal
                .is_some_and(|last| now.saturating_sub(last) < MIN_HEAL_GAP_SECS)
        {
            return Heal::None;
        }

        let heal = match self.stage {
            0 => Heal::RestartClient,
            1 => Heal::RestartAdbThenClient,
            _ => Heal::FullReconnect,
        };

        while self
            .heals
            .front()
            .is_some_and(|&time| now.saturating_sub(time) >= FLAP_WINDOW_SECS)
        {
            self.heals.pop_front();
        }

        if self.heals.len() >= FLAP_MAX
            || (heal == Heal::FullReconnect && self.reconnects >= MAX_RECONNECTS)
        {
            self.tripped = true;
            return Heal::GiveUp;
        }

        self.heals.push_back(now);
        self.last_heal = Some(now);
        if heal == Heal::FullReconnect {
            self.reconnects += 1;
        }
        self.misses = 0;
        self.stage = (self.stage + 1) % 3;
        heal
    }

    pub fn reconnects(&self) -> u32 {
        self.reconnects
    }

    pub fn tripped(&self) -> bool {
        self.tripped
    }
}
