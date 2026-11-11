//! The words and colours the faces show. docs/code-map.md, Parts.

use crate::api::{Health, Zone};

pub fn zone_colour(z: Zone) -> &'static str {
    match z {
        Zone::Green => "#2e9e5b",
        Zone::Yellow => "#e0a030",
        Zone::Red => "#d6453d",
        Zone::Unknown => "#b0b0b0",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthView {
    pub battery: String,
    pub temp: String,
    pub zone: Zone,
    pub colour: String,
}

pub fn health_view(h: Option<&Health>) -> HealthView {
    match h {
        Some(health) => HealthView {
            battery: health
                .battery
                .map(|battery| format!("{battery}%"))
                .unwrap_or_else(|| "unknown".to_string()),
            temp: health
                .temp_c
                .map(|temp| format!("{temp:.1} C"))
                .unwrap_or_else(|| "unknown".to_string()),
            zone: health.zone,
            colour: zone_colour(health.zone).to_string(),
        },
        None => HealthView {
            battery: "unknown".to_string(),
            temp: "unknown".to_string(),
            zone: Zone::Unknown,
            colour: zone_colour(Zone::Unknown).to_string(),
        },
    }
}

pub fn throughput_text(bytes_per_second: u64) -> String {
    if bytes_per_second < 1_000 {
        format!("{bytes_per_second} B/s")
    } else if bytes_per_second < 1_000_000 {
        format!("{:.1} KB/s", bytes_per_second as f64 / 1_000.0)
    } else {
        format!("{:.1} MB/s", bytes_per_second as f64 / 1_000_000.0)
    }
}

pub fn uptime_text(seconds: u64) -> String {
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3_600, (seconds % 3_600) / 60)
    }
}
