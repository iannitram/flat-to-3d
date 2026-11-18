//! `flat3d-inject doctor`: eight read-only checks, each with a fix the user can run.

use super::{Ctx, DoctorCommand};
use crate::{
    api::{AdbSource, ApiError, DoctorCheck, DoctorReport},
    exec::{CommandOutput, CommandRunner},
    injectors::fetch::{self, Pin},
    install::UNIT_NAME,
    link::{
        adb::{bundled_adb, choose_adb, listener_pid, proc_exe_path, AdbChoice, AdbRunner},
        wivrn::{detect_quest_app, detect_server, versions_match},
        QuestApp, WIVRN_FLATPAK,
    },
    paths::EnginePaths,
    steam::DEFAULT_STEAM_ROOT,
    systems,
};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// The detail of a skipped `quest-app` check, so a gate never reads a skip as verified.
const SKIP_NO_HEADSET: &str = "skipped: no headset";
/// The port the adb server listens on.
const ADB_PORT: u16 = 5037;

pub fn run(_cmd: &DoctorCommand, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    let adb = chosen_adb(ctx.paths, ctx.runner);
    let serial = adb_serial(ctx.runner, &adb.path);
    let pins = fetch::parse_pins(fetch::PINS);
    let report = doctor(ctx.paths, ctx.runner, serial.as_deref(), &adb, &pins);
    serde_json::to_value(report).map_err(|e| ApiError::new("io", &e.to_string()))
}

/// The adb in use: a running server's executable, else the bundled one.
fn chosen_adb(p: &EnginePaths, r: &mut dyn CommandRunner) -> AdbChoice {
    let home = p.home.to_string_lossy().into_owned();
    let ss = r.run(&["ss".into(), "-tlnp".into()]);
    let running = ss
        .success
        .then(|| listener_pid(&ss.stdout, ADB_PORT))
        .flatten()
        .and_then(|pid| std::fs::read_link(proc_exe_path(pid)).ok())
        .map(|path| path.to_string_lossy().into_owned());
    let bundled = Some(bundled_adb(&home)).filter(|path| Path::new(path).exists());
    choose_adb(running.as_deref(), bundled.as_deref())
}

/// The serial of the first headset `adb devices` lists as ready, read with the chosen adb.
fn adb_serial(r: &mut dyn CommandRunner, adb: &str) -> Option<String> {
    if adb.is_empty() {
        return None;
    }
    let out = r.run(&[adb.to_string(), "devices".into()]);
    if !out.success {
        return None;
    }
    out.stdout.lines().find_map(|line| {
        let (serial, state) = line.split_once('\t')?;
        (state.trim() == "device" && !serial.trim().is_empty()).then(|| serial.trim().to_string())
    })
}

/// Lets [`AdbRunner`] borrow doctor's runner, so the quest-app check names the adb in use.
struct Borrowed<'a>(&'a mut dyn CommandRunner);

impl CommandRunner for Borrowed<'_> {
    fn run(&mut self, command: &[String]) -> CommandOutput {
        self.0.run(command)
    }
}

/// `detect_quest_app` through the chosen adb. With no adb, it runs nothing and finds nothing.
fn detect_quest_app_with(r: &mut dyn CommandRunner, adb: &str, serial: &str) -> Option<QuestApp> {
    if adb.is_empty() {
        return None;
    }
    detect_quest_app(&mut AdbRunner::new(Borrowed(r), adb, Some(serial)), serial)
}

fn check(name: &str, ok: bool, detail: String, fix: &str) -> DoctorCheck {
    DoctorCheck {
        name: name.into(),
        ok,
        detail,
        fix: if ok { String::new() } else { fix.into() },
    }
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// Run every check, in order. Doctor writes nothing and never fails as a whole: each check
/// catches its own error into its detail.
fn doctor(
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
    serial: Option<&str>,
    adb: &AdbChoice,
    pins: &BTreeMap<String, Pin>,
) -> DoctorReport {
    let home = p.home.to_string_lossy().into_owned();
    let mut checks = Vec::new();

    // 1. The WiVRn server.
    let server = detect_server(r);
    let server_version = server.as_ref().ok().and_then(|dirs| dirs.version.clone());
    checks.push(match &server {
        Ok(dirs) => {
            let mut detail = dirs.app_dir.display().to_string();
            if let Some(version) = &dirs.version {
                detail.push(' ');
                detail.push_str(version);
            }
            check("wivrn-server", true, detail, "")
        }
        Err(message) => check(
            "wivrn-server",
            false,
            message.clone(),
            &format!("flatpak install --user flathub {WIVRN_FLATPAK}"),
        ),
    });

    // 2. The WiVRn app on the headset.
    checks.push(match serial {
        None => check("quest-app", true, SKIP_NO_HEADSET.into(), ""),
        Some(serial) => match detect_quest_app_with(r, &adb.path, serial) {
            None => check(
                "quest-app",
                false,
                "the WiVRn app is not on the headset".into(),
                "flat3d-inject setup install-apps",
            ),
            Some(app) => {
                let mismatch = match (&server_version, &app.version) {
                    (Some(s), Some(a)) if !versions_match(s, a) => Some((s.clone(), a.clone())),
                    _ => None,
                };
                match mismatch {
                    // No command fixes a mismatch yet (questions.md 54).
                    Some((s, a)) => check(
                        "quest-app",
                        false,
                        format!("server {s}, headset {a}"),
                        &format!("update the WiVRn app on the headset to version {s}"),
                    ),
                    None => {
                        let detail = match &app.version {
                            Some(version) => format!("{} {version}", app.package),
                            None => app.package.clone(),
                        };
                        check("quest-app", true, detail, "")
                    }
                }
            }
        },
    });

    // 3. adb: the running server's executable, else the bundled one.
    let source = match adb.source {
        AdbSource::Running => "running",
        AdbSource::Bundled => "bundled",
        AdbSource::Missing => "missing",
    };
    checks.push(check(
        "adb",
        adb.source != AdbSource::Missing,
        source.into(),
        "flat3d-inject install",
    ));

    // 4. The link service.
    let unit = r.run(&[
        "systemctl".into(),
        "--user".into(),
        "is-active".into(),
        UNIT_NAME.into(),
    ]);
    let state = unit.stdout.trim();
    let detail = if !state.is_empty() {
        state.to_string()
    } else if !unit.stderr.trim().is_empty() {
        unit.stderr.trim().to_string()
    } else {
        "not active".to_string()
    };
    checks.push(check(
        "link-service",
        state == "active",
        detail,
        &format!("systemctl --user enable --now {UNIT_NAME}"),
    ));

    // 5. The launch wrapper.
    let wrapper = p.wrapper_bin();
    checks.push(check(
        "wrapper",
        is_executable(&wrapper),
        wrapper.display().to_string(),
        "flat3d-inject install",
    ));

    // 6. Steam has written its library list.
    let vdf = Path::new(&home)
        .join(DEFAULT_STEAM_ROOT)
        .join("steamapps/libraryfolders.vdf");
    checks.push(check(
        "steam",
        vdf.is_file(),
        vdf.display().to_string(),
        "steam",
    ));

    // 7. No pin still waits for its checksum.
    let unpinned: Vec<&str> = pins
        .iter()
        .filter(|(_, pin)| pin.sha256 == fetch::TBD)
        .map(|(key, _)| key.as_str())
        .collect();
    checks.push(match unpinned.first() {
        None => check("pins", true, format!("{} pins", pins.len()), ""),
        Some(first) => check(
            "pins",
            false,
            unpinned.join(", "),
            &format!("packaging/pin.sh {first}"),
        ),
    });

    // 8. The injector executable (WP-14 `injector_path`, WP-15 `--install`).
    let injector = p.data.join("bin/flat3d-uevr-inject.exe");
    checks.push(check(
        "injector",
        injector.exists(),
        injector.display().to_string(),
        "tools/build-win.sh --install",
    ));

    // 9. The shipped system profile and its known problems. An unknown system is normal, and
    // an open known problem is information, so neither turns doctor red.
    let facts = systems::detect(&p.os_release, r);
    match systems::find_system(&p.system_dirs, &facts) {
        Ok(Some(profile)) => {
            checks.push(check("system", true, profile.name.clone(), ""));
            for problem in &profile.problems {
                let status = if problem.is_open() {
                    "open"
                } else {
                    "worked around"
                };
                checks.push(check(
                    &format!("system-{}", problem.id),
                    true,
                    format!(
                        "{status}: {}; workaround: {}",
                        problem.summary, problem.workaround
                    ),
                    "",
                ));
            }
        }
        Ok(None) => checks.push(check("system", true, "unknown system".into(), "")),
        Err(error) => checks.push(check(
            "system",
            false,
            error.to_string(),
            "fix the system file named in the message",
        )),
    }

    DoctorReport {
        ok: checks.iter().all(|c| c.ok),
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeRunner;
    use std::fs;

    const GREEN: &str = "\
sample.version: 1.0
sample.url: file:///nonexistent/sample-1.0.zip
sample.size: 1
sample.sha256: dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275
";
    const WITH_TBD: &str = "\
sample.version: 1.0
sample.url: file:///nonexistent/sample-1.0.zip
sample.size: 1
sample.sha256: dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275
geo11.version: 0.7.12
geo11.url: file:///nonexistent/geo11.zip
geo11.size: 1
geo11.sha256: TBD-at-pin
";
    const ORDER: [&str; 8] = [
        "wivrn-server",
        "quest-app",
        "adb",
        "link-service",
        "wrapper",
        "steam",
        "pins",
        "injector",
    ];

    fn paths(case: &str) -> EnginePaths {
        let home =
            std::env::temp_dir().join(format!("flat3d-doctor-unit-{}-{case}", std::process::id()));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(&home).unwrap();
        let env = BTreeMap::from([("HOME".to_string(), home.to_string_lossy().into_owned())]);
        EnginePaths::from_env(&env, 1000).unwrap()
    }

    fn seed(path: &Path, mode: u32) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"x").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    const SERIAL_ADB: &str = "/opt/flat3d-test/adb";

    /// Files and replies for an all-green machine.
    fn green(case: &str, headset_version: &str) -> (EnginePaths, FakeRunner) {
        let p = paths(case);
        let home = p.home.clone();
        seed(&home.join(".local/share/flat3d/bin/adb"), 0o755);
        seed(&p.wrapper_bin(), 0o755);
        seed(
            &home.join(".local/share/Steam/steamapps/libraryfolders.vdf"),
            0o644,
        );
        seed(&p.data.join("bin/flat3d-uevr-inject.exe"), 0o644);
        let location = format!(
            "{}/.local/share/flatpak/app/io.github.wivrn.wivrn/x86_64/stable/abc\n",
            home.display()
        );
        let adb = home.join(".local/share/flat3d/bin/adb");
        let fake = FakeRunner::new()
            .ok(
                &[adb.to_str().unwrap(), "devices"],
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
                    adb.to_str().unwrap(),
                    "-s",
                    "SERIAL1",
                    "shell",
                    "dumpsys",
                    "package",
                    "org.meumeu.wivrn.github",
                ],
                &format!("    versionName={headset_version}\n"),
            )
            .ok(&["systemctl", "--user", "is-active", UNIT_NAME], "active\n");
        (p, fake)
    }

    fn report(
        p: &EnginePaths,
        fake: &mut FakeRunner,
        serial: Option<&str>,
        pins: &BTreeMap<String, Pin>,
    ) -> DoctorReport {
        let adb = chosen_adb(p, fake);
        doctor(p, fake, serial, &adb, pins)
    }

    fn names(report: &DoctorReport) -> Vec<&str> {
        report.checks.iter().map(|c| c.name.as_str()).collect()
    }

    fn find<'a>(report: &'a DoctorReport, name: &str) -> &'a DoctorCheck {
        report.checks.iter().find(|c| c.name == name).unwrap()
    }

    #[test]
    fn all_green() {
        let (p, mut fake) = green("all-green", "26.9");
        let pins = fetch::parse_pins(GREEN);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        assert!(report.ok, "{report:?}");
        assert_eq!(&names(&report)[..ORDER.len()], &ORDER[..]);
        assert!(report.checks.iter().all(|c| c.ok && c.fix.is_empty()));
    }

    #[test]
    fn tbd_pin_is_red() {
        let (p, mut fake) = green("tbd-pin", "26.9");
        let pins = fetch::parse_pins(WITH_TBD);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        assert!(!report.ok);
        let pins = find(&report, "pins");
        assert!(!pins.ok);
        assert!(pins.detail.contains("geo11"), "{}", pins.detail);
        assert_eq!(pins.fix, "packaging/pin.sh geo11");
    }

    #[test]
    fn no_serial_skips_quest() {
        let mut header_only =
            FakeRunner::new().ok(&[SERIAL_ADB, "devices"], "List of devices attached\n\n");
        assert_eq!(adb_serial(&mut header_only, SERIAL_ADB), None);
        let mut two = FakeRunner::new().ok(
            &[SERIAL_ADB, "devices"],
            "List of devices attached
AAA	unauthorized
BBB	device
",
        );
        assert_eq!(adb_serial(&mut two, SERIAL_ADB).as_deref(), Some("BBB"));
        let (p, mut fake) = green("no-serial", "26.9");
        let pins = fetch::parse_pins(GREEN);
        let report = report(&p, &mut fake, None, &pins);
        let quest = find(&report, "quest-app");
        assert!(quest.ok);
        assert_eq!(quest.detail, SKIP_NO_HEADSET);
        assert_eq!(quest.detail, "skipped: no headset");
        assert!(fake
            .ran
            .iter()
            .all(|argv| !argv.contains(&"dumpsys".to_string())));
    }

    #[test]
    fn serial_lookup_uses_the_chosen_adb_and_never_a_bare_one() {
        let (p, mut fake) = green("serial-adb", "26.9");
        let adb = chosen_adb(&p, &mut fake);
        assert_eq!(adb.source, AdbSource::Bundled);
        assert_eq!(adb_serial(&mut fake, &adb.path).as_deref(), Some("SERIAL1"));
        assert!(fake
            .ran
            .iter()
            .all(|argv| argv.first().map(String::as_str) != Some("adb")));
        assert!(fake
            .ran
            .iter()
            .any(|argv| argv.first().map(String::as_str) == Some(adb.path.as_str())));
    }

    #[test]
    fn quest_check_runs_the_chosen_adb_and_never_a_bare_one() {
        let (p, mut fake) = green("quest-adb", "26.9");
        let pins = fetch::parse_pins(GREEN);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        assert!(find(&report, "quest-app").ok, "{report:?}");
        let adb = p.home.join(".local/share/flat3d/bin/adb");
        let adb = adb.to_str().unwrap();
        assert!(fake.ran.iter().any(|argv| {
            argv.first().map(String::as_str) == Some(adb)
                && argv.get(1).map(String::as_str) == Some("-s")
                && argv.iter().any(|word| word == "dumpsys")
        }));
        assert!(fake
            .ran
            .iter()
            .all(|argv| argv.first().map(String::as_str) != Some("adb")));
    }

    #[test]
    fn quest_check_without_adb_runs_nothing() {
        let p = paths("quest-no-adb");
        let mut fake = FakeRunner::new();
        let pins = fetch::parse_pins(GREEN);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        assert!(!find(&report, "quest-app").ok);
        assert!(fake
            .ran
            .iter()
            .all(|argv| argv.first().map(String::as_str) != Some("adb")));
    }

    #[test]
    fn version_mismatch_is_red() {
        let (p, mut fake) = green("mismatch", "26.8");
        let pins = fetch::parse_pins(GREEN);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        assert!(!report.ok);
        let quest = find(&report, "quest-app");
        assert!(!quest.ok);
        assert!(quest.detail.contains("26.9") && quest.detail.contains("26.8"));
        assert!(quest.fix.contains("26.9"));
    }

    #[test]
    fn report_round_trips() {
        let (p, mut fake) = green("round-trip", "26.9");
        let pins = fetch::parse_pins(WITH_TBD);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        let text = serde_json::to_string(&report).unwrap();
        assert_eq!(serde_json::from_str::<DoctorReport>(&text).unwrap(), report);
    }

    #[test]
    fn every_fix_is_runnable() {
        let p = paths("every-fix");
        let mut fake = FakeRunner::new();
        let pins = fetch::parse_pins(WITH_TBD);
        let report = report(&p, &mut fake, Some("SERIAL1"), &pins);
        assert!(!report.ok);
        assert_eq!(&names(&report)[..ORDER.len()], &ORDER[..]);
        for c in &report.checks {
            if c.name.starts_with("system") {
                continue;
            }
            assert!(!c.ok, "{} should be red", c.name);
            assert!(!c.fix.is_empty(), "{} has no fix", c.name);
            if let Some(rest) = c.fix.strip_prefix("flat3d-inject ") {
                let words: Vec<String> = rest.split_whitespace().map(str::to_string).collect();
                assert!(
                    crate::cli::parse_any(&words).is_ok(),
                    "{}: `{}` does not parse",
                    c.name,
                    c.fix
                );
            } else {
                assert!(
                    [
                        "flatpak",
                        "systemctl",
                        "steam",
                        "packaging/pin.sh",
                        "tools/build-win.sh"
                    ]
                    .iter()
                    .any(|start| c.fix.starts_with(start)),
                    "{}: `{}`",
                    c.name,
                    c.fix
                );
            }
        }
    }
}
