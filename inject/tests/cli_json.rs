//! Locked tests for E1e: `--json` on every `flat3d-inject` command. The command bodies stay
//! stubs here (#27 wires them), so these tests use `render` and stub-proof checks.

use flat3d_inject::api::{ApiError, ERROR_CODES};
use flat3d_inject::cli::{execute, exit_code, parse, render, Cli, Command, USAGE};
use serde_json::{json, Value};

const INJECT_USAGE: &str = "usage: flat3d-inject inject <appid> [--json]";

fn args(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn cli(command: Command, json: bool) -> Result<Cli, String> {
    Ok(Cli { command, json })
}

fn one_json_line(stdout: &str) -> Value {
    assert!(stdout.ends_with('\n'), "{stdout:?}");
    assert_eq!(stdout.trim_end().lines().count(), 1, "{stdout:?}");
    serde_json::from_str(stdout.trim_end()).expect("stdout is one JSON object")
}

#[test]
fn every_command_parses_with_and_without_json_in_any_position() {
    let cases = [
        ("list", Command::List),
        ("inject 2651280", Command::Inject { appid: 2651280 }),
        ("start-3d", Command::Start3d),
        ("stop-3d", Command::Stop3d),
        ("status", Command::Status),
    ];
    for (line, command) in cases {
        assert_eq!(parse(&args(line)), cli(command.clone(), false), "{line}");
        let after = format!("{line} --json");
        assert_eq!(parse(&args(&after)), cli(command.clone(), true), "{after}");
        let before = format!("--json {line}");
        assert_eq!(parse(&args(&before)), cli(command, true), "{before}");
    }
}

#[test]
fn command_names_match_the_words() {
    assert_eq!(Command::List.name(), "list");
    assert_eq!(Command::Inject { appid: 1 }.name(), "inject");
    assert_eq!(Command::Start3d.name(), "start-3d");
    assert_eq!(Command::Stop3d.name(), "stop-3d");
    assert_eq!(Command::Status.name(), "status");
}

#[test]
fn bad_command_lines_are_usage_errors() {
    for line in ["", "fly", "list extra", "list --verbose", "--json"] {
        assert_eq!(parse(&args(line)), Err(USAGE.to_string()), "{line:?}");
    }
    for line in ["inject", "inject abc", "inject -1 --json", "inject 1 2"] {
        assert_eq!(parse(&args(line)), Err(INJECT_USAGE.to_string()), "{line}");
    }
    let usage = "usage: flat3d-inject list | inject <appid> | start-3d | stop-3d | status [--json]";
    assert_eq!(USAGE, usage);
}

#[test]
fn exit_codes_are_2_for_usage_and_not_built_else_1() {
    assert_eq!(exit_code(&ApiError::new("usage", "u")), 2);
    assert_eq!(exit_code(&ApiError::new("not-built", "n")), 2);
    assert_eq!(exit_code(&ApiError::new("refused", "r")), 1);
    assert_eq!(exit_code(&ApiError::new("command-failed", "c")), 1);
}

#[test]
fn render_json_ok_is_one_line_with_ok_command_data() {
    let data = json!({"state": "ready", "message": "m"});
    let out = render("start-3d", Ok(data.clone()), true);
    assert_eq!((out.code, out.stderr.as_str()), (0, ""));
    let expected = json!({"ok": true, "command": "start-3d", "data": data});
    assert_eq!(one_json_line(&out.stdout), expected);
}

#[test]
fn render_json_error_carries_code_and_message_for_each_command() {
    for name in ["list", "inject", "start-3d", "stop-3d", "status"] {
        let out = render(
            name,
            Err(ApiError::new("not-built", "not built yet: x")),
            true,
        );
        assert_eq!((out.code, out.stderr.as_str()), (2, ""));
        let error = json!({"code": "not-built", "message": "not built yet: x"});
        let expected = json!({"ok": false, "command": name, "error": error});
        assert_eq!(one_json_line(&out.stdout), expected);
    }
}

#[test]
fn render_human_error_goes_to_stderr_with_an_error_prefix() {
    let out = render("inject", Err(ApiError::new("refused", "not safe")), false);
    assert_eq!(out.code, 1);
    assert_eq!(out.stdout, "");
    assert_eq!(out.stderr, "error: not safe\n");
}

#[test]
fn render_human_ok_writes_stdout_only() {
    let out = render("list", Ok(json!([])), false);
    assert_eq!((out.code, out.stderr.as_str()), (0, ""));
    assert!(!out.stdout.is_empty());
}

#[test]
fn a_usage_error_with_json_is_a_usage_object_with_an_empty_command() {
    let out = execute(&args("fly --json"));
    assert_eq!((out.code, out.stderr.as_str()), (2, ""));
    let error = json!({"code": "usage", "message": USAGE});
    let expected = json!({"ok": false, "command": "", "error": error});
    assert_eq!(one_json_line(&out.stdout), expected);
}

#[test]
fn a_usage_error_without_json_goes_to_stderr() {
    let out = execute(&args("fly"));
    assert_eq!((out.code, out.stdout.as_str()), (2, ""));
    assert_eq!(out.stderr, format!("error: {USAGE}\n"));
}

#[test]
fn every_command_with_json_prints_one_object_naming_the_command() {
    for line in ["list", "inject 5", "start-3d", "stop-3d", "status"] {
        let out = execute(&args(&format!("{line} --json")));
        let v = one_json_line(&out.stdout);
        assert_eq!(v["command"], json!(line.split(' ').next().unwrap()));
        assert_eq!(v["ok"], json!(out.code == 0), "{v}");
        if out.code != 0 {
            let code = v["error"]["code"].as_str().unwrap();
            assert!(ERROR_CODES.contains(&code), "{code}");
        }
    }
}

#[test]
fn the_binary_prints_one_parseable_object_for_list_json() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_flat3d-inject"))
        .args(["list", "--json"])
        .output()
        .unwrap();
    let v = one_json_line(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(v["command"], json!("list"));
    assert_eq!(v["ok"], json!(out.status.success()));
}
