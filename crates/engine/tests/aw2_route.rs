use flat3d_inject::injectors::{aw2, route_of, Route};
use flat3d_inject::undo::{self, UndoRecord};
use std::fs;
use std::path::{Path, PathBuf};

const APPID: u32 = 302;
const ORIGINAL: &[u8] = b"{\r\n  \"m_eWindowMode\": 1,\r\n  \"m_bFilmGrain\": true\r\n}\r\n";

fn root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("flat3d-aw2-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn setup(label: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
    let root = root(label);
    let game = root.join("game");
    let prefix = root.join("prefix");
    let renderer =
        prefix.join("drive_c/users/steamuser/AppData/Local/Remedy/AlanWake2/renderer.ini");
    let proxy_source = root.join("source/version.dll");
    fs::create_dir_all(renderer.parent().unwrap()).unwrap();
    fs::create_dir_all(&game).unwrap();
    fs::create_dir_all(proxy_source.parent().unwrap()).unwrap();
    fs::write(&renderer, ORIGINAL).unwrap();
    fs::write(game.join("AlanWake2.exe"), b"game exe").unwrap();
    fs::write(&proxy_source, b"proxy dll").unwrap();
    (root.clone(), game, prefix, renderer, proxy_source)
}

fn tree(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    fn visit(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.push((path.strip_prefix(root).unwrap().to_path_buf(), None));
                visit(root, &path, out);
            } else {
                out.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    Some(fs::read(&path).unwrap()),
                ));
            }
        }
    }
    let mut out = Vec::new();
    visit(root, root, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn apply_and_record(data: &Path, game: &Path, prefix: &Path, source: &Path) -> UndoRecord {
    let record = aw2::apply(APPID, game, prefix, source).unwrap();
    undo::save_record(data, &record).unwrap();
    record
}

#[test]
fn aw2_apply_undo_restores_renderer_and_game_folder_byte_for_byte() {
    // Catches: edits that do not use the shared undo record or leave backups/proxy files behind.
    let (root, game, prefix, renderer, source) = setup("roundtrip");
    let before_game = tree(&game);
    let before_renderer = fs::read(&renderer).unwrap();
    let data = root.join("data");
    apply_and_record(&data, &game, &prefix, &source);
    undo::undo_game(&data, APPID).unwrap();
    assert_eq!(tree(&game), before_game);
    assert_eq!(fs::read(renderer).unwrap(), before_renderer);
}

#[test]
fn aw2_apply_twice_keeps_one_record_and_one_installed_proxy() {
    // Catches: a repeat apply replacing its own recovery point or adding duplicate changes.
    let (root, game, prefix, _renderer, source) = setup("twice");
    let data = root.join("data");
    let first = apply_and_record(&data, &game, &prefix, &source);
    let after_first = tree(&game);
    let second = aw2::apply(APPID, &game, &prefix, &source).unwrap();
    assert!(second.created.is_empty() && second.replaced.is_empty());
    assert_eq!(tree(&game), after_first);
    assert_eq!(
        fs::read(undo::record_path(&data, APPID)).unwrap(),
        first.to_text().as_bytes()
    );
}

#[test]
fn aw2_undo_restores_a_proxy_that_was_already_in_the_game_folder() {
    // Catches: replacing the game's original version.dll without retaining its bytes.
    let (root, game, prefix, _renderer, source) = setup("replace-proxy");
    let proxy = game.join("version.dll");
    fs::write(&proxy, b"original version dll").unwrap();
    let before = tree(&game);
    let data = root.join("data");
    apply_and_record(&data, &game, &prefix, &source);
    assert_eq!(fs::read(&proxy).unwrap(), b"proxy dll");
    undo::undo_game(&data, APPID).unwrap();
    assert_eq!(tree(&game), before);
}

#[test]
fn aw2_undo_without_a_record_is_not_found_and_changes_nothing() {
    // Catches: undo mutating a game without a recorded apply.
    let (root, game, _prefix, _renderer, _source) = setup("no-record");
    let before = tree(&game);
    let error = undo::undo_game(&root.join("data"), APPID).unwrap_err();
    assert_eq!(error.code, "not-found");
    assert_eq!(tree(&game), before);
}

#[test]
fn aw2_adds_missing_renderer_keys_and_undo_restores_the_original_bytes() {
    // Catches: partial renderer files being rejected or newly added keys surviving undo.
    let (root, game, prefix, renderer, source) = setup("missing-keys");
    fs::write(&renderer, b"{\"m_eWindowMode\":1}\n").unwrap();
    let original = fs::read(&renderer).unwrap();
    let data = root.join("data");
    apply_and_record(&data, &game, &prefix, &source);
    let applied: serde_json::Value = serde_json::from_slice(&fs::read(&renderer).unwrap()).unwrap();
    for key in [
        "m_eVSync",
        "m_eMotionBlur",
        "m_bFilmGrain",
        "m_bLensDistortion",
        "m_eFSRFGMode",
        "m_eDLSSFGMode",
    ] {
        assert!(applied.get(key).is_some(), "missing {key}");
    }
    assert_eq!(applied["m_eVSync"], 0);
    assert_eq!(applied["m_eMotionBlur"], 0);
    assert!(!applied["m_bFilmGrain"].as_bool().unwrap());
    assert!(!applied["m_bLensDistortion"].as_bool().unwrap());
    assert_eq!(applied["m_eFSRFGMode"], 0);
    assert_eq!(applied["m_eDLSSFGMode"], 0);
    undo::undo_game(&data, APPID).unwrap();
    assert_eq!(fs::read(renderer).unwrap(), original);
}

#[test]
fn aw2_missing_proxy_source_fails_before_changing_anything() {
    // Catches: renderer or game-folder changes before the proxy source is validated.
    let (root, game, prefix, renderer, _source) = setup("missing-source");
    let before_game = tree(&game);
    let before_renderer = fs::read(&renderer).unwrap();
    let error = aw2::apply(APPID, &game, &prefix, &root.join("missing/version.dll")).unwrap_err();
    assert!(error.contains("AW2 proxy DLL is missing"), "{error}");
    assert_eq!(tree(&game), before_game);
    assert_eq!(fs::read(renderer).unwrap(), before_renderer);
}

#[test]
fn aw2_route_is_parsed_and_accepts_no_extra_route_settings() {
    // Catches: recipes naming the W1 AW2 route being treated as an unknown route.
    assert_eq!(route_of("aw2 alternate-eye route"), Route::Aw2);
    assert_eq!(route_of("Alan Wake 2 alternate-eye route"), Route::Aw2);
    assert!(flat3d_inject::recipes::check_route_settings("aw2", &Default::default()).is_ok());
}
