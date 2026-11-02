//! Locked tests for E5e: the state directory, its paths, the request file and staleness.

use flat3d_inject::state::{
    is_stale, parse_request, request_path, request_text, run_record_path, runs_dir, snapshot_path,
    state_dir, Request, STALE_SECS,
};

#[test]
fn the_directory_is_under_the_runtime_dir() {
    assert_eq!(state_dir(Some("/run/user/1000"), 1000), "/run/user/1000/flat3d");
    assert_eq!(state_dir(Some("/run/user/1000/"), 1000), "/run/user/1000/flat3d");
}

#[test]
fn without_a_runtime_dir_it_falls_back_to_tmp() {
    assert_eq!(state_dir(None, 1000), "/tmp/flat3d-1000");
    assert_eq!(state_dir(Some(""), 42), "/tmp/flat3d-42");
}

#[test]
fn the_paths_inside_it() {
    let d = "/run/user/1000/flat3d";
    assert_eq!(snapshot_path(d), "/run/user/1000/flat3d/link.json");
    assert_eq!(request_path(d), "/run/user/1000/flat3d/request");
    assert_eq!(runs_dir(d), "/run/user/1000/flat3d/runs");
    assert_eq!(run_record_path(d, 4242), "/run/user/1000/flat3d/runs/run-4242.txt");
}

#[test]
fn requests_parse_and_print() {
    assert_eq!(parse_request("start-3d\n"), Some(Request::Start3d));
    assert_eq!(parse_request("  stop-3d  "), Some(Request::Stop3d));
    assert_eq!(parse_request("go"), None);
    assert_eq!(parse_request(""), None);
    assert_eq!(request_text(Request::Start3d), "start-3d");
    assert_eq!(request_text(Request::Stop3d), "stop-3d");
    assert_eq!(parse_request(request_text(Request::Stop3d)), Some(Request::Stop3d));
}

#[test]
fn a_snapshot_older_than_ten_seconds_is_stale() {
    assert_eq!(STALE_SECS, 10);
    assert!(!is_stale(100, 110));
    assert!(is_stale(100, 111));
    assert!(!is_stale(200, 100));
}
