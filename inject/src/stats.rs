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
                        let dt = now_ms - previous_ms;

                        // If time step is zero, return (0, 0) and keep the old reading
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

                        let rate_rx = delta_rx * 1000 / dt;
                        let rate_tx = delta_tx * 1000 / dt;

                        // Remember the new reading
                        self.previous = Some((now_ms, rx, tx));

                        (rate_rx, rate_tx)
                    }
                }
            }
        }
    }
}
