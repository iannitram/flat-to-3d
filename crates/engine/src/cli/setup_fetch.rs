use crate::{
    api::{ApiError, SetupReport},
    exec::CommandRunner,
    paths::EnginePaths,
    NotBuilt,
};

pub fn setup_fetch(
    _key: &str,
    _p: &EnginePaths,
    _r: &mut dyn CommandRunner,
) -> Result<SetupReport, ApiError> {
    Err(NotBuilt("cli::setup_fetch::setup_fetch (WP-27)").into())
}
