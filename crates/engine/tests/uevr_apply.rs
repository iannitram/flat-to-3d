#[path = "support/uevr_home.rs"]
mod uevr_home;

use flat3d_inject::injectors::uevr::{self, Rendering, UevrExtras, UevrSettings};
use flat3d_inject::recipes::RouteSettings;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn settings(pairs: &[(&str, &str)]) -> RouteSettings {
    pairs
        .iter()
        .map(|(k, v)| ((*k).into(), (*v).into()))
        .collect()
}
fn file_tree(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.push((path.strip_prefix(root).unwrap().to_path_buf(), None));
                walk(root, &path, out);
            } else {
                out.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    Some(fs::read(&path).unwrap()),
                ));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}
fn code(err: String, expected: &str) {
    assert!(err.starts_with(&format!("{expected}: ")), "{err}");
}
#[test]
fn full_profile_adds_requested_runtime() {
    let s = UevrSettings {
        rendering: Rendering::NativeStereo,
        screen_2d: false,
    };
    let e = UevrExtras {
        exe_stem: "G".into(),
        delay_s: 15,
        native_stereo_fix: None,
        profile: None,
    };
    assert!(uevr::profile_text_full(&s, &e).starts_with(&uevr::profile_text(&s)));
    assert!(
        uevr::profile_text_full(&s, &e).ends_with("Frontend_RequestedRuntime=openxr_loader.dll\n")
    );
}

#[test]
fn native_stereo_fix_line() {
    let s = UevrSettings {
        rendering: Rendering::NativeStereo,
        screen_2d: false,
    };
    let e = UevrExtras {
        exe_stem: "G".into(),
        delay_s: 15,
        native_stereo_fix: Some(true),
        profile: None,
    };
    assert!(uevr::profile_text_full(&s, &e).contains("VR_NativeStereoFix=true\n"));
    let e = UevrExtras {
        native_stereo_fix: Some(false),
        ..e
    };
    assert!(uevr::profile_text_full(&s, &e).contains("VR_NativeStereoFix=false\n"));
}

#[test]
fn extras_need_uevr_exe() {
    code(
        uevr::extras_from(&RouteSettings::new()).unwrap_err(),
        "bad-recipe",
    );
}

#[test]
fn delay_defaults_to_15_and_parses_30() {
    assert_eq!(
        uevr::extras_from(&settings(&[("uevr-exe", "G.exe")]))
            .unwrap()
            .delay_s,
        15
    );
    assert_eq!(
        uevr::extras_from(&settings(&[("uevr-exe", "G.exe"), ("uevr-delay", "30")]))
            .unwrap()
            .delay_s,
        30
    );
}

#[test]
fn bad_delay_is_an_error() {
    code(
        uevr::extras_from(&settings(&[("uevr-exe", "G.exe"), ("uevr-delay", "121")])).unwrap_err(),
        "bad-recipe",
    );
}

#[test]
fn profile_pin_key_is_carried() {
    assert_eq!(
        uevr::extras_from(&settings(&[
            ("uevr-exe", "G.exe"),
            ("uevr-profile", "ac7-profile")
        ]))
        .unwrap()
        .profile
        .as_deref(),
        Some("ac7-profile")
    );
}

#[test]
fn profile_url_form_is_refused() {
    code(
        uevr::extras_from(&settings(&[
            ("uevr-exe", "G.exe"),
            ("uevr-profile", "https://x abc"),
        ]))
        .unwrap_err(),
        "bad-recipe",
    );
}

#[test]
fn apply_writes_profile_marker_and_records_them() {
    let (_root, p, game) = uevr_home::uevr_ready("apply-created");
    let (applied, env) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap();
    let dir = uevr::profile_dir(
        &game
            .install_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("compatdata"),
        game.appid,
        "SWZeroCompany",
    );
    let profile = dir.join(uevr::PROFILE_FILE);
    let marker = dir.join(uevr::MARKER_FILE);
    assert!(profile.is_file() && marker.is_file());
    assert_eq!(applied.record.created, vec![marker, profile]);
    assert!(applied.record.replaced.is_empty());
    assert_eq!(applied.env, env.env);
}

#[test]
fn existing_profile_is_backed_up_and_recorded_replaced() {
    let (_root, p, game) = uevr_home::uevr_ready("apply-existing");
    let dir = uevr::profile_dir(
        &game
            .install_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("compatdata"),
        game.appid,
        "SWZeroCompany",
    );
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(uevr::PROFILE_FILE), "mine\n").unwrap();
    let (applied, _) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join(uevr::BACKUP_FILE)).unwrap(),
        "mine\n"
    );
    assert_eq!(applied.record.replaced.len(), 1);
    assert_eq!(
        applied.record.replaced[0].path,
        dir.join(uevr::PROFILE_FILE)
    );
}

#[test]
fn second_apply_is_refused_and_changes_nothing() {
    let (_root, p, game) = uevr_home::uevr_ready("apply-twice");
    let route = settings(&[("uevr-exe", "SWZeroCompany.exe")]);
    uevr::apply_with(&game, &route, &p).unwrap();
    let dir = uevr::profile_dir(
        &game
            .install_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("compatdata"),
        game.appid,
        "SWZeroCompany",
    );
    let before = fs::read(dir.join(uevr::PROFILE_FILE)).unwrap();
    let err = uevr::apply_with(&game, &route, &p).unwrap_err();
    code(err.clone(), "refused");
    assert!(err.contains("flat3d-inject undo 2075800"));
    assert_eq!(fs::read(dir.join(uevr::PROFILE_FILE)).unwrap(), before);
}

#[test]
fn game_dir_outside_a_library_is_an_error() {
    let (root, p, mut game) = uevr_home::uevr_ready("apply-outside");
    let before = file_tree(&root);
    game.install_dir = PathBuf::from("/G");
    let err = uevr::apply_with(&game, &settings(&[("uevr-exe", "G.exe")]), &p).unwrap_err();
    code(err, "not-found");
    assert_eq!(file_tree(&root), before);
}

#[test]
fn env_is_exactly_the_debug_cmd() {
    let (_root, p, game) = uevr_home::uevr_ready("apply-env");
    let (applied, env) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap();
    assert_eq!(env.env.len(), 1);
    assert_eq!(env.env[0].0, "PROTON_REMOTE_DEBUG_CMD");
    assert_eq!(
        env.env[0].1,
        uevr::debug_cmd(
            &uevr::injector_path(&p),
            "SWZeroCompany",
            15,
            &uevr::uevr_dir(&p)
        )
    );
    assert_eq!(env.unset, ["XR_RUNTIME_JSON"]);
    assert_eq!(env.sidecar, None);
    assert_eq!(applied.env, env.env);
    let value = &env.env[0].1;
    assert!(value.find("openxr_loader.dll").unwrap() < value.find("UEVRBackend.dll").unwrap());
    assert!(value.starts_with("'/"));
    let first_dll = value.find("--dll 'Z:").unwrap();
    let second_dll = value[first_dll + 1..].find("--dll 'Z:").unwrap() + first_dll + 1;
    assert!(first_dll < second_dll);
    assert!(value[first_dll..].contains("openxr_loader.dll"));
    assert!(value[second_dll..].contains("UEVRBackend.dll"));
}

#[test]
fn missing_dll_writes_nothing() {
    let (_root, p, game) = uevr_home::uevr_unfetched("apply-unfetched");
    let err =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap_err();
    code(err.clone(), "not-found");
    assert!(err.contains("flat3d-inject setup fetch uevr"));
    let dir = uevr::profile_dir(
        &game
            .install_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("compatdata"),
        game.appid,
        "SWZeroCompany",
    );
    assert!(!dir.exists());
}

#[test]
fn missing_injector_is_an_error() {
    let (_root, p, game) = uevr_home::uevr_no_injector("apply-no-injector");
    let err =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "SWZeroCompany.exe")]), &p).unwrap_err();
    code(err.clone(), "not-found");
    assert!(err.contains("tools/build-win.sh --install"));
}

#[test]
fn dll_path_contract_example() {
    assert_eq!(uevr::win_path(Path::new("/h/.cache/flat3d/uevr/nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d/openxr_loader.dll")), r"Z:\h\.cache\flat3d\uevr\nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d\openxr_loader.dll");
}

#[test]
fn debug_cmd_matches_golden() {
    let actual = uevr::debug_cmd(
        Path::new("/h/.local/share/flat3d/bin/flat3d-uevr-inject.exe"),
        "SWZeroCompany",
        15,
        Path::new(&format!("/h/.cache/flat3d/uevr/{}", uevr::UEVR_VERSION)),
    );
    assert_eq!(
        actual,
        include_str!("fixtures/uevr-debug-cmd.txt")
            .lines()
            .next()
            .unwrap()
    );
}

#[test]
fn quotes_survive_shlex() {
    let path = Path::new("/h/a space 'quoted'/.local/bin/flat3d-uevr-inject.exe");
    let dir = Path::new("/h/a space 'quoted'/.cache/flat3d/uevr/version");
    let actual = uevr::debug_cmd(path, "A 'B", 0, dir);
    let expected = vec![
        "/h/a space 'quoted'/.local/bin/flat3d-uevr-inject.exe".to_string(),
        "--exe".into(),
        "A 'B".into(),
        "--delay".into(),
        "0".into(),
        "--dll".into(),
        r"Z:\h\a space 'quoted'\.cache\flat3d\uevr\version\openxr_loader.dll".into(),
        "--dll".into(),
        r"Z:\h\a space 'quoted'\.cache\flat3d\uevr\version\UEVRBackend.dll".into(),
    ];
    let output = Command::new("python3").args(["-c", "import os,shlex,sys; print('\\n'.join(shlex.split(sys.argv[1]))); print('ABS='+str(os.path.isabs(shlex.split(sys.argv[1])[0])))", &actual]).output();
    let Ok(output) = output else {
        if std::env::var_os("FLAT3D_REQUIRE_HOST_TOOLS").is_some() {
            panic!("python3 is required by FLAT3D_REQUIRE_HOST_TOOLS=1");
        }
        eprintln!("skip: python3 is not on PATH");
        return;
    };
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let mut lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    assert_eq!(lines.pop().as_deref(), Some("ABS=True"));
    assert_eq!(lines, expected);
    let golden = include_str!("fixtures/uevr-debug-cmd.txt");
    let golden_line = golden.lines().next().unwrap();
    let golden_out = Command::new("python3").args(["-c", "import os,shlex,sys; print('\\n'.join(shlex.split(sys.argv[1]))); print('ABS='+str(os.path.isabs(shlex.split(sys.argv[1])[0])))", golden_line]).output().unwrap();
    assert!(golden_out.status.success());
    let golden_lines = String::from_utf8(golden_out.stdout).unwrap();
    assert!(golden_lines.lines().last() == Some("ABS=True"));
    assert_eq!(
        golden_lines
            .lines()
            .take(expected.len())
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        include_str!("fixtures/uevr-debug-cmd.txt")
            .lines()
            .skip(1)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
}

#[test]
fn uevr_version_matches_pins() {
    assert!(include_str!("../../../packaging/pins")
        .contains(&format!("uevr.version: {}\n", uevr::UEVR_VERSION)));
}

#[test]
fn uevr_exe_must_be_a_file_name() {
    for bad in [
        "../../../tmp/x.exe",
        "a/b.exe",
        "a\\b.exe",
        "..",
        ".",
        ".exe",
        "a\0b",
    ] {
        code(
            uevr::extras_from(&settings(&[("uevr-exe", bad)])).unwrap_err(),
            "bad-recipe",
        );
    }
}

#[test]
fn exe_suffix_is_stripped_once_and_case_insensitively() {
    for (name, stem) in [
        ("Game.Exe", "Game"),
        ("Game.EXE", "Game"),
        ("Foo.exe.exe", "Foo.exe"),
        ("Game", "Game"),
    ] {
        let extras = uevr::extras_from(&settings(&[("uevr-exe", name)])).unwrap();
        assert_eq!(extras.exe_stem, stem);
    }
    let (_root, p, game) = uevr_home::uevr_ready("apply-exe-case");
    let (applied, env) =
        uevr::apply_with(&game, &settings(&[("uevr-exe", "Foo.exe.exe")]), &p).unwrap();
    let compat = game
        .install_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("compatdata");
    let dir = uevr::profile_dir(&compat, game.appid, "Foo.exe.exe");
    assert!(dir.ends_with("UnrealVRMod/Foo.exe"), "{dir:?}");
    assert!(applied.record.created.iter().all(|f| f.starts_with(&dir)));
    assert!(env.env.iter().any(|(_, v)| v.contains("Foo.exe")));
}
