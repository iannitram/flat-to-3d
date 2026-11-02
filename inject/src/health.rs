//! Headset health: battery and temperature zones. docs/code-map.md, Parts.

use crate::api::{Health, Zone};

pub fn health_command() -> Vec<String> {
    vec![
        "adb".into(),
        "shell".into(),
        "dumpsys".into(),
        "battery".into(),
    ]
}

pub fn parse_dumpsys_battery(out: &str) -> Health {
    let mut battery = None;
    let mut status = "Unknown";
    let mut temp_c = None;

    for line in out.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("level:") {
            battery = value
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|level| *level <= 100);
        } else if let Some(value) = line.strip_prefix("status:") {
            status = match value.trim().parse::<u8>().ok() {
                Some(1) => "Unknown",
                Some(2) => "Charging",
                Some(3) => "Discharging",
                Some(4) => "Not Charging",
                Some(5) => "Full",
                _ => "Unknown",
            };
        } else if let Some(value) = line.strip_prefix("temperature:") {
            temp_c = value.trim().parse::<i32>().ok().map(|value| value as f64 / 10.0);
        }
    }

    Health {
        battery,
        status: status.into(),
        temp_c,
        zone: temperature_zone(temp_c),
    }
}

pub fn temperature_zone(temp_c: Option<f64>) -> Zone {
    match temp_c {
        None => Zone::Unknown,
        Some(temp) if temp < 38.0 => Zone::Green,
        Some(temp) if temp <= 42.0 => Zone::Yellow,
        Some(_) => Zone::Red,
    }
}
