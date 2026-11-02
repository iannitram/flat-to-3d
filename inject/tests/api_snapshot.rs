//! Locked tests for E5f: the snapshot types and their exact JSON (docs/code-map.md, API shape).

use flat3d_inject::api::*;
use serde_json::json;

fn full() -> LinkSnapshot {
    LinkSnapshot {
        state: LinkState::Ready,
        message: "ok".into(),
        tether: TetherMode::QuestTether,
        tunnel: true,
        auto_3d: true,
        adb: "/usr/bin/adb".into(),
        adb_source: AdbSource::Running,
        health: Some(Health { battery: Some(80), status: "Discharging".into(), temp_c: Some(36.5), zone: Zone::Green }),
        stats: Some(Stats { rx_bytes_s: 1000, tx_bytes_s: 20, uptime_s: 61, reconnects: 2 }),
        events: vec![LinkEvent { seq: 4, kind: EventKind::LatePlug, text: "t".into() }],
        updated: 77,
    }
}

#[test]
fn a_snapshot_has_exactly_these_keys() {
    let want = json!({
        "state": "ready", "message": "ok", "tether": "quest-tether", "tunnel": true, "auto_3d": true,
        "adb": "/usr/bin/adb", "adb_source": "running",
        "health": {"battery": 80, "status": "Discharging", "temp_c": 36.5, "zone": "green"},
        "stats": {"rx_bytes_s": 1000, "tx_bytes_s": 20, "uptime_s": 61, "reconnects": 2},
        "events": [{"seq": 4, "kind": "late-plug", "text": "t"}],
        "updated": 77
    });
    assert_eq!(serde_json::to_value(full()).unwrap(), want);
}

#[test]
fn a_snapshot_reads_back_equal() {
    let text = serde_json::to_string(&full()).unwrap();
    assert_eq!(serde_json::from_str::<LinkSnapshot>(&text).unwrap(), full());
}

#[test]
fn the_enums_use_these_strings() {
    let kinds = [
        (EventKind::Ready, "ready"), (EventKind::LatePlug, "late-plug"),
        (EventKind::GraceEnded, "grace-ended"), (EventKind::AllowDebugging, "allow-debugging"),
        (EventKind::Warm, "warm"), (EventKind::Hot, "hot"), (EventKind::TetherError, "tether-error"),
    ];
    for (k, s) in kinds {
        assert_eq!(serde_json::to_value(k).unwrap(), json!(s));
    }
    assert_eq!(serde_json::to_value(Zone::Unknown).unwrap(), json!("unknown"));
    assert_eq!(serde_json::to_value(TetherMode::Off).unwrap(), json!("off"));
    assert_eq!(serde_json::to_value(AdbSource::Bundled).unwrap(), json!("bundled"));
}

#[test]
fn a_bad_enum_string_does_not_read() {
    assert!(serde_json::from_value::<Zone>(json!("orange")).is_err());
    assert!(serde_json::from_value::<TetherMode>(json!("Own")).is_err());
}

#[test]
fn the_small_reports_have_these_keys() {
    assert_eq!(serde_json::to_value(SwitchState { appid: 2075800, on: false }).unwrap(), json!({"appid": 2075800, "on": false}));
    assert_eq!(serde_json::to_value(SettingState { key: "tunnel".into(), on: true }).unwrap(), json!({"key": "tunnel", "on": true}));
    let r = InstallReport { written: vec!["a".into()], ran: vec!["b".into()], summary: "s".into() };
    assert_eq!(serde_json::to_value(r).unwrap(), json!({"written": ["a"], "ran": ["b"], "summary": "s"}));
}

#[test]
fn stopped_is_the_documented_snapshot() {
    let want = json!({
        "state": "stopped", "message": "Link service is not running.", "tether": "off", "tunnel": false,
        "auto_3d": false, "adb": "", "adb_source": "missing", "health": null, "stats": null,
        "events": [], "updated": 5
    });
    assert_eq!(serde_json::to_value(LinkSnapshot::stopped(5)).unwrap(), want);
}

#[test]
fn a_stale_snapshot_counts_as_stopped_and_a_fresh_one_is_kept() {
    assert_eq!(full().effective(87), full());
    assert_eq!(full().effective(88), LinkSnapshot::stopped(88));
}
