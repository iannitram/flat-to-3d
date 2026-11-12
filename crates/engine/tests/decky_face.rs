use flat3d_inject::{
    api,
    cli::{self, AnyCommand},
};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::Path};

fn engine_source() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decky/src/engine.ts"))
        .unwrap()
}

fn type_source() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decky/src/types.ts"))
        .unwrap()
}

fn sample_argv() -> Vec<(&'static str, Vec<&'static str>, Vec<&'static str>)> {
    vec![
        ("view", vec!["view"], vec!["view"]),
        ("rows", vec!["rows"], vec!["rows"]),
        ("start3d", vec!["start-3d"], vec!["start-3d"]),
        ("stop3d", vec!["stop-3d"], vec!["stop-3d"]),
        (
            "setSwitch",
            vec!["switch", "set", "5", "on"],
            vec!["switch", "set", "on"],
        ),
        (
            "setSetting",
            vec!["setting", "set", "auto-3d", "off"],
            vec!["setting", "set", "on"],
        ),
        (
            "launch",
            vec!["launch", "add", "--json", "--options", "%command% -dx11"],
            vec!["launch", "--json", "--options"],
        ),
        (
            "launch",
            vec!["launch", "remove", "--json", "--options", "%command% -dx11"],
            vec!["launch", "--json", "--options"],
        ),
        ("doctor", vec!["doctor"], vec!["doctor"]),
        ("collection", vec!["collection"], vec!["collection"]),
        (
            "collection",
            vec!["collection", "--current", "1,2"],
            vec!["collection", "--current"],
        ),
        (
            "collection",
            vec!["collection", "--current", ""],
            vec!["collection", "--current"],
        ),
        ("toasts", vec!["toasts", "7"], vec!["toasts"]),
        (
            "logs",
            vec!["logs", "--limit", "500"],
            vec!["logs", "--limit"],
        ),
    ]
}

#[test]
fn every_decky_wrapper_is_listed_and_dispatches_with_main_json() {
    let source = engine_source();
    let declared: BTreeSet<String> = source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("export const ")?;
            let (name, rhs) = rest.split_once('=')?;
            rhs.contains("engine<").then(|| name.trim().to_string())
        })
        .collect();
    let table: BTreeSet<String> = sample_argv()
        .iter()
        .map(|(name, _, _)| (*name).into())
        .collect();
    assert_eq!(declared, table);
    for (name, argv, literals) in sample_argv() {
        let mut command: Vec<String> = argv.iter().map(|s| (*s).into()).collect();
        command.push("--json".into());
        let parsed = cli::parse_any(&command).unwrap_or_else(|error| panic!("{name}: {error:?}"));
        assert!(matches!(
            parsed.command,
            AnyCommand::View(_)
                | AnyCommand::Link(_)
                | AnyCommand::Engine(_)
                | AnyCommand::Doctor(_)
        ));
        assert!(
            cli::COMMANDS
                .iter()
                .any(|spec| spec.id == parsed.command.id()),
            "{name}: id {:?} is not a COMMANDS row",
            parsed.command.id()
        );
        let start = source
            .find(&format!("export const {name} "))
            .unwrap_or_else(|| panic!("{name} not exported"));
        let after = start + "export const ".len();
        let end = source[after..]
            .find("export")
            .map_or(source.len(), |offset| after + offset);
        let line = &source[start..end];
        let mut at = 0;
        for literal in literals {
            let quoted = format!("\"{literal}\"");
            let found = line[at..]
                .find(&quoted)
                .unwrap_or_else(|| panic!("{name} missing {quoted}"));
            at += found + quoted.len();
        }
    }
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

#[test]
fn decky_error_codes_and_type_mirrors_match_rust() {
    let engine = engine_source();
    let type_source = type_source();
    let error_line = engine
        .lines()
        .find(|line| line.starts_with("export type ErrorCode"))
        .unwrap();
    let codes: BTreeSet<String> = error_line
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect();
    assert_eq!(
        codes,
        api::ERROR_CODES.iter().map(|s| (*s).to_string()).collect()
    );

    let event = api::LinkEvent {
        seq: 1,
        kind: api::EventKind::Ready,
        text: "ready".into(),
    };
    let health = api::Health {
        battery: Some(90),
        status: "warm".into(),
        temp_c: Some(35.0),
        zone: api::Zone::Green,
    };
    let values = [
        ("LinkEvent", json!(event)),
        (
            "HealthView",
            serde_json::to_value(flat3d_inject::face::view::health_view(Some(&health))).unwrap(),
        ),
        (
            "LinkView",
            serde_json::to_value(flat3d_inject::face::link_model::link_view(None, 0)).unwrap(),
        ),
        (
            "GameRow",
            json!(flat3d_inject::face::rows::GameRow {
                appid: 1,
                name: "Game".into(),
                status: "works-now".into(),
                status_label: "Works now".into(),
                status_colour: "#2e9e5b".into(),
                switch_on: true,
                plays_3d: true,
                flat_reason: None,
            }),
        ),
        (
            "LaunchText",
            json!(api::LaunchText {
                options: "x".into()
            }),
        ),
        (
            "CollectionPlan",
            json!(api::CollectionPlan {
                name: "x".into(),
                wanted: vec![],
                add: vec![],
                remove: vec![]
            }),
        ),
        (
            "ToastBatch",
            serde_json::to_value(flat3d_inject::face::toasts::ToastBatch {
                toasts: vec![event],
                last_seq: 1,
            })
            .unwrap(),
        ),
        (
            "LogLines",
            json!(api::LogLines {
                lines: vec![],
                text: "".into()
            }),
        ),
        (
            "LinkReport",
            json!(api::LinkReport {
                state: api::LinkState::Ready,
                message: "ready".into()
            }),
        ),
        (
            "SwitchState",
            json!(api::SwitchState { appid: 1, on: true }),
        ),
        (
            "SettingState",
            json!(api::SettingState {
                key: "auto-3d".into(),
                on: true
            }),
        ),
        (
            "DoctorCheck",
            json!(api::DoctorCheck {
                name: "x".into(),
                ok: true,
                detail: "ok".into(),
                fix: "".into()
            }),
        ),
        (
            "DoctorReport",
            json!(api::DoctorReport {
                ok: true,
                checks: vec![]
            }),
        ),
    ];
    for (name, sample) in values {
        let prefix = format!("export interface {name} {{");
        let declaration = type_source
            .split_once(&prefix)
            .unwrap_or_else(|| panic!("missing {name}"))
            .1
            .split('}')
            .next()
            .unwrap();
        let mirror: BTreeSet<String> = declaration
            .split(';')
            .filter_map(|field| {
                field
                    .trim()
                    .split_once(':')
                    .map(|(key, _)| key.trim().trim_end_matches('?').to_string())
            })
            .collect();
        let rust_keys = keys(&sample);
        let mut with_optional_title = rust_keys.clone();
        with_optional_title.insert("title".into());
        assert!(
            mirror == rust_keys || (name == "ToastBatch" && mirror == with_optional_title),
            "{name}: mirror={mirror:?}, Rust={rust_keys:?}"
        );
    }
}
