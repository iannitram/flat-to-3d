use crate::api::Game;
use crate::api::LinkState;
use crate::switches::Settings;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GameRow {
    pub appid: u32,
    pub name: String,
    pub status: String,
    pub status_label: String,
    pub status_colour: String,
    pub switch_on: bool,
    pub plays_3d: bool,
    pub flat_reason: Option<String>,
}

pub fn game_rows(
    _games: &[Game],
    _filter: &str,
    _settings: &Settings,
    _link: LinkState,
) -> Vec<GameRow> {
    Vec::new()
}
