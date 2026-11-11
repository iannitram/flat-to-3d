use flat3d_app::{
    engine, games_page, headset_page, logs_page, setup_page, update, App, Effect, Message, Tab,
};
use flat3d_inject::api::{ApiError, LinkSnapshot, LinkState};
use flat3d_inject::cli;
use flat3d_inject::rows::GameRow;
use flat3d_inject::setup::Packages;
use flat3d_inject::state::Request;
use serde_json::json;
use std::path::Path;

#[test]
fn selecting_games_reads_snapshot_and_rows() {
    let mut app = App::default();
    assert_eq!(
        update(&mut app, Message::Select(Tab::Games)),
        vec![
            Effect::ReadSnapshot,
            Effect::LoadRows {
                filter: "all".into()
            }
        ]
    );
}

#[test]
fn setup_packages_refresh_every_tenth_tick() {
    let mut app = App {
        active: Tab::Setup,
        ..App::default()
    };
    for n in 1..10 {
        assert_eq!(
            update(&mut app, Message::Tick(n)),
            vec![Effect::ReadSnapshot]
        );
    }
    assert_eq!(app.ticks, 9);
    assert_eq!(
        update(&mut app, Message::Tick(10)),
        vec![Effect::ReadSnapshot, Effect::ReadPackages]
    );
}

#[test]
fn logs_reload_on_tick() {
    let mut app = App {
        active: Tab::Logs,
        ..App::default()
    };
    assert_eq!(
        update(&mut app, Message::Tick(4)),
        vec![Effect::ReadSnapshot, Effect::LoadLogs]
    );
    assert_eq!(app.now, 4);
}

#[test]
fn headset_start_and_done_route() {
    let mut app = App::default();
    assert_eq!(
        update(&mut app, Message::Headset(headset_page::Msg::StartPressed)),
        vec![Effect::Request(Request::Start3d)]
    );
    assert!(update(&mut app, Message::Headset(headset_page::Msg::StartPressed)).is_empty());
    assert_eq!(
        update(&mut app, Message::Headset(headset_page::Msg::Done(Ok(())))),
        vec![Effect::ReadSnapshot]
    );
}

#[test]
fn headset_settings_route() {
    let mut app = App::default();
    assert_eq!(
        update(
            &mut app,
            Message::Headset(headset_page::Msg::SetTunnel(false))
        ),
        vec![Effect::SetSetting {
            key: "tunnel",
            on: false
        }]
    );
}

#[test]
fn games_undo_routes() {
    let mut app = App::default();
    assert_eq!(
        update(&mut app, Message::Games(games_page::Msg::UndoPressed(7))),
        vec![Effect::Undo(7)]
    );
}

#[test]
fn failed_switch_save_sets_error() {
    let mut app = App::default();
    assert!(update(&mut app, Message::SwitchSaved(Err("x".into()))).is_empty());
    assert_eq!(app.games.error.as_deref(), Some("x"));
}

#[test]
fn successful_switch_reloads_current_filter() {
    let mut app = App::default();
    update(
        &mut app,
        Message::Games(games_page::Msg::Filter("works-now".into())),
    );
    assert_eq!(
        update(&mut app, Message::SwitchSaved(Ok(()))),
        vec![Effect::LoadRows {
            filter: "works-now".into()
        }]
    );
}

#[test]
fn copy_joins_visible_logs() {
    let mut app = App::default();
    app.logs.lines = vec!["a".into(), "b".into()];
    assert_eq!(
        update(&mut app, Message::Logs(logs_page::Msg::CopyPressed)),
        vec![Effect::Copy("a\nb".into())]
    );
}

#[test]
fn setup_run_is_busy_and_results_live_in_page_state() {
    let mut app = App::default();
    assert_eq!(
        update(
            &mut app,
            Message::Setup(setup_page::Msg::RunPressed(setup_page::SetupKind::Doctor))
        ),
        vec![Effect::RunSetup(setup_page::SetupKind::Doctor)]
    );
    assert!(update(
        &mut app,
        Message::Setup(setup_page::Msg::RunPressed(setup_page::SetupKind::Packages))
    )
    .is_empty());
    assert_eq!(
        update(&mut app, Message::Setup(setup_page::Msg::Done(Ok(())))),
        vec![Effect::ReadPackages]
    );
    assert!(!app.setup.busy);
    assert_eq!(
        update(
            &mut app,
            Message::Setup(setup_page::Msg::RunPressed(setup_page::SetupKind::Packages))
        ),
        vec![Effect::RunSetup(setup_page::SetupKind::Packages)]
    );
    update(
        &mut app,
        Message::Setup(setup_page::Msg::Packages(Ok(Packages {
            wivrn: true,
            gnirehtet: false,
        }))),
    );
    assert_eq!(
        app.setup.packages,
        Some(Packages {
            wivrn: true,
            gnirehtet: false
        })
    );
}

#[test]
fn page_results_update_their_page_state() {
    let mut app = App::default();
    update(
        &mut app,
        Message::Logs(logs_page::Msg::Lines(vec!["x".into()])),
    );
    assert_eq!(app.logs.lines, ["x"]);
    update(&mut app, Message::Games(games_page::Msg::Rows(Ok(vec![]))));
    assert!(app.games.rows.is_empty());
}

#[test]
fn cli_path_prefers_sibling_then_src_then_bin() {
    let paths = [
        Path::new("/app/flat3d-inject"),
        Path::new("/src/bin/flat3d-inject"),
        Path::new("bin/flat3d-inject"),
    ];
    let exists = |p: &Path| p == paths[0];
    assert_eq!(
        engine::cli_path(
            Path::new("/app/flat3d-app"),
            Some("/src"),
            paths[2],
            &exists
        ),
        Some(paths[0].into())
    );
    let exists = |p: &Path| p == paths[1];
    assert_eq!(
        engine::cli_path(
            Path::new("/app/flat3d-app"),
            Some("/src"),
            paths[2],
            &exists
        ),
        Some(paths[1].into())
    );
    assert_eq!(
        engine::cli_path(Path::new("/app/flat3d-app"), None, paths[2], &|_| false),
        None
    );
}

#[test]
fn snapshot_absence_staleness_and_error_are_normalized() {
    assert_eq!(
        engine::snapshot_or_stopped(Ok(None), 5),
        Ok(LinkSnapshot::stopped(5))
    );
    let mut old = LinkSnapshot::stopped(0);
    old.state = LinkState::Ready;
    assert_eq!(
        engine::snapshot_or_stopped(Ok(Some(old)), 100)
            .unwrap()
            .state,
        LinkState::Stopped
    );
    assert_eq!(
        engine::snapshot_or_stopped(Err("e".into()), 1),
        Err("e".into())
    );
}

#[test]
fn envelope_parsing_preserves_data_and_error_codes() {
    assert_eq!(
        engine::parse_envelope(r#"{"ok":true,"data":{"a":1}}"#).unwrap(),
        json!({"a":1})
    );
    assert_eq!(
        engine::parse_envelope(r#"{"ok":false,"error":{"code":"io","message":"boom"}}"#),
        Err(ApiError::new("io", "boom"))
    );
    assert_eq!(
        engine::parse_envelope("not json").unwrap_err().code,
        "command-failed"
    );
    assert!(
        engine::error_text(&ApiError::new("usage", "bad argv")).starts_with("app/CLI mismatch: ")
    );
}

#[test]
fn game_row_is_built_field_by_field() {
    let row = GameRow {
        appid: 1,
        name: "game".into(),
        status: "works-now".into(),
        status_label: "Works".into(),
        status_colour: "#fff".into(),
        switch_on: true,
        plays_3d: true,
        flat_reason: None,
    };
    let value = serde_json::to_value(&row).unwrap();
    assert_eq!(engine::row_from_json(&value), Some(row));
    let mut missing = value;
    missing.as_object_mut().unwrap().remove("name");
    assert!(engine::row_from_json(&missing).is_none());
}

#[test]
fn effect_argv_matches_the_registered_cli_grammar() {
    let effects = vec![
        Effect::ReadSnapshot,
        Effect::Request(Request::Start3d),
        Effect::LoadLogs,
        Effect::Copy("x".into()),
        Effect::SetSetting {
            key: "auto-3d",
            on: true,
        },
        Effect::SetSetting {
            key: "auto-3d",
            on: false,
        },
        Effect::SetSetting {
            key: "tunnel",
            on: true,
        },
        Effect::SetSetting {
            key: "tunnel",
            on: false,
        },
        Effect::LoadRows {
            filter: "all".into(),
        },
        Effect::SetSwitch { appid: 7, on: true },
        Effect::SetSwitch {
            appid: 7,
            on: false,
        },
        Effect::Undo(7),
        Effect::ReadPackages,
        Effect::RunSetup(setup_page::SetupKind::Packages),
        Effect::RunSetup(setup_page::SetupKind::InstallApps),
        Effect::RunSetup(setup_page::SetupKind::Doctor),
    ];
    for effect in effects {
        let Some(args) = engine::effect_args(&effect) else {
            assert!(matches!(
                effect,
                Effect::ReadSnapshot | Effect::Request(_) | Effect::LoadLogs | Effect::Copy(_)
            ));
            continue;
        };
        assert!(!args.iter().any(|arg| arg == "--json"));
        let mut parse_args = args.clone();
        parse_args.push("--json".into());
        let parsed = cli::parse_any(&parse_args).unwrap();
        assert!(parsed.json);
        assert!(
            cli::COMMANDS
                .iter()
                .any(|row| row.id == parsed.command.id()),
            "{}",
            parsed.command.id()
        );
    }
}

/// Names of forbidden `std` modules imported through a `std::{..}` group, including nested
/// items such as `std::{fs::File, io}` or `std::{io, thread::{self, spawn}}`.
fn forbidden_std_group_members(source: &str) -> Vec<String> {
    const FORBIDDEN: [&str; 4] = ["fs", "process", "net", "thread"];
    let compact: String = source.chars().filter(|c| !c.is_whitespace()).collect();
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = compact[from..].find("std::{") {
        let start = from + at + "std::{".len();
        let mut depth = 1;
        let mut end = compact.len();
        for (i, c) in compact[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let body = &compact[start..end];
        let mut depth = 0;
        let mut item_start = 0;
        for (i, c) in body
            .char_indices()
            .chain(std::iter::once((body.len(), ',')))
        {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                ',' if depth == 0 => {
                    let head: String = body[item_start..i]
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if FORBIDDEN.contains(&head.as_str()) {
                        found.push(format!("std::{head}"));
                    }
                    item_start = i + 1;
                }
                _ => {}
            }
        }
        from = start;
    }
    found
}

#[test]
fn std_group_scanner_sees_grouped_and_nested_imports() {
    assert_eq!(
        forbidden_std_group_members("use std::{fs, thread};"),
        ["std::fs", "std::thread"]
    );
    assert_eq!(
        forbidden_std_group_members("use std::{\n    io::{self, Read},\n    process::Command,\n};"),
        ["std::process"]
    );
    assert_eq!(
        forbidden_std_group_members("use std::{collections::{BTreeMap}, fs::File};"),
        ["std::fs"]
    );
    assert!(forbidden_std_group_members("use std::{path::Path, io};").is_empty());
}

#[test]
fn app_sources_keep_io_in_engine_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs")
            || path.file_name().is_some_and(|n| n == "engine.rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for forbidden in [
            "std::fs",
            "std::process",
            "std::net",
            "std::thread",
            "Command::new",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} uses {forbidden}; return an Action; engine.rs does the I/O",
                path.display()
            );
        }
        let grouped = forbidden_std_group_members(&source);
        assert!(
            grouped.is_empty(),
            "{} uses {} in a use group; return an Action; engine.rs does the I/O",
            path.display(),
            grouped.join(", ")
        );
    }
}

#[test]
fn stand_in_cli_adds_json_and_parses_exit_envelopes() {
    use std::os::unix::fs::PermissionsExt;
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("app_wiring");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("stub.sh");
    let write = |body: &str| {
        std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();
    };
    write("printf '{\"ok\":true,\"command\":\"x\",\"data\":{\"argv\":\"%s\"}}' \"$*\"");
    assert_eq!(
        engine::run_cli_at(&script, &["rows".into(), "--filter".into(), "all".into()]).unwrap()
            ["argv"],
        "rows --filter all --json"
    );
    write("printf '{\"ok\":false,\"error\":{\"code\":\"usage\",\"message\":\"bad\"}}'; exit 2");
    assert_eq!(engine::run_cli_at(&script, &[]).unwrap_err().code, "usage");
    write("printf '{\"ok\":true,\"data\":{\"ok\":false,\"checks\":[{\"name\":\"adb\",\"ok\":false,\"fix\":\"install\"}]}}'; exit 1");
    let error = engine::run_cli_at(&script, &[]).unwrap_err();
    assert_eq!(error.code, "command-failed");
    assert!(error.message.contains("adb: install"));
    write("printf 'not json'");
    assert_eq!(
        engine::run_cli_at(&script, &[]).unwrap_err().code,
        "command-failed"
    );
}

#[test]
fn all_page_views_build_for_named_states() {
    let mut fresh = App::default();
    let mut ready = LinkSnapshot::stopped(10);
    ready.state = LinkState::Ready;
    let link_ready = App {
        headset: headset_page::HeadsetState {
            snapshot: Some(ready.clone()),
            ..Default::default()
        },
        ..App::default()
    };
    let busy = App {
        headset: headset_page::HeadsetState {
            busy: true,
            ..Default::default()
        },
        setup: setup_page::SetupState {
            busy: true,
            ..Default::default()
        },
        ..App::default()
    };
    let errors = App {
        headset: headset_page::HeadsetState {
            error: Some("headset".into()),
            ..Default::default()
        },
        games: games_page::GamesState {
            error: Some("games".into()),
            ..Default::default()
        },
        setup: setup_page::SetupState {
            error: Some("setup".into()),
            ..Default::default()
        },
        logs: logs_page::LogsState {
            error: Some("logs".into()),
            ..Default::default()
        },
        ..App::default()
    };
    let games_rows = App {
        games: games_page::GamesState {
            rows: vec![
                GameRow {
                    appid: 1,
                    name: "one".into(),
                    status: "works-now".into(),
                    status_label: "Works".into(),
                    status_colour: "#ffffff".into(),
                    switch_on: true,
                    plays_3d: true,
                    flat_reason: None,
                },
                GameRow {
                    appid: 2,
                    name: "two".into(),
                    status: "not-safe".into(),
                    status_label: "Unsafe".into(),
                    status_colour: "#ff0000".into(),
                    switch_on: false,
                    plays_3d: false,
                    flat_reason: Some("disabled".into()),
                },
            ],
            filter: "all".into(),
            error: None,
            note: Some("undo".into()),
        },
        ..App::default()
    };
    let setup_done = App {
        setup: setup_page::SetupState {
            packages: Some(Packages {
                wivrn: true,
                gnirehtet: true,
            }),
            ..Default::default()
        },
        headset: link_ready.headset.clone(),
        ..App::default()
    };
    let logs_errors = App {
        logs: logs_page::LogsState {
            filter: logs_page::Filter::Errors,
            lines: vec!["error: failed".into()],
            ..Default::default()
        },
        ..App::default()
    };
    let states = [
        fresh.clone(),
        link_ready,
        busy,
        errors,
        games_rows,
        setup_done,
        logs_errors,
    ];
    for app in &states {
        let _ = flat3d_app::view_headset::view(&app.headset, app.now);
        let _ = flat3d_app::view_games::view(&app.games);
        let _ = flat3d_app::view_setup::view(&app.setup, app.headset.snapshot.as_ref());
        let _ = flat3d_app::view_logs::view(&app.logs);
    }
    // Ensure the fresh state is exactly the default used by the window.
    fresh.now = 0;
    assert_eq!(fresh.active, Tab::Setup);
}
