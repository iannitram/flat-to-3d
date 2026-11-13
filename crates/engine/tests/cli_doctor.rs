//! `flat3d-inject doctor` through `cli::execute_with`: the exit follows the report (ruling D4),
//! and doctor only probes. FakeRunner only: no network, no device.

use flat3d_inject::cli::{execute_with, Ctx};
use flat3d_inject::exec::CommandOutput;
use flat3d_inject::injectors::fetch;
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::testing::FakeRunner;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const CHECKS: [&str; 8] = [
    "wivrn-server",
    "quest-app",
    "adb",
    "link-service",
    "wrapper",
    "steam",
    "pins",
    "injector",
];

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| (*s).to_string()).collect()
}

fn paths(case: &str) -> EnginePaths {
    let home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-doctor-{case}"));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let env = BTreeMap::from([("HOME".to_string(), home.to_string_lossy().into_owned())]);
    EnginePaths::from_env(&env, 1000).unwrap()
}

fn seed_file(path: &Path, mode: u32) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"x").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

/// Everything green except `pins`, which reads the compiled table.
fn green(case: &str, unit_active: bool) -> (EnginePaths, FakeRunner) {
    let p = paths(case);
    let home = p.home.clone();
    seed_file(&home.join(".local/share/flat3d/bin/adb"), 0o755);
    seed_file(&p.wrapper_bin(), 0o755);
    seed_file(
        &home.join(".local/share/Steam/steamapps/libraryfolders.vdf"),
        0o644,
    );
    seed_file(&p.data.join("bin/flat3d-uevr-inject.exe"), 0o644);
    let location = format!(
        "{}/.local/share/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc\n",
        home.display()
    );
    let fake = FakeRunner::new()
        .ok(
            &["adb", "devices"],
            "List of devices attached\nSERIAL1\tdevice\n",
        )
        .ok(
            &[
                "flatpak",
                "info",
                "--show-location",
                "--user",
                "io.github.wivrn.wivrn",
            ],
            &location,
        )
        .ok(
            &["flatpak", "info", "--user", "io.github.wivrn.wivrn"],
            "Version: 26.9\n",
        )
        .ok(
            &[
                "adb",
                "-s",
                "SERIAL1",
                "shell",
                "dumpsys",
                "package",
                "org.meumeu.wivrn.github",
            ],
            "    versionName=26.9\n",
        )
        .reply(
            &["systemctl", "--user", "is-active", "flat3d-link.service"],
            CommandOutput {
                success: unit_active,
                stdout: if unit_active {
                    "active\n"
                } else {
                    "inactive\n"
                }
                .into(),
                stderr: String::new(),
            },
        );
    (p, fake)
}

fn doctor(p: &EnginePaths, fake: &mut FakeRunner) -> (i32, serde_json::Value) {
    let mut process = FakeRunner::new();
    let mut ctx = Ctx {
        paths: p,
        runner: fake,
        process: &mut process,
        json: true,
    };
    let out = execute_with(&args(&["doctor", "--json"]), &mut ctx);
    assert_eq!(out.stdout.lines().count(), 1, "one envelope line");
    (out.code, serde_json::from_str(out.stdout.trim()).unwrap())
}

fn names(data: &serde_json::Value) -> Vec<&str> {
    data["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect()
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                out.insert(rel, Vec::new());
                walk(&path, root, out);
            } else {
                out.insert(rel, fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn exit_follows_report() {
    let (p, mut fake) = green("exit-follows", true);
    let (code, env) = doctor(&p, &mut fake);
    assert_eq!(env["ok"], true, "the envelope: the command ran");
    assert_eq!(env["command"], "doctor");
    let data = &env["data"];
    assert_eq!(names(data), CHECKS);
    for check in data["checks"].as_array().unwrap() {
        if check["name"] != "pins" {
            assert_eq!(check["ok"], true, "{check}");
        }
    }
    let no_tbd = !fetch::parse_pins(fetch::PINS)
        .values()
        .any(|pin| pin.sha256 == fetch::TBD);
    let pins = data["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "pins")
        .unwrap();
    assert_eq!(pins["ok"], no_tbd);
    assert_eq!(data["ok"], no_tbd);
    assert_eq!(code, if data["ok"] == true { 0 } else { 1 });
    // The quest-app line shows a version, not the skip detail.
    let quest = &data["checks"][1];
    assert!(
        quest["detail"].as_str().unwrap().contains("26.9"),
        "{quest}"
    );
}

#[test]
fn red_report_exits_1() {
    let (p, mut fake) = green("red-exits", false);
    let (code, env) = doctor(&p, &mut fake);
    assert_eq!(code, 1);
    let data = &env["data"];
    assert_eq!(data["ok"], false);
    assert_eq!(names(data), CHECKS, "a red report still prints in full");
    let link = &data["checks"][3];
    assert_eq!(link["name"], "link-service");
    assert_eq!(link["ok"], false);
    assert!(link["fix"]
        .as_str()
        .unwrap()
        .contains("systemctl --user enable --now flat3d-link.service"));
}

#[test]
fn doctor_runs_only_probes() {
    let (p, mut fake) = green("only-probes", true);
    let before = tree(&p.home);
    let _ = doctor(&p, &mut fake);
    assert_eq!(tree(&p.home), before, "doctor writes nothing");
    let probes: [&[&str]; 5] = [
        &["adb", "devices"],
        &["adb", "-s", "SERIAL1", "shell", "dumpsys", "package"],
        &["flatpak", "info"],
        &["ss", "-tlnp"],
        &["systemctl", "--user", "is-active"],
    ];
    assert!(!fake.ran.is_empty());
    for argv in &fake.ran {
        assert!(
            probes.iter().any(|probe| argv
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .starts_with(probe)),
            "not a probe: {argv:?}"
        );
    }
}
