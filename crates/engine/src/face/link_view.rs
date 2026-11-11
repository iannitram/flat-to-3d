//! Link wording and the flat reason. docs/code-map.md, Parts.
use crate::api::LinkState;
use crate::status::status_label;

/// The short name of a link state, as both faces show it.
pub fn link_label(s: LinkState) -> &'static str {
    match s {
        LinkState::NoHeadset => "No headset",
        LinkState::Unauthorised => "Not authorised",
        LinkState::Starting => "Starting",
        LinkState::Ready => "Ready",
        LinkState::Stopped => "Stopped",
    }
}

/// What the user should do next for a link state.
pub fn link_hint(s: LinkState) -> &'static str {
    match s {
        LinkState::NoHeadset => "Plug in the Quest with a USB cable.",
        LinkState::Unauthorised => "Put the headset on and allow USB debugging.",
        LinkState::Starting => "Waiting for WiVRn to come up.",
        LinkState::Ready => "Press Play on a 3D Ready game.",
        LinkState::Stopped => "Press Start 3D to bring the headset link up.",
    }
}

/// The reason a game would play flat, or `None` when it would play in 3D.
/// Checked in order: no recipe, status, switch, link.
pub fn flat_reason(
    link: LinkState,
    switch_on: bool,
    has_recipe: bool,
    status: &str,
) -> Option<String> {
    if !has_recipe {
        return Some("No recipe yet".into());
    }
    if status != "works-now" {
        return Some(format!("Status is {}", status_label(status)));
    }
    if !switch_on {
        return Some("3D switch is off".into());
    }
    if link != LinkState::Ready {
        return Some(format!(
            "3D is not ready: {}",
            link_label(link).to_lowercase()
        ));
    }
    None
}
