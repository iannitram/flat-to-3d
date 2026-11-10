use flat3d_inject::switches::{read_settings, write_settings, Settings};
use std::collections::BTreeSet;

#[test]
fn write_then_read_round_trips_and_missing_file_defaults() {
    let path = std::env::temp_dir().join(format!("flat3d-switch-{}.settings", std::process::id()));
    let _ = std::fs::remove_file(&path);
    assert_eq!(read_settings(&path).unwrap(), Settings::default());
    let settings = Settings {
        auto_3d: false,
        tunnel: true,
        off: BTreeSet::from([5, 9]),
    };
    write_settings(&path, &settings).unwrap();
    assert_eq!(read_settings(&path).unwrap(), settings);
    let _ = std::fs::remove_file(path);
}
