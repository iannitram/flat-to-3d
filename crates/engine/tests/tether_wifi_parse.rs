//! Locked tests for E6f: Wi-Fi readers. Pure.

use flat3d_inject::tether::wifi::*;

#[test]
fn the_active_network_is_the_yes_line() {
    assert_eq!(
        parse_active_ssid("no:Cafe\nyes:HomeNet\nno:Other\n"),
        Some("HomeNet".to_string())
    );
}

#[test]
fn escaped_colons_and_backslashes_are_undone() {
    assert_eq!(
        parse_active_ssid("yes:My\\:Net\n"),
        Some("My:Net".to_string())
    );
    assert_eq!(parse_active_ssid("yes:A\\\\B\n"), Some("A\\B".to_string()));
}

#[test]
fn no_active_network_is_none() {
    assert_eq!(parse_active_ssid("no:Cafe\n"), None);
    assert_eq!(parse_active_ssid("yes:\n"), None);
    assert_eq!(parse_active_ssid(""), None);
}

#[test]
fn the_quest_network_is_the_quoted_name() {
    assert_eq!(
        parse_quest_ssid("Wifi is enabled\nWifi is connected to \"HomeNet\"\n"),
        Some("HomeNet".to_string())
    );
    assert_eq!(
        parse_quest_ssid("Wifi is connected to \"My Net\" extra"),
        Some("My Net".to_string())
    );
}

#[test]
fn a_quest_that_is_not_connected_is_none() {
    assert_eq!(parse_quest_ssid("Wifi is disabled\n"), None);
    assert_eq!(parse_quest_ssid("Wifi is connected to \"\"\n"), None);
}

#[test]
fn saved_networks_give_a_key_type_and_passphrase() {
    let (s, p) = parse_credentials("wpa-psk\nhunter 2\n").unwrap();
    assert_eq!((s, p.as_str()), (Security::Wpa2, "hunter 2"));
    let (s, p) = parse_credentials("sae\nabc\n").unwrap();
    assert_eq!((s, p.as_str()), (Security::Wpa3, "abc"));
    assert_eq!(Security::Wpa2.as_str(), "wpa2");
    assert_eq!(Security::Wpa3.as_str(), "wpa3");
}

#[test]
fn unusable_credentials_are_none() {
    assert!(parse_credentials("wpa-eap\nx\n").is_none());
    assert!(parse_credentials("wpa-psk\n").is_none());
    assert!(parse_credentials("wpa-psk\n\n").is_none());
    assert!(parse_credentials("").is_none());
}
