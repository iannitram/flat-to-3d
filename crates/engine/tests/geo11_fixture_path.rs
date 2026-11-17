//! Locked test for the Geo-11 fixture path (refs #52, #51): the whole Geo-11 route through
//! `ensure_applied_in` (pins -> fetch cache -> geo11::apply_in -> env file -> reuse -> undo)
//! on a fake game folder, with a fake pin table and a pre-filled fetch cache, so it runs
//! without the owner's Geo-11 pin. No network, no device: the runner is a FakeRunner that
//! must never be called, and every path is under the test's temp folder.

use flat3d_inject::injectors::{self, fetch, RouteEnv};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::recipes::{Recipe, RouteSettings};
use flat3d_inject::steam::Game;
use flat3d_inject::testing::FakeRunner;
use flat3d_inject::undo;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/uevr_home.rs"]
mod uevr_home;

const GEO11_SHA: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const FIX_SHA: &str = "2222222222222222222222222222222222222222222222222222222222222222";

/// Fake pins. The versions are not the ones in packaging/pins, so the route must look the
/// cache up through this table, not through packaging/pins.
const FAKE_PINS: &str = "\
geo11.fetch: auto
geo11.version: fixture-geo11-1
geo11.url: https://example.invalid/geo11-fixture.7z
geo11.size: 7
geo11.sha256: 1111111111111111111111111111111111111111111111111111111111111111
geo11.license: unstated

control-fix.fetch: auto
control-fix.version: fixture-fix-1
control-fix.url: https://example.invalid/control-fix-fixture.zip
control-fix.size: 9
control-fix.sha256: 2222222222222222222222222222222222222222222222222222222222222222
control-fix.license: unstated
";

fn recipe() -> Recipe {
    Recipe {
        appid: 2075800,
        name: "Game".into(),
        status: "works-now".into(),
        route: "geo-11: sbs".into(),
        engine: "custom".into(),
        api: "dx11".into(),
        mod_url: "none".into(),
        free: "yes".into(),
        safe: "yes".into(),
        checked: "2026-11-17".into(),
    }
}

fn settings() -> RouteSettings {
    BTreeMap::from([
        ("geo11-exe".into(), "Game.exe".into()),
        ("geo11-fix".into(), "control-fix".into()),
        ("geo11-layout".into(), "tab".into()),
        ("geo11-separation".into(), "55".into()),
    ])
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// A fake game folder with a Geo-11 overlay and Control's fix already in the fetch cache,
/// each with the fetch marker naming its fake sha256, and an executable viewer.
fn seeded(case: &str) -> (EnginePaths, Game, BTreeMap<String, fetch::Pin>) {
    let (_root, p, game) = uevr_home::home(case);
    write(&game.install_dir.join("Game.exe"), "exe");
    write(&game.install_dir.join("d3d11.dll"), "game d3d11");
    let pins = fetch::parse_pins(FAKE_PINS);

    let geo11 = fetch::cache_dir(&pins["geo11"], &p);
    assert_eq!(geo11, p.cache().join("geo11").join("fixture-geo11-1"));
    write(&geo11.join(fetch::MARKER), &format!("{GEO11_SHA}\n"));
    write(&geo11.join("3Dmigoto/x64/d3d11.dll"), "geo11 d3d11");
    write(&geo11.join("3Dmigoto/x64/nvapi64.dll"), "geo11 nvapi64");
    write(&geo11.join("3Dmigoto/x64/d3dxdm.ini"), "direct_mode=sbs\n");

    let fix = fetch::cache_dir(&pins["control-fix"], &p);
    assert_eq!(fix, p.cache().join("control-fix").join("fixture-fix-1"));
    write(&fix.join(fetch::MARKER), &format!("{FIX_SHA}\n"));
    write(&fix.join("d3dx.ini"), "[Stereo]\nforce_stereo=0\n");
    write(&fix.join("ShaderFixes/abc-ps.txt"), "shader");
    write(&fix.join("README.txt"), "readme");

    let viewer = p.viewer_bin();
    write(&viewer, "#!/bin/sh\n");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    (p, game, pins)
}

/// Every path below `dir` with its bytes (None for a folder), sorted.
fn tree(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut out = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(d) = dirs.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(dir).unwrap().to_path_buf();
            if path.is_dir() {
                dirs.push(path);
                out.push((rel, None));
            } else {
                out.push((rel, Some(fs::read(&path).unwrap())));
            }
        }
    }
    out.sort();
    out
}

fn file(rel: &str, text: &str) -> (PathBuf, Option<Vec<u8>>) {
    (PathBuf::from(rel), Some(text.as_bytes().to_vec()))
}

fn expected_env(p: &EnginePaths) -> RouteEnv {
    RouteEnv {
        env: vec![("WINEDLLOVERRIDES".into(), "d3d11,nvapi64=n,b".into())],
        unset: Vec::new(),
        sidecar: Some(vec![
            p.viewer_bin().to_string_lossy().into_owned(),
            "--layout".into(),
            "tab".into(),
            "--source".into(),
            "auto".into(),
        ]),
    }
}

#[test]
fn applies_from_a_seeded_cache_with_no_commands() {
    let (p, game, pins) = seeded("geo11-fixture-apply");
    let mut runner = FakeRunner::new();
    let env = injectors::ensure_applied_in(&game, &recipe(), &settings(), &p, &mut runner, &pins)
        .unwrap();
    assert!(runner.ran.is_empty(), "{:?}", runner.ran);
    assert_eq!(env, expected_env(&p));

    let mut want = vec![
        file(".flat3d-applied", ""),
        (PathBuf::from("ShaderFixes"), None),
        file("ShaderFixes/abc-ps.txt", "shader"),
        file("Game.exe", "exe"),
        file("d3d11.dll", "geo11 d3d11"),
        file("d3d11.dll.flat3d-backup", "game d3d11"),
        file("d3dx.ini", "[Stereo]\nforce_stereo=2\n"),
        file(
            "d3dxdm.ini",
            "direct_mode=tab\ndm_separation=55\ndm_convergence=2.0\n",
        ),
        file("nvapi64.dll", "geo11 nvapi64"),
    ];
    want.sort();
    assert_eq!(tree(&game.install_dir), want);
}

#[test]
fn env_file_and_undo_record_describe_the_setup() {
    let (p, game, pins) = seeded("geo11-fixture-env-file");
    let mut runner = FakeRunner::new();
    injectors::ensure_applied_in(&game, &recipe(), &settings(), &p, &mut runner, &pins).unwrap();

    let viewer = p.viewer_bin().to_string_lossy().into_owned();
    let text = fs::read_to_string(p.env_file(game.appid)).unwrap();
    assert_eq!(
        text,
        format!(
            "flat3d-env 1\nroute\tgeo11\n\
             setting\tgeo11-exe\tGame.exe\n\
             setting\tgeo11-fix\tcontrol-fix\n\
             setting\tgeo11-layout\ttab\n\
             setting\tgeo11-separation\t55\n\
             set\tWINEDLLOVERRIDES\td3d11,nvapi64=n,b\n\
             sidecar\t{viewer}\t--layout\ttab\t--source\tauto\n"
        )
    );

    let record = fs::read_to_string(undo::record_path(&p.data, game.appid)).unwrap();
    let record = undo::UndoRecord::from_text(&record).unwrap();
    let g = &game.install_dir;
    assert_eq!(record.appid, 2075800);
    assert_eq!(
        record.created,
        vec![
            g.join(".flat3d-applied"),
            g.join("ShaderFixes/abc-ps.txt"),
            g.join("nvapi64.dll"),
            g.join("d3dx.ini"),
            g.join("d3dxdm.ini"),
        ]
    );
    assert_eq!(record.replaced.len(), 1);
    assert_eq!(record.replaced[0].path, g.join("d3d11.dll"));
    assert_eq!(record.replaced[0].backup, g.join("d3d11.dll.flat3d-backup"));
    assert_eq!(record.created_dirs, vec![g.join("ShaderFixes")]);
}

#[test]
fn second_apply_reuses_the_setup_unchanged() {
    let (p, game, pins) = seeded("geo11-fixture-reuse");
    let mut runner = FakeRunner::new();
    let first = injectors::ensure_applied_in(&game, &recipe(), &settings(), &p, &mut runner, &pins)
        .unwrap();
    let files = tree(&game.install_dir);
    let env_text = fs::read(p.env_file(game.appid)).unwrap();
    let record_text = fs::read(undo::record_path(&p.data, game.appid)).unwrap();
    // Reuse needs no fetch and no cache: with the cache gone, a fresh apply would fail.
    fs::remove_dir_all(p.cache()).unwrap();

    let again = injectors::ensure_applied_in(&game, &recipe(), &settings(), &p, &mut runner, &pins)
        .unwrap();
    assert_eq!(again, first);
    assert!(runner.ran.is_empty(), "{:?}", runner.ran);
    assert_eq!(tree(&game.install_dir), files);
    assert_eq!(fs::read(p.env_file(game.appid)).unwrap(), env_text);
    assert_eq!(
        fs::read(undo::record_path(&p.data, game.appid)).unwrap(),
        record_text
    );
}

#[test]
fn undo_puts_the_game_folder_back_byte_for_byte() {
    let (p, game, pins) = seeded("geo11-fixture-undo");
    let before = tree(&game.install_dir);
    let mut runner = FakeRunner::new();
    injectors::ensure_applied_in(&game, &recipe(), &settings(), &p, &mut runner, &pins).unwrap();
    assert_ne!(tree(&game.install_dir), before);

    undo::undo_game(&p.data, game.appid).unwrap();
    assert_eq!(tree(&game.install_dir), before);
    assert!(!undo::record_path(&p.data, game.appid).exists());
    assert_eq!(
        before,
        vec![file("Game.exe", "exe"), file("d3d11.dll", "game d3d11")]
    );
}

#[test]
fn a_key_missing_from_the_table_is_not_found_and_writes_nothing() {
    let (p, game, mut pins) = seeded("geo11-fixture-missing-pin");
    pins.remove("control-fix");
    let before = tree(&game.install_dir);
    let mut runner = FakeRunner::new();
    let err = injectors::ensure_applied_in(&game, &recipe(), &settings(), &p, &mut runner, &pins)
        .unwrap_err();
    assert_eq!(err, "not-found: unknown pin key control-fix");
    assert!(runner.ran.is_empty());
    assert_eq!(tree(&game.install_dir), before);
    assert!(!undo::record_path(&p.data, game.appid).exists());
    assert!(!p.env_file(game.appid).exists());
}
