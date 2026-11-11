use super::{Command, Ctx};
use crate::{
    api::{ApiError, LinkReport},
    catalog,
    paths::EnginePaths,
    NotBuilt,
};

pub fn run(cmd: &Command, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    let value = match cmd {
        Command::List => serde_json::to_value(list(ctx.paths)?),
        Command::Status => serde_json::to_value(status(ctx.paths)?),
        _ => return Err(NotBuilt("cli::engine::run (WP-01)").into()),
    };
    value.map_err(|error| ApiError::new("command-failed", &error.to_string()))
}

fn list(paths: &EnginePaths) -> Result<Vec<crate::api::Game>, ApiError> {
    Ok(catalog::catalog(paths)?.games)
}

fn status(paths: &EnginePaths) -> Result<LinkReport, ApiError> {
    status_at(paths, crate::sys::unix_now())
}

fn status_at(paths: &EnginePaths, now: u64) -> Result<LinkReport, ApiError> {
    let snapshot = crate::state::read_snapshot(paths)
        .map_err(|error| ApiError::new("io", &error))?
        .map(|snapshot| snapshot.effective(now))
        .unwrap_or_else(|| crate::api::LinkSnapshot::stopped(now));
    Ok(LinkReport {
        state: snapshot.state,
        message: snapshot.message,
    })
}
