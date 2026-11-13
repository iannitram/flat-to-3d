//! The launch wrapper: the 3D-or-flat decision and the exec. docs/code-map.md, Interlinks, the wrapper rule.

use crate::api::LinkState;
use crate::injectors::{check_injectable, ensure_applied, route_of, Route, RouteEnv};
use crate::link::{detect_server, WivrnDirs};
use crate::paths::EnginePaths;
use crate::recipes::Recipe;
use crate::runs::{RunMode, RunRecord};
use crate::{fsx, recipes, state, steam};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// All filesystem locations are supplied by the caller, never read from global env.
pub struct RunPaths {
    pub state: String,
    pub settings: PathBuf,
    pub recipes: PathBuf,
    pub log: PathBuf,
}

/// A launch, including the record written before handing control to the game.
#[derive(Debug)]
pub struct Launch {
    pub command: ExecSpec,
    pub record: RunRecord,
    pub message: String,
}

/// Pure launch decision over supplied file contents and environment.
/// Invalid input always produces the original command with no environment additions.
pub fn prepare(
    argv: &[String],
    env: &BTreeMap<String, String>,
    snapshot: Result<&str, String>,
    settings: Result<&str, String>,
    recipe: Result<Option<&Recipe>, String>,
    pid: u32,
    now: u64,
) -> Launch {
    let appid = appid(env);
    let decision: Result<Plan, String> = (|| {
        let appid = appid.as_ref().map_err(Clone::clone)?;
        let snapshot: crate::api::LinkSnapshot =
            serde_json::from_str(snapshot?).map_err(|e| format!("snapshot: {e}"))?;
        if crate::state::is_stale(snapshot.updated, now) {
            return Err("snapshot is stale".into());
        }
        let settings = settings?;
        if !valid_settings(settings) {
            return Err("invalid settings or game switch".into());
        }
        let plan = plan(
            snapshot.state,
            crate::switches::parse_settings(settings).game_on(*appid),
            recipe?,
        );
        if matches!(plan, Plan::ThreeD { .. }) && env.get("HOME").is_none_or(|home| home.is_empty())
        {
            return Err("HOME is missing".into());
        }
        Ok(plan)
    })();
    let (plan, message) = match decision {
        Ok(plan) => {
            let message = match &plan {
                Plan::Flat { reason } => format!("flat: {}", reason.text()),
                Plan::ThreeD { route } => format!("3d: {route:?}"),
            };
            (plan, message)
        }
        Err(message) => (
            Plan::Flat {
                reason: FlatReason::NotInjectable(message.clone()),
            },
            format!("flat: {message}"),
        ),
    };
    Launch {
        command: exec_spec(
            &plan,
            argv,
            &[],
            env.get("HOME").map(String::as_str).unwrap_or_default(),
            env.get("PRESSURE_VESSEL_FILESYSTEMS_RW")
                .map(String::as_str),
        ),
        record: RunRecord {
            appid: appid.unwrap_or(0),
            pid,
            mode: if matches!(plan, Plan::ThreeD { .. }) {
                RunMode::ThreeD
            } else {
                RunMode::Flat
            },
        },
        message,
    }
}

fn appid(env: &BTreeMap<String, String>) -> Result<u32, String> {
    env.get("SteamAppId")
        .or_else(|| env.get("SteamGameId"))
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|id| *id != 0)
        .ok_or_else(|| "missing or invalid Steam appid".into())
}

fn valid_settings(text: &str) -> bool {
    let mut lines = text.lines();
    lines.next() == Some("flat3d-settings 1")
        && lines.all(|line| match line.split_once('\t') {
            Some(("auto-3d", "on" | "off")) => true,
            Some(("game-off", value)) => value.parse::<u32>().is_ok(),
            Some(("auto-3d", _)) => false,
            // An old `tunnel` line is ignored, so it stays valid whatever its value.
            Some(("tunnel", _)) => true,
            _ => !matches!(line, "auto-3d" | "game-off"),
        })
}

/// Read inputs, persist the decision, then call the launcher even if any IO fails.
/// Only the launcher's own error is returned. Time, pid, env and paths are injected.
pub fn run<T>(
    argv: &[String],
    env: &BTreeMap<String, String>,
    paths: &RunPaths,
    pid: u32,
    now: u64,
    launcher: impl FnOnce(&Launch) -> io::Result<T>,
) -> (Launch, io::Result<T>) {
    let snapshot = std::fs::read_to_string(crate::state::snapshot_path(&paths.state))
        .map_err(|e| format!("snapshot: {e}"));
    let settings = std::fs::read_to_string(&paths.settings).map_err(|e| format!("settings: {e}"));
    let recipe = appid(env).and_then(|id| {
        let path = paths.recipes.join(format!("{id}.recipe"));
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let recipe = crate::recipes::parse_recipe(id, &text).map_err(|e| e.to_string())?;
                crate::recipes::check_recipe(&recipe).map_err(|e| e.to_string())?;
                Ok(Some(recipe))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("recipe: {e}")),
        }
    });
    let mut launch = prepare(
        argv,
        env,
        snapshot.as_deref().map_err(Clone::clone),
        settings.as_deref().map_err(Clone::clone),
        recipe.as_ref().map(Option::as_ref).map_err(Clone::clone),
        pid,
        now,
    );
    let record_path = crate::state::run_record_path(&paths.state, pid);
    let temporary = format!("{record_path}.tmp");
    let saved = (|| {
        std::fs::create_dir_all(crate::state::runs_dir(&paths.state))?;
        std::fs::write(&temporary, launch.record.to_text())?;
        std::fs::rename(&temporary, record_path)
    })();
    if let Err(e) = saved {
        launch.message.push_str(&format!("; run record: {e}"));
    }
    let log = |message: &str| -> io::Result<()> {
        if let Some(parent) = paths.log.parent() {
            std::fs::create_dir_all(parent)?;
        }
        writeln!(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&paths.log)?,
            "{now} appid={} pid={pid} {}",
            launch.record.appid,
            message.replace(['\n', '\r'], " ")
        )
    };
    let _ = log(&launch.message);
    let result = launcher(&launch);
    if let Err(e) = &result {
        let _ = log(&format!("exec: {e}"));
    }
    (launch, result)
}

/// Why the wrapper chose to launch flat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatReason {
    NoRecipe,
    SwitchOff,
    LinkNotReady,
    NotInjectable(String),
}

impl FlatReason {
    /// The reason to include in the wrapper's log.
    pub fn text(&self) -> String {
        match self {
            FlatReason::NoRecipe => "no recipe".to_string(),
            FlatReason::SwitchOff => "3D switch is off".to_string(),
            FlatReason::LinkNotReady => "headset link is not ready".to_string(),
            FlatReason::NotInjectable(message) => message.clone(),
        }
    }
}

/// The wrapper's decision for one game launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    ThreeD { route: Route },
    Flat { reason: FlatReason },
}

/// The command and environment the wrapper execs for one game launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecSpec {
    pub argv: Vec<String>,
    pub env: Vec<(String, String)>,
    pub unset: Vec<String>,
    pub sidecar: Option<Vec<String>>,
}

/// Build the command and environment for a planned launch.
pub fn exec_spec(
    plan: &Plan,
    argv: &[String],
    route_env: &[(String, String)],
    home: &str,
    existing_fs_rw: Option<&str>,
) -> ExecSpec {
    let env = match plan {
        Plan::Flat { .. } => Vec::new(),
        Plan::ThreeD { .. } => {
            let folder = format!("{home}/.local/share/flatpak/app/io.github.wivrn.wivrn");
            let filesystems_rw = match existing_fs_rw.filter(|value| !value.is_empty()) {
                Some(existing) if existing.split(':').any(|part| part == folder) => {
                    existing.to_string()
                }
                Some(existing) => format!("{existing}:{folder}"),
                None => folder,
            };
            let mut env = vec![
                (
                    "PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES".to_string(),
                    "1".to_string(),
                ),
                ("PRESSURE_VESSEL_FILESYSTEMS_RW".to_string(), filesystems_rw),
                (
                    "STEAM_COMPAT_LAUNCHER_SERVICE".to_string(),
                    "proton".to_string(),
                ),
            ];
            env.extend_from_slice(route_env);
            env
        }
    };
    ExecSpec {
        argv: argv.to_vec(),
        env,
        unset: vec![],
        sidecar: None,
    }
}

/// Every reason code a flat launch's run record can carry, in a fixed order.
pub const FLAT_REASONS: [&str; 13] = [
    "appid",
    "snapshot",
    "stale",
    "settings",
    "recipe",
    "no-recipe",
    "switch-off",
    "link-not-ready",
    "not-injectable",
    "home",
    "route",
    "wivrn",
    "sidecar",
];

/// The `FLAT_REASONS` code for a launch message (`flat: <reason>`).
fn reason_code(message: &str) -> &'static str {
    let text = message.strip_prefix("flat: ").unwrap_or(message);
    let text = text.split("; run record: ").next().unwrap_or(text);
    match text {
        "missing or invalid Steam appid" => "appid",
        "snapshot is stale" => "stale",
        "no link snapshot" => "snapshot",
        "no recipe" => "no-recipe",
        "3D switch is off" => "switch-off",
        "headset link is not ready" => "link-not-ready",
        "HOME is missing" => "home",
        _ if text.starts_with("snapshot: ") => "snapshot",
        _ if text.starts_with("settings: ") => "settings",
        _ if text.starts_with("recipe: ") => "recipe",
        _ if text.starts_with("route: ") => "route",
        _ if text.starts_with("wivrn: ") => "wivrn",
        _ if text.starts_with("sidecar: ") => "sidecar",
        _ => "not-injectable",
    }
}

fn one_line(text: &str) -> String {
    text.replace(['\n', '\r'], " ")
}

/// The run record file's text: the record, plus `reason` and `note` lines for a flat launch.
pub fn record_text(launch: &Launch) -> String {
    let mut text = launch.record.to_text();
    if launch.record.mode == RunMode::Flat {
        let note = launch
            .message
            .strip_prefix("flat: ")
            .unwrap_or(&launch.message);
        text.push_str(&format!(
            "reason\t{}\nnote\t{}\n",
            reason_code(&launch.message),
            one_line(note)
        ));
    }
    text
}

fn record_path(paths: &EnginePaths, pid: u32) -> PathBuf {
    PathBuf::from(state::run_record_path(
        &paths.state_dir.to_string_lossy(),
        pid,
    ))
}

/// Write the run record atomically, creating the runs folder first.
fn save_record(paths: &EnginePaths, pid: u32, text: &str) -> io::Result<()> {
    std::fs::create_dir_all(state::runs_dir(&paths.state_dir.to_string_lossy()))?;
    fsx::write_atomic(&record_path(paths, pid), text.as_bytes())
}

/// Append one run.log line for this launch; IO errors are ignored.
fn log_run(paths: &EnginePaths, launch: &Launch, now: u64, message: &str) {
    let _ = fsx::append_line(
        &paths.run_log(),
        &format!(
            "{now} appid={} pid={} {}",
            launch.record.appid,
            launch.record.pid,
            one_line(message)
        ),
    );
}

/// Add a `key\tvalue` line to the launch's run record and a `key: value` line to run.log.
/// IO errors are ignored: the game launches either way.
pub fn note_run(paths: &EnginePaths, launch: &Launch, now: u64, key: &str, value: &str) {
    let path = record_path(paths, launch.record.pid);
    if let Ok(mut text) = std::fs::read_to_string(&path) {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!("{}\t{}\n", one_line(key), one_line(value)));
        let _ = fsx::write_atomic(&path, text.as_bytes());
    }
    log_run(paths, launch, now, &format!("{key}: {value}"));
}

/// The sidecar could not start: record the launch as flat with reason `sidecar`, log it, and
/// return the flat command (argv only).
pub fn flat_after_sidecar_error(
    paths: &EnginePaths,
    launch: &Launch,
    now: u64,
    err: &str,
) -> ExecSpec {
    let err = err.strip_prefix("sidecar: ").unwrap_or(err);
    let flat = Launch {
        command: ExecSpec {
            argv: launch.command.argv.clone(),
            env: Vec::new(),
            unset: Vec::new(),
            sidecar: None,
        },
        record: RunRecord {
            mode: RunMode::Flat,
            ..launch.record
        },
        message: format!("flat: sidecar: {err}"),
    };
    let mut message = flat.message.clone();
    if let Err(error) = save_record(paths, flat.record.pid, &record_text(&flat)) {
        message.push_str(&format!("; run record: {error}"));
    }
    log_run(paths, &flat, now, &message);
    flat.command
}

/// The sidecar's full argv: the program (absolute as given; else beside this executable when it
/// is there, else in `bin`), its arguments, then `--until-pid <until_pid>`.
pub fn sidecar_argv(
    sc: &[String],
    exe_dir: Option<&Path>,
    bin: &Path,
    until_pid: u32,
    exists: &dyn Fn(&Path) -> bool,
) -> Result<Vec<String>, String> {
    let (first, rest) = sc
        .split_first()
        .ok_or_else(|| "sidecar: empty argv".to_string())?;
    let program = if Path::new(first).is_absolute() {
        first.clone()
    } else {
        exe_dir
            .map(|dir| dir.join(first))
            .filter(|path| exists(path))
            .unwrap_or_else(|| bin.join(first))
            .to_string_lossy()
            .into_owned()
    };
    let mut argv = vec![program];
    argv.extend_from_slice(rest);
    argv.push("--until-pid".into());
    argv.push(until_pid.to_string());
    Ok(argv)
}

/// `existing` with `folder` added to its colon list, unless it is already there.
fn filesystems_rw(existing: Option<&str>, folder: String) -> String {
    match existing.filter(|value| !value.is_empty()) {
        Some(existing) if existing.split(':').any(|part| part == folder) => existing.to_string(),
        Some(existing) => format!("{existing}:{folder}"),
        None => folder,
    }
}

/// Build the command and environment for a planned launch with its route's env.
pub fn exec_spec_with(
    plan: &Plan,
    argv: &[String],
    route: &RouteEnv,
    wivrn: &WivrnDirs,
    existing_fs_rw: Option<&str>,
) -> ExecSpec {
    if !matches!(plan, Plan::ThreeD { .. }) {
        return ExecSpec {
            argv: argv.to_vec(),
            env: Vec::new(),
            unset: Vec::new(),
            sidecar: None,
        };
    }
    let folder = wivrn.app_dir.to_string_lossy().into_owned();
    let mut env = vec![
        (
            "PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES".to_string(),
            "1".to_string(),
        ),
        (
            "PRESSURE_VESSEL_FILESYSTEMS_RW".to_string(),
            filesystems_rw(existing_fs_rw, folder),
        ),
        (
            "STEAM_COMPAT_LAUNCHER_SERVICE".to_string(),
            "proton".to_string(),
        ),
    ];
    for (key, value) in &route.env {
        match env.iter_mut().find(|(existing, _)| existing == key) {
            Some(entry) => entry.1 = value.clone(),
            None => env.push((key.clone(), value.clone())),
        }
    }
    let mut unset: Vec<String> = Vec::new();
    for key in &route.unset {
        if !env.iter().any(|(existing, _)| existing == key) && !unset.contains(key) {
            unset.push(key.clone());
        }
    }
    ExecSpec {
        argv: argv.to_vec(),
        env,
        unset,
        sidecar: route.sidecar.clone(),
    }
}

/// The dll names one `WINEDLLOVERRIDES` piece names: the comma list before `=`.
fn dll_names(piece: &str) -> impl Iterator<Item = &str> {
    piece
        .split_once('=')
        .map_or(piece, |(names, _)| names)
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// The inherited `WINEDLLOVERRIDES` (Steam launch options) without the dlls the route names,
/// followed by the route's own value: the route wins for the same dll.
fn merge_dll_overrides(inherited: &str, route: &str) -> String {
    let taken: Vec<&str> = route.split(';').flat_map(dll_names).collect();
    let mut pieces: Vec<String> = Vec::new();
    for piece in inherited.split(';') {
        let kept: Vec<&str> = dll_names(piece)
            .filter(|name| !taken.iter().any(|t| t.eq_ignore_ascii_case(name)))
            .collect();
        if kept.is_empty() {
            continue;
        }
        pieces.push(match piece.split_once('=') {
            Some((_, value)) => format!("{}={value}", kept.join(",")),
            None => kept.join(","),
        });
    }
    pieces.push(route.to_string());
    pieces.join(";")
}

// This public shape is frozen in §2 so later work packages share the same seam.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn prepare_with_route(
    argv: &[String],
    env: &BTreeMap<String, String>,
    snapshot: Result<Option<&crate::api::LinkSnapshot>, String>,
    settings: Result<&crate::switches::Settings, String>,
    recipe: Result<Option<&Recipe>, String>,
    route: &mut dyn FnMut(
        u32,
        &Recipe,
    )
        -> Result<(crate::injectors::RouteEnv, crate::link::WivrnDirs), String>,
    pid: u32,
    now: u64,
) -> Launch {
    let appid = appid(env);
    let home = env.get("HOME").map(String::as_str).unwrap_or_default();
    let decision = (|| -> Result<(Plan, RouteEnv, WivrnDirs), String> {
        let id = *appid.as_ref().map_err(Clone::clone)?;
        let snapshot = snapshot?.ok_or_else(|| "no link snapshot".to_string())?;
        if state::is_stale(snapshot.updated, now) {
            return Err("snapshot is stale".into());
        }
        let settings = settings?;
        let recipe = recipe?;
        let plan = plan(snapshot.state, settings.game_on(id), recipe);
        let (Plan::ThreeD { .. }, Some(recipe)) = (&plan, recipe) else {
            return Ok((
                plan,
                RouteEnv::default(),
                WivrnDirs::user_default(Path::new(home)),
            ));
        };
        if home.is_empty() {
            return Err("HOME is missing".into());
        }
        let (mut route_env, dirs) = route(id, recipe)?;
        if let Some(inherited) = env
            .get("WINEDLLOVERRIDES")
            .filter(|value| !value.is_empty())
        {
            for (key, value) in &mut route_env.env {
                if key == "WINEDLLOVERRIDES" {
                    *value = merge_dll_overrides(inherited, value);
                }
            }
        }
        Ok((plan, route_env, dirs))
    })();
    let (plan, route_env, dirs, message) = match decision {
        Ok((plan, route_env, dirs)) => {
            let message = match &plan {
                Plan::Flat { reason } => format!("flat: {}", reason.text()),
                Plan::ThreeD { route } => format!("3d: {route:?}"),
            };
            (plan, route_env, dirs, message)
        }
        Err(message) => (
            Plan::Flat {
                reason: FlatReason::NotInjectable(message.clone()),
            },
            RouteEnv::default(),
            WivrnDirs::user_default(Path::new(home)),
            format!("flat: {message}"),
        ),
    };
    Launch {
        command: exec_spec_with(
            &plan,
            argv,
            &route_env,
            &dirs,
            env.get("PRESSURE_VESSEL_FILESYSTEMS_RW")
                .map(String::as_str),
        ),
        record: RunRecord {
            appid: appid.unwrap_or(0),
            pid,
            mode: if matches!(plan, Plan::ThreeD { .. }) {
                RunMode::ThreeD
            } else {
                RunMode::Flat
            },
        },
        message,
    }
}

/// Read the launch inputs through `paths`, apply the recipe's route, persist the decision, then
/// call the launcher exactly once even if any IO fails. Only the launcher's own error is returned.
pub fn run_with<T>(
    argv: &[String],
    env: &BTreeMap<String, String>,
    paths: &crate::paths::EnginePaths,
    pid: u32,
    now: u64,
    runner: &mut dyn crate::link::CommandRunner,
    launcher: impl FnOnce(&Launch) -> io::Result<T>,
) -> (Launch, io::Result<T>) {
    let snapshot = state::read_snapshot(paths).map_err(|e| format!("snapshot: {e}"));
    let settings =
        crate::switches::read_settings(&paths.settings()).map_err(|e| format!("settings: {e}"));
    let found = match appid(env) {
        Ok(id) => crate::catalog::find_recipe(&paths.recipe_dirs, id)
            .map(|found| found.map(|(_, text)| (id, text))),
        // prepare_with_route reports the appid first, so the recipe is not read.
        Err(_) => Ok(None),
    };
    let recipe_text = match &found {
        Ok(Some((_, text))) => text.clone(),
        _ => String::new(),
    };
    let recipe = found
        .and_then(|found| {
            found
                .map(|(id, text)| {
                    let recipe = recipes::parse_recipe(id, &text).map_err(|e| e.to_string())?;
                    recipes::check_recipe(&recipe).map_err(|e| e.to_string())?;
                    Ok(recipe)
                })
                .transpose()
        })
        .map_err(|e| format!("recipe: {e}"));
    let mut route = |id: u32, recipe: &Recipe| -> Result<(RouteEnv, WivrnDirs), String> {
        let not_installed = || format!("route: game {id} is not installed");
        let game = steam::installed_games(&paths.home.join(steam::DEFAULT_STEAM_ROOT))
            .map_err(|_| not_installed())?
            .into_iter()
            .find(|game| game.appid == id)
            .ok_or_else(not_installed)?;
        let settings = recipes::route_settings(&recipe_text);
        recipes::check_route_settings(&recipe.route, &settings)
            .map_err(|e| format!("route: {e}"))?;
        let env = ensure_applied(&game, recipe, &settings, paths, &mut *runner)
            .map_err(|e| format!("route: {e}"))?;
        let dirs = detect_server(&mut *runner).map_err(|e| format!("wivrn: {e}"))?;
        Ok((env, dirs))
    };
    let mut launch = prepare_with_route(
        argv,
        env,
        snapshot.as_ref().map(Option::as_ref).map_err(Clone::clone),
        settings.as_ref().map_err(Clone::clone),
        recipe.as_ref().map(Option::as_ref).map_err(Clone::clone),
        &mut route,
        pid,
        now,
    );
    if let Err(error) = save_record(paths, pid, &record_text(&launch)) {
        launch.message.push_str(&format!("; run record: {error}"));
    }
    log_run(paths, &launch, now, &launch.message);
    let result = launcher(&launch);
    if let Err(error) = &result {
        log_run(paths, &launch, now, &format!("exec: {error}"));
    }
    (launch, result)
}

/// Decide whether this launch uses its recipe's 3D route or runs flat.
pub fn plan(link: LinkState, switch_on: bool, recipe: Option<&Recipe>) -> Plan {
    let Some(recipe) = recipe else {
        return Plan::Flat {
            reason: FlatReason::NoRecipe,
        };
    };
    if !switch_on {
        return Plan::Flat {
            reason: FlatReason::SwitchOff,
        };
    }
    if link != LinkState::Ready {
        return Plan::Flat {
            reason: FlatReason::LinkNotReady,
        };
    }
    if let Err(message) = check_injectable(recipe) {
        return Plan::Flat {
            reason: FlatReason::NotInjectable(message),
        };
    }
    Plan::ThreeD {
        route: route_of(&recipe.route),
    }
}
