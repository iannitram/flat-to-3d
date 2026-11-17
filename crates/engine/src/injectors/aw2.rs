//! Alan Wake 2's reversible renderer settings and `version.dll` setup (M32 / W6).

use crate::undo::{Replaced, UndoRecord};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const RENDERER_REL: &str = "drive_c/users/steamuser/AppData/Local/Remedy/AlanWake2/renderer.ini";
const MARKER: &str = ".flat3d-aw2";
const BACKUP_SUFFIX: &str = ".flat3d-backup";

/// Apply W6 to a game folder and Heroic prefix. `proxy_source` is the built `version.dll`.
/// A marker makes a repeated apply a no-op; the first apply's record remains the recovery point.
pub fn apply(
    appid: u32,
    game_dir: &Path,
    prefix: &Path,
    proxy_source: &Path,
) -> Result<UndoRecord, String> {
    let marker = game_dir.join(MARKER);
    if marker.exists() {
        return Ok(empty_record(appid));
    }
    if !proxy_source.is_file() {
        return Err(format!(
            "not-found: AW2 proxy DLL is missing: {}",
            proxy_source.display()
        ));
    }

    let renderer = prefix.join(RENDERER_REL);
    let proxy = game_dir.join("version.dll");
    for path in [&renderer, &proxy] {
        if backup_path(path).exists() {
            return Err(format!(
                "refused: AW2 backup already exists: {}",
                backup_path(path).display()
            ));
        }
    }
    let original = fs::read(&renderer).map_err(|e| io_error("read", &renderer, e))?;
    let mut settings: Value = serde_json::from_slice(&original).map_err(|e| {
        format!(
            "bad-recipe: {}: invalid renderer JSON: {e}",
            renderer.display()
        )
    })?;
    let Some(object) = settings.as_object_mut() else {
        return Err(format!(
            "bad-recipe: {}: renderer settings are not an object",
            renderer.display()
        ));
    };
    object.insert("m_eVSync".into(), Value::from(0));
    object.insert("m_eMotionBlur".into(), Value::from(0));
    object.insert("m_bFilmGrain".into(), Value::from(false));
    object.insert("m_bLensDistortion".into(), Value::from(false));
    object.insert("m_eFSRFGMode".into(), Value::from(0));
    object.insert("m_eDLSSFGMode".into(), Value::from(0));
    let updated = serde_json::to_vec_pretty(&settings)
        .map_err(|e| format!("bad-recipe: cannot encode renderer settings: {e}"))?;

    let mut record = empty_record(appid);
    let result = (|| {
        backup_and_replace(&renderer, &original, &updated, &mut record)?;
        if proxy.exists() {
            let backup = backup_path(&proxy);
            fs::copy(&proxy, &backup).map_err(|e| io_error("back up", &proxy, e))?;
            record.replaced.push(Replaced {
                path: proxy.clone(),
                backup,
            });
        } else {
            record.created.push(proxy.clone());
        }
        fs::copy(proxy_source, &proxy).map_err(|e| io_error("install", &proxy, e))?;
        fs::write(&marker, b"flat3d-aw2\n").map_err(|e| io_error("write", &marker, e))?;
        record.created.push(marker.clone());
        Ok::<(), String>(())
    })();
    if let Err(error) = result {
        rollback(&record);
        return Err(error);
    }
    Ok(record)
}

fn empty_record(appid: u32) -> UndoRecord {
    UndoRecord {
        appid,
        launch_options_before: String::new(),
        created: Vec::new(),
        replaced: Vec::new(),
        created_dirs: Vec::new(),
    }
}

fn backup_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}{}", path.to_string_lossy(), BACKUP_SUFFIX))
}

fn backup_and_replace(
    path: &Path,
    original: &[u8],
    updated: &[u8],
    record: &mut UndoRecord,
) -> Result<(), String> {
    let backup = backup_path(path);
    fs::write(&backup, original).map_err(|e| io_error("back up", path, e))?;
    record.replaced.push(Replaced {
        path: path.to_path_buf(),
        backup,
    });
    crate::fsx::write_atomic(path, updated).map_err(|e| io_error("write", path, e))
}

fn rollback(record: &UndoRecord) {
    for path in record.created.iter().rev() {
        let _ = fs::remove_file(path);
    }
    for item in record.replaced.iter().rev() {
        let _ = fs::copy(&item.backup, &item.path);
        let _ = fs::remove_file(&item.backup);
    }
}

fn io_error(action: &str, path: &Path, error: std::io::Error) -> String {
    format!("io: cannot {action} {}: {error}", path.display())
}
