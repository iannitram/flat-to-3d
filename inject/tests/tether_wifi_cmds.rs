//! Locked tests for E6g: Wi-Fi commands and redaction. Pure.

use flat3d_inject::tether::wifi::*;

fn c(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn the_reading_commands() {
    assert_eq!(active_ssid_command(), c(&["nmcli", "-t", "-f", "active,ssid", "dev", "wifi"]));
    assert_eq!(
        credentials_command("My Net"),
        c(&["nmcli", "-s", "-g", "802-11-wireless-security.key-mgmt,802-11-wireless-security.psk", "connection", "show", "My Net"])
    );
    assert_eq!(quest_ssid_command(), c(&["adb", "shell", "cmd", "wifi", "status"]));
}

#[test]
fn quoting_handles_spaces_and_quotes() {
    assert_eq!(shell_quote("My Net"), "'My Net'");
    assert_eq!(shell_quote("it's"), "'it'\\''s'");
    assert_eq!(shell_quote(""), "''");
}

#[test]
fn the_connect_command_is_one_shell_string() {
    let cmd = connect_network_command("My Net", Security::Wpa2, "s3cret pw");
    assert_eq!(cmd, c(&["adb", "shell", "cmd wifi connect-network 'My Net' wpa2 's3cret pw'"]));
    let cmd = connect_network_command("Home", Security::Wpa3, "x");
    assert_eq!(cmd[2], "cmd wifi connect-network 'Home' wpa3 'x'");
}

#[test]
fn the_log_line_never_holds_the_passphrase() {
    let cmd = connect_network_command("My Net", Security::Wpa2, "s3cret");
    let line = redact_command(&cmd);
    assert_eq!(line, "adb shell cmd wifi connect-network 'My Net' wpa2 '***'");
    assert!(!line.contains("s3cret"));
}

#[test]
fn a_passphrase_with_spaces_and_a_security_word_is_still_hidden() {
    let cmd = connect_network_command("A", Security::Wpa3, "pw wpa2 more");
    let line = redact_command(&cmd);
    assert!(line.ends_with(" wpa3 '***'"), "{line}");
    assert!(!line.contains("more"));
}

#[test]
fn other_commands_are_just_joined() {
    assert_eq!(redact_command(&c(&["adb", "devices"])), "adb devices");
}
