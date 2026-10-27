//! Inject a game by its recipe's stereo route. 1.0 has one route: the Spider-Man 2 port.

use crate::recipes::Recipe;
use crate::steam::Game;
use crate::NotBuilt;

/// Set the game up to launch with its stereo route; returns a one-line summary.
/// Refuses a recipe whose `safe` is not "yes" or whose `status` is not "works-now".
pub fn inject(game: &Game, recipe: &Recipe) -> Result<String, NotBuilt> {
    let _ = (game, recipe);
    Err(NotBuilt("injectors::inject: see the inject issue"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stub_exists() {
        let game = Game { appid: 1, name: "x".into(), install_dir: PathBuf::from("/x") };
        let recipe = Recipe {
            appid: 1,
            name: "x".into(),
            status: "untested".into(),
            route: String::new(),
            engine: String::new(),
            api: "unknown".into(),
            mod_url: "none".into(),
            free: "unclear".into(),
            safe: "no".into(),
            checked: "2026-10-26".into(),
        };
        assert!(inject(&game, &recipe).is_err());
    }
}
