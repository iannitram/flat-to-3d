use flat3d_inject::link::{self, FlatpakScope, WivrnDirs};
use flat3d_inject::testing::FakeRunner;
use std::path::PathBuf;

#[test]
fn user_install_strips_arch_branch_commit() {
    let deploy = "/h/.local/share/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/61d58e1800000000000000000000000000000000000000000000000000000332\n";
    let mut f = FakeRunner::new()
        .ok(&["flatpak", "info", "--show-location", "--user"], deploy)
        .ok(
            &["flatpak", "info", "--user"],
            "Version: 26.9\nBranch: stable\nArch: x86_64\n",
        );
    assert_eq!(
        link::detect_server(&mut f),
        Ok(WivrnDirs {
            scope: FlatpakScope::User,
            app_dir: PathBuf::from("/h/.local/share/flatpak/app/io.github.wivrn.wivrn"),
            deploy: PathBuf::from(deploy.trim()),
            version: Some("26.9".into())
        })
    );
    assert_eq!(f.ran.len(), 2);
}

#[test]
fn system_install_after_user_fails() {
    let mut f = FakeRunner::new()
        .fail(
            &["flatpak", "info", "--show-location", "--user"],
            "not installed",
        )
        .ok(
            &["flatpak", "info", "--show-location", "--system"],
            "/var/lib/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc\n",
        );
    assert_eq!(
        link::detect_server(&mut f).unwrap().scope,
        FlatpakScope::System
    );
    assert_eq!(f.ran.len(), 3);
}

#[test]
fn neither_is_an_error() {
    let mut f = FakeRunner::new()
        .fail(&["flatpak", "info", "--show-location", "--user"], "no user")
        .fail(
            &["flatpak", "info", "--show-location", "--system"],
            "no system",
        );
    let e = link::detect_server(&mut f).unwrap_err();
    assert!(
        e.contains("--user and --system")
            && e.contains("flatpak install --user flathub io.github.wivrn.wivrn")
    );
    assert_eq!(f.ran.len(), 2);
}

#[test]
fn a_short_path_is_an_error() {
    let mut f = FakeRunner::new()
        .ok(&["flatpak", "info", "--show-location", "--user"], "/x/y\n")
        .fail(
            &["flatpak", "info", "--show-location", "--system"],
            "no system",
        );
    let e = link::detect_server(&mut f).unwrap_err();
    assert!(e.contains("--user and --system"));
    assert_eq!(f.ran.len(), 2);
}

#[test]
fn store_app_found_second() {
    let mut f = FakeRunner::new()
        .ok(
            &[
                "adb",
                "-s",
                "SER",
                "shell",
                "dumpsys",
                "package",
                "org.meumeu.wivrn.github",
            ],
            "Unable to find package: org.meumeu.wivrn.github\n",
        )
        .ok(
            &[
                "adb",
                "-s",
                "SER",
                "shell",
                "dumpsys",
                "package",
                "org.meumeu.wivrn",
            ],
            "    versionName=26.9\n",
        );
    assert_eq!(
        link::detect_quest_app(&mut f, "SER").unwrap(),
        link::QuestApp {
            package: "org.meumeu.wivrn".into(),
            version: Some("26.9".into())
        }
    );
    assert!(f.ran[0].starts_with(&[
        "adb".into(),
        "-s".into(),
        "SER".into(),
        "shell".into(),
        "dumpsys".into(),
        "package".into()
    ]));
    assert_eq!(f.ran.len(), 2);
}

#[test]
fn no_candidate_is_none() {
    let mut f = FakeRunner::new();
    assert_eq!(link::detect_quest_app(&mut f, "SER"), None);
    assert_eq!(f.ran.len(), 5);
}

#[test]
fn adb_failure_is_none() {
    let mut f = FakeRunner::new().fail(&["adb"], "error: no devices/emulators found");
    assert_eq!(link::detect_quest_app(&mut f, "SER"), None);
    assert_eq!(f.ran.len(), 1);
}

#[test]
fn detection_runs_only_read_only_probes() {
    let mut f = FakeRunner::new();
    let _ = link::detect_server(&mut f);
    let _ = link::detect_quest_app(&mut f, "SER");
    assert_eq!(f.ran.len(), 7);
    assert!(f
        .ran
        .iter()
        .all(|a| a.starts_with(&["flatpak".into(), "info".into()])
            || a.starts_with(&[
                "adb".into(),
                "-s".into(),
                "SER".into(),
                "shell".into(),
                "dumpsys".into(),
                "package".into()
            ])));
}

#[test]
fn versions_match_ignores_suffix_after_hyphen() {
    assert!(link::wivrn::versions_match("26.9", "26.9-github"));
    assert!(!link::wivrn::versions_match("26.9", "26.8"));
}

#[test]
fn version_parsers_trim_values() {
    assert_eq!(
        link::wivrn::version_from_info("Version: 26.9\n"),
        Some("26.9".into())
    );
    assert_eq!(
        link::wivrn::version_name(" versionName=26.9\n"),
        Some("26.9".into())
    );
    assert_eq!(
        link::wivrn::app_dir_from_location("/app/io.github.wivrn.wivrn/x/y/z\n"),
        Some(PathBuf::from("/app/io.github.wivrn.wivrn"))
    );
}
