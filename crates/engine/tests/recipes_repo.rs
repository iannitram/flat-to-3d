use flat3d_inject::injectors::fetch::{self, FetchError, Pin};
use flat3d_inject::injectors::{check_injectable, route_of, Route};
use flat3d_inject::recipes::{check_route_settings, load_recipes, route_settings, RouteSettings};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn pin_settings(route: Route, settings: &RouteSettings) -> Vec<(&'static str, String)> {
    let mut refs = Vec::new();
    match route {
        Route::Uevr => {
            refs.push(("stereo-route", "uevr".to_string()));
            if let Some(profile) = settings.get("uevr-profile") {
                refs.push(("uevr-profile", profile.clone()));
            }
        }
        Route::Geo11 => {
            refs.push(("stereo-route", "geo11".to_string()));
            if let Some(fix) = settings.get("geo11-fix") {
                refs.push(("geo11-fix", fix.clone()));
            }
        }
        Route::Port | Route::NoRoute => {}
    }
    refs
}

#[test]
fn every_repo_recipe_loads() {
    if let Err(error) = load_recipes(&root().join("recipes")) {
        panic!("recipes/*.recipe: {error}; fix the named recipe");
    }
}

#[test]
fn file_names_are_appid_slug() {
    let dir = root().join("recipes");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "recipe") {
            let file = path.file_name().unwrap().to_string_lossy();
            let stem = file.strip_suffix(".recipe").unwrap_or("");
            let valid = stem.split_once('-').is_some_and(|(appid, slug)| {
                !appid.is_empty()
                    && appid.bytes().all(|byte| byte.is_ascii_digit())
                    && !slug.is_empty()
                    && slug.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            });
            assert!(
                valid,
                "recipes/{file}: name must be <appid>-<slug>.recipe (slug is a-z, 0-9, -); rename it"
            );
        }
    }
}

#[test]
fn works_now_recipes_are_injectable() {
    let dir = root().join("recipes");
    let loaded = load_recipes(&dir)
        .unwrap_or_else(|error| panic!("recipes/*.recipe: {error}; fix the named recipe"));
    let pins = fetch::parse_pins(fetch::PINS);
    for (appid, recipe) in loaded
        .iter()
        .filter(|(_, recipe)| recipe.status == "works-now")
    {
        let path = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{appid}-"))
            })
            .unwrap_or_else(|| {
                panic!("recipes/{appid}-*.recipe: source file missing; restore recipe")
            })
            .path();
        let file = path.file_name().unwrap().to_string_lossy();
        if let Err(error) = check_injectable(recipe) {
            panic!("recipes/{file}: {error}; fix stereo status, safety or route");
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("recipes/{file}: cannot read recipe: {error}; fix file")
        });
        let settings = route_settings(&text);
        if let Err(error) = check_route_settings(&recipe.route, &settings) {
            panic!("recipes/{file}: {error}; fix the route settings");
        }

        let route = route_of(&recipe.route);
        let refs = pin_refs(route, &settings);
        let missing = missing_pins(&refs, &pins);
        let settings_with_pins = pin_settings(route, &settings);
        if let Some(key) = missing.first() {
            let setting = settings_with_pins
                .iter()
                .find(|(_, pin)| pin == key)
                .map_or("stereo-route", |(setting, _)| *setting);
            panic!("recipes/{file}: {setting} names pin {key}, which packaging/pins does not hold as a fetch:auto key; fix the key or add the pin (WP-02)");
        }
        for (setting, key) in settings_with_pins {
            match fetch::pin_in(&pins, &key) {
                Ok(_) => {}
                Err(FetchError::TbdAtPin(_)) => panic!(
                    "recipes/{file}: {setting} pin {key} is TBD-at-pin; run packaging/pin.sh {key}"
                ),
                Err(error) => {
                    panic!("recipes/{file}: {setting} pin {key} is unusable ({error}); fix the pin")
                }
            }
        }
    }
}
