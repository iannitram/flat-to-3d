//! Select and apply the stereo route named by a game's recipe.

use crate::api::{self, ApiError};
use crate::exec::CommandRunner;
use crate::paths::EnginePaths;
use crate::recipes::{self, Recipe, RouteSettings};
use crate::steam::{self, Game};
use crate::undo::{self, UndoRecord};
use std::fs;

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
    game: &Game,
    recipe: &Recipe,
    settings: &RouteSettings,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
) -> Result<RouteEnv, String> {
    let record_path = undo::record_path(&p.data, game.appid);
    let env_path = p.env_file(game.appid);
    if record_path.exists() {
        if let (Ok(text), Ok(env_text)) = (
            fs::read_to_string(&record_path),
            fs::read_to_string(&env_path),
        ) {
            let record = UndoRecord::from_text(&text).ok();
            if record.as_ref().is_some_and(|record| {
                record.appid == game.appid && record.created.iter().all(|path| path.exists())
            }) && parse_env_file(&env_text).is_ok_and(|(route, saved, _)| {
                route == route_of(&recipe.route) && saved == *settings
            }) {
                return parse_env_file(&env_text).map(|(_, _, env)| env);
            }
        }
        undo::undo_game(&p.data, game.appid).map_err(api_error_string)?;
    }

    let route = match route_of(&recipe.route) {
        Route::Uevr => Route::Uevr,
        Route::Geo11 => Route::Geo11,
        Route::Port => return Err("not-built: port route: #27b (WP-27)".into()),
        Route::NoRoute => {
            return Err(format!(
                "refused: {}",
                check_injectable(recipe).unwrap_err()
            ))
        }
    };
    let mut keys = Vec::new();
    match route {
        Route::Uevr => {
            keys.push("uevr".to_string());
            if let Some(key) = settings.get("uevr-profile") {
                keys.push(key.clone());
            }
        }
        Route::Geo11 => {
            keys.push("geo11".to_string());
            if let Some(key) = settings.get("geo11-fix") {
                keys.push(key.clone());
            }
        }
        _ => unreachable!(),
    }
    for key in keys {
        let pin = fetch::pin(&key).map_err(fetch_error_string)?;
        fetch::fetch(&pin, p, r).map_err(fetch_error_string)?;
    }
    let (record, env) = match route {
        Route::Uevr => {
            let (applied, env) = uevr::apply_with(game, settings, p)?;
            (applied.record, env)
        }
        Route::Geo11 => {
            let (applied, env) = geo11::apply(
                &p.cache(),
                &game.install_dir,
                game.appid,
                settings,
                &p.viewer_bin(),
            )?;
            (applied.record, env)
        }
        _ => unreachable!(),
    };
    let rec_path = match undo::save_record(&p.data, &record) {
        Ok(path) => path,
        Err(e) => {
            undo::put_back(&record);
            return Err(api_error_string(e));
        }
    };
    let write_result = (|| {
        let parent = env_path.parent().expect("env path has parent");
        fs::create_dir_all(parent)?;
        crate::fsx::write_atomic(&env_path, env_file_text(route, settings, &env).as_bytes())
    })();
    if let Err(e) = write_result {
        undo::put_back(&record);
        let _ = fs::remove_file(rec_path);
        return Err(format!("io: {}", e));
    }
    Ok(env)
}

fn api_error_string(e: ApiError) -> String {
    format!("{}: {}", e.code, e.message)
}
fn fetch_error_string(e: fetch::FetchError) -> String {
    api_error_string(ApiError::from(e))
}

pub fn env_file_text(route: Route, settings: &RouteSettings, env: &RouteEnv) -> String {
    let mut text = format!(
        "flat3d-env 1\nroute\t{}\n",
        match route {
            Route::Uevr => "uevr",
            Route::Geo11 => "geo11",
            Route::Port => "port",
            Route::NoRoute => "none",
        }
    );
    for (key, value) in settings {
        text.push_str(&format!(
            "setting\t{}\t{}\n",
            undo::escape(key),
            undo::escape(value)
        ));
    }
    for (key, value) in &env.env {
        text.push_str(&format!(
            "set\t{}\t{}\n",
            undo::escape(key),
            undo::escape(value)
        ));
    }
    for key in &env.unset {
        text.push_str(&format!("unset\t{}\n", undo::escape(key)));
    }
    if let Some(args) = &env.sidecar {
        text.push_str("sidecar");
        for arg in args {
            text.push('\t');
            text.push_str(&undo::escape(arg));
        }
        text.push('\n');
    }
    text
}

pub fn parse_env_file(text: &str) -> Result<(Route, RouteSettings, RouteEnv), String> {
    let mut lines = text.lines();
    if lines.next() != Some("flat3d-env 1") {
        return Err("unknown env file header".into());
    }
    let route = match lines.next().and_then(|line| line.strip_prefix("route\t")) {
        Some("uevr") => Route::Uevr,
        Some("geo11") => Route::Geo11,
        Some("port") => Route::Port,
        Some("none") => Route::NoRoute,
        _ => return Err("missing or unknown route tag".into()),
    };
    let mut settings = RouteSettings::new();
    let mut env = RouteEnv::default();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        match fields.as_slice() {
            ["setting", k, v] => {
                settings.insert(undo::unescape(k), undo::unescape(v));
            }
            ["set", k, v] => env.env.push((undo::unescape(k), undo::unescape(v))),
            ["unset", k] => env.unset.push(undo::unescape(k)),
            ["sidecar", args @ ..] => {
                env.sidecar = Some(args.iter().map(|s| undo::unescape(s)).collect())
            }
            _ => {
                return Err(format!(
                    "unknown env file tag: {}",
                    fields.first().unwrap_or(&"")
                ))
            }
        }
    }
    Ok((route, settings, env))
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
pub fn inject(
    p: &EnginePaths,
    appid: u32,
    r: &mut dyn CommandRunner,
) -> Result<api::InjectResult, ApiError> {
    let (_, text) = crate::catalog::find_recipe(&p.recipe_dirs, appid)
        .map_err(|e| ApiError::new("bad-recipe", &e))?
        .ok_or_else(|| ApiError::new("not-found", &format!("no recipe for {appid}")))?;
    let recipe = recipes::parse_recipe(appid, &text)
        .map_err(|e| ApiError::new("bad-recipe", &e.to_string()))?;
    check_injectable(&recipe).map_err(|e| ApiError::new("refused", &e))?;
    let settings = recipes::route_settings(&text);
    recipes::check_route_settings(&recipe.route, &settings)
        .map_err(|e| ApiError::new("bad-recipe", &e))?;
    let games = steam::installed_games(&p.home.join(steam::DEFAULT_STEAM_ROOT))
        .map_err(|e| ApiError::new("io", &e.to_string()))?;
    let game = games
        .into_iter()
        .find(|game| game.appid == appid)
        .ok_or_else(|| ApiError::new("not-found", &format!("{appid} is not installed")))?;
    let record = undo::record_path(&p.data, appid);
    let reusable = record.exists()
        && p.env_file(appid).exists()
        && fs::read_to_string(&p.env_file(appid))
            .ok()
            .and_then(|s| parse_env_file(&s).ok())
            .is_some_and(|(route, saved, _)| route == route_of(&recipe.route) && saved == settings);
    let _env = ensure_applied(&game, &recipe, &settings, p, r)
        .map_err(|error| string_api_error(&error))?;
    let route = route_of(&recipe.route);
    Ok(api::InjectResult {
        appid,
        route: match route {
            Route::Uevr => "uevr",
            Route::Geo11 => "geo11",
            Route::Port => "port",
            Route::NoRoute => "none",
        }
        .into(),
        launched: false,
        summary: if reusable {
            format!("{}: already set up", recipe.name)
        } else {
            format!(
                "{}: set up for {}",
                recipe.name,
                match route {
                    Route::Uevr => "uevr",
                    Route::Geo11 => "geo11",
                    Route::Port => "port",
                    Route::NoRoute => "none",
                }
            )
        },
    })
}

fn string_api_error(text: &str) -> ApiError {
    let Some((code, message)) = text.split_once(": ") else {
        return ApiError::new("command-failed", text);
    };
    if api::ERROR_CODES.contains(&code) {
        ApiError::new(code, message)
    } else {
        ApiError::new("command-failed", text)
    }
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
