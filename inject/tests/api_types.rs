//! Locked tests for E1d: `api`'s data types, the JSON shape both faces read (docs/code-map.md,
//! "API shape"). Key names and values here are the contract; change them only with it.

use flat3d_inject::api::{Game, InjectResult, LinkReport, LinkState, Recipe};
use serde_json::{from_value, json, to_value};

fn sm2_recipe() -> flat3d_inject::recipes::Recipe {
    flat3d_inject::recipes::Recipe {
        appid: 2651280,
        name: "Marvel's Spider-Man 2".into(),
        status: "works-now".into(),
        route: "build: port spidy stereo screen".into(),
        engine: "custom".into(),
        api: "dx12".into(),
        mod_url: "https://github.com/IlyaMez/SpidyVR".into(),
        free: "yes".into(),
        safe: "yes".into(),
        checked: "2026-10-31".into(),
    }
}

#[test]
fn a_game_has_these_keys() {
    let g = Game {
        appid: 2075800,
        name: "STAR WARS Zero Company".into(),
        install_dir: "/home/deck/.local/share/Steam/steamapps/common/Zero Company".into(),
        status: "works-now".into(),
        has_recipe: true,
    };
    let v = json!({"appid": 2075800, "name": "STAR WARS Zero Company",
        "install_dir": "/home/deck/.local/share/Steam/steamapps/common/Zero Company",
        "status": "works-now", "has_recipe": true});
    assert_eq!(to_value(&g).unwrap(), v);
    assert_eq!(from_value::<Game>(v).unwrap(), g);
}

#[test]
fn a_recipe_copies_every_field_and_has_these_keys() {
    let r = Recipe::from(&sm2_recipe());
    let v = json!({"appid": 2651280, "name": "Marvel's Spider-Man 2", "status": "works-now",
        "route": "build: port spidy stereo screen", "engine": "custom", "api": "dx12",
        "mod_url": "https://github.com/IlyaMez/SpidyVR", "free": "yes", "safe": "yes",
        "checked": "2026-10-31"});
    assert_eq!(to_value(&r).unwrap(), v);
    assert_eq!(from_value::<Recipe>(v).unwrap(), r);
}

#[test]
fn link_states_are_kebab_case_strings() {
    let pairs = [
        (LinkState::NoHeadset, "no-headset"),
        (LinkState::Unauthorised, "unauthorised"),
        (LinkState::Starting, "starting"),
        (LinkState::Ready, "ready"),
        (LinkState::Stopped, "stopped"),
    ];
    for (state, text) in pairs {
        assert_eq!(to_value(state).unwrap(), json!(text));
        assert_eq!(from_value::<LinkState>(json!(text)).unwrap(), state);
    }
    assert!(from_value::<LinkState>(json!("Ready")).is_err());
}

#[test]
fn a_link_report_has_state_and_message() {
    let r = LinkReport {
        state: LinkState::Ready,
        message: "WiVRn ready over the cable".into(),
    };
    let v = json!({"state": "ready", "message": "WiVRn ready over the cable"});
    assert_eq!(to_value(&r).unwrap(), v);
    assert_eq!(from_value::<LinkReport>(v).unwrap(), r);
}

#[test]
fn an_inject_result_has_these_keys() {
    let r = InjectResult {
        appid: 2651280,
        route: "build: port spidy stereo screen".into(),
        launched: true,
        summary: "launch options set, game started".into(),
    };
    let v = json!({"appid": 2651280, "route": "build: port spidy stereo screen",
        "launched": true, "summary": "launch options set, game started"});
    assert_eq!(to_value(&r).unwrap(), v);
    assert_eq!(from_value::<InjectResult>(v).unwrap(), r);
}
