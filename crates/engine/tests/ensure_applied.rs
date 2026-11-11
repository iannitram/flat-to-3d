use flat3d_inject::testing::FakeRunner;
use flat3d_inject::{
    injectors::{self, Route, RouteEnv},
    paths::EnginePaths,
    recipes::{Recipe, RouteSettings},
    steam::Game,
    undo,
};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[path = "support/uevr_home.rs"]
mod uevr_home;

fn recipe(route: &str) -> Recipe {
    Recipe {
        appid: 2075800,
        name: "Game".into(),
        status: "works-now".into(),
        route: route.into(),
        engine: "custom".into(),
        api: "unknown".into(),
        mod_url: "none".into(),
        free: "yes".into(),
        safe: "yes".into(),
        checked: "2026-11-11".into(),
    }
}
fn ready(case: &str) -> (PathBuf, EnginePaths, Game) {
    let (root, p, game) = uevr_home::uevr_ready(case);
    let pin = flat3d_inject::injectors::fetch::pin("uevr").unwrap();
    let dir = flat3d_inject::injectors::fetch::cache_dir(&pin, &p);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(flat3d_inject::injectors::fetch::MARKER),
        format!("{}\n", pin.sha256),
    )
    .unwrap();
    (root, p, game)
}

#[test]
fn reuse_is_identical_and_does_not_call_runner() {
    let (_root, p, game) = ready("ensure-reuse");
    let rec = recipe("uevr");
    let settings = BTreeMap::from([("uevr-exe".into(), "Game".into())]);
    let mut runner = FakeRunner::new();
    let first = injectors::ensure_applied(&game, &rec, &settings, &p, &mut runner).unwrap();
    let ran = runner.ran.clone();
    let second = injectors::ensure_applied(&game, &rec, &settings, &p, &mut runner).unwrap();
    assert_eq!(first, second);
    assert_eq!(runner.ran, ran);
}

#[test]
fn settings_change_undoes_then_reapplies_and_updates_env() {
    let (_root, p, game) = ready("ensure-stale");
    let rec = recipe("uevr");
    let old = BTreeMap::from([("uevr-exe".into(), "Game".into())]);
    let mut runner = FakeRunner::new();
    injectors::ensure_applied(&game, &rec, &old, &p, &mut runner).unwrap();
    let new = BTreeMap::from([
        ("uevr-exe".into(), "Game".into()),
        ("uevr-delay".into(), "20".into()),
    ]);
    injectors::ensure_applied(&game, &rec, &new, &p, &mut runner).unwrap();
    let text = fs::read_to_string(p.env_file(game.appid)).unwrap();
    assert!(text.contains("setting\tuevr-delay\t20\n"));
    assert_eq!(injectors::parse_env_file(&text).unwrap().1, new);
    assert!(undo::record_path(&p.data, game.appid).exists());
}

#[test]
fn port_route_is_tagged_not_built_and_errors_are_codes() {
    let (_root, p, game) = ready("ensure-port");
    let mut runner = FakeRunner::new();
    let err = injectors::ensure_applied(
        &game,
        &recipe("build me"),
        &RouteSettings::new(),
        &p,
        &mut runner,
    )
    .unwrap_err();
    assert!(err.starts_with("not-built: "));
    assert!(flat3d_inject::api::ERROR_CODES
        .iter()
        .any(|c| err.starts_with(&format!("{c}: "))));
}

#[test]
fn env_format_round_trips_escaped_fields() {
    let settings = BTreeMap::from([("uevr-exe".into(), "tab\tline\nslash\\".into())]);
    let env = RouteEnv {
        env: vec![("K\t".into(), "V\n\\".into())],
        unset: vec!["U".into()],
        sidecar: Some(vec!["arg\t\n\\".into()]),
    };
    let text = injectors::env_file_text(Route::Uevr, &settings, &env);
    assert_eq!(
        injectors::parse_env_file(&text).unwrap(),
        (Route::Uevr, settings, env)
    );
}
