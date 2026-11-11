use crate::{
    api::{ApiError, SetupReport},
    exec::CommandRunner,
    paths::EnginePaths,
};

pub fn setup_fetch(
    key: &str,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
) -> Result<SetupReport, ApiError> {
    let pin = crate::injectors::fetch::pin(key).map_err(ApiError::from)?;
    let cached = crate::injectors::fetch::is_fetched(&pin, p);
    let dir = crate::injectors::fetch::fetch(&pin, p, r).map_err(ApiError::from)?;
    let dir = dir.display().to_string();
    Ok(SetupReport {
        steps: vec![format!(
            "{} {}",
            if cached { "cached" } else { "fetched" },
            dir
        )],
        summary: format!("{} {} is ready in {}", key, pin.version, dir),
    })
}
