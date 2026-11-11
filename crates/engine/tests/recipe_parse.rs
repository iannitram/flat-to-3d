//! Locked tests for E1a: `recipes::parse_recipe`, one recipe file's text to a `Recipe`.
//! Pure: no files. Value checks (allowed status, api, ...) are E1b, not here.

use flat3d_inject::recipes::{parse_recipe, Recipe, RecipeError};

const GOOD: &str = "\
# Marvel's Spider-Man 2: port of SpidyVR's stereo screen
name: Marvel's Spider-Man 2
stereo-status: works-now
stereo-route: build: port spidy stereo screen
stereo-engine: custom
stereo-api: dx12

stereo-mod: https://github.com/IlyaMez/SpidyVR
stereo-free: yes
stereo-safe: yes
stereo-checked: 2026-10-31
";

fn good_recipe() -> Recipe {
    Recipe {
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
fn a_full_recipe_parses_to_every_field() {
    assert_eq!(parse_recipe(2651280, GOOD), Ok(good_recipe()));
}

#[test]
fn the_appid_comes_from_the_argument() {
    assert_eq!(parse_recipe(7, GOOD).unwrap().appid, 7);
}

#[test]
fn a_value_keeps_colons_after_the_first() {
    let r = parse_recipe(1, GOOD).unwrap();
    assert_eq!(r.route, "build: port spidy stereo screen");
    assert_eq!(r.mod_url, "https://github.com/IlyaMez/SpidyVR");
}

#[test]
fn spaces_around_keys_and_values_are_trimmed() {
    let text = GOOD.replace("stereo-api: dx12", "   stereo-api   :   dx12   ");
    assert_eq!(parse_recipe(1, &text).unwrap().api, "dx12");
}

#[test]
fn one_pair_of_double_quotes_around_a_value_is_removed() {
    let text = GOOD
        .replace("stereo-free: yes", "stereo-free: \"yes\"")
        .replace("stereo-safe: yes", "stereo-safe: \"no\"");
    let r = parse_recipe(1, &text).unwrap();
    assert_eq!(r.free, "yes");
    assert_eq!(r.safe, "no");
}

#[test]
fn comment_and_blank_lines_are_skipped() {
    let text = format!("\n# top comment\n   # indented comment\n\n{GOOD}\n\n");
    assert_eq!(parse_recipe(2651280, &text), Ok(good_recipe()));
}

#[test]
fn an_unknown_key_is_ignored() {
    let text = format!("{GOOD}uevr-profile: zero-company\n");
    assert_eq!(parse_recipe(2651280, &text), Ok(good_recipe()));
}

#[test]
fn values_are_not_checked_here() {
    let text = GOOD.replace("stereo-status: works-now", "stereo-status: maybe");
    assert_eq!(parse_recipe(1, &text).unwrap().status, "maybe");
}

#[test]
fn a_line_without_a_colon_is_a_bad_line_with_its_1_based_number() {
    let text = GOOD.replace("stereo-engine: custom", "stereo-engine custom");
    assert_eq!(
        parse_recipe(1, &text),
        Err(RecipeError::BadLine {
            line: 5,
            text: "stereo-engine custom".into()
        })
    );
}

#[test]
fn a_key_given_twice_is_a_duplicate_key_at_the_second_line() {
    let text = format!("{GOOD}stereo-api: dx11\n");
    assert_eq!(
        parse_recipe(1, &text),
        Err(RecipeError::DuplicateKey {
            key: "stereo-api".into(),
            line: 12
        })
    );
}

#[test]
fn each_missing_key_is_named() {
    let keys = [
        "name",
        "stereo-status",
        "stereo-route",
        "stereo-engine",
        "stereo-api",
        "stereo-mod",
        "stereo-free",
        "stereo-safe",
        "stereo-checked",
    ];
    for key in keys {
        let text: String = GOOD
            .lines()
            .filter(|l| !l.starts_with(&format!("{key}:")))
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(
            parse_recipe(1, &text),
            Err(RecipeError::MissingKey(key.into())),
            "without {key}"
        );
    }
}

#[test]
fn an_error_displays_as_one_line_naming_what_is_wrong() {
    let shown = RecipeError::MissingKey("stereo-api".into()).to_string();
    assert!(shown.contains("stereo-api"), "{shown}");
    assert!(!shown.contains('\n'), "{shown}");
}
