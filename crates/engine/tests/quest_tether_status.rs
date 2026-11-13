//! WP-38: reading Quest Tether's state, read-only. The one command is `status --json`; only the
//! `state` text is kept; a missing program, a failed run or junk reads as "not installed".

use flat3d_inject::exec::CommandOutput;
use flat3d_inject::link::quest_tether::{
    parse_status, quest_tether_status, status_command, QuestTetherStatus,
};
use flat3d_inject::testing::FakeRunner;

/// A `status --json` reply with every field of Quest Tether's client contract (version 1).
const FULL: &str = r#"{"version":1,"state":"connected","rx_bps":1200,"tx_bps":800,"uptime_s":93,"battery_level":81,"battery_status":"Discharging","reconnects":0,"relay_clients":1,"temp_c":31.5,"zone":"green","device_serial":"TESTSERIAL","device_model":"Quest","vpn_ip":"10.0.0.2","adb_path":"/opt/adb/adb","error":null}"#;

fn status(state: &str) -> Option<QuestTetherStatus> {
    Some(QuestTetherStatus {
        state: state.to_string(),
    })
}

#[test]
fn the_command_is_exactly_three_words() {
    assert_eq!(status_command(), ["quest-tether", "status", "--json"]);
}

#[test]
fn a_reply_with_every_documented_field_gives_connected() {
    assert_eq!(parse_status(FULL), status("connected"));
}

#[test]
fn a_reply_with_only_state_parses() {
    assert_eq!(parse_status(r#"{"state":"idle"}"#), status("idle"));
}

#[test]
fn every_documented_state_is_kept_as_it_is() {
    for state in [
        "idle",
        "starting",
        "connected",
        "reconnecting",
        "stopping",
        "error",
    ] {
        let reply = format!(r#"{{"state":"{state}"}}"#);
        assert_eq!(parse_status(&reply), status(state), "{state}");
    }
}

#[test]
fn a_surrounding_newline_is_fine() {
    assert_eq!(parse_status("{\"state\":\"idle\"}\n"), status("idle"));
}

#[test]
fn junk_or_a_missing_state_gives_none() {
    for out in [
        "{}",
        "[]",
        "",
        "   \n",
        "not json",
        "null",
        r#""connected""#,
        r#"[{"state":"connected"}]"#,
        r#"{"state":null}"#,
        r#"{"state":7}"#,
        r#"{"state":""}"#,
        r#"{"State":"connected"}"#,
        r#"{"state":"connected""#,
    ] {
        assert_eq!(parse_status(out), None, "{out:?}");
    }
}

#[test]
fn the_runner_gets_the_one_command() {
    let mut runner = FakeRunner::new().ok(&["quest-tether", "status", "--json"], FULL);
    assert_eq!(quest_tether_status(&mut runner), status("connected"));
    assert_eq!(runner.ran, vec![status_command()]);
}

#[test]
fn a_program_that_is_not_installed_gives_none() {
    let mut runner = FakeRunner::new().fail(
        &["quest-tether"],
        "quest-tether: No such file or directory (os error 2)",
    );
    assert_eq!(quest_tether_status(&mut runner), None);
    assert_eq!(runner.ran.len(), 1);
}

#[test]
fn a_non_zero_exit_gives_none_even_with_json_on_stdout() {
    let mut runner = FakeRunner::new().reply(
        &["quest-tether"],
        CommandOutput {
            success: false,
            stdout: r#"{"state":"connected"}"#.into(),
            stderr: "status failed".into(),
        },
    );
    assert_eq!(quest_tether_status(&mut runner), None);
}

#[test]
fn unparseable_output_gives_none() {
    let mut runner = FakeRunner::new().ok(&["quest-tether"], "Quest Tether is not running\n");
    assert_eq!(quest_tether_status(&mut runner), None);
}
