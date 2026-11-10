use super::{Command, Ctx};
use crate::api::ApiError;

pub fn run(cmd: &Command, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    let Command::Undo { appid } = cmd else {
        return Err(ApiError::new("usage", "undo requires an appid"));
    };
    let result = crate::undo::undo_game(&ctx.paths.data, *appid)?;
    serde_json::to_value(result).map_err(|e| ApiError::new("command-failed", &e.to_string()))
}
