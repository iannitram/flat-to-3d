#[cfg(not(windows))]
fn main() {
    eprintln!("flat3d-uevr-inject: windows only");
    std::process::exit(3);
}

#[cfg(windows)]
fn main() {
    eprintln!("flat3d-uevr-inject: not built (WP-15)");
    std::process::exit(5);
}
