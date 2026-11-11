use flat3d_inject::injectors::fetch::{self, FetchError, Pin};
use flat3d_inject::injectors::{check_injectable, route_of, Route};
use flat3d_inject::recipes::{check_route_settings, load_recipes, route_settings, RouteSettings};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn seeds() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recipe-seeds")
}

fn pin_refs(route: Route, settings: &RouteSettings) -> Vec<String> {
    let mut refs = Vec::new();
    match route {
        Route::Uevr => {
            refs.push("uevr".to_string());
            if let Some(profile) = settings.get("uevr-profile") {
                refs.push(profile.clone());
            }
        }
        Route::Geo11 => {
            refs.push("geo11".to_string());
            if let Some(fix) = settings.get("geo11-fix") {
                refs.push(fix.clone());
            }
        }
        Route::Port | Route::NoRoute => {}
    }
    refs
}

fn missing_pins(refs: &[String], pins: &BTreeMap<String, Pin>) -> Vec<String> {
    refs.iter()
        .filter(|key| matches!(fetch::pin_in(pins, key), Err(FetchError::UnknownKey(_))))
        .cloned()
        .collect()
}

fn pin_setting(route: Route, settings: &RouteSettings, pin: &str) -> &'static str {
    match route {
        Route::Uevr if settings.get("uevr-profile").is_some_and(|key| key == pin) => "uevr-profile",
        Route::Geo11 if settings.get("geo11-fix").is_some_and(|key| key == pin) => "geo11-fix",
        Route::Uevr | Route::Geo11 => "stereo-route",
        Route::Port | Route::NoRoute => "stereo-route",
    }
}

#[test]
fn the_seeds_are_the_five_games() {
    let loaded = load_recipes(&seeds())
        .unwrap_or_else(|error| panic!("recipe-seeds/*.recipe: {error}; fix the seed recipe"));
    let mut appids: Vec<_> = loaded.keys().copied().collect();
    appids.sort_unstable();
    assert_eq!(
        appids,
        vec![502500, 870780, 1608230, 2075800, 2651280],
        "recipe-seeds/*.recipe: expected exactly the five assigned appids; fix the seed set"
    );
}

#[test]
fn no_seed_is_works_now() {
    let loaded = load_recipes(&seeds()).expect("recipe-seeds/*.recipe: fix invalid seed recipes");
    for (appid, recipe) in loaded {
        let file = format!("recipe-seeds/{appid}-*.recipe");
        assert_eq!(
            recipe.status, "untested",
            "{file}: stereo-status must stay untested until its session passes; fix the seed"
        );
        let error = match check_injectable(&recipe) {
            Ok(()) => panic!(
                "{file}: untested seed is injectable; keep it untested until its session passes"
            ),
            Err(error) => error,
        };
        assert!(
            error.contains("is not works-now"),
            "{file}: check_injectable returned {error}; keep seed untested until its session passes"
        );
    }
}

#[test]
fn each_seed_names_its_route() {
    let loaded = load_recipes(&seeds()).expect("recipe-seeds/*.recipe: fix invalid seed recipes");
    let expected = [
        (2075800, Route::Uevr),
        (502500, Route::Uevr),
        (1608230, Route::Geo11),
        (870780, Route::Geo11),
        (2651280, Route::Port),
    ];
    for (appid, route) in expected {
        let recipe = loaded.get(&appid).unwrap_or_else(|| {
            panic!("recipe-seeds/{appid}-*.recipe: seed is missing; add the assigned fixture")
        });
        assert_eq!(
            route_of(&recipe.route),
            route,
            "recipe-seeds/{appid}-*.recipe: stereo-route names the wrong route; fix the seed route"
        );
    }
}

#[test]
fn route_keys_pass_the_frozen_check() {
    let loaded = load_recipes(&seeds()).expect("recipe-seeds/*.recipe: fix invalid seed recipes");
    for (appid, recipe) in loaded {
        let file = format!("recipe-seeds/{appid}-*.recipe");
        let path = fs::read_dir(seeds())
            .unwrap_or_else(|error| panic!("{file}: cannot read seed folder: {error}; fix fixture"))
            .filter_map(Result::ok)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{appid}-"))
            })
            .unwrap_or_else(|| panic!("{file}: seed file is missing; add the fixture"))
            .path();
        let text = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("{}: cannot read seed: {error}; fix fixture", path.display())
        });
        let settings = route_settings(&text);
        if let Err(error) = check_route_settings(&recipe.route, &settings) {
            panic!(
                "{}: {error}; fix the route key in the seed",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }
}

#[test]
fn pin_refs_name_real_pins() {
    let loaded = load_recipes(&seeds()).expect("recipe-seeds/*.recipe: fix invalid seed recipes");
    let pins = fetch::parse_pins(fetch::PINS);
    let mut all_refs = Vec::new();
    for (appid, recipe) in loaded {
        let path = fs::read_dir(seeds())
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{appid}-"))
            })
            .unwrap()
            .path();
        let file = path.file_name().unwrap().to_string_lossy();
        let text = fs::read_to_string(&path).unwrap();
        let settings = route_settings(&text);
        let refs = pin_refs(route_of(&recipe.route), &settings);
        let missing = missing_pins(&refs, &pins);
        if let Some(key) = missing.first() {
            let setting = pin_setting(route_of(&recipe.route), &settings, key);
            panic!("{file}: {setting} names pin {key}, which packaging/pins does not hold as a fetch:auto key; fix the key or add the pin (WP-02)");
        }
        for key in &refs {
            match fetch::pin_in(&pins, key) {
                Ok(_) | Err(FetchError::TbdAtPin(_)) => {}
                Err(error) => {
                    panic!("{file}: pin {key} is unusable ({error}); fix the pin reference")
                }
            }
        }
        all_refs.extend(refs);
    }
    all_refs.sort();
    all_refs.dedup();
    assert_eq!(
        all_refs,
        vec!["ac7-profile", "control-fix", "geo11", "lana-fix", "uevr"],
        "recipe-seeds/*.recipe: unexpected route pin references; fix seed keys and packaging/pins"
    );
}

#[test]
fn an_unknown_pin_is_refused() {
    let settings = route_settings("uevr-exe: X\nuevr-profile: no-such-pin");
    let pins = fetch::parse_pins(fetch::PINS);
    assert_eq!(
        missing_pins(&pin_refs(Route::Uevr, &settings), &pins),
        vec!["no-such-pin"],
        "recipe-seeds/2075800-star-wars-zero-company.recipe: unknown pin probe must find no-such-pin; fix pin lookup"
    );
}

#[test]
fn seeds_stay_out_of_recipes_until_works_now() {
    let recipes = root().join("recipes");
    let seed_appids = [502500, 870780, 1608230, 2075800, 2651280];
    let loaded = load_recipes(&recipes)
        .unwrap_or_else(|error| panic!("recipes/*.recipe: {error}; fix the repo recipe"));
    for entry in fs::read_dir(&recipes).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "recipe") {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let appid = stem
                .split_once('-')
                .and_then(|(left, _)| left.parse::<u32>().ok());
            if appid.is_some_and(|id| seed_appids.contains(&id)) {
                let id = appid.unwrap();
                let recipe = loaded.get(&id).unwrap_or_else(|| {
                    panic!(
                        "recipes/{}: parsed recipe is missing; fix the repo recipe",
                        path.file_name().unwrap().to_string_lossy()
                    )
                });
                assert_eq!(
                    recipe.status,
                    "works-now",
                    "recipes/{}: stereo-status is {}, not works-now; keep it in crates/engine/tests/fixtures/recipe-seeds/ until a session passes",
                    path.file_name().unwrap().to_string_lossy(),
                    recipe.status
                );
            }
        }
    }
}
