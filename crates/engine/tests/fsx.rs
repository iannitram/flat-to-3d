use flat3d_inject::fsx;
use std::path::PathBuf;

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("flat3d-{name}-{}", std::process::id()))
}

#[test]
fn atomic_write_leaves_no_temporary_file() {
    let dir = temp("fsx-atomic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("file.txt");
    fsx::write_atomic(&path, b"hello").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"hello");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn append_creates_parent_dirs_and_read_opt_returns_none_for_missing_file() {
    let dir = temp("fsx-append");
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("nested/log.txt");
    assert_eq!(fsx::read_opt(&path).unwrap(), None);
    fsx::append_line(&path, "line one").unwrap();
    assert_eq!(fsx::read_opt(&path).unwrap().as_deref(), Some("line one\n"));
    let _ = std::fs::remove_dir_all(dir);
}
