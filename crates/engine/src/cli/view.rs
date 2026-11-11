use super::{Ctx, ViewCommand};
use crate::{
    api::{self, ApiError},
    face::{self, headset_view, link_model, toasts},
};

const VIEW_USAGE: &str = "usage: flat3d-inject view | rows [--filter <status>|all] | launch add|remove --options <string> | collection [--current <appids>] | toasts <last_seq> | logs [--limit <n>]";

fn usage() -> String {
    VIEW_USAGE.to_string()
}

pub fn parse(args: &[String]) -> Option<Result<ViewCommand, String>> {
    let command = args.first().map(String::as_str)?;
    let parsed = match command {
        "view" if args.len() == 1 => Ok(ViewCommand::View),
        "view" => Err(usage()),
        "rows" => match args {
            [_] => Ok(ViewCommand::Rows {
                filter: "all".into(),
            }),
            [_, flag, filter] if flag == "--filter" => Ok(ViewCommand::Rows {
                filter: filter.clone(),
            }),
            _ => Err(usage()),
        },
        "launch" => match args {
            [_, verb, flag, options]
                if flag == "--options" && (verb == "add" || verb == "remove") =>
            {
                Ok(ViewCommand::Launch {
                    add: verb == "add",
                    options: options.clone(),
                })
            }
            _ => Err(usage()),
        },
        "collection" => match args {
            [_] => Ok(ViewCommand::Collection { current: None }),
            [_, flag, csv] if flag == "--current" => {
                if csv.is_empty() {
                    Ok(ViewCommand::Collection {
                        current: Some(Vec::new()),
                    })
                } else {
                    csv.split(',')
                        .map(str::parse::<u32>)
                        .collect::<Result<Vec<_>, _>>()
                        .map(|current| ViewCommand::Collection {
                            current: Some(current),
                        })
                        .map_err(|_| usage())
                }
            }
            _ => Err(usage()),
        },
        "toasts" => match args {
            [_, last_seq] => last_seq
                .parse::<u64>()
                .map(|last_seq| ViewCommand::Toasts { last_seq })
                .map_err(|_| usage()),
            _ => Err(usage()),
        },
        "logs" => match args {
            [_] => Ok(ViewCommand::Logs {
                limit: headset_view::DEFAULT_LOG_LINES,
            }),
            [_, flag, n] if flag == "--limit" => n
                .parse::<usize>()
                .ok()
                .filter(|limit| *limit > 0)
                .map(|limit| ViewCommand::Logs { limit })
                .ok_or_else(usage),
            _ => Err(usage()),
        },
        _ => return None,
    };
    Some(parsed)
}

pub fn run(cmd: &ViewCommand, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    let p = ctx.paths;
    let now = crate::sys::unix_now();
    match cmd {
        ViewCommand::View => serde_json::to_value(link_model::link_view(
            crate::state::read_snapshot(p).map_err(|e| ApiError::new("io", &e))?,
            now,
        ))
        .map_err(|e| ApiError::new("command-failed", &e.to_string())),
        ViewCommand::Rows { filter } => serde_json::to_value(rows(p, filter, now)?)
            .map_err(|e| ApiError::new("command-failed", &e.to_string())),
        ViewCommand::Launch { add, options } => {
            let options = if *add {
                crate::launch_options::add_wrapper_at(options, &p.wrapper_bin().to_string_lossy())
            } else {
                crate::launch_options::remove_wrapper(options)
            };
            serde_json::to_value(api::LaunchText { options })
                .map_err(|e| ApiError::new("command-failed", &e.to_string()))
        }
        ViewCommand::Collection { current } => serde_json::to_value(collection(p, current)?)
            .map_err(|e| ApiError::new("command-failed", &e.to_string())),
        ViewCommand::Toasts { last_seq } => {
            let events = crate::state::read_snapshot(p)
                .map_err(|e| ApiError::new("io", &e))?
                .map(|snapshot| snapshot.events)
                .unwrap_or_default();
            let mut value = serde_json::to_value(toasts::toasts_since(&events, *last_seq))
                .map_err(|e| ApiError::new("command-failed", &e.to_string()))?;
            value["title"] = serde_json::Value::String(api::TOAST_TITLE.into());
            Ok(value)
        }
        ViewCommand::Logs { limit } => serde_json::to_value(logs(p, *limit))
            .map_err(|e| ApiError::new("command-failed", &e.to_string())),
    }
}

fn rows(
    p: &crate::paths::EnginePaths,
    filter: &str,
    now: u64,
) -> Result<Vec<face::rows::GameRow>, ApiError> {
    let games = crate::catalog::catalog(p)?.games;
    let settings =
        crate::switches::read_settings(&p.settings()).map_err(|e| ApiError::new("io", &e))?;
    let link = crate::state::read_snapshot(p)
        .map_err(|e| ApiError::new("io", &e))?
        .map(|snapshot| snapshot.effective(now).state)
        .unwrap_or(crate::api::LinkState::Stopped);
    Ok(face::rows::game_rows(&games, filter, &settings, link))
}

fn collection(
    p: &crate::paths::EnginePaths,
    current: &Option<Vec<u32>>,
) -> Result<api::CollectionPlan, ApiError> {
    let wanted = face::collection::three_d_ready_appids(&crate::catalog::catalog(p)?.games);
    let diff = face::collection::collection_diff(current.as_deref().unwrap_or_default(), &wanted);
    Ok(api::CollectionPlan {
        name: face::collection::COLLECTION_NAME.into(),
        wanted,
        add: diff.add,
        remove: diff.remove,
    })
}

fn logs(p: &crate::paths::EnginePaths, limit: usize) -> api::LogLines {
    let lines = face::logs::read_logs(p, limit);
    api::LogLines {
        text: headset_view::log_copy_text(&lines),
        lines,
    }
}
