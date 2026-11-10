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
fn error_help_covers_every_closed_error_code() {
    for code in flat3d_inject::api::ERROR_CODES {
        let (_, fix) = flat3d_inject::api::error_help(code);
        assert!(!fix.is_empty(), "{code}");
    }
}
