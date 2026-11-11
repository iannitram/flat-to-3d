use flat3d_uevr_inject::{args, pick};

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn dlls(value: &[&str]) -> Vec<String> {
    value.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn parses_defaults_and_keeps_dll_order_and_bytes() {
    let parsed = args::parse(&argv(&[
        "--exe",
        "SWZeroCompany",
        "--dll",
        r"Z:\c\openxr_loader.dll",
        "--dll",
        r"Z:\c\UEVRBackend.dll",
    ]))
    .unwrap();
    assert!(matches!(parsed.target, args::Target::Exe(ref stem) if stem == "SWZeroCompany"));
    assert_eq!(parsed.delay_s, 15);
    assert_eq!(parsed.wait_s, 120);
    assert_eq!(
        parsed.dlls,
        dlls(&[r"Z:\c\openxr_loader.dll", r"Z:\c\UEVRBackend.dll"])
    );
}

#[test]
fn strips_exe_extension_ascii_case_insensitively() {
    let parsed = args::parse(&argv(&["--exe", "Ace7Game.EXE", "--dll", r"Z:\x.dll"])).unwrap();
    assert!(matches!(parsed.target, args::Target::Exe(ref stem) if stem == "Ace7Game"));
}

#[test]
fn accepts_pid_and_zero_delay() {
    let parsed = args::parse(&argv(&[
        "--pid",
        "4242",
        "--delay",
        "0",
        "--dll",
        r"Z:\x.dll",
    ]))
    .unwrap();
    assert!(matches!(parsed.target, args::Target::Pid(4242)));
    assert_eq!(parsed.delay_s, 0);
}

#[test]
fn rejects_invalid_argument_shapes() {
    for input in [
        vec![],
        argv(&["--exe", "a", "--pid", "1", "--dll", r"Z:\x.dll"]),
        argv(&["--exe", "a"]),
        argv(&["--exe", "a", "--dll", r"Z:\x.dll", "--delay", "abc"]),
        argv(&["--pid", "0", "--dll", r"Z:\x.dll"]),
        argv(&["--attach=x"]),
        argv(&["--exe", "a", "--dll"]),
        argv(&["--exe", "a", "--exe", "b", "--dll", r"Z:\x.dll"]),
        argv(&["--exe", "a", "--dll", "x"]),
        argv(&["--exe", "a", "--dll", "/c/x.dll"]),
        argv(&["--exe", "a", "--dll", r"Z:/x.dll"]),
        argv(&["--exe", "a", "--dll", r"Z:\x.dll", "--wait", "-1"]),
    ] {
        assert!(args::parse(&input).is_err(), "accepted {input:?}");
    }
}

#[test]
fn contract_path_is_preserved_byte_for_byte() {
    let path = r"Z:\h\.cache\flat3d\uevr\nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d\openxr_loader.dll";
    let parsed = args::parse(&argv(&["--exe", "SWZeroCompany", "--dll", path])).unwrap();
    assert_eq!(parsed.dlls, dlls(&[path]));
}

#[test]
fn debug_cmd_golden_argv_parses() {
    let tokens: Vec<_> = include_str!("../../engine/tests/fixtures/uevr-debug-cmd.txt")
        .lines()
        .skip(1)
        .collect();
    let parsed = args::parse(
        &tokens
            .iter()
            .skip(1)
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert!(matches!(parsed.target, args::Target::Exe(ref stem) if stem == "SWZeroCompany"));
    assert_eq!(parsed.delay_s, 15);
    assert_eq!(parsed.wait_s, 120);
    assert_eq!(parsed.dlls, dlls(&[tokens[6], tokens[8]]));
}

#[test]
fn error_line_and_exit_codes_are_frozen() {
    assert_eq!(
        args::error_line("usage", "bad", "x"),
        "flat3d-uevr-inject: error[usage] bad; fix: x"
    );
    assert_eq!(args::EXIT_USAGE, 2);
    assert_eq!(args::EXIT_NOT_WINDOWS, 3);
    assert_eq!(args::EXIT_NO_GAME, 4);
    assert_eq!(args::EXIT_INJECT, 5);
}

#[test]
fn preferred_game_path_beats_bootstrap() {
    let procs = vec![
        (10, r"C:\Games\SWZeroCompany.exe".to_owned()),
        (
            12,
            r"C:\Games\SWZeroCompany\Binaries\Win64\SWZeroCompany.exe".to_owned(),
        ),
    ];
    assert_eq!(
        pick::choose(&procs, "SWZeroCompany"),
        Some(pick::Pick {
            pid: 12,
            preferred: true
        })
    );
}

#[test]
fn bootstrap_fallback_and_ascii_path_matching() {
    assert_eq!(
        pick::choose(
            &[(10, r"C:\Games\SWZeroCompany.exe".to_owned())],
            "SWZeroCompany"
        ),
        Some(pick::Pick {
            pid: 10,
            preferred: false
        })
    );
    assert_eq!(
        pick::choose(
            &[(20, r"C:\G\GAME\BINARIES\WIN64\ACE7GAME.EXE".to_owned())],
            "Ace7Game"
        ),
        Some(pick::Pick {
            pid: 20,
            preferred: true
        })
    );
    assert_eq!(
        pick::choose(
            &[(21, "/g/Game/Binaries/Win64/Ace7Game.exe".to_owned())],
            "Ace7Game"
        ),
        Some(pick::Pick {
            pid: 21,
            preferred: true
        })
    );
    assert_eq!(
        pick::choose(
            &[(22, r"C:\g\SWZeroCompanyLauncher.exe".to_owned())],
            "SWZeroCompany"
        ),
        None
    );
}

#[test]
fn last_preferred_and_last_fallback_match_win() {
    let procs = vec![
        (11, r"C:\1\GAME\Binaries\Win64\game.exe".to_owned()),
        (12, r"C:\2\GAME\Binaries\Win64\game.exe".to_owned()),
    ];
    assert_eq!(pick::choose(&procs, "game").unwrap().pid, 12);
    assert_eq!(
        pick::choose(
            &[
                (31, r"C:\one\game.exe".to_owned()),
                (32, r"C:\two\game.exe".to_owned())
            ],
            "game"
        )
        .unwrap()
        .pid,
        32
    );
}

#[test]
fn module_comparison_uses_case_insensitive_basename() {
    assert!(pick::has_module(
        &["OPENXR_LOADER.DLL".to_owned()],
        r"Z:\x\openxr_loader.dll"
    ));
}

fn snapshots(paths: &[(u32, &str)]) -> Vec<(u32, String)> {
    paths
        .iter()
        .map(|(pid, path)| (*pid, (*path).to_owned()))
        .collect()
}

#[test]
fn waiter_selects_preferred_immediately_and_fallback_after_ten_seconds() {
    let preferred = snapshots(&[(12, r"C:\SWZeroCompany\Binaries\Win64\SWZeroCompany.exe")]);
    let mut waiter = pick::Waiter::new("SWZeroCompany");
    assert_eq!(waiter.step(&preferred, 0), Some(12));

    let bootstrap = snapshots(&[(10, r"C:\SWZeroCompany.exe")]);
    let mut waiter = pick::Waiter::new("SWZeroCompany");
    for now_ms in (0..=9_500).step_by(500) {
        assert_eq!(waiter.step(&bootstrap, now_ms), None);
    }
    assert_eq!(waiter.step(&bootstrap, 10_000), Some(10));
}

#[test]
fn waiter_resets_on_absence_preference_or_new_pid() {
    let bootstrap = snapshots(&[(10, r"C:\SWZeroCompany.exe")]);
    let preferred = snapshots(&[(12, r"C:\SWZeroCompany\Binaries\Win64\SWZeroCompany.exe")]);
    let mut waiter = pick::Waiter::new("SWZeroCompany");
    assert_eq!(waiter.step(&bootstrap, 0), None);
    assert_eq!(waiter.step(&bootstrap, 5_000), None);
    assert_eq!(waiter.step(&preferred, 5_500), Some(12));

    let mut waiter = pick::Waiter::new("SWZeroCompany");
    assert_eq!(waiter.step(&bootstrap, 0), None);
    assert_eq!(waiter.step(&bootstrap, 4_000), None);
    assert_eq!(waiter.step(&[], 4_500), None);
    assert_eq!(waiter.step(&bootstrap, 5_000), None);
    assert_eq!(waiter.step(&bootstrap, 14_500), None);
    assert_eq!(waiter.step(&bootstrap, 15_000), Some(10));

    let mut waiter = pick::Waiter::new("SWZeroCompany");
    assert_eq!(waiter.step(&bootstrap, 0), None);
    assert_eq!(
        waiter.step(&snapshots(&[(11, r"C:\SWZeroCompany.exe")]), 5_000),
        None
    );
    assert_eq!(
        waiter.step(&snapshots(&[(11, r"C:\SWZeroCompany.exe")]), 14_500),
        None
    );
    assert_eq!(
        waiter.step(&snapshots(&[(11, r"C:\SWZeroCompany.exe")]), 15_000),
        Some(11)
    );
}
