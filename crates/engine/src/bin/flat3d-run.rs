//! Steam's launch wrapper; preserve the original OS arguments and inherited env.

use flat3d_inject::wrapper::{run, RunPaths};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[cfg(unix)]
fn uid() -> u32 {
    extern "C" {
        fn getuid() -> u32;
    }
    // SAFETY: getuid takes no pointers and always returns the calling user's id.
    unsafe { getuid() }
}

#[cfg(not(unix))]
fn uid() -> u32 {
    0
}

fn main() {
    let argv: Vec<_> = std::env::args_os().skip(1).collect();
    let env: BTreeMap<_, _> = std::env::vars_os()
        .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)))
        .collect();
    let home = PathBuf::from(env.get("HOME").map(String::as_str).unwrap_or_default());
    let paths = RunPaths {
        state: flat3d_inject::state::state_dir(
            env.get("XDG_RUNTIME_DIR").map(String::as_str),
            uid(),
        ),
        settings: home.join(".config/flat3d/settings"),
        recipes: home.join(".local/share/flat3d/recipes"),
        log: home.join(".local/state/flat3d/run.log"),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default();
    let (_, result) = run(
        &argv
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        &env,
        &paths,
        std::process::id(),
        now,
        |launch| {
            let program = argv.first().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "no game command")
            })?;
            let mut command = std::process::Command::new(program);
            command
                .args(&argv[1..])
                .envs(launch.command.env.iter().cloned());
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                Err::<i32, _>(command.exec())
            }
            #[cfg(not(unix))]
            {
                command.status().map(|status| status.code().unwrap_or(1))
            }
        },
    );
    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("flat3d-run: {error}");
            std::process::exit(1);
        }
    }
}
