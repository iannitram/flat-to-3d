//! Locked tests for E5d: manual Start/Stop in `watch::step`, `link_commands` and the note texts.

use flat3d_inject::link::{start_3d_commands, stop_3d_commands};
use flat3d_inject::watch::{
    link_commands, note_kind, note_text, step, Effect, Event, Inputs, Note, Phase, Watch,
};

fn inputs() -> Inputs {
    Inputs {
        auto_3d: false,
        tunnel_on: true,
        game_3d_running: false,
        game_flat_running: false,
    }
}
fn run(p: Phase, e: Event) -> (Phase, Vec<Effect>) {
    let (n, fx) = step(&Watch { phase: p }, &e, 10, &inputs());
    (n.phase, fx)
}

#[test]
fn start_from_plugged_brings_the_link_up() {
    let (p, fx) = run(Phase::Plugged, Event::StartRequested);
    assert_eq!(p, Phase::Up);
    assert_eq!(fx, vec![Effect::StartLink, Effect::Notify(Note::Ready)]);
}

#[test]
fn start_in_a_grace_reattaches() {
    let (p, fx) = run(Phase::Grace { since: 1 }, Event::StartRequested);
    assert_eq!((p, fx), (Phase::Up, vec![Effect::Reattach]));
}

#[test]
fn start_with_no_headset_or_when_up_does_nothing() {
    assert_eq!(
        run(Phase::Absent, Event::StartRequested),
        (Phase::Absent, vec![])
    );
    assert_eq!(run(Phase::Up, Event::StartRequested), (Phase::Up, vec![]));
}

#[test]
fn stop_takes_the_link_down_and_keeps_the_tether() {
    assert_eq!(
        run(Phase::Up, Event::StopRequested),
        (Phase::Plugged, vec![Effect::StopLink])
    );
    assert_eq!(
        run(Phase::Grace { since: 3 }, Event::StopRequested),
        (Phase::Plugged, vec![Effect::StopLink])
    );
}

#[test]
fn stop_when_there_is_no_link_does_nothing() {
    assert_eq!(
        run(Phase::Plugged, Event::StopRequested),
        (Phase::Plugged, vec![])
    );
    assert_eq!(
        run(Phase::Absent, Event::StopRequested),
        (Phase::Absent, vec![])
    );
}

#[test]
fn link_effects_run_these_commands() {
    assert_eq!(link_commands(&Effect::StartLink), start_3d_commands());
    assert_eq!(link_commands(&Effect::StopLink), stop_3d_commands());
    assert_eq!(
        link_commands(&Effect::Reattach),
        start_3d_commands()[2..].to_vec()
    );
    assert_eq!(link_commands(&Effect::Reattach).len(), 2);
}

#[test]
fn the_other_effects_run_no_link_commands() {
    for e in [
        Effect::StartTether,
        Effect::StopTether,
        Effect::Notify(Note::Ready),
    ] {
        assert!(link_commands(&e).is_empty());
    }
}

#[test]
fn the_toast_wording_is_fixed() {
    assert_eq!(
        note_text(Note::Ready),
        "3D is ready. Press Play on a 3D Ready game."
    );
    assert_eq!(
        note_text(Note::LateRestartFor3d),
        "Headset connected. Restart the game to play it in 3D."
    );
    assert_eq!(
        note_text(Note::StoppedRestartForFlat),
        "Headset link closed. Restart the game to play it flat."
    );
    assert_eq!(
        note_text(Note::AllowUsbDebugging),
        "Put the headset on and allow USB debugging."
    );
}

#[test]
fn each_note_has_a_kind() {
    assert_eq!(note_kind(Note::Ready), "ready");
    assert_eq!(note_kind(Note::LateRestartFor3d), "late-plug");
    assert_eq!(note_kind(Note::StoppedRestartForFlat), "grace-ended");
    assert_eq!(note_kind(Note::AllowUsbDebugging), "allow-debugging");
}
