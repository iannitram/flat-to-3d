use super::{Ctx, DoctorCommand};
use crate::{api::ApiError, NotBuilt};

pub fn run(_cmd: &DoctorCommand, _ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    Err(NotBuilt("cli::doctor::run (WP-29)").into())
}
