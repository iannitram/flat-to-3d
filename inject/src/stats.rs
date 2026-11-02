//! Throughput and soak figures. docs/code-map.md, Parts.

#[derive(Debug, Clone, Default)]
pub struct RateMeter {
    previous: Option<(u64, u64, u64)>, // (ms, rx, tx)
}

impl RateMeter {
    pub fn new() -> RateMeter {
        RateMeter { previous: None }
    }

    pub fn sample(&mut self, now_ms: u64, counters: Option<(u64, u64)>) -> (u64, u64) {
        match counters {
            None => {
                // Forget the previous reading
                self.previous = None;
                (0, 0)
            }
            Some((rx, tx)) => {
                match self.previous {
                    None => {
                        // First reading - remember it and return (0, 0)
                        self.previous = Some((now_ms, rx, tx));
                        (0, 0)
                    }
                    Some((previous_ms, previous_rx, previous_tx)) => {
                        let dt = now_ms.saturating_sub(previous_ms);

                        // If time did not advance, return (0, 0) and keep the old reading
                        if dt == 0 {
                            return (0, 0);
                        }

                        // If any counter went backwards, return (0, 0) and remember the new reading
                        if rx < previous_rx || tx < previous_tx {
                            self.previous = Some((now_ms, rx, tx));
                            return (0, 0);
                        }

                        // Calculate rates in bytes per second
                        let delta_rx = rx - previous_rx;
                        let delta_tx = tx - previous_tx;

                        // Saturate rather than overflow on a huge counter step
                        let rate_rx = delta_rx.saturating_mul(1000) / dt;
                        let rate_tx = delta_tx.saturating_mul(1000) / dt;

                        // Remember the new reading
                        self.previous = Some((now_ms, rx, tx));

                        (rate_rx, rate_tx)
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoakSummary {
    pub start: u64,
    pub end: u64,
    pub peak_bytes_s: u64,
    pub avg_bytes_s: u64,
    pub reconnects: u32,
    pub mem_start_kb: u64,
    pub mem_end_kb: u64,
    pub orphans: u32,
}

impl SoakSummary {
    pub fn to_text(&self) -> String {
        format!(
            "flat3d-soak 1\nstart\t{}\nend\t{}\npeak_bytes_s\t{}\navg_bytes_s\t{}\nreconnects\t{}\nmem_start_kb\t{}\nmem_end_kb\t{}\norphans\t{}\n",
            self.start,
            self.end,
            self.peak_bytes_s,
            self.avg_bytes_s,
            self.reconnects,
            self.mem_start_kb,
            self.mem_end_kb,
            self.orphans
        )
    }
}

#[derive(Debug, Clone)]
pub struct Soak {
    start: u64,
    mem_start_kb: u64,
    peak: u64,
    sum: u64,
    count: u64,
}

impl Soak {
    pub fn new(start_unix: u64, mem_start_kb: u64) -> Soak {
        Soak {
            start: start_unix,
            mem_start_kb,
            peak: 0,
            sum: 0,
            count: 0,
        }
    }

    pub fn sample(&mut self, bytes_s: u64) {
        self.peak = self.peak.max(bytes_s);
        self.sum = self.sum.saturating_add(bytes_s);
        self.count = self.count.saturating_add(1);
    }

    pub fn finish(
        self,
        end_unix: u64,
        mem_end_kb: u64,
        reconnects: u32,
        orphans: u32,
    ) -> SoakSummary {
        SoakSummary {
            start: self.start,
            end: end_unix,
            peak_bytes_s: self.peak,
            avg_bytes_s: self.sum.checked_div(self.count).unwrap_or(0),
            reconnects,
            mem_start_kb: self.mem_start_kb,
            mem_end_kb,
            orphans,
        }
    }
}

pub fn mem_used_kb(meminfo: &str) -> Option<u64> {
    let mut total = None;
    let mut available = None;
    for line in meminfo.lines() {
        let (key, value) = line.split_once(':')?;
        let value = value.split_whitespace().next()?.parse::<u64>().ok()?;
        match key {
            "MemTotal" => total = Some(value),
            "MemAvailable" => available = Some(value),
            _ => {}
        }
    }
    Some(total?.saturating_sub(available?))
}

pub fn soak_path(home: &str, start_unix: u64) -> String {
    format!("{home}/.local/state/flat3d/soak-{start_unix}.txt")
}
