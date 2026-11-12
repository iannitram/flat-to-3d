//! Locked tests for E9c: health wording. Pure.

use flat3d_inject::api::{Health, Zone};
use flat3d_inject::view::{health_view, zone_colour, HealthView};

fn h(battery: Option<u8>, temp_c: Option<f64>, zone: Zone) -> Health {
    Health {
        battery,
        status: "Charging".into(),
        temp_c,
        zone,
    }
}
fn v(battery: &str, temp: &str, zone: Zone, colour: &str) -> HealthView {
    HealthView {
        battery: battery.into(),
        temp: temp.into(),
        zone,
        colour: colour.into(),
    }
}

#[test]
fn each_zone_has_its_own_colour() {
    let got: Vec<&str> = [Zone::Green, Zone::Yellow, Zone::Red, Zone::Unknown]
        .into_iter()
        .map(zone_colour)
        .collect();
    assert_eq!(got, vec!["#2e9e5b", "#e0a030", "#d6453d", "#b0b0b0"]);
}

#[test]
fn the_health_figures_are_written_out() {
    assert_eq!(
        health_view(Some(&h(Some(85), Some(31.2), Zone::Green))),
        v("85%", "31.2 C", Zone::Green, "#2e9e5b")
    );
    assert_eq!(
        health_view(Some(&h(Some(5), Some(43.0), Zone::Red))),
        v("5%", "43.0 C", Zone::Red, "#d6453d")
    );
}

#[test]
fn missing_data_says_unknown() {
    let unknown = v("unknown", "unknown", Zone::Unknown, "#b0b0b0");
    assert_eq!(health_view(None), unknown);
    assert_eq!(health_view(Some(&h(None, None, Zone::Unknown))), unknown);
}
