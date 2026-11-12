//! The launch wrapper: the 3D-or-flat decision and the exec. docs/code-map.md, Interlinks, the wrapper rule.

use crate::api::LinkState;
use crate::injectors::{check_injectable, route_of, Route};
use crate::recipes::Recipe;
use crate::runs::{RunMode, RunRecord};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::PathBuf;

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

pub fn exec_spec_with(
    plan: &Plan,
    argv: &[String],
    route: &crate::injectors::RouteEnv,
    wivrn: &crate::link::WivrnDirs,
    existing_fs_rw: Option<&str>,
) -> ExecSpec {
    let mut spec = exec_spec(plan, argv, &route.env, "", existing_fs_rw);
    if matches!(plan, Plan::ThreeD { .. }) {
        let folder = wivrn.app_dir.to_string_lossy().into_owned();
        let rw = match existing_fs_rw.filter(|value| !value.is_empty()) {
            Some(existing) if existing.split(':').any(|part| part == folder) => {
                existing.to_string()
            }
            Some(existing) => format!("{existing}:{folder}"),
            None => folder,
        };
        if let Some((_, value)) = spec
            .env
            .iter_mut()
            .find(|(key, _)| key == "PRESSURE_VESSEL_FILESYSTEMS_RW")
        {
            *value = rw;
        }
        spec.unset = route.unset.clone();
        spec.sidecar = route.sidecar.clone();
    }
    spec
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
    let decision =
        (|| -> Result<(Plan, crate::injectors::RouteEnv, crate::link::WivrnDirs), String> {
            let id = appid.as_ref().map_err(Clone::clone)?;
            let snapshot = snapshot?.ok_or_else(|| "snapshot is missing".to_string())?;
            if crate::state::is_stale(snapshot.updated, now) {
                return Err("snapshot is stale".into());
            }
            let settings = settings?;
            let recipe = recipe?;
            let plan = plan(snapshot.state, settings.game_on(*id), recipe);
            if matches!(plan, Plan::ThreeD { .. })
                && env.get("HOME").is_none_or(|home| home.is_empty())
            {
                return Err("HOME is missing".into());
            }
            match (&plan, recipe) {
                (Plan::ThreeD { .. }, Some(recipe)) => {
                    let (route_env, dirs) = route(*id, recipe)?;
                    Ok((plan, route_env, dirs))
                }
                _ => Ok((
                    plan,
                    crate::injectors::RouteEnv::default(),
                    crate::link::WivrnDirs::user_default(
                        env.get("HOME")
                            .map_or(std::path::Path::new(""), |h| std::path::Path::new(h)),
                    ),
                )),
            }
        })();
    let (plan, route_env, dirs, message) = match decision {
        Ok((plan, route_env, dirs)) => {
            let message = match &plan {
                Plan::Flat { reason } => format!("flat: {}", reason.text()),
                Plan::ThreeD { route } => format!("3d: {route:?}"),
            };
            (plan, route_env, dirs, message)
        }
        Err(message) => {
            let plan = Plan::Flat {
                reason: FlatReason::NotInjectable(message.clone()),
            };
            (
                plan,
                crate::injectors::RouteEnv::default(),
                crate::link::WivrnDirs::user_default(
                    env.get("HOME")
                        .map_or(std::path::Path::new(""), |h| std::path::Path::new(h)),
                ),
                format!("flat: {message}"),
            )
        }
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

pub fn run_with<T>(
    argv: &[String],
    env: &BTreeMap<String, String>,
    paths: &crate::paths::EnginePaths,
    pid: u32,
    now: u64,
    _runner: &mut dyn crate::link::CommandRunner,
    launcher: impl FnOnce(&Launch) -> io::Result<T>,
) -> (Launch, io::Result<T>) {
    let snapshot = crate::state::read_snapshot(paths);
    let settings = crate::switches::read_settings(&paths.settings());
    let mut recipe_text = None;
    let recipe_result = match appid(env)
        .and_then(|id| crate::catalog::find_recipe(&paths.recipe_dirs, id).map(|found| (id, found)))
    {
        Ok((id, Some((_, text)))) => {
            recipe_text = Some(text.clone());
            crate::recipes::parse_recipe(id, &text)
                .map_err(|error| error.to_string())
                .and_then(|recipe| {
                    crate::recipes::check_recipe(&recipe)
                        .map(|()| recipe)
                        .map_err(|error| error.to_string())
                })
                .map(Some)
        }
        Ok((_, None)) => Ok(None),
        Err(error) => Err(error),
    };
    let mut route_fn = |appid: u32, recipe: &Recipe| {
        let steam_root = paths.home.join(crate::steam::DEFAULT_STEAM_ROOT);
        let game = crate::steam::installed_games(&steam_root)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|game| game.appid == appid)
            .ok_or_else(|| "game is not installed".to_string())?;
        let _text = recipe_text.as_deref();
        let _route_settings = _text.map(crate::recipes::route_settings);
        let _ = (&game, recipe, &_route_settings, paths);
        Err("not-built: route (WP-31)".into())
    };
    let mut launch = prepare_with_route(
        argv,
        env,
        snapshot.as_ref().map(Option::as_ref).map_err(Clone::clone),
        settings.as_ref().map_err(Clone::clone),
        recipe_result
            .as_ref()
            .map(|recipe| recipe.as_ref())
            .map_err(Clone::clone),
        &mut route_fn,
        pid,
        now,
    );
    let record_path = crate::state::run_record_path(&paths.state_dir.to_string_lossy(), pid);
    let temporary = format!("{record_path}.tmp");
    let saved = (|| {
        std::fs::create_dir_all(crate::state::runs_dir(&paths.state_dir.to_string_lossy()))?;
        std::fs::write(&temporary, launch.record.to_text())?;
        std::fs::rename(&temporary, record_path)
    })();
    if let Err(error) = saved {
        launch.message.push_str(&format!("; run record: {error}"));
    }
    let log = |message: &str| -> io::Result<()> {
        std::fs::create_dir_all(&paths.log_dir)?;
        writeln!(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(paths.run_log())?,
            "{now} appid={} pid={pid} {}",
            launch.record.appid,
            message.replace(['\n', '\r'], " ")
        )
    };
    let _ = log(&launch.message);
    let result = launcher(&launch);
    if let Err(error) = &result {
        let _ = log(&format!("exec: {error}"));
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
