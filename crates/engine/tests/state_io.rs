use flat3d_inject::paths::EnginePaths;
use flat3d_inject::state::{parse_request, read_snapshot, write_request, Request};
use std::collections::BTreeMap;

#[test]
fn missing_snapshot_and_request_round_trip() {
    let home = std::env::temp_dir().join(format!("flat3d-state-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    let runtime = home.join("runtime");
    let paths = EnginePaths::from_env(
        &BTreeMap::from([
            ("HOME".into(), home.to_string_lossy().into_owned()),
            ("XDG_RUNTIME_DIR".into(), runtime.to_string_lossy().into_owned()),
        ]),
        12,
    )
    .unwrap();
    assert_eq!(read_snapshot(&paths).unwrap(), None);
    write_request(&paths, Request::Stop3d).unwrap();
    let text = std::fs::read_to_string(flat3d_inject::state::request_path(
        &paths.state_dir.to_string_lossy(),
    ))
    .unwrap();
    assert_eq!(parse_request(&text), Some(Request::Stop3d));
    let _ = std::fs::remove_dir_all(home);
}
