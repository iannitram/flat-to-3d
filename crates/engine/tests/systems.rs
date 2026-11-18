//! System profiles (#412): matching, no match, a malformed file, detection through fixtures,
//! and the repository's own shipped profile.

use flat3d_inject::systems::{
    detect, find_system, load_systems, matches, parse_gpu_name, parse_os_release, SystemError,
    SystemFacts, SystemProfile,
};
use flat3d_inject::testing::FakeRunner;
use std::fs;
use std::path::{Path, PathBuf};

const STEAMOS: &str = "\
# fixture
name: SteamOS on Strix Halo
match-os: steamos
match-version: >=3.0
match-gpu: Strix Halo | Radeon 8060S | 8060S
desktop-session: plasma
game-session: gamescope
steam-install: preinstalled
wivrn-install: flatpak io.github.wivrn.wivrn
tool: gamescope
known-problem: pipewire-capture
problem-summary: Capture must use PipeWire because x11grab gives black frames.
problem-workaround: Capture through PipeWire instead of x11grab.
problem-status: worked-around
known-problem: gamescope-vulkan-crash
problem-summary: gamescope 3.16.30 can crash in the Radeon Vulkan driver; the cause is unknown.
problem-workaround: none yet
problem-status: open
";

fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("systems-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (file, text) in files {
        fs::write(dir.join(file), text).unwrap();
    }
    dir
}

fn facts(os_id: &str, version: &str, gpu: &str) -> SystemFacts {
    SystemFacts {
        os_id: os_id.into(),
        os_version: Some(version.into()),
        gpu: Some(gpu.into()),
    }
}

fn one(text: &str) -> SystemProfile {
    let dir = folder("one", &[("steamos-strix-halo.system", text)]);
    load_systems(&dir)
        .unwrap()
        .remove("steamos-strix-halo")
        .unwrap()
}

#[test]
fn a_profile_matches_the_detected_facts() {
    let dir = folder("match", &[("steamos-strix-halo.system", STEAMOS)]);
    let detected = facts("steamos", "3.6", "AMD Radeon 8060S Graphics");
    let found = find_system(&[dir], &detected).unwrap().unwrap();
    assert_eq!(found.id, "steamos-strix-halo");
    assert_eq!(found.game_session, "gamescope");
    assert_eq!(found.wivrn_install, "flatpak io.github.wivrn.wivrn");
    assert_eq!(found.problems.len(), 2);
    assert!(found.problems.iter().any(|p| p.status == "open"));
}

#[test]
fn no_match_is_unknown_and_not_an_error() {
    let dir = folder("no-match", &[("steamos-strix-halo.system", STEAMOS)]);
    for detected in [
        facts("arch", "3.6", "AMD Radeon 8060S Graphics"),
        facts("steamos", "2.9", "AMD Radeon 8060S Graphics"),
        facts("steamos", "3.6", "Intel Arc B580"),
    ] {
        assert!(find_system(std::slice::from_ref(&dir), &detected)
            .unwrap()
            .is_none());
    }
    let mut facts = facts("steamos", "3.6", "AMD Radeon 8060S Graphics");
    facts.os_id.clear();
    assert!(!matches(&one(STEAMOS), &facts));
}

#[test]
fn a_malformed_file_is_an_error_naming_the_file() {
    let cases = [
        ("broken.system", "name: Broken\n"),
        (
            "bad-problem.system",
            "name: Bad\nmatch-os: steamos\ndesktop-session: plasma\ngame-session: gamescope\n\
             steam-install: preinstalled\nwivrn-install: flatpak io.github.wivrn.wivrn\n\
             known-problem: half\nproblem-summary: only half\n",
        ),
        (
            "bad-status.system",
            "name: Bad\nmatch-os: steamos\ndesktop-session: plasma\ngame-session: gamescope\n\
             steam-install: preinstalled\nwivrn-install: flatpak io.github.wivrn.wivrn\n\
             known-problem: odd\nproblem-summary: odd\nproblem-workaround: none yet\n\
             problem-status: maybe\n",
        ),
    ];
    for (file, text) in cases {
        let dir = folder(
            &format!("bad-{}", file.trim_end_matches(".system")),
            &[(file, text)],
        );
        match load_systems(&dir) {
            Err(SystemError::InFile { file: named, .. }) => assert_eq!(named, file),
            other => panic!("expected InFile naming {file}, got {other:?}"),
        }
    }
}

#[test]
fn detection_reads_the_os_release_and_gpu_fixtures() {
    let dir = folder("detect", &[]);
    let os_release = dir.join("os-release");
    fs::write(&os_release, "ID=steamos\nVERSION_ID=3.6\n").unwrap();
    let mut runner = FakeRunner::new().ok(
        &["lspci", "-nn"],
        "03:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon 8050S/8060S] (rev c1)\n",
    );
    let detected = detect(&os_release, &mut runner);
    assert_eq!(detected.os_id, "steamos");
    assert_eq!(detected.os_version.as_deref(), Some("3.6"));
    assert!(detected
        .gpu
        .as_deref()
        .is_some_and(|name| name.contains("Strix Halo")));
    assert_eq!(parse_os_release("ID=arch\n").0, "arch");
    assert!(parse_gpu_name("Device: AMD Radeon 8060S Graphics\n")
        .is_some_and(|name| name.contains("8060S")));
}

#[test]
fn the_repo_ships_a_steamos_strix_halo_profile_that_loads() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profiles = load_systems(&root.join("systems"))
        .unwrap_or_else(|error| panic!("systems/*.system: {error}"));
    let profile = profiles
        .get("steamos-strix-halo")
        .expect("systems/steamos-strix-halo.system");
    assert_eq!(profile.os_id, "steamos");
    assert_eq!(profile.game_session, "gamescope");
    assert_eq!(profile.wivrn_install, "flatpak io.github.wivrn.wivrn");
    assert!(profile
        .problems
        .iter()
        .any(|p| p.id == "virtual-display-headless"));
    assert!(profile.problems.iter().any(|p| p.id == "pipewire-capture"));
    assert!(profile
        .problems
        .iter()
        .any(|p| p.id == "gamescope-vulkan-crash" && p.is_open()));
    assert!(profile
        .problems
        .iter()
        .any(|p| p.id == "lsfg-vk-getinstanceprocaddr" && p.is_open()));
}
