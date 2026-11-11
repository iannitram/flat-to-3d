use super::link_view::flat_reason;
use crate::api::Game;
use crate::api::LinkState;
use crate::status::{filter_by_status, sort_games, status_colour, status_label};
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
    games: &[Game],
    filter: &str,
    settings: &Settings,
    link: LinkState,
) -> Vec<GameRow> {
    sort_games(&filter_by_status(games, filter))
        .into_iter()
        .map(|game| {
            let switch_on = settings.game_on(game.appid);
            let flat_reason = flat_reason(link, switch_on, game.has_recipe, &game.status);
            GameRow {
                appid: game.appid,
                name: game.name.clone(),
                status_label: status_label(&game.status).to_string(),
                status_colour: status_colour(&game.status).to_string(),
                status: game.status.clone(),
                switch_on,
                plays_3d: flat_reason.is_none(),
                flat_reason,
            }
        })
        .collect()
}
