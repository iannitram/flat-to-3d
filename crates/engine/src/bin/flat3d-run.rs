//! Steam's launch wrapper; preserve the original OS arguments and inherited env.

use flat3d_inject::paths::EnginePaths;
use flat3d_inject::{exec::SystemRunner, wrapper::run_with};
use std::collections::BTreeMap;

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

/// Run the original command untouched; never returns.
fn play_flat(argv: &[std::ffi::OsString]) -> ! {
    let Some((program, rest)) = argv.split_first() else {
        eprintln!("flat3d-run: no game command");
        std::process::exit(1)
    };
    let mut command = std::process::Command::new(program);
    command.args(rest);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        eprintln!("flat3d-run: {error}");
        std::process::exit(1)
    }
    #[cfg(not(unix))]
    {
        match command.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => {
                eprintln!("flat3d-run: {error}");
                std::process::exit(1)
            }
        }
    }
}

fn main() {
    let argv: Vec<_> = std::env::args_os().skip(1).collect();
    let env: BTreeMap<_, _> = std::env::vars_os()
        .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)))
        .collect();
    let Ok(paths) = EnginePaths::from_env(&env, uid()) else {
        // Bad input plays flat: run the original command unchanged.
        play_flat(&argv)
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default();
    let mut runner = SystemRunner;
    let (_, result) = run_with(
        &argv
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        &env,
        &paths,
        std::process::id(),
        now,
        &mut runner,
        |launch| {
            let program = argv.first().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "no game command")
            })?;
            let mut command = std::process::Command::new(program);
            command
                .args(&argv[1..])
                .envs(launch.command.env.iter().cloned());
            for key in &launch.command.unset {
                command.env_remove(key);
            }
            if let Some(sidecar) = &launch.command.sidecar {
                if let Some((program, args)) = sidecar.split_first() {
                    let mut sidecar_cmd = std::process::Command::new(program);
                    sidecar_cmd
                        .args(args)
                        .arg("--until-pid")
                        .arg(std::process::id().to_string());
                    sidecar_cmd.spawn()?;
                }
            }
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
