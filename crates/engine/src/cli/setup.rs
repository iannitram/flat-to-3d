use super::{Ctx, SetupCommand};
use crate::{api::ApiError, NotBuilt};

pub fn run(_cmd: &SetupCommand, _ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    Err(NotBuilt("cli::setup::run (WP-29)").into())
}
