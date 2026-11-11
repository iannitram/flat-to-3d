//! The app's IO boundary and CLI adapter. Pure mapping helpers live here too.
use crate::{Effect, Message};
use flat3d_inject::api::{ApiError, LinkSnapshot};
use flat3d_inject::face::headset_view::DEFAULT_LOG_LINES;
use flat3d_inject::face::logs::read_logs;
use flat3d_inject::paths::EnginePaths;
use flat3d_inject::rows::GameRow;
use flat3d_inject::setup::Packages;
use flat3d_inject::{state, sys};
use serde_json::Value;
use std::future::Future;
use std::path::{Path, PathBuf};

pub fn paths() -> Result<EnginePaths, ApiError> {
    EnginePaths::from_env(&std::env::vars().collect(), sys::current_uid())
}

pub fn cli_path(
    exe: &Path,
    src_dir: Option<&str>,
    bin: &Path,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let sibling = exe.parent().map(|parent| parent.join("flat3d-inject"));
    sibling
        .filter(|path| exists(path))
        .or_else(|| {
            src_dir
                .map(|dir| Path::new(dir).join("bin/flat3d-inject"))
                .filter(|path| exists(path))
        })
        .or_else(|| exists(bin).then(|| bin.to_path_buf()))
}

pub fn snapshot_or_stopped(
    result: Result<Option<LinkSnapshot>, String>,
    now: u64,
) -> Result<LinkSnapshot, String> {
    result.map(|snapshot| snapshot.map_or_else(|| LinkSnapshot::stopped(now), |s| s.effective(now)))
}

pub fn read_snapshot(p: &EnginePaths, now: u64) -> Result<LinkSnapshot, String> {
    snapshot_or_stopped(state::read_snapshot(p), now)
}

pub fn request(p: &EnginePaths, request: flat3d_inject::state::Request) -> Result<(), String> {
    state::write_request(p, request).map_err(|e| format!("io: {e}"))
}

pub fn effect_args(effect: &Effect) -> Option<Vec<String>> {
    match effect {
        Effect::SetSetting { key, on } => Some(vec![
            "setting".into(),
            "set".into(),
            (*key).into(),
            bool_word(*on).into(),
        ]),
        Effect::LoadRows { filter } => Some(vec!["rows".into(), "--filter".into(), filter.clone()]),
        Effect::SetSwitch { appid, on } => Some(vec![
            "switch".into(),
            "set".into(),
            appid.to_string(),
            bool_word(*on).into(),
        ]),
        Effect::Undo(id) => Some(vec!["undo".into(), id.to_string()]),
        Effect::ReadPackages => Some(
            crate::setup_page::SetupKind::Packages
                .args()
                .iter()
                .map(|s| (*s).into())
                .collect(),
        ),
        Effect::RunSetup(kind) => Some(kind.args().iter().map(|s| (*s).into()).collect()),
        Effect::ReadSnapshot | Effect::Request(_) | Effect::LoadLogs | Effect::Copy(_) => None,
    }
}

fn bool_word(on: bool) -> &'static str {
    if on {
        "on"
    } else {
        "off"
    }
}

pub fn parse_envelope(stdout: &str) -> Result<Value, ApiError> {
    let parsed: Value = serde_json::from_str(stdout).map_err(|_| {
        ApiError::new(
            "command-failed",
            &format!(
                "bad engine output: {}",
                stdout.chars().take(200).collect::<String>()
            ),
        )
    })?;
    if parsed.get("ok").and_then(Value::as_bool) == Some(true) {
        return Ok(parsed.get("data").cloned().unwrap_or(Value::Null));
    }
    if parsed.get("ok").and_then(Value::as_bool) == Some(false) {
        let error = parsed
            .get("error")
            .ok_or_else(|| ApiError::new("command-failed", "bad engine output: missing error"))?;
        let code = error.get("code").and_then(Value::as_str).ok_or_else(|| {
            ApiError::new("command-failed", "bad engine output: missing error code")
        })?;
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ApiError::new("command-failed", "bad engine output: missing error message")
            })?;
        return Err(ApiError::new(code, message));
    }
    Err(ApiError::new(
        "command-failed",
        "bad engine output: missing ok flag",
    ))
}

pub fn error_text(error: &ApiError) -> String {
    if error.code == "usage" {
        format!("app/CLI mismatch: {}", error.message)
    } else {
        format!("{}: {}", error.code, error.message)
    }
}

pub fn row_from_json(value: &Value) -> Option<GameRow> {
    Some(GameRow {
        appid: value.get("appid")?.as_u64()?.try_into().ok()?,
        name: value.get("name")?.as_str()?.to_owned(),
        status: value.get("status")?.as_str()?.to_owned(),
        status_label: value.get("status_label")?.as_str()?.to_owned(),
        status_colour: value.get("status_colour")?.as_str()?.to_owned(),
        switch_on: value.get("switch_on")?.as_bool()?,
        plays_3d: value.get("plays_3d")?.as_bool()?,
        flat_reason: match value.get("flat_reason")? {
            Value::Null => None,
            Value::String(s) => Some(s.clone()),
            _ => return None,
        },
    })
}

pub fn run_cli_at(cli: &Path, args: &[String]) -> Result<Value, ApiError> {
    let output = std::process::Command::new(cli)
        .args(args)
        .arg("--json")
        .output()
        .map_err(|e| ApiError::new("io", &format!("{}: {e}", cli.display())))?;
    let data = parse_envelope(&String::from_utf8_lossy(&output.stdout))?;
    if output.status.success() {
        return Ok(data);
    }
    let message = if let Some(checks) = data.get("checks").and_then(Value::as_array) {
        let red: Vec<String> = checks
            .iter()
            .filter(|c| c.get("ok").and_then(Value::as_bool) == Some(false))
            .filter_map(|c| {
                Some(format!(
                    "{}: {}",
                    c.get("name")?.as_str()?,
                    c.get("fix")?.as_str()?
                ))
            })
            .collect();
        if red.is_empty() {
            format!("CLI exited with {}", output.status)
        } else {
            red.join("; ")
        }
    } else {
        format!("CLI exited with {}", output.status)
    };
    Err(ApiError::new("command-failed", &message))
}

pub fn run_cli(args: &[String]) -> Result<Value, ApiError> {
    let exe = std::env::current_exe().map_err(|e| ApiError::new("io", &e.to_string()))?;
    let src_dir = std::env::var("FLAT3D_SRC_DIR").ok();
    let cli = cli_path(
        &exe,
        src_dir.as_deref(),
        Path::new("bin/flat3d-inject"),
        &|p| p.exists(),
    )
    .ok_or_else(|| {
        ApiError::new(
            "not-found",
            "flat3d-inject not found next to flat3d-app, in $FLAT3D_SRC_DIR/bin or in bin/",
        )
    })?;
    run_cli_at(&cli, args)
}

pub fn off_thread<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> impl Future<Output = Option<T>> {
    let (sender, receiver) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = sender.send(f());
    });
    async move { receiver.await.ok() }
}

/// Execute one effect on a worker and translate its result back into a UI message.
pub fn perform(effect: Effect) -> Message {
    use crate::{games_page, headset_page, logs_page, setup_page};
    let fail = |text: String| match &effect {
        Effect::ReadSnapshot => Message::Headset(headset_page::Msg::Snapshot(Err(text))),
        Effect::Request(_) | Effect::SetSetting { .. } => {
            Message::Headset(headset_page::Msg::Done(Err(text)))
        }
        Effect::LoadRows { .. } => Message::Games(games_page::Msg::Rows(Err(text))),
        Effect::SetSwitch { .. } => Message::SwitchSaved(Err(text)),
        Effect::Undo(_) => Message::Games(games_page::Msg::UndoDone(Err(text))),
        Effect::ReadPackages => Message::Setup(setup_page::Msg::Packages(Err(text))),
        Effect::RunSetup(_) => Message::Setup(setup_page::Msg::Done(Err(text))),
        Effect::LoadLogs => Message::Logs(logs_page::Msg::Failed(text)),
        Effect::Copy(_) => unreachable!("Copy is handled on the UI thread"),
    };
    let p = match paths() {
        Ok(p) => p,
        Err(e) => return fail(error_text(&e)),
    };
    match &effect {
        Effect::ReadSnapshot => Message::Headset(headset_page::Msg::Snapshot(read_snapshot(
            &p,
            sys::unix_now(),
        ))),
        Effect::Request(req) => Message::Headset(headset_page::Msg::Done(request(&p, *req))),
        Effect::LoadLogs => Message::Logs(logs_page::Msg::Lines(read_logs(&p, DEFAULT_LOG_LINES))),
        Effect::Copy(_) => unreachable!("Copy is handled on the UI thread"),
        Effect::ReadPackages => cli_result(&effect)
            .and_then(|data| {
                let wivrn = data
                    .get("wivrn")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| "bad packages data: wivrn".to_string())?;
                let gnirehtet = data
                    .get("gnirehtet")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| "bad packages data: gnirehtet".to_string())?;
                Ok(Packages { wivrn, gnirehtet })
            })
            .map_or_else(
                |e| Message::Setup(setup_page::Msg::Packages(Err(e))),
                |v| Message::Setup(setup_page::Msg::Packages(Ok(v))),
            ),
        Effect::Undo(_) => cli_result(&effect)
            .and_then(|data| {
                data.get("summary")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or_else(|| "bad undo data: summary".into())
            })
            .map_or_else(
                |e| Message::Games(games_page::Msg::UndoDone(Err(e))),
                |v| Message::Games(games_page::Msg::UndoDone(Ok(v))),
            ),
        Effect::LoadRows { .. } => cli_result(&effect)
            .and_then(|data| {
                data.as_array()
                    .ok_or_else(|| "bad rows data: expected array".to_string())?
                    .iter()
                    .map(|v| row_from_json(v).ok_or_else(|| format!("bad row: {v}")))
                    .collect()
            })
            .map_or_else(
                |e| Message::Games(games_page::Msg::Rows(Err(e))),
                |v| Message::Games(games_page::Msg::Rows(Ok(v))),
            ),
        Effect::RunSetup(_) => cli_result(&effect).map(|_| ()).map_or_else(
            |e| Message::Setup(setup_page::Msg::Done(Err(e))),
            |v| Message::Setup(setup_page::Msg::Done(Ok(v))),
        ),
        Effect::SetSetting { .. } => cli_result(&effect).map(|_| ()).map_or_else(
            |e| Message::Headset(headset_page::Msg::Done(Err(e))),
            |v| Message::Headset(headset_page::Msg::Done(Ok(v))),
        ),
        Effect::SetSwitch { .. } => cli_result(&effect).map(|_| ()).map_or_else(
            |e| Message::SwitchSaved(Err(e)),
            |_| Message::SwitchSaved(Ok(())),
        ),
    }
}

pub fn worker_stopped(effect: &Effect) -> Message {
    let text = "worker stopped".to_string();
    match effect {
        Effect::ReadSnapshot => Message::Headset(crate::headset_page::Msg::Snapshot(Err(text))),
        Effect::Request(_) | Effect::SetSetting { .. } => {
            Message::Headset(crate::headset_page::Msg::Done(Err(text)))
        }
        Effect::LoadRows { .. } => Message::Games(crate::games_page::Msg::Rows(Err(text))),
        Effect::SetSwitch { .. } => Message::SwitchSaved(Err(text)),
        Effect::Undo(_) => Message::Games(crate::games_page::Msg::UndoDone(Err(text))),
        Effect::ReadPackages => Message::Setup(crate::setup_page::Msg::Packages(Err(text))),
        Effect::RunSetup(_) => Message::Setup(crate::setup_page::Msg::Done(Err(text))),
        Effect::LoadLogs => Message::Logs(crate::logs_page::Msg::Failed(text)),
        Effect::Copy(_) => unreachable!("Copy is handled on the UI thread"),
    }
}

fn cli_result(effect: &Effect) -> Result<Value, String> {
    let args = effect_args(effect).expect("CLI effect has argv");
    run_cli(&args).map_err(|e| error_text(&e))
}
