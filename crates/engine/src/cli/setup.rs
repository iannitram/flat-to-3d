//! `flat3d-inject setup packages | install-apps | launch-env [<appid>] | fetch <key>`.

use super::{Ctx, SetupCommand};
use crate::{
    api::{ApiError, LaunchText, SetupReport},
    exec::CommandRunner,
    injectors::fetch,
    link::wivrn::detect_server,
    paths::EnginePaths,
    setup::{missing, packages_command, parse_packages, Packages},
};
use std::path::PathBuf;

/// The pin for the WiVRn app on the headset.
const WIVRN_APK_KEY: &str = "wivrn-apk";

pub fn run(cmd: &SetupCommand, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    match cmd {
        SetupCommand::Packages => {
            let found = packages(ctx.runner)?;
            Ok(serde_json::json!({"wivrn": found.wivrn, "missing": missing(&found)}))
        }
        SetupCommand::InstallApps => to_value(install_apps(ctx.paths, ctx.runner)?),
        SetupCommand::LaunchEnv { appid } => to_value(launch_env(*appid, ctx.paths, ctx.runner)?),
        SetupCommand::Fetch { key } => to_value(crate::cli::setup_fetch::setup_fetch(
            key, ctx.paths, ctx.runner,
        )?),
    }
}

fn to_value<T: serde::Serialize>(report: T) -> Result<serde_json::Value, ApiError> {
    serde_json::to_value(report).map_err(|e| ApiError::new("io", &e.to_string()))
}

/// Ask the headset which setup apps it has.
fn packages(r: &mut dyn CommandRunner) -> Result<Packages, ApiError> {
    let out = r.run(&packages_command());
    if !out.success {
        return Err(ApiError::new("command-failed", out.stderr.trim()));
    }
    Ok(parse_packages(&out.stdout))
}

/// The last non-empty line of a command's stderr (or a fallback), for one-line errors.
fn last_line(stderr: &str, fallback: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

/// The one `*.apk` directly inside the fetched folder.
fn cached_apk(dir: &std::path::Path) -> Result<PathBuf, ApiError> {
    let not_found = || {
        ApiError::new(
            "not-found",
            &format!(
                "expected exactly one .apk file in {}; run: flat3d-inject setup fetch {WIVRN_APK_KEY}",
                dir.display()
            ),
        )
    };
    let mut apks = std::fs::read_dir(dir)
        .map_err(|_| not_found())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "apk"));
    match (apks.next(), apks.next()) {
        (Some(apk), None) => Ok(apk),
        _ => Err(not_found()),
    }
}

fn install_apps(p: &EnginePaths, r: &mut dyn CommandRunner) -> Result<SetupReport, ApiError> {
    let before = missing(&packages(r)?);
    if before.is_empty() {
        return Ok(SetupReport {
            steps: Vec::new(),
            summary: "nothing to install".into(),
        });
    }
    let mut steps = Vec::new();
    // WiVRn is the only app (ruling 42).
    if before.contains(&"WiVRn") {
        let fetched = crate::cli::setup_fetch::setup_fetch(WIVRN_APK_KEY, p, r)?;
        steps.extend(fetched.steps);
        let pin = fetch::pin(WIVRN_APK_KEY).map_err(ApiError::from)?;
        let apk = cached_apk(&fetch::cache_dir(&pin, p))?;
        let apk_arg = apk.to_string_lossy().into_owned();
        let out = r.run(&["adb".into(), "install".into(), "-r".into(), apk_arg]);
        if !out.success {
            return Err(ApiError::new(
                "command-failed",
                &last_line(&out.stderr, "adb install failed"),
            ));
        }
        steps.push(format!("installed {}", apk.display()));
    }
    let after = missing(&packages(r)?);
    if !after.is_empty() {
        return Err(ApiError::new(
            "command-failed",
            &format!(
                "still missing after install: {} ({})",
                after.join(", "),
                crate::setup::WIVRN_PACKAGE
            ),
        ));
    }
    Ok(SetupReport {
        steps,
        summary: format!("installed {}", before.join(", ")),
    })
}

/// The launch options a game needs to reach the WiVRn runtime, or the wrapper form for one game.
fn launch_env(
    appid: Option<u32>,
    p: &EnginePaths,
    r: &mut dyn CommandRunner,
) -> Result<LaunchText, ApiError> {
    if appid.is_some() {
        return Ok(LaunchText {
            options: format!("{} %command%", p.wrapper_bin().display()),
        });
    }
    let dirs = detect_server(r).map_err(|message| ApiError::new("not-found", &message))?;
    // The same 3 pairs as `wrapper::exec_spec`.
    Ok(LaunchText {
        options: format!(
            "PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 PRESSURE_VESSEL_FILESYSTEMS_RW={} STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%",
            dirs.app_dir.display()
        ),
    })
}
