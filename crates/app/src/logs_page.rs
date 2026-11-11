//! The Logs page: state, messages, update. docs/code-map.md, Parts, crates/app/src.
use flat3d_inject::face::logs::is_error_line;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    All,
    Errors,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogsState {
    pub lines: Vec<String>,
    pub filter: Filter,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Refresh,
    Lines(Vec<String>),
    Failed(String),
    SetFilter(Filter),
    CopyPressed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Load,
    Copy(String),
}

pub fn update(state: &mut LogsState, msg: Msg) -> Option<Action> {
    match msg {
        Msg::Refresh if state.loading => None,
        Msg::Refresh => {
            state.loading = true;
            Some(Action::Load)
        }
        Msg::Lines(lines) => {
            state.lines = lines;
            state.error = None;
            state.loading = false;
            None
        }
        Msg::Failed(error) => {
            state.error = Some(error);
            state.loading = false;
            None
        }
        Msg::SetFilter(filter) => {
            state.filter = filter;
            None
        }
        Msg::CopyPressed => {
            let lines = visible(state);
            (!lines.is_empty()).then(|| Action::Copy(lines.join("\n")))
        }
    }
}

pub fn visible(state: &LogsState) -> Vec<&str> {
    state
        .lines
        .iter()
        .filter(|line| state.filter == Filter::All || is_error_line(line))
        .map(String::as_str)
        .collect()
}
