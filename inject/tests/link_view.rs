//! Locked tests for E10b: link wording and the reason a game would play flat. Pure.

use flat3d_inject::api::LinkState::{self, *};
use flat3d_inject::link_view::{flat_reason, link_hint, link_label};

const ALL: [LinkState; 5] = [NoHeadset, Unauthorised, Starting, Ready, Stopped];

#[test]
fn each_state_has_a_short_name() {
    let got: Vec<&str> = ALL.iter().map(|s| link_label(*s)).collect();
    assert_eq!(got, vec!["No headset", "Not authorised", "Starting", "Ready", "Stopped"]);
}

#[test]
fn each_state_says_what_to_do() {
    let got: Vec<&str> = ALL.iter().map(|s| link_hint(*s)).collect();
    assert_eq!(
        got,
        vec![
            "Plug in the Quest with a USB cable.",
            "Put the headset on and allow USB debugging.",
            "Waiting for WiVRn to come up.",
            "Press Play on a 3D Ready game.",
            "Press Start 3D to bring the headset link up.",
        ]
    );
}

#[test]
fn a_good_game_with_a_ready_link_plays_in_3d() {
    assert_eq!(flat_reason(Ready, true, true, "works-now"), None);
}

#[test]
fn a_game_without_a_recipe_has_no_recipe_yet() {
    assert_eq!(flat_reason(Ready, true, false, "untested"), Some("No recipe yet".into()));
}

#[test]
fn a_status_that_is_not_works_now_is_named() {
    assert_eq!(flat_reason(Ready, true, true, "likely"), Some("Status is Likely".into()));
    assert_eq!(flat_reason(Ready, true, true, "build"), Some("Status is Needs a build".into()));
}

#[test]
fn a_switch_that_is_off_is_named() {
    assert_eq!(flat_reason(Ready, false, true, "works-now"), Some("3D switch is off".into()));
}

#[test]
fn a_link_that_is_not_ready_is_named() {
    assert_eq!(flat_reason(NoHeadset, true, true, "works-now"), Some("3D is not ready: no headset".into()));
    assert_eq!(flat_reason(Unauthorised, true, true, "works-now"), Some("3D is not ready: not authorised".into()));
    assert_eq!(flat_reason(Starting, true, true, "works-now"), Some("3D is not ready: starting".into()));
    assert_eq!(flat_reason(Stopped, true, true, "works-now"), Some("3D is not ready: stopped".into()));
}

#[test]
fn the_reasons_are_checked_in_order() {
    assert_eq!(flat_reason(Stopped, false, false, "likely"), Some("No recipe yet".into()));
    assert_eq!(flat_reason(Stopped, false, true, "likely"), Some("Status is Likely".into()));
    assert_eq!(flat_reason(Stopped, false, true, "works-now"), Some("3D switch is off".into()));
}
