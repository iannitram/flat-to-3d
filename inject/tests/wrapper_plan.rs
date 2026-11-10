//! Locked tests for E8a: the wrapper's 3D-or-flat decision. Pure.

use flat3d_inject::api::LinkState;
use flat3d_inject::injectors::Route;
use flat3d_inject::recipes::Recipe;
use flat3d_inject::wrapper::{plan, FlatReason, Plan};

fn recipe(route: &str, status: &str, safe: &str) -> Recipe {
    Recipe {
        appid: 2075800,
        name: "Zero Company".into(),
        status: status.into(),
        route: route.into(),
        engine: "Unreal 5.6".into(),
        api: "DX12".into(),
        mod_url: "https://example.com".into(),
        free: "yes".into(),
        safe: safe.into(),
        checked: "2026-11-01".into(),
    }
}

#[test]
fn a_good_launch_is_3d_with_the_recipes_route() {
    let r = recipe("uevr + vrto3d", "works-now", "yes");
    assert_eq!(
        plan(LinkState::Ready, true, Some(&r)),
        Plan::ThreeD { route: Route::Uevr }
    );
    let g = recipe("geo-11", "works-now", "yes");
    assert_eq!(
        plan(LinkState::Ready, true, Some(&g)),
        Plan::ThreeD {
            route: Route::Geo11
        }
    );
}

#[test]
fn no_recipe_is_flat() {
    assert_eq!(
        plan(LinkState::Ready, true, None),
        Plan::Flat {
            reason: FlatReason::NoRecipe
        }
    );
}

#[test]
fn a_switch_that_is_off_is_flat() {
    let r = recipe("uevr + vrto3d", "works-now", "yes");
    assert_eq!(
        plan(LinkState::Ready, false, Some(&r)),
        Plan::Flat {
            reason: FlatReason::SwitchOff
        }
    );
}

#[test]
fn a_link_that_is_not_ready_is_flat() {
    let r = recipe("uevr + vrto3d", "works-now", "yes");
    for s in [
        LinkState::NoHeadset,
        LinkState::Unauthorised,
        LinkState::Starting,
        LinkState::Stopped,
    ] {
        assert_eq!(
            plan(s, true, Some(&r)),
            Plan::Flat {
                reason: FlatReason::LinkNotReady
            }
        );
    }
}

#[test]
fn an_uninjectable_recipe_is_flat_with_the_reason() {
    let r = recipe("uevr + vrto3d", "likely", "yes");
    match plan(LinkState::Ready, true, Some(&r)) {
        Plan::Flat {
            reason: FlatReason::NotInjectable(m),
        } => assert!(m.contains("works-now"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_reasons_are_checked_in_order() {
    let bad = recipe("none", "likely", "no");
    assert_eq!(
        plan(LinkState::Stopped, false, None),
        Plan::Flat {
            reason: FlatReason::NoRecipe
        }
    );
    assert_eq!(
        plan(LinkState::Stopped, false, Some(&bad)),
        Plan::Flat {
            reason: FlatReason::SwitchOff
        }
    );
    assert_eq!(
        plan(LinkState::Stopped, true, Some(&bad)),
        Plan::Flat {
            reason: FlatReason::LinkNotReady
        }
    );
}

#[test]
fn the_log_text() {
    assert_eq!(FlatReason::NoRecipe.text(), "no recipe");
    assert_eq!(FlatReason::SwitchOff.text(), "3D switch is off");
    assert_eq!(FlatReason::LinkNotReady.text(), "headset link is not ready");
    assert_eq!(
        FlatReason::NotInjectable("because".into()).text(),
        "because"
    );
}
