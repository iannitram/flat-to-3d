use flat3d_app::logs_page::{self, Action, Filter, LogsState, Msg};

fn lines() -> Vec<String> {
    [
        "3 link: Tether command failed: exit 1",
        "2 link: ready",
        "1 run: Could not run tether command",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[test]
fn new_page_is_empty() {
    let state = LogsState::default();
    assert!(state.lines.is_empty());
    assert_eq!(state.filter, Filter::All);
    assert!(!state.loading);
    assert_eq!(state.error, None);
}

#[test]
fn refresh_loads_once_while_loading() {
    let mut state = LogsState::default();
    assert_eq!(
        logs_page::update(&mut state, Msg::Refresh),
        Some(Action::Load)
    );
    assert!(state.loading);
    assert_eq!(logs_page::update(&mut state, Msg::Refresh), None);
}

#[test]
fn lines_stores_and_clears_error() {
    let mut state = LogsState::default();
    logs_page::update(&mut state, Msg::Failed("io: x".into()));
    logs_page::update(&mut state, Msg::Lines(lines()));
    assert_eq!(state.lines, lines());
    assert_eq!(state.error, None);
    assert!(!state.loading);
}

#[test]
fn failed_keeps_old_lines() {
    let mut state = LogsState::default();
    logs_page::update(&mut state, Msg::Lines(lines()));
    logs_page::update(&mut state, Msg::Failed("io: x".into()));
    assert_eq!(state.lines, lines());
    assert_eq!(state.error.as_deref(), Some("io: x"));
}

#[test]
fn errors_filter_shows_only_error_lines() {
    let mut state = LogsState {
        lines: lines(),
        ..LogsState::default()
    };
    logs_page::update(&mut state, Msg::SetFilter(Filter::Errors));
    assert_eq!(
        logs_page::visible(&state),
        [
            "3 link: Tether command failed: exit 1",
            "1 run: Could not run tether command"
        ]
    );
}

#[test]
fn copy_copies_visible_lines() {
    let mut state = LogsState {
        lines: lines(),
        ..LogsState::default()
    };
    logs_page::update(&mut state, Msg::SetFilter(Filter::Errors));
    assert_eq!(
        logs_page::update(&mut state, Msg::CopyPressed),
        Some(Action::Copy(
            "3 link: Tether command failed: exit 1\n1 run: Could not run tether command".into()
        ))
    );
}

#[test]
fn copy_with_nothing_is_none() {
    let mut state = LogsState::default();
    assert_eq!(logs_page::update(&mut state, Msg::CopyPressed), None);
}

#[test]
fn page_does_no_io() {
    let source = include_str!("../src/logs_page.rs");
    for forbidden in [
        "std::fs",
        "std::process",
        "std::net",
        "std::thread",
        "Command::new",
        "read_logs",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden source token: {forbidden}"
        );
    }
}
