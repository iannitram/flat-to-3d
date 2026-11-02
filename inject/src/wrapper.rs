//! The launch wrapper: the 3D-or-flat decision and the exec. docs/code-map.md, Interlinks, the wrapper rule.

use crate::api::LinkState;
use crate::injectors::{check_injectable, route_of, Route};
use crate::recipes::Recipe;

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
                Some(existing)
                    if existing.split(':').any(|part| part == folder) =>
                {
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
                (
                    "PRESSURE_VESSEL_FILESYSTEMS_RW".to_string(),
                    filesystems_rw,
                ),
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
    }
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
