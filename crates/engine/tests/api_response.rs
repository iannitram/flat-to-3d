//! Locked tests for E1e: `api`'s error and response envelope, the one JSON object every
//! `--json` command prints (docs/code-map.md, "API shape"). Keys here are the contract.

use flat3d_inject::api::{ApiError, Game, LinkReport, LinkState, Response, ERROR_CODES};
use flat3d_inject::NotBuilt;
use serde_json::{from_value, json, to_value};

#[test]
fn the_error_codes_are_exactly_these() {
    let codes = "usage not-built not-found bad-recipe refused link-not-ready command-failed io";
    assert_eq!(ERROR_CODES.join(" "), codes);
}

#[test]
fn api_error_new_and_its_keys() {
    let e = ApiError::new("refused", "stereo-safe is not yes");
    assert_eq!(e.code, "refused");
    assert_eq!(e.message, "stereo-safe is not yes");
    assert_eq!(
        to_value(&e).unwrap(),
        json!({"code": "refused", "message": "stereo-safe is not yes"})
    );
}

#[test]
fn not_built_becomes_the_not_built_code() {
    let e = ApiError::from(NotBuilt("link::start_3d"));
    assert_eq!(
        e,
        ApiError::new("not-built", "not built yet: link::start_3d")
    );
}

#[test]
fn a_success_response_has_ok_command_and_data_and_no_error_key() {
    let resp = Response::success(
        "start-3d",
        LinkReport {
            state: LinkState::Ready,
            message: "m".into(),
        },
    );
    let v = json!({"ok": true, "command": "start-3d", "data": {"state": "ready", "message": "m"}});
    assert_eq!(to_value(&resp).unwrap(), v);
    assert_eq!(from_value::<Response<LinkReport>>(v).unwrap(), resp);
}

#[test]
fn a_failure_response_has_ok_command_and_error_and_no_data_key() {
    let resp: Response<Vec<Game>> = Response::failure("list", ApiError::new("io", "no Steam"));
    let v = json!({"ok": false, "command": "list", "error": {"code": "io", "message": "no Steam"}});
    assert_eq!(to_value(&resp).unwrap(), v);
    assert_eq!(from_value::<Response<Vec<Game>>>(v).unwrap(), resp);
}

#[test]
fn a_list_response_carries_an_array_of_games() {
    let resp = Response::success("list", Vec::<Game>::new());
    assert_eq!(
        to_value(&resp).unwrap(),
        json!({"ok": true, "command": "list", "data": []})
    );
}

#[test]
fn the_code_map_says_api_rs_is_present() {
    let map = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/code-map.md");
    let text = std::fs::read_to_string(map).unwrap();
    assert!(text.contains("| `crates/engine/src/api.rs` | present |"));
}
