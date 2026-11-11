pub enum Target {
    Exe(String),
    Pid(u32),
}

pub struct Args {
    pub target: Target,
    pub delay_s: u64,
    pub wait_s: u64,
    pub dlls: Vec<String>,
}

pub const DEFAULT_DELAY_S: u64 = 15;
pub const DEFAULT_WAIT_S: u64 = 120;
pub const EXIT_USAGE: u8 = 2;
pub const EXIT_NOT_WINDOWS: u8 = 3;
pub const EXIT_NO_GAME: u8 = 4;
pub const EXIT_INJECT: u8 = 5;
pub const USAGE: &str = "usage: flat3d-uevr-inject (--exe <stem> | --pid <n>) [--delay <secs>] [--wait <secs>] --dll <path> [--dll <path>]...";

fn is_absolute_windows_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\'
}

fn exe_stem(value: &str) -> Result<String, String> {
    let suffix_start = value.len().saturating_sub(4);
    let stem = if value
        .get(suffix_start..)
        .is_some_and(|suffix| suffix.eq_ignore_ascii_case(".exe"))
    {
        value.get(..suffix_start).unwrap_or(value)
    } else {
        value
    };
    if stem.is_empty() || stem.contains(['/', '\\']) || stem == "." || stem == ".." {
        return Err("--exe must be a process stem, optionally ending in .exe".into());
    }
    Ok(stem.to_owned())
}

pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut exe = None;
    let mut pid = None;
    let mut delay_s = None;
    let mut wait_s = None;
    let mut dlls = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].as_str();
        if flag.contains('=') {
            return Err(format!(
                "flags take their value in the next argument: {flag}"
            ));
        }
        i += 1;
        let value = match flag {
            "--exe" | "--pid" | "--delay" | "--wait" | "--dll" => {
                let Some(value) = argv.get(i) else {
                    return Err(format!("missing value for {flag}"));
                };
                if value.starts_with("--") {
                    return Err(format!("missing value for {flag}"));
                }
                i += 1;
                value
            }
            _ => return Err(format!("unknown flag: {flag}")),
        };
        match flag {
            "--exe" => {
                if exe.is_some() {
                    return Err("--exe may appear only once".into());
                }
                exe = Some(exe_stem(value)?);
            }
            "--pid" => {
                if pid.is_some() {
                    return Err("--pid may appear only once".into());
                }
                let parsed = value
                    .parse::<u32>()
                    .map_err(|_| "--pid must be a positive u32")?;
                if parsed == 0 {
                    return Err("--pid must be greater than zero".into());
                }
                pid = Some(parsed);
            }
            "--delay" => {
                if delay_s.is_some() {
                    return Err("--delay may appear only once".into());
                }
                delay_s = Some(value.parse::<u64>().map_err(|_| "--delay must be a u64")?);
            }
            "--wait" => {
                if wait_s.is_some() {
                    return Err("--wait may appear only once".into());
                }
                wait_s = Some(value.parse::<u64>().map_err(|_| "--wait must be a u64")?);
            }
            "--dll" => {
                if !is_absolute_windows_path(value) {
                    return Err(format!("--dll requires an absolute Windows path: {value}"));
                }
                dlls.push(value.clone());
            }
            _ => unreachable!(),
        }
    }
    let target = match (exe, pid) {
        (Some(stem), None) => Target::Exe(stem),
        (None, Some(pid)) => Target::Pid(pid),
        (Some(_), Some(_)) => return Err("choose exactly one of --exe or --pid".into()),
        (None, None) => return Err("one of --exe or --pid is required".into()),
    };
    if dlls.is_empty() {
        return Err("at least one --dll is required".into());
    }
    Ok(Args {
        target,
        delay_s: delay_s.unwrap_or(DEFAULT_DELAY_S),
        wait_s: wait_s.unwrap_or(DEFAULT_WAIT_S),
        dlls,
    })
}

pub fn error_line(token: &str, msg: &str, fix: &str) -> String {
    format!("flat3d-uevr-inject: error[{token}] {msg}; fix: {fix}")
}
