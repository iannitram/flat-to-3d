#[path = "support/scenes.rs"]
mod scenes;

use flat3d_inject::{
    api::{AdbSource, EventKind, LinkSnapshot, LinkState, Stats, TetherMode},
    cli::{self, AnyCommand, Ctx, ViewCommand},
    face::link_model,
    testing::FakeRunner,
};
use serde_json::{json, Value};

fn words(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_string()).collect()
}

fn view(args: &[&str]) -> ViewCommand {
    match cli::parse_any(&words(args)).unwrap().command {
        AnyCommand::View(command) => command,
        other => panic!("expected view command, got {other:?}"),
    }
}

#[test]
fn parses_all_view_forms_and_json_positions() {
    assert_eq!(view(&["view"]), ViewCommand::View);
    assert!(cli::parse_any(&words(&["view", "--json"])).unwrap().json);
    assert_eq!(
        view(&["rows"]),
        ViewCommand::Rows {
            filter: "all".into()
        }
    );
    assert_eq!(
        view(&["rows", "--filter", "works-now"]),
        ViewCommand::Rows {
            filter: "works-now".into()
        }
    );
    assert_eq!(
        view(&["launch", "add", "--options", ""]),
        ViewCommand::Launch {
            add: true,
            options: String::new()
        }
    );
    assert_eq!(
        view(&["launch", "add", "--json", "--options", "%command% -dx11"]),
        ViewCommand::Launch {
            add: true,
            options: "%command% -dx11".into()
        }
    );
    assert!(
        cli::parse_any(&words(&["launch", "add", "--options", "x", "--json"]))
            .unwrap()
            .json
    );
    assert_eq!(
        view(&["launch", "add", "--options", "FOO=1 %command% -- -dx11"]),
        ViewCommand::Launch {
            add: true,
            options: "FOO=1 %command% -- -dx11".into()
        }
    );
    assert_eq!(
        view(&["launch", "add", "--options", "-- %command%"]),
        ViewCommand::Launch {
            add: true,
            options: "-- %command%".into()
        }
    );
    assert_eq!(
        view(&["launch", "remove", "--options", "flat3d-run %command%"]),
        ViewCommand::Launch {
            add: false,
            options: "flat3d-run %command%".into()
        }
    );
    assert_eq!(
        view(&["collection"]),
        ViewCommand::Collection { current: None }
    );
    assert_eq!(
        view(&["collection", "--current", "1,2"]),
        ViewCommand::Collection {
            current: Some(vec![1, 2])
        }
    );
    assert_eq!(
        view(&["collection", "--current", ""]),
        ViewCommand::Collection {
            current: Some(vec![])
        }
    );
    assert_eq!(view(&["toasts", "7"]), ViewCommand::Toasts { last_seq: 7 });
    assert_eq!(view(&["logs"]), ViewCommand::Logs { limit: 500 });
    assert_eq!(
        view(&["logs", "--limit", "3"]),
        ViewCommand::Logs { limit: 3 }
    );
}

#[test]
fn bad_view_args_are_usage_errors() {
    for args in [
        vec!["rows", "--filter"],
        vec!["launch"],
        vec!["launch", "add"],
        vec!["launch", "add", "--options"],
        vec!["launch", "frob", "--options", "x"],
        vec!["launch", "add", "--options", "a", "b"],
        vec!["collection", "--current", "a"],
        vec!["toasts"],
        vec!["toasts", "-1"],
        vec!["logs", "--limit", "0"],
    ] {
        let error = cli::parse_any(&words(&args)).unwrap_err();
        assert_eq!(error.code, "usage", "argv: {args:?}");
        assert!(error.message.starts_with("usage: flat3d-inject"));
    }
}

fn expected_stopped() -> Value {
    json!({"state":"stopped","label":"Stopped","hint":"Press Start 3D to bring the headset link up.","start":true,"stop":false,"health":{"battery":"unknown","temp":"unknown","zone":"unknown","colour":"#b0b0b0"},"rx":"—","tx":"—","uptime":"—","tether":"off","tunnel":false,"auto_3d":false,"adb_source":"missing"})
}

#[test]
fn link_model_handles_absent_stale_and_fresh_snapshots() {
    let now = 100;
    assert_eq!(
        serde_json::to_value(link_model::link_view(None, now)).unwrap(),
        expected_stopped()
    );
    let mut stale = LinkSnapshot::stopped(now - 11);
    stale.state = LinkState::Ready;
    assert_eq!(
        serde_json::to_value(link_model::link_view(Some(stale), now)).unwrap(),
        expected_stopped()
    );
    let mut fresh = LinkSnapshot::stopped(now);
    fresh.state = LinkState::Ready;
    fresh.tether = TetherMode::Own;
    fresh.tunnel = true;
    fresh.auto_3d = true;
    fresh.adb_source = AdbSource::Bundled;
    fresh.stats = Some(Stats {
        rx_bytes_s: 1536,
        tx_bytes_s: 512,
        uptime_s: 185,
        reconnects: 0,
    });
    assert_eq!(
        serde_json::to_value(link_model::link_view(Some(fresh), now)).unwrap(),
        json!({"state":"ready","label":"Ready","hint":"Press Play on a 3D Ready game.","start":false,"stop":true,"health":{"battery":"unknown","temp":"unknown","zone":"unknown","colour":"#b0b0b0"},"rx":"1.5 KB/s","tx":"512 B/s","uptime":"3m 5s","tether":"own","tunnel":true,"auto_3d":true,"adb_source":"bundled"})
    );
}

fn with_ctx<T>(
    case: &str,
    f: impl FnOnce(&flat3d_inject::paths::EnginePaths, &mut Ctx<'_>) -> T,
) -> T {
    let paths = scenes::no_steam(case);
    let mut runner = FakeRunner::new();
    let mut relay = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &paths,
        runner: &mut runner,
        relay: &mut relay,
        json: true,
    };
    f(&paths, &mut ctx)
}

#[test]
fn view_commands_return_expected_values() {
    with_ctx("commands", |p, ctx| {
        assert_eq!(
            cli::view::run(&ViewCommand::View, ctx).unwrap(),
            expected_stopped()
        );
        let launch = cli::view::run(
            &ViewCommand::Launch {
                add: true,
                options: String::new(),
            },
            ctx,
        )
        .unwrap();
        assert_eq!(
            launch["options"],
            format!("{} %command%", p.wrapper_bin().display())
        );
        for original in ["", "-- %command%", "FOO=1 %command% -- -dx11"] {
            let added = flat3d_inject::launch_options::add_wrapper_at(
                original,
                &p.wrapper_bin().to_string_lossy(),
            );
            let removed = cli::view::run(
                &ViewCommand::Launch {
                    add: false,
                    options: added,
                },
                ctx,
            )
            .unwrap();
            assert_eq!(removed["options"], original);
        }
        let added = flat3d_inject::launch_options::add_wrapper_at(
            "%command% -dx11",
            &p.wrapper_bin().to_string_lossy(),
        );
        assert_eq!(
            cli::view::run(
                &ViewCommand::Launch {
                    add: false,
                    options: added
                },
                ctx
            )
            .unwrap()["options"],
            "-dx11"
        );
        let plan = cli::view::run(
            &ViewCommand::Collection {
                current: Some(vec![]),
            },
            ctx,
        )
        .unwrap();
        assert_eq!(
            plan,
            json!({"name":"3D Ready","wanted":[],"add":[],"remove":[]})
        );
        let plan = cli::view::run(
            &ViewCommand::Collection {
                current: Some(vec![1, 2]),
            },
            ctx,
        )
        .unwrap();
        assert_eq!(plan["remove"], json!([1, 2]));
        let batch = cli::view::run(&ViewCommand::Toasts { last_seq: 0 }, ctx).unwrap();
        assert_eq!(
            batch,
            json!({"toasts":[],"last_seq":0,"title":"Flat to 3D"})
        );
        let logs = cli::view::run(&ViewCommand::Logs { limit: 500 }, ctx).unwrap();
        assert_eq!(logs, json!({"lines":[],"text":""}));
    });
}

#[test]
fn toast_sequence_and_log_tail_are_projected() {
    with_ctx("tail", |p, ctx| {
        let mut snapshot = LinkSnapshot::stopped(flat3d_inject::sys::unix_now());
        snapshot.events = [1, 2, 3]
            .into_iter()
            .map(|seq| flat3d_inject::api::LinkEvent {
                seq,
                kind: EventKind::Ready,
                text: format!("event {seq}"),
            })
            .collect();
        scenes::write_snapshot(p, &snapshot);
        assert_eq!(
            cli::view::run(&ViewCommand::Toasts { last_seq: 1 }, ctx).unwrap(),
            json!({"toasts":[{"seq":2,"kind":"ready","text":"event 2"},{"seq":3,"kind":"ready","text":"event 3"}],"last_seq":3,"title":"Flat to 3D"})
        );
        std::fs::create_dir_all(p.log_dir.clone()).unwrap();
        std::fs::write(p.link_log(), "1 a\n2 b\n3 c\n4 d\n5 e\n").unwrap();
        assert_eq!(
            cli::view::run(&ViewCommand::Logs { limit: 3 }, ctx).unwrap(),
            json!({"lines":["5 link: e","4 link: d","3 link: c"],"text":"5 link: e\n4 link: d\n3 link: c"})
        );
    });
}

#[test]
fn view_registry_rows_match_parser_and_read_only_effects() {
    let samples: &[(&str, &[&str])] = &[
        ("view", &["view"]),
        ("rows", &["rows", "--filter", "works-now"]),
        ("launch", &["launch", "add", "--options", "x"]),
        ("collection", &["collection", "--current", "1,2"]),
        ("toasts", &["toasts", "7"]),
        ("logs", &["logs", "--limit", "3"]),
    ];
    // Row -> parser: every View-group row parses to its own variant.
    let mut row_ids = Vec::new();
    for spec in cli::COMMANDS.iter().filter(|spec| spec.group == "View") {
        let (_, sample) = samples
            .iter()
            .find(|(id, _)| *id == spec.id)
            .unwrap_or_else(|| panic!("no sample for row {}", spec.id));
        let command = view(sample);
        let matches = match spec.id {
            "view" => matches!(command, ViewCommand::View),
            "rows" => matches!(command, ViewCommand::Rows { .. }),
            "launch" => matches!(command, ViewCommand::Launch { .. }),
            "collection" => matches!(command, ViewCommand::Collection { .. }),
            "toasts" => matches!(command, ViewCommand::Toasts { .. }),
            "logs" => matches!(command, ViewCommand::Logs { .. }),
            other => panic!("unexpected view row {other}"),
        };
        assert!(matches, "{}: wrong variant {command:?}", spec.id);
        assert_eq!(command_id(sample), spec.id);
        row_ids.push(spec.id);
    }
    // Variant -> row: every ViewCommand variant has exactly one row, read-only.
    for id in samples.iter().map(|(id, _)| *id) {
        let rows: Vec<_> = cli::COMMANDS.iter().filter(|spec| spec.id == id).collect();
        assert_eq!(rows.len(), 1, "{id}");
        assert_eq!(rows[0].group, "View", "{id}");
        assert_eq!(rows[0].effects, &[cli::SideEffect::ReadOnly], "{id}");
        assert!(row_ids.contains(&id), "{id}");
    }
    assert_eq!(row_ids.len(), samples.len());
    // The bare forms parse too.
    for bare in [&["rows"][..], &["collection"][..], &["logs"][..]] {
        view(bare);
    }
}

fn command_id(args: &[&str]) -> &'static str {
    cli::parse_any(&words(args)).unwrap().command.id()
}

fn files(root: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(
        root: &std::path::Path,
        at: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn every_view_verb_is_read_only() {
    let p = scenes::two_libs_new_vdf("readonly");
    std::fs::create_dir_all(&p.config).unwrap();
    std::fs::write(p.settings(), "tunnel=on\n").unwrap();
    let mut snapshot = LinkSnapshot::stopped(flat3d_inject::sys::unix_now());
    snapshot.state = LinkState::Ready;
    scenes::write_snapshot(&p, &snapshot);
    let root = p.home.parent().unwrap();
    let before = files(root);
    let mut runner = FakeRunner::new();
    let mut relay = FakeRunner::new();
    {
        let mut ctx = Ctx {
            paths: &p,
            runner: &mut runner,
            relay: &mut relay,
            json: true,
        };
        for command in [
            ViewCommand::View,
            ViewCommand::Rows {
                filter: "all".into(),
            },
            ViewCommand::Launch {
                add: true,
                options: "%command%".into(),
            },
            ViewCommand::Launch {
                add: false,
                options: "flat3d-run %command%".into(),
            },
            ViewCommand::Collection {
                current: Some(vec![1]),
            },
            ViewCommand::Toasts { last_seq: 0 },
            ViewCommand::Logs { limit: 500 },
        ] {
            cli::view::run(&command, &mut ctx).unwrap();
        }
    }
    assert_eq!(before, files(root));
    assert!(runner.ran.is_empty());
    assert!(relay.ran.is_empty());
}
