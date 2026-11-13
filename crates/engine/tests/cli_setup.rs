//! `flat3d-inject setup packages | install-apps | launch-env | fetch` through `cli::setup::run`
//! and `cli::execute_with`. FakeRunner only: no network, no device.

use flat3d_inject::api::{LaunchText, SetupReport};
use flat3d_inject::cli::{execute_with, setup, Ctx, SetupCommand};
use flat3d_inject::injectors::fetch;
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::testing::FakeRunner;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const PM_LIST: [&str; 5] = ["adb", "shell", "pm", "list", "packages"];
const WIVRN_LINE: &str = "package:com.oculus.vrshell\npackage:org.meumeu.wivrn.github\n";

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| (*s).to_string()).collect()
}

fn paths(case: &str) -> EnginePaths {
    let home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-setup-{case}"));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let env = BTreeMap::from([("HOME".to_string(), home.to_string_lossy().into_owned())]);
    EnginePaths::from_env(&env, 1000).unwrap()
}

/// Put the WiVRn APK in the fetch cache with its marker, as `setup fetch wivrn-apk` leaves it.
fn seed_apk(p: &EnginePaths) -> PathBuf {
    let pin = fetch::pin("wivrn-apk").unwrap();
    let dir = fetch::cache_dir(&pin, p);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(fetch::MARKER),
        format!("{}\n", pin.sha256.to_ascii_lowercase()),
    )
    .unwrap();
    let apk = dir.join("WiVRn-release.apk");
    fs::write(&apk, b"apk").unwrap();
    apk
}

fn call(
    p: &EnginePaths,
    fake: &mut FakeRunner,
    cmd: SetupCommand,
) -> Result<serde_json::Value, flat3d_inject::api::ApiError> {
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: p,
        runner: fake,
        process: &mut process,
        json: true,
    };
    setup::run(&cmd, &mut ctx)
}

fn pm_lists(fake: &FakeRunner) -> usize {
    fake.ran.iter().filter(|argv| **argv == PM_LIST).count()
}

fn installs(fake: &FakeRunner) -> Vec<Vec<String>> {
    fake.ran
        .iter()
        .filter(|argv| argv.starts_with(&args(&["adb", "install"])))
        .cloned()
        .collect()
}

fn flatpak_location(p: &EnginePaths) -> String {
    format!(
        "{}/.local/share/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc\n",
        p.home.display()
    )
}

#[test]
fn packages_missing_wivrn() {
    let p = paths("packages-missing");
    let mut fake = FakeRunner::new().ok(&PM_LIST, "");
    let got = call(&p, &mut fake, SetupCommand::Packages).unwrap();
    // `missing` names the app as the setup page shows it (setup.rs, locked setup_packages).
    assert_eq!(
        got,
        serde_json::json!({"wivrn": false, "missing": ["WiVRn"]})
    );
    assert_eq!(fake.ran, vec![args(&PM_LIST)]);
}

#[test]
fn install_apps_installs_wivrn() {
    let p = paths("install-ok");
    let apk = seed_apk(&p);
    let mut fake = FakeRunner::new().ok(&PM_LIST, "").ok(&PM_LIST, WIVRN_LINE);
    let got = call(&p, &mut fake, SetupCommand::InstallApps).unwrap();
    let report: SetupReport = serde_json::from_value(got).unwrap();
    assert_eq!(report.summary, "installed WiVRn");
    assert_eq!(report.steps.len(), 2, "one fetch line and one install line");
    assert!(fake.ran.iter().all(|argv| argv[0] != "curl"));
    assert_eq!(
        installs(&fake),
        vec![args(&["adb", "install", "-r", &apk.to_string_lossy()])]
    );
    assert_eq!(pm_lists(&fake), 2);
}

#[test]
fn install_fails_command_failed() {
    let p = paths("install-fails");
    seed_apk(&p);
    let mut fake = FakeRunner::new().ok(&PM_LIST, "").fail(
        &["adb", "install"],
        "Performing Streamed Install\nFailure [INSTALL_FAILED]\n",
    );
    let err = call(&p, &mut fake, SetupCommand::InstallApps).unwrap_err();
    assert_eq!(err.code, "command-failed");
    assert!(
        err.message.contains("Failure [INSTALL_FAILED]"),
        "{}",
        err.message
    );
    assert_eq!(pm_lists(&fake), 1, "no re-read after a failed install");
}

#[test]
fn still_missing_after_install() {
    let p = paths("still-missing");
    seed_apk(&p);
    let mut fake = FakeRunner::new().ok(&PM_LIST, "").ok(&PM_LIST, "");
    let err = call(&p, &mut fake, SetupCommand::InstallApps).unwrap_err();
    assert_eq!(err.code, "command-failed");
    assert!(
        err.message.starts_with("still missing after install: "),
        "{}",
        err.message
    );
    assert!(err.message.contains("WiVRn"), "{}", err.message);
    assert!(err.message.contains("wivrn"), "{}", err.message);
    assert_eq!(installs(&fake).len(), 1);
    assert_eq!(pm_lists(&fake), 2);
}

#[test]
fn nothing_to_install() {
    let p = paths("nothing");
    let mut fake = FakeRunner::new().ok(&PM_LIST, WIVRN_LINE);
    let got = call(&p, &mut fake, SetupCommand::InstallApps).unwrap();
    let report: SetupReport = serde_json::from_value(got).unwrap();
    assert!(report.steps.is_empty());
    assert_eq!(report.summary, "nothing to install");
    assert!(installs(&fake).is_empty());
    assert_eq!(pm_lists(&fake), 1);
    assert_eq!(fake.ran.len(), 1);
}

#[test]
fn launch_env_base() {
    let p = paths("launch-base");
    let app_dir = format!(
        "{}/.local/share/flatpak/app/io.github.wivrn.wivrn",
        p.home.display()
    );
    let mut fake = FakeRunner::new()
        .ok(
            &[
                "flatpak",
                "info",
                "--show-location",
                "--user",
                "io.github.wivrn.wivrn",
            ],
            &flatpak_location(&p),
        )
        .ok(
            &["flatpak", "info", "--user", "io.github.wivrn.wivrn"],
            "Version: 26.9\n",
        );
    let got = call(&p, &mut fake, SetupCommand::LaunchEnv { appid: None }).unwrap();
    let text: LaunchText = serde_json::from_value(got).unwrap();
    assert!(
        text.options
            .contains(&format!("PRESSURE_VESSEL_FILESYSTEMS_RW={app_dir}")),
        "{}",
        text.options
    );
    assert_eq!(
        text.options,
        format!(
            "PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 PRESSURE_VESSEL_FILESYSTEMS_RW={app_dir} STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%"
        )
    );
    assert!(text.options.ends_with("%command%"));
    assert!(!fake.ran.is_empty());
    for argv in &fake.ran {
        assert!(argv.starts_with(&args(&["flatpak", "info"])), "{argv:?}");
    }
}

#[test]
fn launch_env_appid() {
    let p = paths("launch-appid");
    let mut fake = FakeRunner::new();
    let got = call(&p, &mut fake, SetupCommand::LaunchEnv { appid: Some(5) }).unwrap();
    let text: LaunchText = serde_json::from_value(got).unwrap();
    assert_eq!(
        text.options,
        format!("{} %command%", p.wrapper_bin().display())
    );
    assert!(fake.ran.is_empty());
}

#[test]
fn launch_env_bad_appid_usage() {
    let p = paths("launch-bad");
    let mut fake = FakeRunner::new();
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &p,
        runner: &mut fake,
        process: &mut process,
        json: true,
    };
    let out = execute_with(&args(&["setup", "launch-env", "abc", "--json"]), &mut ctx);
    assert_eq!(out.code, 2);
    let env: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "usage");
    assert!(fake.ran.is_empty());
}

#[test]
fn setup_fetch_no_key_usage() {
    let p = paths("fetch-no-key");
    let mut fake = FakeRunner::new();
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: &p,
        runner: &mut fake,
        process: &mut process,
        json: true,
    };
    let out = execute_with(&args(&["setup", "fetch", "--json"]), &mut ctx);
    assert_eq!(out.code, 2);
    let env: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    assert_eq!(env["error"]["code"], "usage");
    assert!(fake.ran.is_empty(), "parsing fails before any runner call");
}
