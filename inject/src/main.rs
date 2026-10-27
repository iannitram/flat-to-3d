//! `flat3d-inject list | inject <appid> | start-3d`

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(flat3d_inject::cli::run(&args));
}
