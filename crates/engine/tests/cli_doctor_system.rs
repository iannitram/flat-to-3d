//! `flat3d-inject doctor` (#412): one check names the matched system profile or says
//! `unknown system`, and the profile's known problems are listed as information.

use flat3d_inject::cli::{execute_with, Ctx};
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::testing::FakeRunner;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const SYSTEM: &str = "\
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

fn paths(case: &str) -> EnginePaths {
    let home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-doctor-system-{case}"));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let os_release = home.join("os-release");
    fs::write(&os_release, "ID=steamos\nVERSION_ID=3.6\n").unwrap();
    let env = BTreeMap::from([
        ("HOME".to_string(), home.to_string_lossy().into_owned()),
        (
            "FLAT3D_OS_RELEASE".to_string(),
            os_release.to_string_lossy().into_owned(),
        ),
    ]);
    EnginePaths::from_env(&env, 1000).unwrap()
}

fn doctor(p: &EnginePaths, runner: &mut FakeRunner) -> serde_json::Value {
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: p,
        runner,
        process: &mut process,
        json: true,
    };
    let out = execute_with(&["doctor".to_string(), "--json".to_string()], &mut ctx);
    assert_eq!(out.stdout.lines().count(), 1, "one envelope line");
    serde_json::from_str(out.stdout.trim()).unwrap()
}

fn check<'a>(data: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    data["data"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("no check named {name}: {data}"))
}

#[test]
fn doctor_names_the_matched_system_and_lists_its_known_problems() {
    let p = paths("matched");
    let dir = p.data.join("systems");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("steamos-strix-halo.system"), SYSTEM).unwrap();
    let mut runner = FakeRunner::new().ok(
        &["lspci", "-nn"],
        "03:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon 8050S/8060S] (rev c1)\n",
    );
    let data = doctor(&p, &mut runner);

    let system = check(&data, "system");
    assert_eq!(system["ok"], true);
    assert_eq!(system["detail"], "SteamOS on Strix Halo");

    let open = check(&data, "system-gamescope-vulkan-crash");
    assert_eq!(open["ok"], true, "an open known problem is information");
    assert!(open["detail"]
        .as_str()
        .unwrap()
        .contains("cause is unknown"));
    assert!(open["detail"].as_str().unwrap().contains("none yet"));

    let worked = check(&data, "system-pipewire-capture");
    assert_eq!(worked["ok"], true);
    assert!(worked["detail"]
        .as_str()
        .unwrap()
        .starts_with("worked around:"));
}

#[test]
fn doctor_says_unknown_system_when_nothing_matches() {
    let p = paths("unknown");
    let mut runner = FakeRunner::new().ok(
        &["lspci", "-nn"],
        "03:00.0 VGA compatible controller: Intel Corporation Arc B580\n",
    );
    let data = doctor(&p, &mut runner);
    let system = check(&data, "system");
    assert_eq!(system["ok"], true);
    assert_eq!(system["detail"], "unknown system");
    assert!(data["data"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| !c["name"].as_str().unwrap().starts_with("system-")));
}
