//! Locked tests for E10c: the rows both faces draw. Pure.

use flat3d_inject::api::{Game, LinkState};
use flat3d_inject::rows::game_rows;
use flat3d_inject::switches::Settings;
use std::collections::BTreeSet;

fn g(appid: u32, name: &str, status: &str, has_recipe: bool) -> Game {
    Game {
        appid,
        name: name.into(),
        install_dir: format!("/g/{appid}"),
        status: status.into(),
        has_recipe,
    }
}
fn games() -> Vec<Game> {
    vec![
        g(3, "Untested One", "untested", false),
        g(2, "Control", "likely", true),
        g(1, "Spider-Man 2", "works-now", true),
    ]
}
fn settings(off: &[u32]) -> Settings {
    Settings {
        auto_3d: true,
        tunnel: true,
        off: BTreeSet::from_iter(off.iter().copied()),
    }
}

#[test]
fn rows_are_sorted_with_label_and_colour() {
    let got = game_rows(&games(), "all", &settings(&[]), LinkState::Ready);
    assert_eq!(
        got.iter().map(|r| r.appid).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        got.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
        vec!["Spider-Man 2", "Control", "Untested One"]
    );
    assert_eq!(
        got.iter()
            .map(|r| r.status_label.as_str())
            .collect::<Vec<_>>(),
        vec!["Works now", "Likely", "Untested"]
    );
    assert_eq!(
        got.iter()
            .map(|r| r.status_colour.as_str())
            .collect::<Vec<_>>(),
        vec!["#2e9e5b", "#8bc34a", "#b0b0b0"]
    );
}

#[test]
fn the_filter_keeps_one_status() {
    let s = settings(&[]);
    let got = game_rows(&games(), "likely", &s, LinkState::Ready);
    assert_eq!(got.iter().map(|r| r.appid).collect::<Vec<_>>(), vec![2]);
    assert!(game_rows(&games(), "build", &s, LinkState::Ready).is_empty());
}

#[test]
fn only_a_works_now_game_with_a_recipe_a_switch_and_a_ready_link_plays_in_3d() {
    let got = game_rows(&games(), "all", &settings(&[]), LinkState::Ready);
    assert_eq!(
        got.iter().map(|r| r.plays_3d).collect::<Vec<_>>(),
        vec![true, false, false]
    );
    assert_eq!(got[0].flat_reason, None);
    assert_eq!(got[1].flat_reason, Some("Status is Likely".into()));
    assert_eq!(got[2].flat_reason, Some("No recipe yet".into()));
}

#[test]
fn the_switch_comes_from_the_settings() {
    let got = game_rows(&games(), "all", &settings(&[1]), LinkState::Ready);
    assert!(!got[0].switch_on);
    assert!(got[1].switch_on);
    assert!(!got[0].plays_3d);
    assert_eq!(got[0].flat_reason, Some("3D switch is off".into()));
}

#[test]
fn a_link_that_is_not_ready_is_the_reason() {
    let got = game_rows(&games(), "all", &settings(&[]), LinkState::NoHeadset);
    assert_eq!(
        got[0].flat_reason,
        Some("3D is not ready: no headset".into())
    );
    assert!(got.iter().all(|r| !r.plays_3d));
}

#[test]
fn the_json_has_exactly_these_keys() {
    let got = game_rows(&games(), "all", &settings(&[]), LinkState::Ready);
    let v = serde_json::to_value(&got[0]).unwrap();
    let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "appid",
            "flat_reason",
            "name",
            "plays_3d",
            "status",
            "status_colour",
            "status_label",
            "switch_on"
        ]
    );
    assert!(v["flat_reason"].is_null());
}
