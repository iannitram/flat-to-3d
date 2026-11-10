use crate::api::{AdbSource, LinkSnapshot, LinkState, LinkView, TetherMode};

pub fn link_view(_snap: Option<LinkSnapshot>, _now: u64) -> LinkView {
    let state = LinkState::Stopped;
    let buttons = crate::face::headset_view::headset_buttons(state);
    LinkView {
        state,
        label: crate::face::link_view::link_label(state).to_string(),
        hint: crate::face::link_view::link_hint(state).to_string(),
        start: buttons.start,
        stop: buttons.stop,
        health: crate::face::view::health_view(None),
        rx: crate::face::view::throughput_text(0),
        tx: crate::face::view::throughput_text(0),
        uptime: crate::face::view::uptime_text(0),
        tether: TetherMode::Off,
        tunnel: false,
        auto_3d: true,
        adb_source: AdbSource::Missing,
    }
}
