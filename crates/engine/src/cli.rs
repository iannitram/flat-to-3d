//! Command line: `flat3d-inject list | inject <appid> | start-3d | stop-3d | status | undo <appid>`, each with
//! an optional `--json` that prints exactly one `Response` object on one line. Human text is the
//! default. The Decky backend (decky/main.py) calls these same commands with `--json`.

pub mod doctor;
pub mod engine;
pub mod inject;
pub mod link;
pub mod setup;
pub mod setup_fetch;
pub mod undo;
pub mod view;

use crate::api::{ApiError, Response};
use crate::exec::{CommandRunner, LinkRunner, ProcessRunner, SystemRunner};
use crate::paths::EnginePaths;
use serde_json::Value;
use std::collections::BTreeMap;

/// A parsed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Installed games with their stereo status.
    List,
    /// Inject a game by Steam appid.
    Inject { appid: u32 },
    /// Bring up the headset link.
    Start3d,
    /// Take the headset link down.
    Stop3d,
    /// The link's state.
    Status,
    /// Put a game back to flat from its undo record.
    Undo { appid: u32 },
}

impl Command {
    /// The command's word on the command line.
    pub fn name(&self) -> &'static str {
        match self {
            Command::List => "list",
            Command::Inject { .. } => "inject",
            Command::Start3d => "start-3d",
            Command::Stop3d => "stop-3d",
            Command::Status => "status",
            Command::Undo { .. } => "undo",
        }
    }
}

/// A parsed command line: the command, and whether `--json` was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub command: Command,
    pub json: bool,
}

/// The usage line for any bad command line.
pub const USAGE: &str =
    "usage: flat3d-inject list | inject <appid> | start-3d | stop-3d | status [--json]";
pub const INJECT_USAGE: &str = "usage: flat3d-inject inject <appid> [--json]";
pub const UNDO_USAGE: &str = "usage: flat3d-inject undo <appid> [--json]";

/// Parse the arguments after the program name. `--json` may stand anywhere. `Err` carries a
/// one-line usage message.
pub fn parse(args: &[String]) -> Result<Cli, String> {
    let json = args.iter().any(|a| a == "--json");
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json")
        .collect();
    let command = match words.as_slice() {
        ["list"] => Command::List,
        ["start-3d"] => Command::Start3d,
        ["stop-3d"] => Command::Stop3d,
        ["status"] => Command::Status,
        ["inject", appid] => match appid.parse::<u32>() {
            Ok(appid) => Command::Inject { appid },
            Err(_) => return Err(INJECT_USAGE.to_string()),
        },
        ["inject", ..] => return Err(INJECT_USAGE.to_string()),
        ["undo", appid] => match appid.parse::<u32>() {
            Ok(appid) => Command::Undo { appid },
            Err(_) => return Err(UNDO_USAGE.to_string()),
        },
        ["undo", ..] => return Err(UNDO_USAGE.to_string()),
        _ => return Err(USAGE.to_string()),
    };
    Ok(Cli { command, json })
}

/// What one run prints and returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 2 for a usage or not-built error, 1 for any other.
pub fn exit_code(error: &ApiError) -> i32 {
    match error.code.as_str() {
        "usage" | "not-built" => 2,
        _ => 1,
    }
}

/// Turn a command's result into what is printed. With `json`, stdout is one `Response` object on
/// one line; without, people's text goes to stdout and an error to stderr.
pub fn render(command: &str, result: Result<Value, ApiError>, json: bool) -> Output {
    let code = match &result {
        Ok(_) => 0,
        Err(e) => exit_code(e),
    };
    if json {
        let line = match result {
            Ok(v) => serde_json::to_string(&Response::success(command, v)),
            Err(e) => serde_json::to_string(&Response::<Value>::failure(command, e)),
        }
        .unwrap_or_default();
        return Output {
            code,
            stdout: format!("{line}\n"),
            stderr: String::new(),
        };
    }
    match result {
        Ok(v) => Output {
            code,
            stdout: format!("{}\n", serde_json::to_string_pretty(&v).unwrap_or_default()),
            stderr: String::new(),
        },
        Err(e) => Output {
            code,
            stdout: String::new(),
            stderr: format!("error: {}\n", e.message),
        },
    }
}

// The shared command model is expanded below; the locked parser/render functions above remain stable.

pub struct Ctx<'a> {
    pub paths: &'a EnginePaths,
    pub runner: &'a mut dyn CommandRunner,
    pub process: &'a mut dyn LinkRunner,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyCli {
    pub command: AnyCommand,
    pub json: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyCommand {
    Engine(Command),
    Link(link::LinkCommand),
    View(ViewCommand),
    Setup(SetupCommand),
    Doctor(DoctorCommand),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewCommand {
    View,
    Rows { filter: String },
    Launch { add: bool, options: String },
    Collection { current: Option<Vec<u32>> },
    Toasts { last_seq: u64 },
    Logs { limit: usize },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupCommand {
    Packages,
    InstallApps,
    LaunchEnv { appid: Option<u32> },
    Fetch { key: String },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorCommand {
    Run,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
    ReadOnly,
    Probes,
    WritesFiles,
    RunsCommands,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub id: &'static str,
    pub group: &'static str,
    pub args: &'static str,
    pub output: &'static str,
    pub effects: &'static [SideEffect],
}

const RO: &[SideEffect] = &[SideEffect::ReadOnly];
const PROBE: &[SideEffect] = &[SideEffect::Probes];
const WRITE: &[SideEffect] = &[SideEffect::WritesFiles];
const WRUN: &[SideEffect] = &[SideEffect::WritesFiles, SideEffect::RunsCommands];
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "list",
        group: "Engine",
        args: "",
        output: "Game (array)",
        effects: RO,
    },
    CommandSpec {
        id: "inject",
        group: "Engine",
        args: "<appid>",
        output: "InjectResult",
        effects: WRUN,
    },
    CommandSpec {
        id: "start-3d",
        group: "Engine",
        args: "",
        output: "LinkReport",
        effects: WRITE,
    },
    CommandSpec {
        id: "stop-3d",
        group: "Engine",
        args: "",
        output: "LinkReport",
        effects: WRITE,
    },
    CommandSpec {
        id: "status",
        group: "Engine",
        args: "",
        output: "LinkReport",
        effects: RO,
    },
    CommandSpec {
        id: "undo",
        group: "Engine",
        args: "<appid>",
        output: "UndoResult",
        effects: WRITE,
    },
    CommandSpec {
        id: "watch",
        group: "Link",
        args: "",
        output: "",
        effects: WRUN,
    },
    CommandSpec {
        id: "snapshot",
        group: "Link",
        args: "",
        output: "LinkSnapshot",
        effects: RO,
    },
    CommandSpec {
        id: "install",
        group: "Link",
        args: "",
        output: "InstallReport",
        effects: WRUN,
    },
    CommandSpec {
        id: "uninstall",
        group: "Link",
        args: "",
        output: "InstallReport",
        effects: WRUN,
    },
    CommandSpec {
        id: "switch get",
        group: "Link",
        args: "<appid>",
        output: "SwitchState",
        effects: RO,
    },
    CommandSpec {
        id: "switch set",
        group: "Link",
        args: "<appid> on|off",
        output: "SwitchState",
        effects: WRITE,
    },
    CommandSpec {
        id: "setting get",
        group: "Link",
        args: "<key>",
        output: "SettingState",
        effects: RO,
    },
    CommandSpec {
        id: "setting set",
        group: "Link",
        args: "<key> on|off",
        output: "SettingState",
        effects: WRITE,
    },
    CommandSpec {
        id: "run",
        group: "Link",
        args: "<appid> -- <command...>",
        output: "",
        effects: WRUN,
    },
    CommandSpec {
        id: "view",
        group: "View",
        args: "",
        output: "LinkView",
        effects: RO,
    },
    CommandSpec {
        id: "rows",
        group: "View",
        args: "[--filter <status>|all]",
        output: "GameRow (array)",
        effects: RO,
    },
    CommandSpec {
        id: "launch",
        group: "View",
        args: "add|remove --options <options>",
        output: "LaunchText",
        effects: RO,
    },
    CommandSpec {
        id: "collection",
        group: "View",
        args: "[--current <appids>]",
        output: "CollectionPlan",
        effects: RO,
    },
    CommandSpec {
        id: "toasts",
        group: "View",
        args: "<last_seq>",
        output: "ToastBatch",
        effects: RO,
    },
    CommandSpec {
        id: "logs",
        group: "View",
        args: "[--limit <n>]",
        output: "LogLines",
        effects: RO,
    },
    CommandSpec {
        id: "setup packages",
        group: "Setup",
        args: "",
        output: "{wivrn,missing}",
        effects: PROBE,
    },
    CommandSpec {
        id: "setup install-apps",
        group: "Setup",
        args: "",
        output: "SetupReport",
        effects: WRUN,
    },
    CommandSpec {
        id: "setup launch-env",
        group: "Setup",
        args: "[<appid>]",
        output: "LaunchText",
        effects: PROBE,
    },
    CommandSpec {
        id: "setup fetch",
        group: "Setup",
        args: "<key>",
        output: "SetupReport",
        effects: WRUN,
    },
    CommandSpec {
        id: "doctor",
        group: "Doctor",
        args: "",
        output: "DoctorReport",
        effects: PROBE,
    },
];

impl AnyCommand {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Engine(c) => c.name(),
            Self::Link(c) => match c {
                link::LinkCommand::Watch => "watch",
                link::LinkCommand::Snapshot => "snapshot",
                link::LinkCommand::Install => "install",
                link::LinkCommand::Uninstall => "uninstall",
                link::LinkCommand::SwitchGet { .. } => "switch get",
                link::LinkCommand::SwitchSet { .. } => "switch set",
                link::LinkCommand::SettingGet { .. } => "setting get",
                link::LinkCommand::SettingSet { .. } => "setting set",
                link::LinkCommand::Run { .. } => "run",
            },
            Self::View(c) => match c {
                ViewCommand::View => "view",
                ViewCommand::Rows { .. } => "rows",
                ViewCommand::Launch { .. } => "launch",
                ViewCommand::Collection { .. } => "collection",
                ViewCommand::Toasts { .. } => "toasts",
                ViewCommand::Logs { .. } => "logs",
            },
            Self::Setup(c) => match c {
                SetupCommand::Packages => "setup packages",
                SetupCommand::InstallApps => "setup install-apps",
                SetupCommand::LaunchEnv { .. } => "setup launch-env",
                SetupCommand::Fetch { .. } => "setup fetch",
            },
            Self::Doctor(_) => "doctor",
        }
    }
}

/// Whether `--json` stands before any `--` separator.
fn json_before_separator(args: &[String]) -> bool {
    let sep = args.iter().position(|x| x == "--").unwrap_or(args.len());
    args[..sep].iter().any(|x| x == "--json")
}

pub fn parse_any(args: &[String]) -> Result<AnyCli, ApiError> {
    let sep = args.iter().position(|x| x == "--").unwrap_or(args.len());
    let mut head: Vec<String> = args[..sep]
        .iter()
        .filter(|x| x.as_str() != "--json")
        .cloned()
        .collect();
    let json = json_before_separator(args);
    let mut full = head.clone();
    if sep < args.len() {
        full.push("--".into());
        full.extend(args[sep + 1..].iter().cloned());
    }
    if let Some(result) = link::parse_link(&full) {
        return result
            .map(|c| AnyCli {
                command: AnyCommand::Link(c),
                json,
            })
            .map_err(|e| ApiError::new("usage", &e));
    }
    if let Some(result) = view::parse(&full) {
        return result
            .map(|c| AnyCli {
                command: AnyCommand::View(c),
                json,
            })
            .map_err(|e| ApiError::new("usage", &e));
    }
    let setup = match head.as_slice() {
        [a,b] if a=="setup" && b=="packages" => Some(Ok(SetupCommand::Packages)),
        [a,b] if a=="setup" && b=="install-apps" => Some(Ok(SetupCommand::InstallApps)),
        [a,b] if a=="setup" && b=="launch-env" => Some(Ok(SetupCommand::LaunchEnv{appid:None})),
        [a,b,n] if a=="setup" && b=="launch-env" => Some(n.parse::<u32>().map(|appid|SetupCommand::LaunchEnv{appid:Some(appid)}).map_err(|_|"usage: flat3d-inject setup launch-env [<appid>]".to_string())),
        [a,b,key] if a=="setup" && b=="fetch" => Some(Ok(SetupCommand::Fetch{key:key.clone()})),
        [a,b] if a=="setup" && b=="fetch" => Some(Err("usage: flat3d-inject setup fetch <key>".into())),
        [a,..] if a=="setup" => Some(Err("usage: flat3d-inject setup packages | setup install-apps | setup launch-env [<appid>] | setup fetch <key>".into())),
        _ => None,
    };
    if let Some(result) = setup {
        return result
            .map(|c| AnyCli {
                command: AnyCommand::Setup(c),
                json,
            })
            .map_err(|e| ApiError::new("usage", &e));
    }
    if head.as_slice() == ["doctor"] {
        return Ok(AnyCli {
            command: AnyCommand::Doctor(DoctorCommand::Run),
            json,
        });
    }
    if head.first().is_some_and(|x| x == "doctor") {
        return Err(ApiError::new("usage", "usage: flat3d-inject doctor"));
    }
    head.extend(if sep < args.len() {
        let mut x = vec!["--".to_string()];
        x.extend(args[sep + 1..].iter().cloned());
        x
    } else {
        vec![]
    });
    parse(&head)
        .map(|c| AnyCli {
            command: AnyCommand::Engine(c.command),
            json: json || c.json,
        })
        .map_err(|e| ApiError::new("usage", &e))
}

pub fn ok_exit(cmd: &AnyCommand, data: &serde_json::Value) -> i32 {
    if matches!(cmd, AnyCommand::Doctor(DoctorCommand::Run))
        && data.get("ok") == Some(&serde_json::Value::Bool(false))
    {
        1
    } else {
        0
    }
}

fn dispatch(cmd: &AnyCommand, ctx: &mut Ctx<'_>) -> Result<serde_json::Value, ApiError> {
    match cmd {
        AnyCommand::Engine(c @ (Command::List | Command::Status)) => engine::run(c, ctx),
        AnyCommand::Engine(c @ Command::Inject { .. }) => inject::run(c, ctx),
        AnyCommand::Engine(c @ Command::Undo { .. }) => undo::run(c, ctx),
        AnyCommand::Engine(Command::Start3d) => {
            let report = link::request_3d(
                crate::state::Request::Start3d,
                ctx.paths,
                crate::sys::unix_now(),
            )?;
            serde_json::to_value(report)
                .map_err(|e| ApiError::new("command-failed", &e.to_string()))
        }
        AnyCommand::Engine(Command::Stop3d) => {
            let report = link::request_3d(
                crate::state::Request::Stop3d,
                ctx.paths,
                crate::sys::unix_now(),
            )?;
            serde_json::to_value(report)
                .map_err(|e| ApiError::new("command-failed", &e.to_string()))
        }
        AnyCommand::Link(c) => link::run(c, ctx),
        AnyCommand::View(c) => view::run(c, ctx),
        AnyCommand::Setup(c) => setup::run(c, ctx),
        AnyCommand::Doctor(c) => doctor::run(c, ctx),
    }
}

pub fn execute_with(args: &[String], ctx: &mut Ctx<'_>) -> Output {
    match parse_any(args) {
        Ok(parsed) => {
            let name = match &parsed.command {
                AnyCommand::Engine(c) => c.name(),
                _ => parsed.command.id().split_whitespace().next().unwrap_or(""),
            };
            let result = dispatch(&parsed.command, ctx);
            let hook = result
                .as_ref()
                .ok()
                .map(|data| ok_exit(&parsed.command, data));
            let mut out = render(name, result, parsed.json);
            if let Some(code) = hook {
                out.code = code;
            }
            out
        }
        Err(e) => render("", Err(e), json_before_separator(args)),
    }
}

pub fn execute(args: &[String]) -> Output {
    let parsed = match parse_any(args) {
        Ok(p) => p,
        Err(e) => return render("", Err(e), json_before_separator(args)),
    };
    let env: BTreeMap<String, String> = std::env::vars().collect();
    let paths = match EnginePaths::from_env(&env, crate::sys::current_uid()) {
        Ok(p) => p,
        Err(e) => return render("", Err(e), parsed.json),
    };
    let mut runner = SystemRunner;
    let mut process = ProcessRunner;
    let mut ctx = Ctx {
        paths: &paths,
        runner: &mut runner,
        process: &mut process,
        json: parsed.json,
    };
    execute_with(args, &mut ctx)
}

/// Run a command line: print its output and return the process exit code.
pub fn run(args: &[String]) -> i32 {
    let out = execute(args);
    print!("{}", out.stdout);
    eprint!("{}", out.stderr);
    out.code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_parses_without_json() {
        let expected = Cli {
            command: Command::List,
            json: false,
        };
        assert_eq!(parse(&["list".to_string()]), Ok(expected));
    }

    #[test]
    fn json_flag_may_stand_between_command_and_argument() {
        let expected = Cli {
            command: Command::Inject { appid: 2651280 },
            json: true,
        };
        let args = [
            "inject".to_string(),
            "--json".to_string(),
            "2651280".to_string(),
        ];
        assert_eq!(parse(&args), Ok(expected));
    }

    #[test]
    fn human_not_built_result_goes_to_stderr() {
        let error = crate::NotBuilt("cli::engine::run (WP-07)").into();
        let out = render("list", Err(error), false);

        assert_eq!(out.code, 2);
        assert!(out.stdout.is_empty());
        assert_eq!(
            out.stderr,
            "error: not built yet: cli::engine::run (WP-07)\n"
        );
    }

    #[test]
    fn dispatch_routes_each_family() {
        let rows: &[&[&str]] = &[
            &["list"],
            &["status"],
            &["inject", "5"],
            &["undo", "5"],
            &["start-3d"],
            &["stop-3d"],
            &["snapshot"],
            &["view"],
            &["setup", "packages"],
            &["doctor"],
        ];
        let home = std::env::temp_dir().join(format!("flat3d-cli-dispatch-{}", std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        for row in rows {
            let av: Vec<String> = row.iter().map(|s| (*s).to_owned()).collect();
            let cmd = parse_any(&av).unwrap().command;
            let mut env = BTreeMap::new();
            env.insert("HOME".to_string(), home.display().to_string());
            env.insert("XDG_RUNTIME_DIR".to_string(), home.display().to_string());
            let paths = EnginePaths::from_env(&env, 1).unwrap();
            let mut runner = crate::testing::FakeRunner::new();
            let mut process = ProcessRunner;
            let mut ctx = Ctx {
                paths: &paths,
                runner: &mut runner,
                process: &mut process,
                json: false,
            };
            let outcome = dispatch(&cmd, &mut ctx);
            if let Err(err) = outcome {
                if err.code == "not-built" {
                    let expected = match cmd {
                        AnyCommand::Engine(Command::Start3d | Command::Stop3d) => {
                            "cli::link::request_3d"
                        }
                        AnyCommand::Engine(Command::List | Command::Status) => "cli::engine::run",
                        AnyCommand::Engine(Command::Inject { .. }) => "cli::inject::run",
                        AnyCommand::Link(_) => "cli::link::run",
                        AnyCommand::View(_) => "cli::view::run",
                        AnyCommand::Setup(_) => "cli::setup::run",
                        AnyCommand::Doctor(_) => "cli::doctor::run",
                        AnyCommand::Engine(Command::Undo { .. }) => {
                            panic!("undo is already implemented")
                        }
                    };
                    assert!(err.message.contains(expected), "{row:?}: {}", err.message);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&home);
        assert_eq!(
            ok_exit(
                &AnyCommand::Doctor(DoctorCommand::Run),
                &serde_json::json!({"ok":false})
            ),
            1
        );
        assert_eq!(
            ok_exit(
                &AnyCommand::Doctor(DoctorCommand::Run),
                &serde_json::json!({"ok":true})
            ),
            0
        );
        assert_eq!(
            ok_exit(
                &AnyCommand::Engine(Command::List),
                &serde_json::json!({"ok":false})
            ),
            0
        );
    }
}
