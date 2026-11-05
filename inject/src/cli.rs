//! Command line: `flat3d-inject list | inject <appid> | start-3d | stop-3d | status`, each with
//! an optional `--json` that prints exactly one `Response` object on one line. Human text is the
//! default. The Decky backend (decky/main.py) calls these same commands with `--json`.

use crate::api::{ApiError, Response};
use crate::NotBuilt;
use serde_json::Value;

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
const INJECT_USAGE: &str = "usage: flat3d-inject inject <appid> [--json]";

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

/// Every command body is wired to the engine by #27.
fn dispatch(_command: &Command) -> Result<Value, ApiError> {
    Err(NotBuilt("cli: the command bodies are wired by #27").into())
}

/// Parse and run a command line without printing.
pub fn execute(args: &[String]) -> Output {
    match parse(args) {
        Ok(cli) => render(cli.command.name(), dispatch(&cli.command), cli.json),
        Err(usage) => {
            let json = args.iter().any(|a| a == "--json");
            render("", Err(ApiError::new("usage", &usage)), json)
        }
    }
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
}
