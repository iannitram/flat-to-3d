use super::{Command, Ctx};
use crate::api::ApiError;

pub fn run(cmd: &Command, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    let Command::Inject { appid } = cmd else {
        return Err(ApiError::new("usage", "inject requires an appid"));
    };
    let result = crate::injectors::inject(ctx.paths, *appid, ctx.runner)?;
    serde_json::to_value(result).map_err(|e| ApiError::new("io", &e.to_string()))
}
