use flat3d_inject::recipes::{load_recipes, load_recipes_keyed, RecipeError, StoreKey};
use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    manifest_dir().join("../..")
}

fn control_seed() -> String {
    let path = manifest_dir().join("tests/fixtures/recipe-seeds/870780-control.recipe");
    fs::read_to_string(&path).expect("fixture recipe must exist")
}

#[test]
fn an_epic_keyed_recipe_loads() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recipe-store-key-epic");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let control_text = control_seed();
    fs::write(dir.join("870780-control.recipe"), &control_text).unwrap();

    let alan_wake_text = "name: Alan Wake 2\n\
         stereo-status: untested\n\
         stereo-route: custom\n\
         stereo-engine: northlight\n\
         stereo-api: dx12\n\
         stereo-mod: none\n\
         stereo-free: yes\n\
         stereo-safe: yes\n\
         stereo-checked: 2026-11-17\n\
         store-key: epic:Hydrangea\n";
    fs::write(dir.join("epic-alan-wake-2.recipe"), alan_wake_text).unwrap();

    let keyed = load_recipes_keyed(&dir).unwrap();
    assert_eq!(keyed.len(), 2);
    let epic_key = StoreKey::Epic("Hydrangea".to_string());
    let steam_key = StoreKey::Steam(870780);
    assert!(keyed.contains_key(&epic_key));
    assert!(keyed.contains_key(&steam_key));
    assert_eq!(keyed.get(&epic_key).unwrap().appid, 0);
    assert_eq!(keyed.get(&steam_key).unwrap().appid, 870780);

    let steam_only = load_recipes(&dir).unwrap();
    assert_eq!(steam_only.len(), 1);
    assert!(steam_only.contains_key(&870780));
    assert_eq!(steam_only.get(&870780), keyed.get(&steam_key));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn every_shipped_steam_recipe_loads_unchanged() {
    let roots = [
        manifest_dir().join("tests/fixtures/recipe-seeds"),
        repo_root().join("recipes"),
    ];
    // recipes/ may hold no recipe yet; the seed fixtures always do, so the total is never 0.
    let mut checked = 0;
    for dir in roots {
        let steam_loaded = load_recipes(&dir).unwrap();
        let keyed_loaded = load_recipes_keyed(&dir).unwrap();
        checked += steam_loaded.len();
        assert_eq!(steam_loaded.len(), keyed_loaded.len());
        for (appid, steam_recipe) in &steam_loaded {
            let key = StoreKey::Steam(*appid);
            let keyed_recipe = keyed_loaded.get(&key).unwrap_or_else(|| {
                panic!("missing key {key} in {}", dir.display());
            });
            assert_eq!(steam_recipe, keyed_recipe);
            assert_eq!(keyed_recipe.appid, *appid);
        }
    }
    assert!(checked > 0, "no shipped Steam recipes found");
}

#[test]
fn an_unknown_store_is_a_clear_error() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recipe-store-key-unknown");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("gog-witcher.recipe"), "name: Witcher\n").unwrap();
    let err = load_recipes_keyed(&dir).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("gog"),
        "expected error containing 'gog', got: {msg}"
    );
    assert!(
        msg.contains("gog-witcher.recipe"),
        "expected error containing file name, got: {msg}"
    );

    let parse_err = "gog:1".parse::<StoreKey>().unwrap_err();
    assert!(
        matches!(parse_err, RecipeError::UnknownStore(_)),
        "expected UnknownStore, got: {parse_err:?}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_non_steam_recipe_without_a_store_key_is_an_error() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("recipe-store-key-missing");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let control_text = control_seed();
    fs::write(dir.join("epic-alan-wake-2.recipe"), &control_text).unwrap();
    let err = load_recipes_keyed(&dir).unwrap_err();
    match err {
        RecipeError::InFile { file, error } => {
            assert_eq!(file, "epic-alan-wake-2.recipe");
            assert_eq!(*error, RecipeError::MissingKey("store-key".to_string()));
        }
        other => panic!("expected InFile(MissingKey), got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_store_key_round_trips() {
    let steam = StoreKey::Steam(870780);
    let steam_display = steam.to_string();
    assert_eq!(steam_display, "steam:870780");
    let parsed_steam: StoreKey = steam_display.parse().unwrap();
    assert_eq!(parsed_steam, steam);

    let epic = StoreKey::Epic("Hydrangea".to_string());
    let epic_display = epic.to_string();
    assert_eq!(epic_display, "epic:Hydrangea");
    let parsed_epic: StoreKey = epic_display.parse().unwrap();
    assert_eq!(parsed_epic, epic);
}
