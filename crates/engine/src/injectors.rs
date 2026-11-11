//! Select and apply the stereo route named by a game's recipe.

use crate::recipes::Recipe;
use crate::steam::Game;
use crate::NotBuilt;

pub mod fetch;
pub mod geo11;
pub mod port;
pub mod uevr;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteEnv {
    pub env: Vec<(String, String)>,
    pub unset: Vec<String>,
    pub sidecar: Option<Vec<String>>,
}

pub fn ensure_applied(
    _game: &Game,
    _recipe: &Recipe,
    _settings: &crate::recipes::RouteSettings,
    _p: &crate::paths::EnginePaths,
    _r: &mut dyn crate::link::CommandRunner,
) -> Result<RouteEnv, String> {
    Err("not-built: injectors::ensure_applied (WP-27)".into())
}

/// Which injector module a recipe's `stereo-route` text names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Uevr,
    Geo11,
    Port,
    NoRoute,
}

/// Say which injector module a recipe's `stereo-route` text names.
/// `build` wins over a tool name in the same text; anything unknown has no route.
pub fn route_of(stereo_route: &str) -> Route {
    let text = stereo_route.trim().to_lowercase();
    if text.starts_with("build") {
        Route::Port
    } else if text.contains("uevr") {
        Route::Uevr
    } else if text.contains("geo-11") {
        Route::Geo11
    } else {
        Route::NoRoute
    }
}

/// The refusal rule for injecting a game: the first problem found, in order.
/// The `refused` API error code is attached by the caller; this returns plain text.
pub fn check_injectable(recipe: &Recipe) -> Result<(), String> {
    if recipe.safe != "yes" {
        return Err(format!(
            "{} is not safe to inject (stereo-safe is \"{}\")",
            recipe.name, recipe.safe
        ));
    }
    if recipe.status != "works-now" {
        return Err(format!(
            "{} is not works-now (stereo-status is \"{}\")",
            recipe.name, recipe.status
        ));
    }
    if route_of(&recipe.route) == Route::NoRoute {
        return Err(format!(
            "{} has no stereo route (stereo-route is \"{}\")",
            recipe.name, recipe.route
        ));
    }
    Ok(())
}

/// Set the game up to launch with its stereo route; returns a one-line summary.
/// Refuses a recipe whose `safe` is not "yes" or whose `status` is not "works-now".
pub fn inject(game: &Game, recipe: &Recipe) -> Result<String, NotBuilt> {
    let _ = (game, recipe);
    Err(NotBuilt("injectors::inject (WP-27)"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stub_exists() {
        let game = Game {
            appid: 1,
            name: "x".into(),
            install_dir: PathBuf::from("/x"),
        };
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
