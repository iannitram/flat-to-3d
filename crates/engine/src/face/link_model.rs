use crate::api::{AdbSource, LinkSnapshot, LinkState, LinkView, TetherMode};

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
    let stats = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.stats.as_ref());
    LinkView {
        state,
        label: crate::face::link_view::link_label(state).to_string(),
        hint: crate::face::link_view::link_hint(state).to_string(),
        start: buttons.start,
        stop: buttons.stop,
        health: crate::face::view::health_view(health),
        rx: stats
            .map(|stats| crate::face::view::throughput_text(stats.rx_bytes_s))
            .unwrap_or_else(|| "—".into()),
        tx: stats
            .map(|stats| crate::face::view::throughput_text(stats.tx_bytes_s))
            .unwrap_or_else(|| "—".into()),
        uptime: stats
            .map(|stats| crate::face::view::uptime_text(stats.uptime_s))
            .unwrap_or_else(|| "—".into()),
        tether: snapshot
            .as_ref()
            .map(|snapshot| snapshot.tether)
            .unwrap_or(TetherMode::Off),
        tunnel: snapshot.as_ref().is_some_and(|snapshot| snapshot.tunnel),
        auto_3d: snapshot.as_ref().is_some_and(|snapshot| snapshot.auto_3d),
        adb_source: snapshot
            .as_ref()
            .map(|snapshot| snapshot.adb_source)
            .unwrap_or(AdbSource::Missing),
    }
}
