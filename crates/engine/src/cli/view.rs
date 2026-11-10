use super::{Ctx, ViewCommand};
use crate::{api::ApiError, NotBuilt};

pub fn run(_cmd: &ViewCommand, _ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    Err(NotBuilt("cli::view::run (WP-28)").into())
}

pub fn parse(args: &[String]) -> Option<Result<ViewCommand, String>> {
    match args.first().map(String::as_str) {
        Some("view") if args.len() == 1 => Some(Ok(ViewCommand::View)),
        Some("view") => Some(Err("usage: flat3d-inject view".into())),
        Some("logs") => {
            let parsed = match args {
                [_, flag, n] if flag == "--limit" => n
                    .parse::<usize>()
                    .map(|limit| ViewCommand::Logs { limit })
                    .map_err(|_| "usage: flat3d-inject logs [--limit <n>]".to_string()),
                [_] => Ok(ViewCommand::Logs {
                    limit: crate::face::headset_view::DEFAULT_LOG_LINES,
                }),
                _ => Err("usage: flat3d-inject logs [--limit <n>]".into()),
            };
            Some(parsed)
        }
        _ => None,
    }
}
