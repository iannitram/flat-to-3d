//! Locked tests for E4a: which stereo route a recipe names, and when injecting is refused.
//! Recipe `stereo-route` values look like `uevr + vrto3d`, `geo-11`, `build: port spidy stereo
//! screen`, `none` (docs/notes/DECISIONS.md, "Recipe keys").

use flat3d_inject::injectors::{check_injectable, route_of, Route};
use flat3d_inject::recipes::Recipe;

fn recipe(route: &str, status: &str, safe: &str) -> Recipe {
    Recipe {
        appid: 2651280,
        name: "Marvel's Spider-Man 2".into(),
        status: status.into(),
        route: route.into(),
        engine: "custom".into(),
        api: "dx12".into(),
        mod_url: "none".into(),
        free: "yes".into(),
        safe: safe.into(),
        checked: "2026-10-31".into(),
    }
}

#[test]
fn a_build_route_is_the_port_route() {
    assert_eq!(route_of("build: port spidy stereo screen"), Route::Port);
    assert_eq!(route_of("Build: something else"), Route::Port);
}

#[test]
fn a_route_naming_uevr_is_the_uevr_route() {
    assert_eq!(route_of("uevr + vrto3d"), Route::Uevr);
    assert_eq!(route_of("UEVR"), Route::Uevr);
}

#[test]
fn a_route_naming_geo_11_is_the_geo11_route() {
    assert_eq!(route_of("geo-11"), Route::Geo11);
    assert_eq!(route_of("Geo-11 + game fix"), Route::Geo11);
}

#[test]
fn none_empty_and_unknown_routes_are_no_route() {
    assert_eq!(route_of("none"), Route::NoRoute);
    assert_eq!(route_of(""), Route::NoRoute);
    assert_eq!(route_of("reshade depth3d"), Route::NoRoute);
}

#[test]
fn build_wins_over_a_tool_name_in_the_same_route() {
    assert_eq!(route_of("build: uevr hook"), Route::Port);
}

#[test]
fn a_safe_works_now_recipe_with_a_route_is_injectable() {
    assert_eq!(check_injectable(&recipe("build: port spidy stereo screen", "works-now", "yes")), Ok(()));
}

#[test]
fn an_unsafe_recipe_is_refused_with_this_message() {
    let r = recipe("uevr", "works-now", "no");
    assert_eq!(
        check_injectable(&r),
        Err("Marvel's Spider-Man 2 is not safe to inject (stereo-safe is \"no\")".to_string())
    );
}

#[test]
fn a_recipe_that_is_not_works_now_is_refused_with_this_message() {
    let r = recipe("uevr", "likely", "yes");
    assert_eq!(
        check_injectable(&r),
        Err("Marvel's Spider-Man 2 is not works-now (stereo-status is \"likely\")".to_string())
    );
}

#[test]
fn a_recipe_with_no_route_is_refused_with_this_message() {
    let r = recipe("none", "works-now", "yes");
    assert_eq!(
        check_injectable(&r),
        Err("Marvel's Spider-Man 2 has no stereo route (stereo-route is \"none\")".to_string())
    );
}

#[test]
fn unsafe_is_reported_before_status_and_status_before_route() {
    let both = recipe("none", "build", "no");
    assert!(check_injectable(&both).unwrap_err().contains("not safe"));
    let status_and_route = recipe("none", "build", "yes");
    assert!(check_injectable(&status_and_route).unwrap_err().contains("not works-now"));
}
