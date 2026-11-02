//! Locked tests for E7a: battery dump and temperature zones. Pure.

use flat3d_inject::api::{Health, Zone};
use flat3d_inject::health::{health_command, parse_dumpsys_battery, temperature_zone};

fn dump(status: &str, level: &str, temp: &str) -> String {
    format!(
        "Current Battery Service state:\n  AC powered: false\n  USB powered: true\n  Max charging current: 900000\n  status: {status}\n  health: 2\n  present: true\n  level: {level}\n  scale: 100\n  voltage: 4100\n  temperature: {temp}\n  technology: Li-ion\n"
    )
}

#[test]
fn the_command() {
    assert_eq!(health_command(), vec!["adb", "shell", "dumpsys", "battery"]);
}

#[test]
fn a_normal_dump() {
    let h = parse_dumpsys_battery(&dump("2", "85", "312"));
    assert_eq!(
        h,
        Health { battery: Some(85), status: "Charging".into(), temp_c: Some(31.2), zone: Zone::Green }
    );
}

#[test]
fn every_status_code() {
    let names = [("1", "Unknown"), ("2", "Charging"), ("3", "Discharging"), ("4", "Not Charging"), ("5", "Full"), ("9", "Unknown")];
    for (code, name) in names {
        assert_eq!(parse_dumpsys_battery(&dump(code, "50", "300")).status, name);
    }
}

#[test]
fn missing_fields_are_unknown() {
    let h = parse_dumpsys_battery("");
    assert_eq!(h, Health { battery: None, status: "Unknown".into(), temp_c: None, zone: Zone::Unknown });
    let h = parse_dumpsys_battery(&dump("3", "x", "y"));
    assert_eq!((h.battery, h.temp_c, h.zone), (None, None, Zone::Unknown));
}

#[test]
fn a_level_over_100_is_not_believed() {
    assert_eq!(parse_dumpsys_battery(&dump("3", "250", "300")).battery, None);
}

#[test]
fn the_zone_follows_the_temperature() {
    assert_eq!(parse_dumpsys_battery(&dump("3", "50", "385")).zone, Zone::Yellow);
    assert_eq!(parse_dumpsys_battery(&dump("3", "50", "425")).zone, Zone::Red);
}

#[test]
fn the_zone_edges() {
    assert_eq!(temperature_zone(Some(37.9)), Zone::Green);
    assert_eq!(temperature_zone(Some(38.0)), Zone::Yellow);
    assert_eq!(temperature_zone(Some(42.0)), Zone::Yellow);
    assert_eq!(temperature_zone(Some(42.1)), Zone::Red);
    assert_eq!(temperature_zone(None), Zone::Unknown);
}
