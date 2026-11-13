//! Steam's launch wrapper; preserve the original OS arguments and inherited env.

use flat3d_inject::paths::EnginePaths;
use flat3d_inject::wrapper::{
    flat_after_sidecar_error, note_run, run_with, sidecar_argv, ExecSpec, Launch,
};
use flat3d_inject::{exec::SystemRunner, sys};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::process::{Command, Stdio};

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

/// viewer.log in the log folder, opened for append; `None` when it cannot be opened.
fn viewer_log(paths: &EnginePaths) -> Option<File> {
    std::fs::create_dir_all(&paths.log_dir).ok()?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.log_dir.join("viewer.log"))
        .ok()
}

/// Start the route's sidecar, tied to this process's pid (the game keeps it through exec).
fn spawn_sidecar(paths: &EnginePaths, sidecar: &[String]) -> Result<(u32, Vec<String>), String> {
    let exe = std::env::current_exe().ok();
    let argv = sidecar_argv(
        sidecar,
        exe.as_deref().and_then(|path| path.parent()),
        &paths.bin,
        std::process::id(),
        &|path| path.exists(),
    )?;
    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]).stdin(Stdio::null());
    match viewer_log(paths) {
        Some(log) => {
            let err = log.try_clone().map_or_else(|_| Stdio::null(), Stdio::from);
            command.stdout(Stdio::from(log)).stderr(err);
        }
        None => {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }
    }
    let child = command.spawn().map_err(|e| e.to_string())?;
    Ok((child.id(), argv))
}

fn launch(paths: &EnginePaths, argv: &[OsString], now: u64, launch: &Launch) -> io::Result<i32> {
    let mut spec = launch.command.clone();
    if let Some(sidecar) = &launch.command.sidecar {
        match spawn_sidecar(paths, sidecar) {
            Ok((pid, sidecar_argv)) => note_run(
                paths,
                launch,
                now,
                "sidecar",
                &format!("pid {pid} {}", sidecar_argv.join(" ")),
            ),
            Err(error) => spec = flat_after_sidecar_error(paths, launch, now, &error),
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
