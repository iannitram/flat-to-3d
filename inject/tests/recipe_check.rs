//! Locked tests for E1b: `recipes::check_recipe`, the allowed values from
//! docs/notes/DECISIONS.md ("Recipe keys"). Pure: no files.

use flat3d_inject::recipes::{check_recipe, Recipe, RecipeError, APIS, FREES, SAFES, STATUSES};

fn good() -> Recipe {
    Recipe {
        appid: 2075800,
        name: "STAR WARS Zero Company".into(),
        status: "works-now".into(),
        route: "uevr: native stereo + 2d screen".into(),
        engine: "unreal-5.6".into(),
        api: "dx12".into(),
        mod_url: "https://github.com/praydog/UEVR".into(),
        free: "yes".into(),
        safe: "yes".into(),
        checked: "2026-10-31".into(),
    }
}

/// `good()` with one change applied.
fn with(change: impl FnOnce(&mut Recipe)) -> Recipe {
    let mut r = good();
    change(&mut r);
    r
}

fn bad(key: &str, value: &str) -> Result<(), RecipeError> {
    let (key, value) = (key.to_string(), value.to_string());
    Err(RecipeError::BadValue { key, value })
}

#[test]
fn the_allowed_value_lists_are_exactly_these() {
    let statuses = [
        "works-now",
        "likely",
        "build",
        "not-safe",
        "wont-work",
        "untested",
    ];
    assert_eq!(STATUSES, statuses);
    assert_eq!(APIS, ["dx9", "dx11", "dx12", "vulkan", "opengl", "unknown"]);
    assert_eq!(FREES, ["yes", "no", "unclear"]);
    assert_eq!(SAFES, ["yes", "no"]);
}

#[test]
fn a_good_recipe_passes() {
    assert_eq!(check_recipe(&good()), Ok(()));
}

#[test]
fn every_allowed_status_api_free_and_safe_passes() {
    for s in STATUSES {
        assert_eq!(check_recipe(&with(|r| r.status = s.into())), Ok(()), "{s}");
    }
    for a in APIS {
        assert_eq!(check_recipe(&with(|r| r.api = a.into())), Ok(()), "{a}");
    }
    for f in FREES {
        assert_eq!(check_recipe(&with(|r| r.free = f.into())), Ok(()), "{f}");
    }
    for s in SAFES {
        assert_eq!(check_recipe(&with(|r| r.safe = s.into())), Ok(()), "{s}");
    }
}

#[test]
fn an_unknown_status_is_a_bad_stereo_status() {
    let r = with(|r| r.status = "maybe".into());
    assert_eq!(check_recipe(&r), bad("stereo-status", "maybe"));
}

#[test]
fn values_are_case_sensitive() {
    let r = with(|r| r.status = "Works-Now".into());
    assert_eq!(check_recipe(&r), bad("stereo-status", "Works-Now"));
}

#[test]
fn an_unknown_api_is_a_bad_stereo_api() {
    let r = with(|r| r.api = "dx10".into());
    assert_eq!(check_recipe(&r), bad("stereo-api", "dx10"));
}

#[test]
fn an_unknown_free_is_a_bad_stereo_free() {
    let r = with(|r| r.free = "maybe".into());
    assert_eq!(check_recipe(&r), bad("stereo-free", "maybe"));
}

#[test]
fn unclear_is_not_allowed_for_safe() {
    let r = with(|r| r.safe = "unclear".into());
    assert_eq!(check_recipe(&r), bad("stereo-safe", "unclear"));
}

#[test]
fn a_checked_date_not_shaped_yyyy_mm_dd_is_a_bad_stereo_checked() {
    for d in [
        "2026-1-31",
        "31-10-2026",
        "2026/10/31",
        "2026-10-31x",
        "today",
        "",
    ] {
        let r = with(|r| r.checked = d.into());
        assert_eq!(check_recipe(&r), bad("stereo-checked", d), "{d:?}");
    }
}

#[test]
fn route_engine_and_mod_are_free_text() {
    let r = with(|r| {
        r.route = "none".into();
        r.engine = String::new();
        r.mod_url = "none".into();
    });
    assert_eq!(check_recipe(&r), Ok(()));
}

#[test]
fn the_first_bad_key_in_order_status_api_free_safe_checked_is_reported() {
    let mut r = with(|r| {
        r.status = "s".into();
        r.api = "a".into();
        r.free = "f".into();
        r.safe = "x".into();
        r.checked = "c".into();
    });
    assert_eq!(check_recipe(&r), bad("stereo-status", "s"));
    r.status = "likely".into();
    assert_eq!(check_recipe(&r), bad("stereo-api", "a"));
    r.api = "dx11".into();
    assert_eq!(check_recipe(&r), bad("stereo-free", "f"));
    r.free = "no".into();
    assert_eq!(check_recipe(&r), bad("stereo-safe", "x"));
    r.safe = "no".into();
    assert_eq!(check_recipe(&r), bad("stereo-checked", "c"));
}
