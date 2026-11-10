//! Recipe lookup across the configured recipe directories.

use std::fs;
use std::path::PathBuf;

/// Find the exact appid recipe first, then the first sorted dashed recipe.
pub fn find_recipe(dirs: &[PathBuf], appid: u32) -> Result<Option<(PathBuf, String)>, String> {
    let exact_name = format!("{appid}.recipe");
    let prefix = format!("{appid}-");
    for dir in dirs {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("{}: {error}", dir.display())),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(&prefix) && name.ends_with(".recipe") && entry.path().is_file() {
                names.push(name);
            }
        }
        names.sort();
        let exact = dir.join(&exact_name);
        let found = if exact.is_file() {
            Some(exact)
        } else {
            names.first().map(|name| dir.join(name))
        };
        if let Some(path) = found {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            return Ok(Some((path, text)));
        }
    }
    Ok(None)
}
