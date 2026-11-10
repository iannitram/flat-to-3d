use std::fs;
use std::path::{Path, PathBuf};

const ALLOWLIST: &[(&str, &str)] = &[
    ("bin/flat3d-run.rs", "WP-31"),
    ("cli/link.rs", "WP-08"),
    ("watch/run.rs", "follow-up"),
    ("watch/service.rs", "follow-up"),
];

fn rust_files(dir: &Path, root: &Path, files: &mut Vec<(String, String)>) {
    for item in fs::read_dir(dir).unwrap() {
        let item = item.unwrap();
        let path = item.path();
        if path.is_dir() {
            rust_files(&path, root, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            files.push((name, fs::read_to_string(path).unwrap()));
        }
    }
}

fn tagged(value: &str) -> bool {
    let Some((_, tag)) = value.rsplit_once("(WP-") else {
        return false;
    };
    tag.strip_suffix(')').is_some_and(valid_wp_tag)
}

fn valid_wp_tag(tag: &str) -> bool {
    let number = tag
        .strip_suffix('a')
        .or_else(|| tag.strip_suffix('b'))
        .unwrap_or(tag);
    let Ok(wp) = number.parse::<u8>() else {
        return false;
    };
    if wp == 25 {
        matches!(tag, "25a" | "25b")
    } else {
        tag == number && (wp <= 24 || (26..=34).contains(&wp))
    }
}

#[test]
fn only_exec_spawns_outside_tagged_architecture_allowlist() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &root, &mut files);
    for (name, contents) in &files {
        if name == "exec.rs" {
            continue;
        }
        for (index, line) in contents.lines().enumerate() {
            if line.contains("Command::new") {
                let entry = ALLOWLIST.iter().find(|(file, _)| *file == name.as_str());
                let (file, tag) = entry.unwrap_or_else(|| {
                    panic!(
                        "{name}:{}: move the spawn into exec.rs behind a runner trait",
                        index + 1
                    )
                });
                if *tag == "follow-up" {
                    assert!(
                        contents.contains(&format!("architecture-allowlist: {tag}")),
                        "{file}:{}: missing architecture allowlist tag {tag}",
                        index + 1
                    );
                }
            }
        }
    }
}

#[test]
fn every_stub_tag_names_a_real_roadmap_wp() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &root, &mut files);
    for (name, contents) in files {
        for (index, line) in contents.lines().enumerate() {
            for needle in ["NotBuilt(\"", "not-built: "] {
                let mut rest = line;
                while let Some(position) = rest.find(needle) {
                    rest = &rest[position + needle.len()..];
                    let end = rest.find('"').unwrap_or(rest.len());
                    let literal = &rest[..end];
                    assert!(
                        tagged(literal),
                        "{name}:{}: stub tag is missing or invalid: {literal}",
                        index + 1
                    );
                    let tag = literal
                        .rsplit_once("(WP-")
                        .unwrap()
                        .1
                        .strip_suffix(')')
                        .unwrap();
                    assert!(
                        valid_wp_tag(tag),
                        "{name}:{}: WP-{tag} is not in the roadmap index",
                        index + 1
                    );
                }
            }
        }
    }
}
