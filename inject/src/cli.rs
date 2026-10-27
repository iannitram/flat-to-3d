//! Command line: `flat3d-inject list | inject <appid> | start-3d`. The Decky backend
//! (decky/main.py) calls these same commands as a subprocess.

/// A parsed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Installed games with their stereo status.
    List,
    /// Inject a game by Steam appid.
    Inject { appid: u32 },
    /// Bring up the headset link.
    Start3d,
}

/// Parse the arguments after the program name. `Err` carries a one-line usage message.
pub fn parse(args: &[String]) -> Result<Command, String> {
    match args.first().map(String::as_str) {
        Some("list") => Ok(Command::List),
        Some("start-3d") => Ok(Command::Start3d),
        Some("inject") => args
            .get(1)
            .and_then(|a| a.parse::<u32>().ok())
            .map(|appid| Command::Inject { appid })
            .ok_or_else(|| "usage: flat3d-inject inject <appid>".to_string()),
        _ => Err("usage: flat3d-inject list | inject <appid> | start-3d".to_string()),
    }
}

/// Run a command line; returns the process exit code (0 ok, 2 not built or bad usage).
pub fn run(args: &[String]) -> i32 {
    match parse(args) {
        Ok(cmd) => {
            eprintln!("{cmd:?}: not built yet (skeleton)");
            2
        }
        Err(usage) => {
            eprintln!("{usage}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_exists() {
        assert_eq!(parse(&["list".to_string()]), Ok(Command::List));
        assert_eq!(run(&["start-3d".to_string()]), 2);
    }
}
