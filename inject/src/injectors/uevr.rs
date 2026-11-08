use crate::undo::{Replaced, UndoRecord};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rendering {
    NativeStereo,
    SynchronizedSequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UevrSettings {
    pub rendering: Rendering,
    pub screen_2d: bool,
}

pub fn profile_text(settings: &UevrSettings) -> String {
    let mut text = match settings.rendering {
        Rendering::NativeStereo => "VR_RenderingMethod=0\n".to_owned(),
        Rendering::SynchronizedSequential => {
            "VR_RenderingMethod=1\nVR_SyncedSequentialMethod=1\n".to_owned()
        }
    };
    text.push_str(if settings.screen_2d {
        "VR_2DScreenMode=true\n"
    } else {
        "VR_2DScreenMode=false\n"
    });
    text
}

pub fn profile_dir(compatdata: &Path, appid: u32, exe_name: &str) -> PathBuf {
    let stem = exe_name
        .get(..exe_name.len().saturating_sub(4))
        .filter(|_| {
            exe_name
                .get(exe_name.len().saturating_sub(4)..)
                .is_some_and(|suffix| suffix.eq_ignore_ascii_case(".exe"))
        })
        .unwrap_or(exe_name);
    compatdata
        .join(appid.to_string())
        .join("pfx/drive_c/users/steamuser/AppData/Roaming/UnrealVRMod")
        .join(stem)
}

/// The profile file UEVR reads inside the profile directory.
pub const PROFILE_FILE: &str = "config.txt";
/// Copy of the game's own profile, made before we replace it.
pub const BACKUP_FILE: &str = "config.txt.flat3d-backup";
/// Where the backup is copied first. It is renamed to `BACKUP_FILE` only once the copy is
/// whole, so a failed copy never leaves a partial backup that would refuse every later apply.
pub const BACKUP_PARTIAL_FILE: &str = "config.txt.flat3d-backup.partial";
/// Marks a profile directory that we have already set up. It is recorded in `created`, so
/// undo removes it, and while it exists a second apply is refused.
pub const MARKER_FILE: &str = ".flat3d-applied";

/// What applying the UEVR route did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    /// Every change, for undo.
    pub record: UndoRecord,
    /// The environment the wrapper starts the game with. Empty for now: the real UEVR
    /// environment comes with the device half of #51, which waits on #32.
    pub env: Vec<(String, String)>,
}

/// Write the UEVR profile into the game's Proton prefix and record every change.
/// An existing profile is copied to a backup first and recorded in `replaced`; a profile we
/// create is recorded in `created`. A second apply is refused, whichever of the two happened
/// the first time, until the first one is undone: otherwise the second record would back up
/// our own profile and undo would leave the game in 3D. On failure nothing is left changed.
/// Fetching UEVR and .NET into the prefix is not done here (the device half of #51).
pub fn apply(
    compatdata: &Path,
    appid: u32,
    exe_name: &str,
    settings: &UevrSettings,
    launch_options_before: &str,
) -> Result<Applied, String> {
    let dir = profile_dir(compatdata, appid, exe_name);
    let profile = dir.join(PROFILE_FILE);
    let backup = dir.join(BACKUP_FILE);
    let marker = dir.join(MARKER_FILE);
    if marker.exists() || backup.exists() {
        return Err(format!(
            "the UEVR profile for {} is already set up; undo it first",
            appid
        ));
    }
    let mut record = UndoRecord {
        appid,
        launch_options_before: launch_options_before.to_owned(),
        created: Vec::new(),
        replaced: Vec::new(),
    };
    if let Err(e) = write_all(&dir, &profile, &backup, &marker, settings, &mut record) {
        rollback(&record);
        return Err(e);
    }
    Ok(Applied { record, env: Vec::new() })
}

fn write_all(
    dir: &Path,
    profile: &Path,
    backup: &Path,
    marker: &Path,
    settings: &UevrSettings,
    record: &mut UndoRecord,
) -> Result<(), String> {
    let io = |what: &str, path: &Path, e: std::io::Error| format!("{what} {}: {e}", path.display());
    fs::create_dir_all(dir).map_err(|e| io("cannot create", dir, e))?;
    fs::write(marker, "").map_err(|e| io("cannot write", marker, e))?;
    record.created.push(marker.to_path_buf());
    if profile.exists() {
        let partial = dir.join(BACKUP_PARTIAL_FILE);
        if let Err(e) = fs::copy(profile, &partial).and_then(|_| fs::rename(&partial, backup)) {
            let _ = fs::remove_file(&partial);
            return Err(io("cannot back up to", backup, e));
        }
        record.replaced.push(Replaced {
            path: profile.to_path_buf(),
            backup: backup.to_path_buf(),
        });
    } else {
        record.created.push(profile.to_path_buf());
    }
    fs::write(profile, profile_text(settings)).map_err(|e| io("cannot write", profile, e))
}

/// Undo a half-finished apply: put back replaced files, remove created ones.
fn rollback(record: &UndoRecord) {
    for r in record.replaced.iter().rev() {
        let _ = fs::rename(&r.backup, &r.path);
    }
    for p in record.created.iter().rev() {
        let _ = fs::remove_file(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::undo::{undo_actions, UndoAction};

    const SETTINGS: UevrSettings = UevrSettings {
        rendering: Rendering::NativeStereo,
        screen_2d: true,
    };

    fn fake_compatdata(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("flat3d-uevr-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    /// Run the file steps of an undo record (the real `undo` is #84).
    fn undo(record: &UndoRecord) {
        for action in undo_actions(record) {
            match action {
                UndoAction::RemoveFile(p) => fs::remove_file(p).unwrap(),
                UndoAction::RestoreFile { backup, path } => fs::rename(backup, path).unwrap(),
                UndoAction::SetLaunchOptions(_) => {}
            }
        }
    }

    fn files_in(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .map(|d| d.map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        v.sort();
        v
    }

    #[test]
    fn a_fresh_prefix_gets_the_profile_and_no_env_yet() {
        let root = fake_compatdata("fresh");
        let a = apply(&root, 7, "Game-Win64-Shipping.exe", &SETTINGS, "-x %command%").unwrap();
        let dir = profile_dir(&root, 7, "Game-Win64-Shipping.exe");
        assert_eq!(fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(), profile_text(&SETTINGS));
        assert_eq!(a.record.appid, 7);
        assert_eq!(a.record.launch_options_before, "-x %command%");
        assert_eq!(a.record.created, vec![dir.join(MARKER_FILE), dir.join(PROFILE_FILE)]);
        assert!(a.record.replaced.is_empty());
        assert!(a.env.is_empty());
    }

    #[test]
    fn undo_on_a_fresh_prefix_leaves_no_files() {
        let root = fake_compatdata("fresh-undo");
        let a = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        undo(&a.record);
        assert!(files_in(&profile_dir(&root, 7, "G.exe")).is_empty());
    }

    #[test]
    fn an_old_profile_is_backed_up_and_undo_restores_it() {
        let root = fake_compatdata("old");
        let dir = profile_dir(&root, 7, "G.exe");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(PROFILE_FILE), "mine\n").unwrap();
        let a = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        assert_eq!(fs::read_to_string(dir.join(BACKUP_FILE)).unwrap(), "mine\n");
        assert_eq!(a.record.replaced.len(), 1);
        undo(&a.record);
        assert_eq!(fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(), "mine\n");
        assert_eq!(files_in(&dir), vec![PROFILE_FILE.to_owned()]);
    }

    #[test]
    fn a_second_apply_on_a_fresh_prefix_is_refused_and_undo_still_goes_flat() {
        let root = fake_compatdata("twice-fresh");
        let first = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        let dir = profile_dir(&root, 7, "G.exe");
        let before = files_in(&dir);
        let other = UevrSettings { rendering: Rendering::SynchronizedSequential, screen_2d: false };
        assert!(apply(&root, 7, "G.exe", &other, "").is_err());
        assert_eq!(files_in(&dir), before);
        assert_eq!(fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(), profile_text(&SETTINGS));
        undo(&first.record);
        assert!(files_in(&dir).is_empty());
    }

    #[test]
    fn a_second_apply_over_an_old_profile_is_refused_and_changes_nothing() {
        let root = fake_compatdata("twice-old");
        let dir = profile_dir(&root, 7, "G.exe");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(PROFILE_FILE), "mine\n").unwrap();
        let first = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        assert!(apply(&root, 7, "G.exe", &SETTINGS, "").is_err());
        assert_eq!(fs::read_to_string(dir.join(BACKUP_FILE)).unwrap(), "mine\n");
        undo(&first.record);
        assert_eq!(fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(), "mine\n");
    }

    #[test]
    fn apply_can_run_again_after_undo() {
        let root = fake_compatdata("again");
        let a = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        undo(&a.record);
        assert!(apply(&root, 7, "G.exe", &SETTINGS, "").is_ok());
    }

    #[test]
    fn a_failed_apply_changes_nothing() {
        let root = fake_compatdata("fail");
        let dir = profile_dir(&root, 7, "G.exe");
        // A directory where the profile file belongs makes the write fail after the marker.
        fs::create_dir_all(dir.join(PROFILE_FILE)).unwrap();
        assert!(apply(&root, 7, "G.exe", &SETTINGS, "").is_err());
        assert_eq!(files_in(&dir), vec![PROFILE_FILE.to_owned()]);
    }

    #[test]
    fn a_failed_backup_leaves_no_backup_and_a_later_apply_works() {
        let root = fake_compatdata("fail-backup");
        let dir = profile_dir(&root, 7, "G.exe");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(PROFILE_FILE), "mine\n").unwrap();
        // A directory where the partial copy goes makes the backup copy fail.
        fs::create_dir_all(dir.join(BACKUP_PARTIAL_FILE)).unwrap();
        assert!(apply(&root, 7, "G.exe", &SETTINGS, "").is_err());
        assert!(!dir.join(BACKUP_FILE).exists());
        assert!(!dir.join(MARKER_FILE).exists());
        assert_eq!(fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(), "mine\n");
        fs::remove_dir(dir.join(BACKUP_PARTIAL_FILE)).unwrap();
        let a = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        assert_eq!(fs::read_to_string(dir.join(BACKUP_FILE)).unwrap(), "mine\n");
        assert!(!dir.join(BACKUP_PARTIAL_FILE).exists());
        undo(&a.record);
        assert_eq!(files_in(&dir), vec![PROFILE_FILE.to_owned()]);
    }
}
