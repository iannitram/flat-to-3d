//! Locked tests for E5c: the grace period in `watch::step`. Pure: nothing is run.

use flat3d_inject::watch::{step, Effect, Event, Inputs, Note, Phase, Watch, GRACE_SECS};

fn inp(tunnel_on: bool, g3d: bool) -> Inputs {
    Inputs {
        auto_3d: true,
        tunnel_on,
        game_3d_running: g3d,
        game_flat_running: false,
    }
}
fn grace(since: u64) -> Watch {
    Watch {
        phase: Phase::Grace { since },
    }
}

#[test]
fn the_grace_is_120_seconds() {
    assert_eq!(GRACE_SECS, 120);
}

#[test]
fn nothing_happens_before_the_grace_is_over() {
    let (n, fx) = step(&grace(100), &Event::Tick, 219, &inp(true, true));
    assert_eq!(n.phase, Phase::Grace { since: 100 });
    assert_eq!(fx, vec![]);
}

#[test]
fn at_120_seconds_the_link_stops_and_the_player_is_told() {
    let (n, fx) = step(&grace(100), &Event::Tick, 220, &inp(true, true));
    assert_eq!(n.phase, Phase::Absent);
    assert_eq!(
        fx,
        vec![
            Effect::StopLink,
            Effect::StopTether,
            Effect::Notify(Note::StoppedRestartForFlat)
        ]
    );
}

#[test]
fn with_the_tunnel_off_there_is_no_tether_to_stop() {
    let (_, fx) = step(&grace(100), &Event::Tick, 400, &inp(false, true));
    assert_eq!(
        fx,
        vec![
            Effect::StopLink,
            Effect::Notify(Note::StoppedRestartForFlat)
        ]
    );
}

#[test]
fn if_the_game_quits_during_the_grace_the_stop_is_silent() {
    let (n, fx) = step(&grace(100), &Event::Tick, 150, &inp(true, false));
    assert_eq!(n.phase, Phase::Absent);
    assert_eq!(fx, vec![Effect::StopLink, Effect::StopTether]);
}

#[test]
fn a_game_quit_after_the_grace_is_over_is_still_silent() {
    let (_, fx) = step(&grace(100), &Event::Tick, 900, &inp(true, false));
    assert_eq!(fx, vec![Effect::StopLink, Effect::StopTether]);
}

#[test]
fn a_clock_that_went_backwards_does_not_end_the_grace() {
    let (n, fx) = step(&grace(500), &Event::Tick, 100, &inp(true, true));
    assert_eq!((n.phase, fx), (Phase::Grace { since: 500 }, vec![]));
}

#[test]
fn ticks_in_the_other_phases_do_nothing() {
    for p in [Phase::Absent, Phase::Plugged, Phase::Up] {
        let (n, fx) = step(&Watch { phase: p }, &Event::Tick, 1000, &inp(true, true));
        assert_eq!((n.phase, fx), (p, vec![]));
    }
}
