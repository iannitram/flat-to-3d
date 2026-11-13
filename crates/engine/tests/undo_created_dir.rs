//! The `created-dir` line of the undo record (owner ruling 2026-11-12, `docs/notes/DECISIONS.md`
//! ruling 54): the folders an apply made are listed, and undo removes them after the files,
//! deepest first and only when empty. A record without the line reads and undoes as before.

#[path = "support/uevr_home.rs"]
mod uevr_home;

use flat3d_inject::injectors::{geo11, uevr};
use flat3d_inject::recipes::RouteSettings;
use flat3d_inject::undo::{record_path, undo_actions, undo_game, Replaced, UndoAction, UndoRecord};
use std::fs;
use std::path::{Path, PathBuf};

fn root(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("undo-created-dir")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn put(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn settings(pairs: &[(&str, &str)]) -> RouteSettings {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// Write `rec` where `undo <appid>` looks for it.
fn save(data: &Path, rec: &UndoRecord) {
    let path = record_path(data, rec.appid);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, rec.to_text()).unwrap();
}

/// Every folder under `root`, sorted, as paths relative to it.
fn folders(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
                walk(root, &path, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

fn old_sample() -> UndoRecord {
    UndoRecord {
        appid: 2651280,
        launch_options_before: "-novid".into(),
        created: vec![PathBuf::from("/g/a.dll")],
        replaced: vec![Replaced {
            path: PathBuf::from("/g/d3d11.dll"),
            backup: PathBuf::from("/g/d3d11.dll.flat3d-backup"),
        }],
        created_dirs: vec![],
    }
}

const OLD_TEXT: &str = "flat3d-undo 1\nappid\t2651280\nlaunch-options\t-novid\n\
created\t/g/a.dll\nreplaced\t/g/d3d11.dll\t/g/d3d11.dll.flat3d-backup\n";

#[test]
fn the_created_dir_line_round_trips() {
    let mut r = old_sample();
    r.created_dirs = vec![
        PathBuf::from("/g/ShaderFixes"),
        PathBuf::from("/g/we\tird\nname\\x"),
    ];
    let text = r.to_text();
    assert_eq!(
        text,
        format!("{OLD_TEXT}created-dir\t/g/ShaderFixes\ncreated-dir\t/g/we\\tird\\nname\\\\x\n")
    );
    assert_eq!(UndoRecord::from_text(&text), Ok(r));
}

#[test]
fn an_old_record_without_the_line_reads_and_lists_the_same_steps() {
    let r = UndoRecord::from_text(OLD_TEXT).unwrap();
    assert_eq!(r, old_sample());
    assert!(r.created_dirs.is_empty());
    assert_eq!(r.to_text(), OLD_TEXT);
    assert_eq!(
        undo_actions(&r),
        vec![
            UndoAction::RemoveFile(PathBuf::from("/g/a.dll")),
            UndoAction::RestoreFile {
                backup: PathBuf::from("/g/d3d11.dll.flat3d-backup"),
                path: PathBuf::from("/g/d3d11.dll"),
            },
            UndoAction::SetLaunchOptions("-novid".into()),
        ]
    );
}

#[test]
fn an_old_record_undoes_on_disk_exactly_as_before() {
    let r = root("old-on-disk");
    let game = r.join("game");
    put(game.join("sub/a.dll"), "ours");
    let rec = UndoRecord {
        appid: 31,
        launch_options_before: String::new(),
        created: vec![game.join("sub/a.dll")],
        replaced: vec![],
        created_dirs: vec![],
    };
    let data = r.join("data");
    save(&data, &rec);
    let got = undo_game(&data, 31).unwrap();
    assert_eq!(
        got.restored,
        vec![format!("removed {}", game.join("sub/a.dll").display())]
    );
    assert!(got.summary.contains("1 change(s)"));
    // Without a created-dir line no folder is touched.
    assert!(game.join("sub").is_dir());
    assert!(!record_path(&data, 31).exists());
}

#[test]
fn folders_come_after_the_files_deepest_first_before_the_launch_options() {
    let mut r = old_sample();
    // Recorded out of order on purpose: depth decides, not the line order.
    r.created_dirs = vec![
        PathBuf::from("/g/a/b"),
        PathBuf::from("/g/a"),
        PathBuf::from("/g/a/b/c"),
    ];
    let got = undo_actions(&r);
    assert_eq!(got.len(), 6);
    assert!(matches!(got[0], UndoAction::RemoveFile(_)));
    assert!(matches!(got[1], UndoAction::RestoreFile { .. }));
    assert_eq!(got[2], UndoAction::RemoveDir(PathBuf::from("/g/a/b/c")));
    assert_eq!(got[3], UndoAction::RemoveDir(PathBuf::from("/g/a/b")));
    assert_eq!(got[4], UndoAction::RemoveDir(PathBuf::from("/g/a")));
    assert_eq!(got[5], UndoAction::SetLaunchOptions("-novid".into()));
}

#[test]
fn nested_created_levels_are_all_removed_deepest_first() {
    let r = root("nested");
    let (a, b, c) = (r.join("a"), r.join("a/b"), r.join("a/b/c"));
    put(c.join("f.txt"), "ours");
    let rec = UndoRecord {
        appid: 32,
        launch_options_before: String::new(),
        created: vec![c.join("f.txt")],
        replaced: vec![],
        created_dirs: vec![a.clone(), b.clone(), c.clone()],
    };
    let data = r.join("data");
    save(&data, &rec);
    let got = undo_game(&data, 32).unwrap();
    assert_eq!(
        got.restored,
        vec![
            format!("removed {}", c.join("f.txt").display()),
            format!("removed folder {}", c.display()),
            format!("removed folder {}", b.display()),
            format!("removed folder {}", a.display()),
        ]
    );
    assert!(!a.exists());
    assert!(got.summary.contains("4 change(s)"));
}

#[test]
fn a_created_folder_that_is_not_empty_is_kept_and_named() {
    let r = root("kept");
    let (a, b) = (r.join("a"), r.join("a/b"));
    put(b.join("ours.dll"), "ours");
    put(b.join("users-own.txt"), "not ours");
    let rec = UndoRecord {
        appid: 33,
        launch_options_before: String::new(),
        created: vec![b.join("ours.dll")],
        replaced: vec![],
        created_dirs: vec![a.clone(), b.clone()],
    };
    let data = r.join("data");
    save(&data, &rec);
    let got = undo_game(&data, 33).unwrap();
    assert_eq!(
        got.restored,
        vec![
            format!("removed {}", b.join("ours.dll").display()),
            format!("kept {} (not empty)", b.display()),
            format!("kept {} (not empty)", a.display()),
        ]
    );
    assert_eq!(fs::read(b.join("users-own.txt")).unwrap(), b"not ours");
    // The undo still finishes: the record is gone and a second undo has nothing to do.
    assert!(!record_path(&data, 33).exists());
    assert!(got.summary.contains("1 change(s)"));
}

#[test]
fn a_created_folder_already_gone_counts_as_done() {
    let r = root("dir-gone");
    let gone = r.join("gone");
    let rec = UndoRecord {
        appid: 34,
        launch_options_before: String::new(),
        created: vec![],
        replaced: vec![],
        created_dirs: vec![gone.clone()],
    };
    let data = r.join("data");
    save(&data, &rec);
    let got = undo_game(&data, 34).unwrap();
    assert_eq!(
        got.restored,
        vec![format!("already gone {}", gone.display())]
    );
}

// --- UEVR ---

fn uevr_profile_dir(game: &flat3d_inject::steam::Game) -> PathBuf {
    uevr::profile_dir(&compatdata(game), game.appid, "SWZeroCompany")
}

fn compatdata(game: &flat3d_inject::steam::Game) -> PathBuf {
    game.install_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("compatdata")
}

#[test]
fn uevr_apply_then_undo_leaves_no_folder_it_made() {
    let (root, p, game) = uevr_home::uevr_ready("undo-created-dir-uevr");
    let library = game.install_dir.parent().unwrap().parent().unwrap();
    let before = folders(library);
    let (applied, _) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap();
    let profile_dir = uevr_profile_dir(&game);
    assert!(profile_dir.is_dir());
    // The fake library has no compatdata yet, so every level from there down was made.
    let made = applied.record.created_dirs.clone();
    assert_eq!(made.first(), Some(&compatdata(&game)));
    assert_eq!(made.last(), Some(&profile_dir));
    assert_eq!(
        made.len(),
        profile_dir.components().count() - library.components().count()
    );
    save(&p.data, &applied.record);
    undo_game(&p.data, game.appid).unwrap();
    assert_eq!(folders(library), before);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn uevr_never_removes_a_prefix_folder_that_was_there_before() {
    let (root, p, game) = uevr_home::uevr_ready("undo-created-dir-uevr-prefix");
    let profile_dir = uevr_profile_dir(&game);
    let unrealvrmod = profile_dir.parent().unwrap().to_path_buf();
    let roaming = unrealvrmod.parent().unwrap().to_path_buf();
    fs::create_dir_all(&roaming).unwrap();
    let (applied, _) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap();
    assert_eq!(
        applied.record.created_dirs,
        vec![unrealvrmod.clone(), profile_dir.clone()]
    );
    save(&p.data, &applied.record);
    undo_game(&p.data, game.appid).unwrap();
    assert!(roaming.is_dir());
    assert!(!unrealvrmod.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn uevr_an_existing_profile_folder_is_kept() {
    let (root, p, game) = uevr_home::uevr_ready("undo-created-dir-uevr-own");
    let profile_dir = uevr_profile_dir(&game);
    put(profile_dir.join(uevr::PROFILE_FILE), "the game's own");
    let (applied, _) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap();
    assert!(applied.record.created_dirs.is_empty());
    save(&p.data, &applied.record);
    undo_game(&p.data, game.appid).unwrap();
    assert_eq!(
        fs::read(profile_dir.join(uevr::PROFILE_FILE)).unwrap(),
        b"the game's own"
    );
    let _ = fs::remove_dir_all(root);
}

// --- Geo-11 ---

fn viewer_exe(r: &Path) -> PathBuf {
    let viewer = r.join("viewer");
    put(&viewer, "");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o755)).unwrap();
    }
    viewer
}

/// A fix with a ShaderFixes folder, a two-level nested folder and a `bin/` file, plus the
/// Geo-11 overlay. The game already has a `bin/` folder of its own.
fn geo11_game(r: &Path) -> (PathBuf, PathBuf, RouteSettings) {
    let cache = r.join("cache");
    let game = r.join("game");
    for arch in ["x64", "x32"] {
        let v = cache.join("geo11/geo-11_v0.7.12").join(arch);
        put(v.join("d3d11.dll"), arch);
        put(v.join("nvapi64.dll"), arch);
        put(
            v.join("d3dxdm.ini"),
            "direct_mode=sbs\ndm_separation=30\ndm_convergence=1.0\n",
        );
    }
    put(cache.join("dir-fix/d3dx.ini"), "[Stereo]\nx = 0\n");
    put(cache.join("dir-fix/d3d11.dll"), "fix");
    put(cache.join("dir-fix/ShaderFixes/a-ps.txt"), "shader");
    put(cache.join("dir-fix/deep/er/f.txt"), "deep");
    put(cache.join("dir-fix/bin/new.dll"), "new");
    put(game.join("Game.exe"), "exe");
    put(game.join("bin/own.dll"), "the game's own");
    let m = settings(&[("geo11-fix", "dir-fix"), ("geo11-exe", "Game.exe")]);
    (cache, game, m)
}

#[test]
fn geo11_apply_then_undo_leaves_no_folder_it_made() {
    let r = root("geo11");
    let (cache, game, m) = geo11_game(&r);
    let before = folders(&game);
    let (applied, _) = geo11::apply(&cache, &game, 41, &m, &viewer_exe(&r)).unwrap();
    let mut made = applied.record.created_dirs.clone();
    made.sort();
    assert_eq!(
        made,
        vec![
            game.join("ShaderFixes"),
            game.join("deep"),
            game.join("deep/er"),
        ]
    );
    let data = r.join("data");
    save(&data, &applied.record);
    undo_game(&data, 41).unwrap();
    assert_eq!(folders(&game), before);
    // The game's own folder stays, with its own file.
    assert_eq!(
        fs::read(game.join("bin/own.dll")).unwrap(),
        b"the game's own"
    );
    assert!(!game.join("bin/new.dll").exists());
}

#[test]
fn geo11_a_made_folder_the_game_wrote_into_is_kept_and_named() {
    let r = root("geo11-kept");
    let (cache, game, m) = geo11_game(&r);
    let (applied, _) = geo11::apply(&cache, &game, 42, &m, &viewer_exe(&r)).unwrap();
    // The game wrote a shader cache file of its own next to ours.
    put(game.join("ShaderFixes/cache.bin"), "the game's");
    let data = r.join("data");
    save(&data, &applied.record);
    let got = undo_game(&data, 42).unwrap();
    assert!(got.restored.contains(&format!(
        "kept {} (not empty)",
        game.join("ShaderFixes").display()
    )));
    assert_eq!(
        fs::read(game.join("ShaderFixes/cache.bin")).unwrap(),
        b"the game's"
    );
    assert!(!game.join("ShaderFixes/a-ps.txt").exists());
    assert!(!game.join("deep").exists());
}

#[test]
fn geo11_rollback_removes_the_folders_it_made() {
    let r = root("geo11-rollback");
    let (cache, game, m) = geo11_game(&r);
    // A folder where a later file must go makes the apply fail after the folders are made.
    put(cache.join("dir-fix/zzz.dll"), "late");
    fs::create_dir_all(game.join("zzz.dll")).unwrap();
    let before = folders(&game);
    let err = geo11::apply(&cache, &game, 43, &m, &viewer_exe(&r)).unwrap_err();
    assert!(err.starts_with("io: "), "{err}");
    assert_eq!(folders(&game), before);
}
