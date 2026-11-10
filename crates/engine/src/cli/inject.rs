use super::{Command, Ctx};
use crate::{api::ApiError, NotBuilt};

pub fn run(_cmd: &Command, _ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    Err(NotBuilt("cli::inject::run (WP-27)").into())
}
