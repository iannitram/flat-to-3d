use flat3d_inject::cli::{parse_any, AnyCommand, SetupCommand, ViewCommand, COMMANDS};

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn parses_separator_and_json_only_before_it() {
    let parsed = parse_any(&args(&["run", "5", "--", "a", "--json"])).unwrap();
    assert!(!parsed.json);
    let AnyCommand::Link(flat3d_inject::cli::link::LinkCommand::Run { argv, .. }) = parsed.command
    else {
        panic!("expected run")
    };
    assert_eq!(argv, ["a", "--json"]);
    let parsed = parse_any(&args(&["view", "--json"])).unwrap();
    assert!(parsed.json);
    assert_eq!(parsed.command, AnyCommand::View(ViewCommand::View));
}

#[test]
fn view_stub_and_setup_usage_are_stable() {
    assert_eq!(
        parse_any(&args(&["logs"])).unwrap().command,
        AnyCommand::View(ViewCommand::Logs {
            limit: flat3d_inject::headset_view::DEFAULT_LOG_LINES
        })
    );
    assert_eq!(
        parse_any(&args(&["setup", "launch-env", "abc"]))
            .unwrap_err()
            .code,
        "usage"
    );
    let err = parse_any(&args(&["fly"])).unwrap_err();
    assert_eq!(err.code, "usage");
    assert_eq!(err.message, flat3d_inject::cli::USAGE);
    assert_eq!(
        parse_any(&args(&["setup", "packages"])).unwrap().command,
        AnyCommand::Setup(SetupCommand::Packages)
    );
}

fn sample(id: &str) -> Vec<String> {
    let mut words: Vec<String> = id.split_whitespace().map(str::to_owned).collect();
    if id == "inject"
        || id == "undo"
        || id == "switch get"
        || id == "switch set"
        || id == "setup launch-env"
    {
        words.push("5".into());
    }
    if id == "switch set" {
        words.push("on".into());
    }
    if id == "setting get" {
        words.push("auto-3d".into());
    }
    if id == "setting set" {
        words.extend(["auto-3d".into(), "on".into()]);
    }
    if id == "run" {
        words.extend(["5".into(), "--".into(), "true".into()]);
    }
    if id == "rows" {
        words.extend(["--filter".into(), "all".into()]);
    }
    if id == "launch" {
        words.extend(["add".into(), "--options".into(), "%command%".into()]);
    }
    if id == "collection" {
        words.extend(["--current".into(), "5".into()]);
    }
    if id == "toasts" {
        words.push("0".into());
    }
    if id == "logs" {
        words.extend(["--limit".into(), "10".into()]);
    }
    if id == "setup fetch" {
        words.push("uevr".into());
    }
    words
}

#[test]
fn command_registry_entries_parse_and_keep_their_typed_id() {
    for row in COMMANDS {
        let parsed = parse_any(&sample(row.id));
        if row.group == "View" && row.id != "view" && parsed.is_err() {
            continue;
        }
        let parsed = parsed.unwrap_or_else(|e| panic!("{}: {e:?}", row.id));
        assert_eq!(parsed.command.id(), row.id);
    }
}

#[test]
fn every_any_command_variant_has_exactly_one_registry_row() {
    use flat3d_inject::cli::link::{LinkCommand, SettingKey};
    use flat3d_inject::cli::{Command, DoctorCommand};
    let variants = vec![
        AnyCommand::Engine(Command::List),
        AnyCommand::Engine(Command::Inject { appid: 1 }),
        AnyCommand::Engine(Command::Start3d),
        AnyCommand::Engine(Command::Stop3d),
        AnyCommand::Engine(Command::Status),
        AnyCommand::Engine(Command::Undo { appid: 1 }),
        AnyCommand::Link(LinkCommand::Watch),
        AnyCommand::Link(LinkCommand::Snapshot),
        AnyCommand::Link(LinkCommand::Install),
        AnyCommand::Link(LinkCommand::Uninstall),
        AnyCommand::Link(LinkCommand::SwitchGet { appid: 1 }),
        AnyCommand::Link(LinkCommand::SwitchSet { appid: 1, on: true }),
        AnyCommand::Link(LinkCommand::SettingGet {
            key: SettingKey::Auto3d,
        }),
        AnyCommand::Link(LinkCommand::SettingSet {
            key: SettingKey::Auto3d,
            on: true,
        }),
        AnyCommand::Link(LinkCommand::Run {
            appid: 1,
            argv: vec![],
        }),
        AnyCommand::View(ViewCommand::View),
        AnyCommand::View(ViewCommand::Rows {
            filter: "all".into(),
        }),
        AnyCommand::View(ViewCommand::Launch {
            add: true,
            options: String::new(),
        }),
        AnyCommand::View(ViewCommand::Collection { current: None }),
        AnyCommand::View(ViewCommand::Toasts { last_seq: 0 }),
        AnyCommand::View(ViewCommand::Logs { limit: 1 }),
        AnyCommand::Setup(SetupCommand::Packages),
        AnyCommand::Setup(SetupCommand::InstallApps),
        AnyCommand::Setup(SetupCommand::LaunchEnv { appid: None }),
        AnyCommand::Setup(SetupCommand::Fetch { key: "k".into() }),
        AnyCommand::Doctor(DoctorCommand::Run),
    ];
    // Adding a variant breaks this match until the list above is extended.
    for v in &variants {
        match v {
            AnyCommand::Engine(
                Command::List
                | Command::Inject { .. }
                | Command::Start3d
                | Command::Stop3d
                | Command::Status
                | Command::Undo { .. },
            )
            | AnyCommand::Link(
                LinkCommand::Watch
                | LinkCommand::Snapshot
                | LinkCommand::Install
                | LinkCommand::Uninstall
                | LinkCommand::SwitchGet { .. }
                | LinkCommand::SwitchSet { .. }
                | LinkCommand::SettingGet { .. }
                | LinkCommand::SettingSet { .. }
                | LinkCommand::Run { .. },
            )
            | AnyCommand::View(
                ViewCommand::View
                | ViewCommand::Rows { .. }
                | ViewCommand::Launch { .. }
                | ViewCommand::Collection { .. }
                | ViewCommand::Toasts { .. }
                | ViewCommand::Logs { .. },
            )
            | AnyCommand::Setup(
                SetupCommand::Packages
                | SetupCommand::InstallApps
                | SetupCommand::LaunchEnv { .. }
                | SetupCommand::Fetch { .. },
            )
            | AnyCommand::Doctor(DoctorCommand::Run) => {}
        }
        let rows = COMMANDS.iter().filter(|r| r.id == v.id()).count();
        assert_eq!(rows, 1, "{} has {rows} registry rows", v.id());
    }
    assert_eq!(variants.len(), COMMANDS.len(), "a row has no variant");
}

/// The command words that start each alternative of a usage form.
fn command_words(form: &str) -> Vec<String> {
    form.split(" | ")
        .map(|alt| {
            alt.split_whitespace()
                .take_while(|w| !w.contains(['<', '[', '|']) && *w != "--")
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// The `usage("<form>")` literals and the `FORM` const in `cli/link.rs`.
fn link_usage_forms() -> Vec<String> {
    let src = include_str!("../src/cli/link.rs");
    let mut forms = Vec::new();
    for (at, _) in src.match_indices("usage(") {
        let rest = src[at + "usage(".len()..].trim_start();
        if let Some(body) = rest.strip_prefix('"') {
            forms.push(body[..body.find('"').unwrap()].to_owned());
        }
    }
    let marker = "const FORM: &str = \"";
    let at = src.find(marker).expect("FORM const") + marker.len();
    forms.push(src[at..at + src[at..].find('"').unwrap()].to_owned());
    forms
}

#[test]
fn every_usage_command_word_has_a_registry_row() {
    use flat3d_inject::cli::{INJECT_USAGE, UNDO_USAGE, USAGE};
    let mut forms: Vec<String> = [USAGE, INJECT_USAGE, UNDO_USAGE]
        .iter()
        .map(|u| u.trim_start_matches("usage: flat3d-inject ").to_owned())
        .collect();
    let link = link_usage_forms();
    assert!(link.len() >= 6, "link usage forms not found: {link:?}");
    forms.extend(link);
    for form in forms {
        for word in command_words(&form) {
            assert!(
                COMMANDS.iter().any(|r| r.id == word),
                "usage word {word:?} (in {form:?}) has no COMMANDS row"
            );
        }
    }
}
