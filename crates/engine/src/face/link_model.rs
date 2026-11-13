use crate::api::{AdbSource, LinkSnapshot, LinkState, LinkView};

pub fn link_view(snap: Option<LinkSnapshot>, now: u64) -> LinkView {
    let snapshot = snap.map(|snapshot| snapshot.effective(now));
    let state = snapshot
        .as_ref()
        .map(|snapshot| snapshot.state)
        .unwrap_or(LinkState::Stopped);
    let buttons = crate::face::headset_view::headset_buttons(state);
    let health = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.health.as_ref());
    LinkView {
        state,
        label: crate::face::link_view::link_label(state).to_string(),
        hint: crate::face::link_view::link_hint(state).to_string(),
        start: buttons.start,
        stop: buttons.stop,
        health: crate::face::view::health_view(health),
        auto_3d: snapshot.as_ref().is_some_and(|snapshot| snapshot.auto_3d),
        adb_source: snapshot
            .as_ref()
            .map(|snapshot| snapshot.adb_source)
            .unwrap_or(AdbSource::Missing),
        quest_tether: quest_tether_line(
            snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.quest_tether.as_deref()),
        )
        .to_string(),
    }
}

/// The Quest Tether line both faces draw: its state as a read-only fact, nothing to press.
fn quest_tether_line(state: Option<&str>) -> &'static str {
    match state {
        Some("connected") => "Quest Tether: connected",
        Some(_) => "Quest Tether: not connected",
        None => "Quest Tether: not installed",
    }
}
