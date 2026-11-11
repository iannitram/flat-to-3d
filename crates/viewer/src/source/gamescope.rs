use super::pipewire::{self, Offer, PwSource, Remote, Target};
use std::time::Duration;

pub const NODE_NAME: &str = "gamescope";

pub fn open(offer: &Offer, wait: Duration) -> Result<Option<PwSource>, String> {
    match pipewire::find_node(Remote::Default, NODE_NAME, wait)? {
        Some(_) => pipewire::open(
            Remote::Default,
            Target::NodeName(NODE_NAME.to_owned()),
            offer,
        )
        .map(Some),
        None => Ok(None),
    }
}
