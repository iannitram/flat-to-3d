//! Locked tests for E5b: `watch::step` for plug-in and unplug. Pure: nothing is run.

use flat3d_inject::watch::{step, Effect, Event, Inputs, Note, Phase, Watch};

fn inp(auto_3d: bool, tunnel_on: bool, g3d: bool, gflat: bool) -> Inputs {
    Inputs { auto_3d, tunnel_on, game_3d_running: g3d, game_flat_running: gflat }
}
fn w(phase: Phase) -> Watch {
    Watch { phase }
}

#[test]
fn a_new_watch_starts_absent() {
    assert_eq!(Watch::new().phase, Phase::Absent);
}

#[test]
fn plug_in_with_auto_3d_brings_everything_up() {
    let (n, fx) = step(&w(Phase::Absent), &Event::Plugged, 10, &inp(true, true, false, false));
    assert_eq!(n.phase, Phase::Up);
    assert_eq!(fx, vec![Effect::StartTether, Effect::StartLink, Effect::Notify(Note::Ready)]);
}

#[test]
fn plug_in_with_the_tunnel_off_skips_the_tether() {
    let (n, fx) = step(&w(Phase::Absent), &Event::Plugged, 10, &inp(true, false, false, false));
    assert_eq!(n.phase, Phase::Up);
    assert_eq!(fx, vec![Effect::StartLink, Effect::Notify(Note::Ready)]);
}

#[test]
fn plug_in_with_auto_3d_off_brings_up_only_the_tether() {
    let (n, fx) = step(&w(Phase::Absent), &Event::Plugged, 10, &inp(false, true, false, false));
    assert_eq!(n.phase, Phase::Plugged);
    assert_eq!(fx, vec![Effect::StartTether]);
    let (n, fx) = step(&w(Phase::Absent), &Event::Plugged, 10, &inp(false, false, false, false));
    assert_eq!((n.phase, fx), (Phase::Plugged, vec![]));
}

#[test]
fn plug_in_while_a_game_runs_flat_says_restart_for_3d() {
    let (n, fx) = step(&w(Phase::Absent), &Event::Plugged, 10, &inp(true, true, false, true));
    assert_eq!(n.phase, Phase::Up);
    assert_eq!(
        fx,
        vec![Effect::StartTether, Effect::StartLink, Effect::Notify(Note::LateRestartFor3d)]
    );
}

#[test]
fn unplug_with_no_3d_game_stops_everything() {
    let (n, fx) = step(&w(Phase::Up), &Event::Unplugged, 50, &inp(true, true, false, false));
    assert_eq!(n.phase, Phase::Absent);
    assert_eq!(fx, vec![Effect::StopLink, Effect::StopTether]);
    let (_, fx) = step(&w(Phase::Up), &Event::Unplugged, 50, &inp(true, false, false, false));
    assert_eq!(fx, vec![Effect::StopLink]);
}

#[test]
fn unplug_with_a_3d_game_starts_the_grace_and_does_nothing_else() {
    let (n, fx) = step(&w(Phase::Up), &Event::Unplugged, 100, &inp(true, true, true, false));
    assert_eq!(n.phase, Phase::Grace { since: 100 });
    assert_eq!(fx, vec![]);
}

#[test]
fn unplug_before_the_link_was_up_only_drops_the_tether() {
    let (n, fx) = step(&w(Phase::Plugged), &Event::Unplugged, 5, &inp(false, true, false, false));
    assert_eq!((n.phase, fx), (Phase::Absent, vec![Effect::StopTether]));
    let (n, fx) = step(&w(Phase::Plugged), &Event::Unplugged, 5, &inp(false, false, false, false));
    assert_eq!((n.phase, fx), (Phase::Absent, vec![]));
}

#[test]
fn a_replug_inside_the_grace_reattaches_only() {
    let (n, fx) = step(&w(Phase::Grace { since: 100 }), &Event::Plugged, 130, &inp(true, true, true, false));
    assert_eq!(n.phase, Phase::Up);
    assert_eq!(fx, vec![Effect::Reattach]);
}

#[test]
fn repeated_events_change_nothing() {
    let i = inp(true, true, false, false);
    for (p, e) in [
        (Phase::Up, Event::Plugged),
        (Phase::Plugged, Event::Plugged),
        (Phase::Absent, Event::Unplugged),
        (Phase::Grace { since: 1 }, Event::Unplugged),
        (Phase::Up, Event::Tick),
    ] {
        let (n, fx) = step(&w(p), &e, 9, &i);
        assert_eq!((n.phase, fx), (p, vec![]));
    }
}
