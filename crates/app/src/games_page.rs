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
pub fn update(_state: &mut GamesState, _msg: Msg) -> Option<Action> {
    None
}
