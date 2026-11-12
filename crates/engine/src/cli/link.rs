//! The link service's commands. docs/code-map.md, API shape.
//!
//! Parsing only. `parse_link` is called first by the command line; it answers `None` for
//! any word that is not ours, so the caller can try its own commands.

/// A setting the link service can switch on or off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingKey {
    /// Automatic stereo injection (`auto-3d`).
    Auto3d,
}

impl SettingKey {
    /// The word used on the command line and in the Decky panel.
    pub fn as_str(&self) -> &'static str {
        match self {
            SettingKey::Auto3d => "auto-3d",
        }
    }
}

/// A parsed link command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkCommand {
    /// Watch for games starting and stopping.
    Watch,
    /// Take a snapshot of the link state.
    Snapshot,
    /// Install the link service.
    Install,
    /// Remove the link service.
    Uninstall,
    /// Read whether a game's stereo switch is on.
    SwitchGet { appid: u32 },
    /// Turn a game's stereo switch on or off.
    SwitchSet { appid: u32, on: bool },
    /// Read a setting.
    SettingGet { key: SettingKey },
    /// Turn a setting on or off.
    SettingSet { key: SettingKey, on: bool },
    /// Run a game's command words; `argv` is kept as given.
    Run { appid: u32, argv: Vec<String> },
}

pub fn run(
    cmd: &LinkCommand,
    ctx: &mut super::Ctx<'_>,
) -> Result<serde_json::Value, crate::api::ApiError> {
    let now = crate::sys::unix_now();
    let pid = std::process::id();
    let env: std::collections::BTreeMap<String, String> = std::env::vars().collect();
    let src_dir = std::env::current_exe()
        .map_err(|error| crate::api::ApiError::new("io", &error.to_string()))?
        .parent()
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| crate::api::ApiError::new("io", "executable has no parent directory"))?;
    let appid_added = match cmd {
        LinkCommand::Run { appid, .. }
            if !env.contains_key("SteamAppId") && !env.contains_key("SteamGameId") =>
        {
            Some(*appid)
        }
        _ => None,
    };
    let mut io = LinkIo {
        now,
        pid,
        env,
        src_dir,
        exec: &mut |launch| exec_launch(launch, appid_added, pid),
        watch: &mut |paths| crate::watch::run(&paths.watch_paths()),
    };
    run_with(cmd, ctx, &mut io)
}

pub fn request_3d(
    r: crate::state::Request,
    p: &crate::paths::EnginePaths,
    now: u64,
) -> Result<crate::api::LinkReport, crate::api::ApiError> {
    let snapshot = crate::state::read_snapshot(p).map_err(|_| {
        crate::api::ApiError::new(
            "link-not-ready",
            "Link service is not running. Start it: systemctl --user start flat3d-link.service (first time: flat3d-inject install).",
        )
    })?;
    let Some(snapshot) = snapshot else {
        return Err(crate::api::ApiError::new(
            "link-not-ready",
            "Link service is not running. Start it: systemctl --user start flat3d-link.service (first time: flat3d-inject install).",
        ));
    };
    if crate::state::is_stale(snapshot.updated, now) {
        return Err(crate::api::ApiError::new(
            "link-not-ready",
            &format!(
                "Link service stopped updating (last update {} s ago). Restart it: systemctl --user restart flat3d-link.service.",
                now.saturating_sub(snapshot.updated)
            ),
        ));
    }
    crate::state::write_request(p, r)
        .map_err(|error| crate::api::ApiError::new("io", &error.to_string()))?;
    Ok(crate::api::LinkReport {
        state: snapshot.state,
        message: format!("Requested {}.", crate::state::request_text(r)),
    })
}

struct LinkIo<'a> {
    now: u64,
    pid: u32,
    env: std::collections::BTreeMap<String, String>,
    src_dir: std::path::PathBuf,
    exec: &'a mut dyn FnMut(&crate::wrapper::Launch) -> std::io::Result<()>,
    watch: &'a mut dyn FnMut(&crate::paths::EnginePaths),
}

fn run_with(
    cmd: &LinkCommand,
    ctx: &mut super::Ctx<'_>,
    io: &mut LinkIo<'_>,
) -> Result<serde_json::Value, crate::api::ApiError> {
    use crate::api::{ApiError, LinkSnapshot, SettingState, SwitchState};
    let value = match cmd {
        LinkCommand::Watch => {
            (io.watch)(ctx.paths);
            serde_json::Value::Null
        }
        LinkCommand::Snapshot => {
            let snapshot = crate::state::read_snapshot(ctx.paths)
                .map_err(|error| ApiError::new("io", &error))?
                .unwrap_or_else(|| LinkSnapshot::stopped(io.now));
            serde_json::to_value(snapshot)
                .map_err(|error| ApiError::new("command-failed", &error.to_string()))?
        }
        LinkCommand::SwitchGet { appid } => {
            let settings = crate::switches::read_settings(&ctx.paths.settings())
                .map_err(|error| ApiError::new("io", &error))?;
            serde_json::to_value(SwitchState {
                appid: *appid,
                on: settings.game_on(*appid),
            })
            .map_err(|error| ApiError::new("command-failed", &error.to_string()))?
        }
        LinkCommand::SwitchSet { appid, on } => {
            let path = ctx.paths.settings();
            let mut settings = crate::switches::read_settings(&path)
                .map_err(|error| ApiError::new("io", &error))?;
            settings.set_game(*appid, *on);
            crate::switches::write_settings(&path, &settings)
                .map_err(|error| ApiError::new("io", &error.to_string()))?;
            serde_json::to_value(SwitchState {
                appid: *appid,
                on: *on,
            })
            .map_err(|error| ApiError::new("command-failed", &error.to_string()))?
        }
        LinkCommand::SettingGet { key } | LinkCommand::SettingSet { key, .. } => {
            let path = ctx.paths.settings();
            let mut settings = crate::switches::read_settings(&path)
                .map_err(|error| ApiError::new("io", &error))?;
            let on = match cmd {
                LinkCommand::SettingGet { .. } => match key {
                    SettingKey::Auto3d => settings.auto_3d,
                },
                LinkCommand::SettingSet { on, .. } => {
                    match key {
                        SettingKey::Auto3d => settings.auto_3d = *on,
                    }
                    crate::switches::write_settings(&path, &settings)
                        .map_err(|error| ApiError::new("io", &error.to_string()))?;
                    *on
                }
                _ => unreachable!(),
            };
            serde_json::to_value(SettingState {
                key: key.as_str().into(),
                on,
            })
            .map_err(|error| ApiError::new("command-failed", &error.to_string()))?
        }
        LinkCommand::Install => {
            let home = ctx.paths.home.to_string_lossy();
            let src_dir = io.src_dir.to_string_lossy();
            let report = crate::install::execute(
                &crate::install::install_plan(&home, &src_dir),
                ctx.runner,
            )?;
            serde_json::to_value(report)
                .map_err(|error| ApiError::new("command-failed", &error.to_string()))?
        }
        LinkCommand::Uninstall => {
            let home = ctx.paths.home.to_string_lossy();
            let report = crate::install::execute_uninstall(
                &crate::install::uninstall_plan(&home),
                ctx.runner,
            )?;
            serde_json::to_value(report)
                .map_err(|error| ApiError::new("command-failed", &error.to_string()))?
        }
        LinkCommand::Run { appid, argv } => {
            let mut env = io.env.clone();
            if !env.contains_key("SteamAppId") && !env.contains_key("SteamGameId") {
                env.insert("SteamAppId".into(), appid.to_string());
            }
            let (launch, result) = crate::wrapper::run_with(
                argv,
                &env,
                ctx.paths,
                io.pid,
                io.now,
                ctx.runner,
                |launch| (io.exec)(launch),
            );
            result.map_err(|error| ApiError::new("command-failed", &error.to_string()))?;
            serde_json::json!({"message": launch.message})
        }
    };
    Ok(value)
}

fn exec_launch(
    launch: &crate::wrapper::Launch,
    appid_added: Option<u32>,
    pid: u32,
) -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;

    let argv = &launch.command.argv;
    let program = argv
        .first()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no game command"))?;
    let mut command = std::process::Command::new(program);
    command
        .args(&argv[1..])
        .envs(launch.command.env.iter().cloned());
    for key in &launch.command.unset {
        command.env_remove(key);
    }
    if let Some(appid) = appid_added {
        command.env("SteamAppId", appid.to_string());
    }
    if let Some(sidecar) = &launch.command.sidecar {
        if let Some((program, args)) = sidecar.split_first() {
            std::process::Command::new(program)
                .args(args)
                .arg("--until-pid")
                .arg(pid.to_string())
                .spawn()?;
        }
    }
    Err(command.exec())
}

/// Parse the arguments after the program name. `None` when the first word is not one of
/// ours (or there are no words); `Some(Err)` carries a usage message for a bad form.
pub fn parse_link(args: &[String]) -> Option<Result<LinkCommand, String>> {
    match args.first().map(String::as_str) {
        Some("watch") => Some(Ok(LinkCommand::Watch)),
        Some("snapshot") => Some(Ok(LinkCommand::Snapshot)),
        Some("install") => Some(Ok(LinkCommand::Install)),
        Some("uninstall") => Some(Ok(LinkCommand::Uninstall)),
        Some("switch") => Some(parse_switch(&args[1..])),
        Some("setting") => Some(parse_setting(&args[1..])),
        Some("run") => Some(parse_run(&args[1..])),
        _ => None,
    }
}

/// A usage message for the given form, e.g. `switch set <appid> on|off`.
fn usage(form: &str) -> String {
    format!("usage: flat3d-inject {form}")
}

fn parse_appid(word: &str) -> Option<u32> {
    word.parse().ok()
}

fn parse_on_off(word: &str) -> Option<bool> {
    match word {
        "on" => Some(true),
        "off" => Some(false),
        _ => None,
    }
}

fn parse_key(word: &str) -> Option<SettingKey> {
    match word {
        "auto-3d" => Some(SettingKey::Auto3d),
        _ => None,
    }
}

fn parse_switch(rest: &[String]) -> Result<LinkCommand, String> {
    match rest {
        [verb, appid] if verb == "get" => parse_appid(appid)
            .map(|appid| LinkCommand::SwitchGet { appid })
            .ok_or_else(|| usage("switch get <appid>")),
        [verb, appid, state] if verb == "set" => match (parse_appid(appid), parse_on_off(state)) {
            (Some(appid), Some(on)) => Ok(LinkCommand::SwitchSet { appid, on }),
            _ => Err(usage("switch set <appid> on|off")),
        },
        _ => Err(usage("switch get <appid> | switch set <appid> on|off")),
    }
}

fn parse_setting(rest: &[String]) -> Result<LinkCommand, String> {
    match rest {
        [verb, key] if verb == "get" => parse_key(key)
            .map(|key| LinkCommand::SettingGet { key })
            .ok_or_else(|| usage("setting get <name>")),
        [verb, key, state] if verb == "set" => match (parse_key(key), parse_on_off(state)) {
            (Some(key), Some(on)) => Ok(LinkCommand::SettingSet { key, on }),
            _ => Err(usage("setting set <name> on|off")),
        },
        _ => Err(usage("setting get <name> | setting set <name> on|off")),
    }
}

fn parse_run(rest: &[String]) -> Result<LinkCommand, String> {
    const FORM: &str = "run <appid> -- <command words...>";
    match rest {
        [appid, dashes, words @ ..] if dashes == "--" && !words.is_empty() => {
            match parse_appid(appid) {
                Some(appid) => Ok(LinkCommand::Run {
                    appid,
                    argv: words.to_vec(),
                }),
                None => Err(usage(FORM)),
            }
        }
        _ => Err(usage(FORM)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::ProcessRunner;
    use crate::testing::FakeRunner;
    use std::collections::BTreeMap;

    fn setup(name: &str) -> (std::path::PathBuf, crate::paths::EnginePaths) {
        let root =
            std::env::temp_dir().join(format!("flat3d-cli-link-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut env = BTreeMap::new();
        env.insert(
            "HOME".into(),
            root.join("home").to_string_lossy().into_owned(),
        );
        env.insert(
            "XDG_RUNTIME_DIR".into(),
            root.join("run").to_string_lossy().into_owned(),
        );
        let paths = crate::paths::EnginePaths::from_env(&env, 1000).unwrap();
        std::fs::create_dir_all(&paths.home).unwrap();
        (root, paths)
    }

    #[test]
    fn run_keeps_argv_records_before_exec_and_maps_exec_errors() {
        let (_root, paths) = setup("run");
        let env = BTreeMap::from([
            ("HOME".into(), paths.home.to_string_lossy().into_owned()),
            (
                "XDG_RUNTIME_DIR".into(),
                paths
                    .state_dir
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            ),
        ]);
        let mut runner = FakeRunner::new();
        let mut process = ProcessRunner;
        let mut ctx = super::super::Ctx {
            paths: &paths,
            runner: &mut runner,
            process: &mut process,
            json: false,
        };
        let argv = vec!["game".into(), "--json".into()];
        let observed = std::rc::Rc::new(std::cell::RefCell::new(None));
        let observed_exec = observed.clone();
        let state_dir = paths.state_dir.clone();
        let mut exec = move |launch: &crate::wrapper::Launch| {
            let record = std::fs::read_to_string(crate::state::run_record_path(
                &state_dir.to_string_lossy(),
                41,
            ))
            .unwrap();
            *observed_exec.borrow_mut() =
                Some((launch.command.argv.clone(), launch.record.appid, record));
            Ok(())
        };
        let mut watch = |_: &crate::paths::EnginePaths| {};
        let mut link_io = LinkIo {
            now: 1000,
            pid: 41,
            env: env.clone(),
            src_dir: std::env::temp_dir(),
            exec: &mut exec,
            watch: &mut watch,
        };
        let value = run_with(
            &LinkCommand::Run {
                appid: 5,
                argv: argv.clone(),
            },
            &mut ctx,
            &mut link_io,
        )
        .unwrap();
        assert_eq!(value["message"], "flat: snapshot is missing");
        let observed = observed.borrow_mut().take().unwrap();
        assert_eq!(observed.0, argv);
        assert_eq!(observed.1, 5);
        assert!(observed.2.contains("appid\t5\n"));

        let mut exec = |_: &crate::wrapper::Launch| Err(std::io::Error::other("exec failed"));
        let mut watch = |_: &crate::paths::EnginePaths| {};
        let mut link_io = LinkIo {
            now: 1000,
            pid: 42,
            env,
            src_dir: std::env::temp_dir(),
            exec: &mut exec,
            watch: &mut watch,
        };
        let error =
            run_with(&LinkCommand::Run { appid: 5, argv }, &mut ctx, &mut link_io).unwrap_err();
        assert_eq!(error.code, "command-failed");
        assert_eq!(super::super::render("run", Err(error), true).code, 1);
    }

    #[test]
    fn watch_calls_its_closure_once() {
        let (_root, paths) = setup("watch");
        let mut runner = FakeRunner::new();
        let mut process = ProcessRunner;
        let mut ctx = super::super::Ctx {
            paths: &paths,
            runner: &mut runner,
            process: &mut process,
            json: false,
        };
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let observed_calls = calls.clone();
        let mut exec = |_: &crate::wrapper::Launch| Ok(());
        let mut watch =
            move |_: &crate::paths::EnginePaths| observed_calls.set(observed_calls.get() + 1);
        let mut link_io = LinkIo {
            now: 1000,
            pid: 1,
            env: BTreeMap::new(),
            src_dir: std::env::temp_dir(),
            exec: &mut exec,
            watch: &mut watch,
        };
        assert_eq!(
            run_with(&LinkCommand::Watch, &mut ctx, &mut link_io).unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn install_and_uninstall_use_the_frozen_install_plans() {
        let (root, paths) = setup("install");
        let src = root.join("release");
        std::fs::create_dir_all(src.join("tools")).unwrap();
        for path in [
            "flat3d-inject",
            "flat3d-run",
            "tools/adb",
            "tools/gnirehtet",
            "tools/gnirehtet.apk",
        ] {
            std::fs::write(src.join(path), b"").unwrap();
        }
        let env = BTreeMap::from([("HOME".into(), paths.home.to_string_lossy().into_owned())]);
        let mut runner = FakeRunner::new();
        let mut process = ProcessRunner;
        {
            let mut ctx = super::super::Ctx {
                paths: &paths,
                runner: &mut runner,
                process: &mut process,
                json: false,
            };
            let mut exec = |_: &crate::wrapper::Launch| Ok(());
            let mut watch = |_: &crate::paths::EnginePaths| {};
            let mut link_io = LinkIo {
                now: 1000,
                pid: 1,
                env,
                src_dir: src,
                exec: &mut exec,
                watch: &mut watch,
            };
            let report = run_with(&LinkCommand::Install, &mut ctx, &mut link_io).unwrap();
            assert!(serde_json::from_value::<crate::api::InstallReport>(report).is_ok());
            let report = run_with(&LinkCommand::Uninstall, &mut ctx, &mut link_io).unwrap();
            assert!(serde_json::from_value::<crate::api::InstallReport>(report).is_ok());
        }
        assert_eq!(runner.ran[0], ["systemctl", "--user", "daemon-reload"]);
        assert_eq!(
            runner.ran[1],
            [
                "systemctl",
                "--user",
                "enable",
                "--now",
                "flat3d-link.service"
            ]
        );
    }
}
