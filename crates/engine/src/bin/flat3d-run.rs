//! Steam's launch wrapper; preserve the original OS arguments and inherited env.

use flat3d_inject::paths::EnginePaths;
use flat3d_inject::wrapper::{
    flat_after_sidecar_error, note_run, run_with, spawn_sidecar_with, ExecSpec, Launch,
};
use flat3d_inject::{exec::SystemRunner, sys};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io;
use std::process::Command;

/// Exec the original command with `spec`'s env additions and removals.
/// On Unix this only returns on failure; elsewhere it waits and returns the exit code.
fn exec(argv: &[OsString], spec: &ExecSpec) -> io::Result<i32> {
    let Some((program, rest)) = argv.split_first() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no game command",
        ));
    };
    let mut command = Command::new(program);
    command.args(rest).envs(spec.env.iter().cloned());
    for key in &spec.unset {
        command.env_remove(key);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec())
    }
    #[cfg(not(unix))]
    {
        command.status().map(|status| status.code().unwrap_or(1))
    }
}

fn launch(paths: &EnginePaths, argv: &[OsString], now: u64, launch: &Launch) -> io::Result<i32> {
    let mut spec = launch.command.clone();
    if let Some(sidecar) = &launch.command.sidecar {
        let exe = std::env::current_exe().ok();
        let exe_dir = exe.as_deref().and_then(|path| path.parent());
        match spawn_sidecar_with(paths, sidecar, exe_dir, std::process::id()) {
            Ok((pid, sidecar_argv)) => note_run(
                paths,
                launch,
                now,
                "sidecar",
                &format!("pid {pid} {}", sidecar_argv.join(" ")),
            ),
            Err(error) => spec = flat_after_sidecar_error(paths, launch, now, &error.to_string()),
        }
    }
    exec(argv, &spec)
}

fn main() {
    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    let env: BTreeMap<_, _> = std::env::vars_os()
        .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)))
        .collect();
    let result = match EnginePaths::from_env(&env, sys::current_uid()) {
        Ok(paths) => {
            let now = sys::unix_now();
            let mut runner = SystemRunner;
            let lossy: Vec<String> = argv
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            let (_, result) = run_with(
                &lossy,
                &env,
                &paths,
                std::process::id(),
                now,
                &mut runner,
                |planned| launch(&paths, &argv, now, planned),
            );
            result
        }
        Err(error) => {
            // No paths: play flat with the original command and write no record.
            eprintln!("flat3d-run: paths: {}: {}", error.code, error.message);
            let flat = ExecSpec {
                argv: Vec::new(),
                env: Vec::new(),
                unset: Vec::new(),
                sidecar: None,
            };
            exec(&argv, &flat)
        }
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("flat3d-run: {error}");
            std::process::exit(1);
        }
    }
}
