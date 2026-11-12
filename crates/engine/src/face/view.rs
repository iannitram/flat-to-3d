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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
