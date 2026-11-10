use flat3d_inject::catalog::find_recipe;
use std::path::PathBuf;

fn dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("flat3d-catalog-{name}-{}", std::process::id()))
}

#[test]
fn exact_then_sorted_dashed_then_first_directory_and_missing_dir_skip() {
    let root = dir("lookup");
    let _ = std::fs::remove_dir_all(&root);
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    std::fs::write(first.join("7-z.recipe"), "z").unwrap();
    std::fs::write(first.join("7-a.recipe"), "a").unwrap();
    std::fs::write(first.join("7.recipe"), "exact").unwrap();
    std::fs::write(second.join("7.recipe"), "second").unwrap();
    let missing = root.join("missing");
    let (path, text) = find_recipe(&[missing, first.clone(), second], 7)
        .unwrap()
        .unwrap();
    assert_eq!(path, first.join("7.recipe"));
    assert_eq!(text, "exact");
    std::fs::remove_file(path).unwrap();
    let (path, text) = find_recipe(std::slice::from_ref(&first), 7)
        .unwrap()
        .unwrap();
    assert_eq!(path, first.join("7-a.recipe"));
    assert_eq!(text, "a");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn no_match_returns_none() {
    let root = dir("empty");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    assert_eq!(find_recipe(std::slice::from_ref(&root), 99).unwrap(), None);
    let _ = std::fs::remove_dir_all(root);
}
