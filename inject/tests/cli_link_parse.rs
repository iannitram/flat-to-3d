//! Locked tests for E5l: parsing the link commands.

use flat3d_inject::cli_link::{parse_link, LinkCommand, SettingKey};

fn a(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}
fn ok(words: &[&str]) -> LinkCommand {
    parse_link(&a(words)).expect("ours").expect("valid")
}
fn usage(words: &[&str]) -> String {
    let e = parse_link(&a(words)).expect("ours").expect_err("invalid");
    assert!(e.starts_with("usage: flat3d-inject "), "{e}");
    e
}

#[test]
fn words_that_are_not_ours_are_left_alone() {
    assert_eq!(parse_link(&a(&["list"])), None);
    assert_eq!(parse_link(&a(&["inject", "5"])), None);
    assert_eq!(parse_link(&a(&[])), None);
}

#[test]
fn the_plain_commands() {
    assert_eq!(ok(&["watch"]), LinkCommand::Watch);
    assert_eq!(ok(&["snapshot"]), LinkCommand::Snapshot);
    assert_eq!(ok(&["install"]), LinkCommand::Install);
    assert_eq!(ok(&["uninstall"]), LinkCommand::Uninstall);
}

#[test]
fn switch_get_and_set() {
    assert_eq!(ok(&["switch", "get", "2075800"]), LinkCommand::SwitchGet { appid: 2075800 });
    assert_eq!(ok(&["switch", "set", "2075800", "off"]), LinkCommand::SwitchSet { appid: 2075800, on: false });
    assert_eq!(ok(&["switch", "set", "5", "on"]), LinkCommand::SwitchSet { appid: 5, on: true });
}

#[test]
fn setting_get_and_set() {
    assert_eq!(ok(&["setting", "get", "auto-3d"]), LinkCommand::SettingGet { key: SettingKey::Auto3d });
    assert_eq!(ok(&["setting", "set", "tunnel", "off"]), LinkCommand::SettingSet { key: SettingKey::Tunnel, on: false });
    assert_eq!(SettingKey::Auto3d.as_str(), "auto-3d");
    assert_eq!(SettingKey::Tunnel.as_str(), "tunnel");
}

#[test]
fn run_takes_the_words_after_the_double_dash() {
    assert_eq!(
        ok(&["run", "2075800", "--", "mangohud", "game", "-novid"]),
        LinkCommand::Run { appid: 2075800, argv: a(&["mangohud", "game", "-novid"]) }
    );
}

#[test]
fn bad_arguments_give_a_usage_message() {
    assert!(usage(&["switch", "set", "5", "maybe"]).contains("switch set"));
    assert!(usage(&["switch", "get"]).contains("switch get"));
    assert!(usage(&["switch"]).contains("switch"));
    assert!(usage(&["setting", "get", "volume"]).contains("setting"));
    assert!(usage(&["setting", "set", "tunnel"]).contains("setting set"));
    assert!(usage(&["run", "5", "game"]).contains("run"));
    assert!(usage(&["run", "5", "--"]).contains("run"));
    assert!(usage(&["run", "x", "--", "game"]).contains("run"));
}
