//! The Games page: state, messages, update. docs/code-map.md, Parts, crates/app/src.
use flat3d_inject::rows::GameRow;

#[derive(Debug, Clone, PartialEq)]
pub struct GamesState {
    pub rows: Vec<GameRow>,
    pub filter: String,
    pub error: Option<String>,
    pub note: Option<String>,
}

impl Default for GamesState {
    fn default() -> Self {
        Self {
            rows: vec![],
            filter: "all".into(),
            error: None,
            note: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Rows(Result<Vec<GameRow>, String>),
    Filter(String),
    Switch { appid: u32, on: bool },
    UndoPressed(u32),
    UndoDone(Result<String, String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    LoadRows { filter: String },
    SetSwitch { appid: u32, on: bool },
    Undo(u32),
}

pub fn update(state: &mut GamesState, msg: Msg) -> Option<Action> {
    match msg {
        Msg::Rows(Ok(rows)) => {
            state.rows = rows;
            state.error = None;
            None
        }
        Msg::Rows(Err(error)) => {
            state.error = Some(error);
            None
        }
        Msg::Filter(filter) => {
            state.filter = filter.clone();
            Some(Action::LoadRows { filter })
        }
        Msg::Switch { appid, on } => {
            if let Some(row) = state.rows.iter_mut().find(|row| row.appid == appid) {
                row.switch_on = on;
            }
            Some(Action::SetSwitch { appid, on })
        }
        Msg::UndoPressed(appid) => {
            state.note = None;
            state.error = None;
            Some(Action::Undo(appid))
        }
        Msg::UndoDone(Ok(summary)) => {
            state.note = Some(summary);
            Some(Action::LoadRows {
                filter: state.filter.clone(),
            })
        }
        Msg::UndoDone(Err(error)) => {
            state.error = Some(error);
            None
        }
    }
}
