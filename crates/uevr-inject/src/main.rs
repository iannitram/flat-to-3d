use flat3d_uevr_inject::args;

fn main() -> std::process::ExitCode {
    let parsed = match args::parse(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", args::error_line("usage", &error, args::USAGE));
            return std::process::ExitCode::from(args::EXIT_USAGE);
        }
    };

    #[cfg(windows)]
    {
        flat3d_uevr_inject::win::run(&parsed)
    }

    #[cfg(not(windows))]
    {
        let _ = parsed;
        eprintln!(
            "{}",
            args::error_line(
                "not-windows",
                "Windows only",
                "let Proton start it through PROTON_REMOTE_DEBUG_CMD (WP-14 debug_cmd)",
            )
        );
        std::process::ExitCode::from(args::EXIT_NOT_WINDOWS)
    }
}
