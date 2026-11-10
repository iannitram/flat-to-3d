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
pub fn update(_state: &mut LogsState, _msg: Msg) -> Option<Action> {
    None
}
pub fn visible(_state: &LogsState) -> Vec<&str> {
    Vec::new()
}
