//! Select and apply the stereo route named by a game's recipe.

use crate::api::{self, ApiError};
use crate::exec::CommandRunner;
use crate::paths::EnginePaths;
use crate::recipes::{self, Recipe, RouteSettings};
use crate::steam::{self, Game};
use crate::undo::{self, UndoRecord};
use std::fs;

pub mod aw2;
pub mod fetch;
pub mod geo11;
pub mod port;
pub mod uevr;
pub mod uevr_profile;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteEnv {
    pub env: Vec<(String, String)>,
    pub unset: Vec<String>,
    pub sidecar: Option<Vec<String>>,
}

/// Make sure the game is set up for its stereo route and return the environment the wrapper
/// starts it with. A matching earlier setup is reused with no fetch, writes or commands; a
/// stale one is undone first. On Err nothing new is left on disk. Every Err starts with a code
/// from [`api::ERROR_CODES`] and `": "`.
pub fn ensure_applied(
    game: &Game,
    recipe: &Recipe,
    settings: &RouteSettings,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
) -> Result<RouteEnv, String> {
    ensure_applied_in(
        game,
        recipe,
        settings,
        p,
        r,
        &fetch::parse_pins(fetch::PINS),
    )
}

/// Like [`ensure_applied`], using the supplied pin table.
pub fn ensure_applied_in(
    game: &Game,
    recipe: &Recipe,
    settings: &RouteSettings,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
    pins: &std::collections::BTreeMap<String, fetch::Pin>,
) -> Result<RouteEnv, String> {
    // A game with no Steam appid carries its store key as its name; an `epic:` key goes to the
    // Epic Games Launcher hook, never to the Steam route below.
    if game.appid == 0 && game.name.starts_with("epic:") {
        return ensure_applied_epic(game, recipe, settings, p, r);
    }
    if let Some(env) = reuse(game.appid, recipe, settings, p) {
        return Ok(env);
    }
    let env_path = p.env_file(game.appid);
    if undo::record_path(&p.data, game.appid).exists() {
        undo::undo_game(&p.data, game.appid).map_err(api_error_string)?;
        // The env file described the setup just undone; never leave it behind on its own.
        match fs::remove_file(&env_path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("io: cannot remove {}: {e}", env_path.display())),
        }
    }

    let route = route_of(&recipe.route);
    let keys =
        match route {
            Route::Uevr => {
                let mut keys = vec!["uevr".to_string()];
                keys.extend(settings.get("uevr-profile").cloned());
                keys
            }
            Route::Geo11 => {
                let mut keys = vec!["geo11".to_string()];
                keys.extend(settings.get("geo11-fix").cloned());
                keys
            }
            Route::Port => return Err(PORT_NOT_BUILT.into()),
            Route::Aw2 => return Err(
                "not-built: AW2 route needs the Heroic prefix through ensure_applied: #379 (WP-31)"
                    .into(),
            ),
            Route::NoRoute => {
                let why = check_injectable(recipe)
                    .err()
                    .unwrap_or_else(|| format!("{} has no stereo route", recipe.name));
                return Err(format!("refused: {why}"));
            }
        };
    for key in keys {
        let pin = fetch::pin_in(pins, &key).map_err(fetch_error_string)?;
        fetch::fetch(&pin, p, r).map_err(fetch_error_string)?;
    }
    let (record, env) = match route {
        Route::Uevr => {
            let (applied, env) = uevr::apply_with(game, settings, p)?;
            (applied.record, env)
        }
        Route::Geo11 => {
            let (applied, env) = geo11::apply_in(
                &p.cache(),
                &game.install_dir,
                game.appid,
                settings,
                &p.viewer_bin(),
                pins,
            )?;
            (applied.record, env)
        }
        Route::Port | Route::Aw2 | Route::NoRoute => unreachable!("returned above"),
    };

    let rec_path = match undo::save_record(&p.data, &record) {
        Ok(path) => path,
        Err(e) => {
            undo::put_back(&record);
            return Err(api_error_string(e));
        }
    };
    let written = env_path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| {
            crate::fsx::write_atomic(&env_path, env_file_text(route, settings, &env).as_bytes())
        });
    if let Err(e) = written {
        undo::put_back(&record);
        let _ = fs::remove_file(&rec_path);
        return Err(format!("io: cannot write {}: {e}", env_path.display()));
    }
    Ok(env)
}

/// The Steam path's refusal of a `Port` route; the Epic path gives the same one.
const PORT_NOT_BUILT: &str = "not-built: port route: #27b (WP-27)";

/// Turn 3D on for an `epic:<app name>` game: write its block into the launcher's
/// `user_settings.py`. The block's env comes from the inputs the Steam wrapper uses: the
/// recipe's route, the injector env, the WiVRn folders from `detect_server`, and
/// `PRESSURE_VESSEL_FILESYSTEMS_RW` merged with the value already set (at launch time, in the
/// block). Returns the injector env, as the Steam branch does. The exe comes from the recipe's
/// `exe:` line, never from the app name.
fn ensure_applied_epic(
    game: &Game,
    recipe: &Recipe,
    settings: &RouteSettings,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
) -> Result<RouteEnv, String> {
    let key: recipes::StoreKey = game
        .name
        .parse()
        .map_err(|e: recipes::RecipeError| format!("refused: {e}"))?;
    let recipes::StoreKey::Epic(app_name) = key else {
        return Err(format!("refused: not an Epic key: {}", game.name));
    };
    crate::heroic_wrapper::check_app_name(&app_name).map_err(api_error_string)?;
    let exe = settings
        .get(recipes::EPIC_EXE_KEY)
        .ok_or_else(|| format!("bad-recipe: the recipe for epic:{app_name} has no `exe:` line"))?;
    crate::epic_user_settings::check_exe(exe).map_err(api_error_string)?;
    let (route, injector) = epic_injector_env(recipe, settings)?;
    let wivrn = crate::link::detect_server(r).map_err(|e| format!("not-found: wivrn: {e}"))?;
    // The inherited value is not known until launch; the block merges it there.
    let env = crate::epic_user_settings::three_d_env(route, &injector, &wivrn, None);
    crate::epic_user_settings::apply(p, &app_name, exe, &env).map_err(api_error_string)?;
    Ok(injector)
}

/// The route step for an Epic game, refusing what the Steam path refuses: a recipe its
/// wrapper would run flat (`check_injectable`), bad route settings, a `Port` route, no route.
/// UEVR and Geo-11 work on a Steam game folder and are not built for an Epic game. The AW2
/// route sets no launch variables of its own (`aw2.rs` changes files), so its env is empty.
pub fn epic_injector_env(
    recipe: &Recipe,
    settings: &RouteSettings,
) -> Result<(Route, RouteEnv), String> {
    check_injectable(recipe).map_err(|why| format!("refused: {why}"))?;
    recipes::check_route_settings(&recipe.route, settings)
        .map_err(|e| format!("bad-recipe: {e}"))?;
    match route_of(&recipe.route) {
        Route::Port => Err(PORT_NOT_BUILT.into()),
        Route::NoRoute => Err(format!("refused: {} has no stereo route", recipe.name)),
        route @ (Route::Uevr | Route::Geo11) => Err(format!(
            "not-built: the {} route needs a Steam game folder; it is not built for an Epic game",
            route_name(route)
        )),
        Route::Aw2 => Ok((Route::Aw2, RouteEnv::default())),
    }
}

/// The reuse check (`ensure_applied` step 1): the record and the env file both exist, every
/// file the record created is still there, and the env file names this route and settings.
fn reuse(
    appid: u32,
    recipe: &Recipe,
    settings: &RouteSettings,
    p: &EnginePaths,
) -> Option<RouteEnv> {
    let record = fs::read_to_string(undo::record_path(&p.data, appid)).ok()?;
    let record = UndoRecord::from_text(&record).ok()?;
    if record.appid != appid || !record.created.iter().all(|path| path.exists()) {
        return None;
    }
    let (route, saved, env) = parse_env_file(&fs::read_to_string(p.env_file(appid)).ok()?).ok()?;
    (route == route_of(&recipe.route) && saved == *settings).then_some(env)
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Uevr => "uevr",
        Route::Geo11 => "geo11",
        Route::Port => "port",
        Route::Aw2 => "aw2",
        Route::NoRoute => "none",
    }
}

fn api_error_string(e: ApiError) -> String {
    format!("{}: {}", e.code, e.message)
}

fn fetch_error_string(e: fetch::FetchError) -> String {
    api_error_string(ApiError::from(e))
}

pub fn env_file_text(route: Route, settings: &RouteSettings, env: &RouteEnv) -> String {
    let mut text = format!("flat3d-env 1\nroute\t{}\n", route_name(route));
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
        Some("aw2") => Route::Aw2,
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
    Aw2,
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
    } else if text.contains("aw2") || text.contains("alan wake 2") {
        Route::Aw2
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

/// Set the game up to launch with its stereo route (`inject <appid>`). Nothing is written
/// before the recipe, the refusal rule, the route settings and the installed game all check
/// out. Never writes Steam launch options and never starts the game.
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
    let reused = reuse(appid, &recipe, &settings, p).is_some();
    ensure_applied(&game, &recipe, &settings, p, r).map_err(|e| string_api_error(&e))?;
    let route = route_name(route_of(&recipe.route));
    Ok(api::InjectResult {
        appid,
        route: route.into(),
        launched: false,
        summary: if reused {
            format!("{}: already set up", recipe.name)
        } else {
            format!("{}: set up for {}", recipe.name, route)
        },
    })
}

/// Split an `ensure_applied` error into its code and message; text without a known code
/// prefix is `command-failed` with the whole text.
pub(crate) fn string_api_error(text: &str) -> ApiError {
    match text.split_once(": ") {
        Some((code, message)) if api::ERROR_CODES.contains(&code) => ApiError::new(code, message),
        _ => ApiError::new("command-failed", text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_strings_split_on_a_known_code_only() {
        let e = string_api_error("not-found: no pin `x`");
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("not-found", "no pin `x`")
        );
        let e = string_api_error("weird: thing");
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("command-failed", "weird: thing")
        );
        let e = string_api_error("no prefix");
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("command-failed", "no prefix")
        );
    }

    #[test]
    fn env_file_rejects_unknown_header_and_tag() {
        assert!(parse_env_file("flat3d-env 2\nroute\tuevr\n").is_err());
        assert!(parse_env_file("flat3d-env 1\nroute\tbogus\n").is_err());
        assert!(parse_env_file("flat3d-env 1\nroute\tuevr\nextra\tx\n").is_err());
        assert!(parse_env_file("flat3d-env 1\nroute\tuevr\nset\tonly-key\n").is_err());
    }
}
