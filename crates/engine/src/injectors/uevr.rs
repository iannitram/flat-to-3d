use crate::injectors::{fetch, uevr_profile, RouteEnv};
use crate::paths::EnginePaths;
use crate::recipes::RouteSettings;
use crate::steam::Game;
use crate::undo::{Replaced, UndoRecord};
use std::fs;
use std::path::{Path, PathBuf};

pub const UEVR_VERSION: &str = "nightly-01143-4ee5c6b6162dee2291fc75f9dfc57667f6d45a2d";
pub const UEVR_DLLS: [&str; 2] = ["openxr_loader.dll", "UEVRBackend.dll"];
pub const INJECTOR_EXE: &str = "flat3d-uevr-inject.exe";
pub const DEFAULT_DELAY_S: u32 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UevrExtras {
    pub exe_stem: String,
    pub delay_s: u32,
    pub native_stereo_fix: Option<bool>,
    pub profile: Option<String>,
}

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

fn strip_exe(name: &str) -> &str {
    let cut = name.len().saturating_sub(4);
    match name.get(cut..) {
        Some(suffix) if suffix.eq_ignore_ascii_case(".exe") => &name[..cut],
        _ => name,
    }
}

fn profile_dir_for_stem(compatdata: &Path, appid: u32, stem: &str) -> PathBuf {
    compatdata
        .join(appid.to_string())
        .join("pfx/drive_c/users/steamuser/AppData/Roaming/UnrealVRMod")
        .join(stem)
}

pub fn profile_dir(compatdata: &Path, appid: u32, exe_name: &str) -> PathBuf {
    profile_dir_for_stem(compatdata, appid, strip_exe(exe_name))
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

/// Convert route settings to the UEVR profile fields.
pub fn settings_from(s: &RouteSettings) -> Result<UevrSettings, String> {
    let rendering = match s
        .get("uevr-rendering")
        .map(String::as_str)
        .unwrap_or("native")
    {
        "native" => Rendering::NativeStereo,
        "synced" => Rendering::SynchronizedSequential,
        _ => return Err("bad-recipe: uevr-rendering must be native or synced".into()),
    };
    let screen_2d = match s.get("uevr-2d-screen").map(String::as_str).unwrap_or("no") {
        "yes" => true,
        "no" => false,
        _ => return Err("bad-recipe: uevr-2d-screen must be yes or no".into()),
    };
    Ok(UevrSettings {
        rendering,
        screen_2d,
    })
}

/// Read launch-time values from the UEVR route settings.
pub fn extras_from(s: &RouteSettings) -> Result<UevrExtras, String> {
    let exe = s
        .get("uevr-exe")
        .map(String::as_str)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "bad-recipe: uevr-exe is missing".to_owned())?;
    let exe_stem = strip_exe(exe).to_owned();
    if exe_stem.is_empty()
        || exe_stem == "."
        || exe_stem == ".."
        || exe_stem.contains(['/', '\\', '\0'])
    {
        return Err("bad-recipe: uevr-exe must be a file name, not a path".to_owned());
    }
    let delay_s = match s.get("uevr-delay") {
        None => DEFAULT_DELAY_S,
        Some(value) => value
            .parse::<u32>()
            .ok()
            .filter(|n| *n <= 120)
            .ok_or_else(|| "bad-recipe: uevr-delay must be a u32 from 0 to 120".to_owned())?,
    };
    let native_stereo_fix = match s.get("uevr-native-stereo-fix").map(String::as_str) {
        None => None,
        Some("yes") => Some(true),
        Some("no") => Some(false),
        Some(_) => return Err("bad-recipe: uevr-native-stereo-fix must be yes or no".into()),
    };
    let profile = match s.get("uevr-profile") {
        None => None,
        Some(value)
            if !value.is_empty()
                && value
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') =>
        {
            Some(value.clone())
        }
        Some(_) => return Err("bad-recipe: uevr-profile must be a pin key ([a-z0-9-]+)".into()),
    };
    Ok(UevrExtras {
        exe_stem,
        delay_s,
        native_stereo_fix,
        profile,
    })
}

pub fn profile_text_full(settings: &UevrSettings, extras: &UevrExtras) -> String {
    let mut text = profile_text(settings);
    if let Some(value) = extras.native_stereo_fix {
        text.push_str(if value {
            "VR_NativeStereoFix=true\n"
        } else {
            "VR_NativeStereoFix=false\n"
        });
    }
    text.push_str("Frontend_RequestedRuntime=openxr_loader.dll\n");
    text
}

pub fn uevr_dir(p: &EnginePaths) -> PathBuf {
    p.uevr_cache().join(UEVR_VERSION)
}

pub fn injector_path(p: &EnginePaths) -> PathBuf {
    p.data.join("bin").join(INJECTOR_EXE)
}

pub fn win_path(unix: &Path) -> String {
    format!("Z:{}", unix.to_string_lossy().replace('/', "\\"))
}

fn shlex_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn debug_cmd(injector: &Path, stem: &str, delay: u32, dir: &Path) -> String {
    let mut args = vec![
        shlex_quote(&injector.to_string_lossy()),
        "--exe".into(),
        shlex_quote(stem),
        "--delay".into(),
        delay.to_string(),
    ];
    for dll in UEVR_DLLS {
        args.push("--dll".into());
        args.push(shlex_quote(&win_path(&dir.join(dll))));
    }
    args.join(" ")
}

fn route_error(code: &str, detail: impl std::fmt::Display) -> String {
    format!("{code}: {detail}")
}

/// Apply the UEVR route. When the recipe names a profile pin (`uevr-profile`), that pin must be
/// fetched; its files go into the profile folder and our keys are merged into its `config.txt`.
pub fn apply_with(
    game: &Game,
    settings: &RouteSettings,
    p: &EnginePaths,
) -> Result<(Applied, RouteEnv), String> {
    let extras = extras_from(settings)?;
    let fetched = match &extras.profile {
        None => None,
        Some(key) => {
            let pin = fetch::pin(key).map_err(|e| {
                let e = crate::api::ApiError::from(e);
                format!("{}: {}", e.code, e.message)
            })?;
            if !fetch::is_fetched(&pin, p) {
                return Err(route_error(
                    "not-found",
                    format!("the UEVR profile {key} is not fetched; run flat3d-inject setup fetch {key}"),
                ));
            }
            Some(fetch::cache_dir(&pin, p))
        }
    };
    apply_with_profile(game, settings, p, fetched.as_deref())
}

/// [`apply_with`] with the fetched profile folder given directly (`None`: no profile).
#[doc(hidden)]
pub fn apply_with_profile(
    game: &Game,
    settings: &RouteSettings,
    p: &EnginePaths,
    fetched_profile: Option<&Path>,
) -> Result<(Applied, RouteEnv), String> {
    let profile_settings = settings_from(settings)?;
    let extras = extras_from(settings)?;
    let profile_pin = fetched_profile
        .map(uevr_profile::read)
        .transpose()
        .map_err(|e| route_error("refused", e))?;
    let compatdata = game
        .install_dir
        .parent()
        .and_then(Path::parent)
        .map(|s| s.join("compatdata"))
        .ok_or_else(|| {
            route_error(
                "not-found",
                format!(
                    "game folder {} is not under steamapps/common",
                    game.install_dir.display()
                ),
            )
        })?;
    let dir = uevr_dir(p);
    for dll in UEVR_DLLS {
        if !dir.join(dll).is_file() {
            return Err(route_error(
                "not-found",
                "UEVR is not fetched; run flat3d-inject setup fetch uevr",
            ));
        }
    }
    let injector = injector_path(p);
    if !injector.is_file() {
        return Err(route_error(
            "not-found",
            "flat3d-uevr-inject.exe is not installed; run tools/build-win.sh --install",
        ));
    }
    let profile_dir = profile_dir_for_stem(&compatdata, game.appid, &extras.exe_stem);
    let profile = profile_dir.join(PROFILE_FILE);
    let backup = profile_dir.join(BACKUP_FILE);
    let marker = profile_dir.join(MARKER_FILE);
    if marker.exists() || backup.exists() {
        return Err(route_error(
            "refused",
            format!(
                "the UEVR profile for {} is already set up; run flat3d-inject undo {} first",
                game.appid, game.appid
            ),
        ));
    }
    let mut record = UndoRecord {
        appid: game.appid,
        launch_options_before: String::new(),
        created: Vec::new(),
        replaced: Vec::new(),
    };
    let ours = profile_text_full(&profile_settings, &extras);
    let full_text = match &profile_pin {
        Some(pin) => uevr_profile::merge_config(&pin.config, &ours),
        None => ours,
    };
    let files = profile_pin.as_ref().map_or(&[][..], |pin| &pin.files[..]);
    if let Err(error) = write_all_text(
        &profile_dir,
        &profile,
        &backup,
        &marker,
        &full_text,
        files,
        &mut record,
    ) {
        rollback(&record);
        return Err(route_error("io", error));
    }
    let env = RouteEnv {
        env: vec![(
            "PROTON_REMOTE_DEBUG_CMD".into(),
            debug_cmd(&injector, &extras.exe_stem, extras.delay_s, &dir),
        )],
        unset: vec!["XR_RUNTIME_JSON".into()],
        sidecar: None,
    };
    let applied = Applied {
        record,
        env: env.env.clone(),
    };
    Ok((applied, env))
}

/// Suffix of the backup made of a prefix file a profile file replaces.
pub const FILE_BACKUP_SUFFIX: &str = ".flat3d-backup";

#[allow(clippy::too_many_arguments)]
fn write_all_text(
    dir: &Path,
    profile: &Path,
    backup: &Path,
    marker: &Path,
    text: &str,
    files: &[uevr_profile::ProfileFile],
    record: &mut UndoRecord,
) -> Result<(), String> {
    let io = |what: &str, path: &Path, e: std::io::Error| format!("{what} {}: {e}", path.display());
    fs::create_dir_all(dir).map_err(|e| io("cannot create", dir, e))?;
    fs::write(marker, "").map_err(|e| io("cannot write", marker, e))?;
    record.created.push(marker.to_path_buf());
    for file in files {
        copy_over(&file.src, &dir.join(&file.rel), record)?;
    }
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
    fs::write(profile, text).map_err(|e| io("cannot write", profile, e))
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
    Ok(Applied {
        record,
        env: Vec::new(),
    })
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

/// Copy one profile file into place. A file already there is moved to its backup first and
/// recorded in `replaced`; a new one is recorded in `created`. The copy goes to a staging name
/// and is renamed into place only once it is whole.
fn copy_over(src: &Path, target: &Path, record: &mut UndoRecord) -> Result<(), String> {
    let io = |what: &str, path: &Path, e: std::io::Error| format!("{what} {}: {e}", path.display());
    let with_suffix = |suffix: &str| {
        let mut name = target.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    };
    let staging = with_suffix(".flat3d-new");
    let backup = with_suffix(FILE_BACKUP_SUFFIX);
    if backup.exists() {
        return Err(format!(
            "{} is left from an earlier setup; run flat3d-inject undo first",
            backup.display()
        ));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| io("cannot create", parent, e))?;
    }
    if let Err(e) = fs::copy(src, &staging) {
        let _ = fs::remove_file(&staging);
        return Err(io("cannot write", target, e));
    }
    if target.is_dir() {
        let _ = fs::remove_file(&staging);
        return Err(format!("{} is a folder in the prefix", target.display()));
    }
    let existed = target.exists();
    if existed {
        if let Err(e) = fs::rename(target, &backup) {
            let _ = fs::remove_file(&staging);
            return Err(io("cannot back up", target, e));
        }
    }
    if let Err(e) = fs::rename(&staging, target) {
        let _ = fs::remove_file(&staging);
        if existed {
            let _ = fs::rename(&backup, target);
        }
        return Err(io("cannot write", target, e));
    }
    if existed {
        record.replaced.push(Replaced {
            path: target.to_path_buf(),
            backup,
        });
    } else {
        record.created.push(target.to_path_buf());
    }
    Ok(())
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
        let root =
            std::env::temp_dir().join(format!("flat3d-uevr-{}-{}", name, std::process::id()));
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
            .map(|d| {
                d.map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    }

    #[test]
    fn a_fresh_prefix_gets_the_profile_and_no_env_yet() {
        let root = fake_compatdata("fresh");
        let a = apply(
            &root,
            7,
            "Game-Win64-Shipping.exe",
            &SETTINGS,
            "-x %command%",
        )
        .unwrap();
        let dir = profile_dir(&root, 7, "Game-Win64-Shipping.exe");
        assert_eq!(
            fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(),
            profile_text(&SETTINGS)
        );
        assert_eq!(a.record.appid, 7);
        assert_eq!(a.record.launch_options_before, "-x %command%");
        assert_eq!(
            a.record.created,
            vec![dir.join(MARKER_FILE), dir.join(PROFILE_FILE)]
        );
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
        assert_eq!(
            fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(),
            "mine\n"
        );
        assert_eq!(files_in(&dir), vec![PROFILE_FILE.to_owned()]);
    }

    #[test]
    fn a_second_apply_on_a_fresh_prefix_is_refused_and_undo_still_goes_flat() {
        let root = fake_compatdata("twice-fresh");
        let first = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        let dir = profile_dir(&root, 7, "G.exe");
        let before = files_in(&dir);
        let other = UevrSettings {
            rendering: Rendering::SynchronizedSequential,
            screen_2d: false,
        };
        assert!(apply(&root, 7, "G.exe", &other, "").is_err());
        assert_eq!(files_in(&dir), before);
        assert_eq!(
            fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(),
            profile_text(&SETTINGS)
        );
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
        assert_eq!(
            fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(),
            "mine\n"
        );
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
        assert_eq!(
            fs::read_to_string(dir.join(PROFILE_FILE)).unwrap(),
            "mine\n"
        );
        fs::remove_dir(dir.join(BACKUP_PARTIAL_FILE)).unwrap();
        let a = apply(&root, 7, "G.exe", &SETTINGS, "").unwrap();
        assert_eq!(fs::read_to_string(dir.join(BACKUP_FILE)).unwrap(), "mine\n");
        assert!(!dir.join(BACKUP_PARTIAL_FILE).exists());
        undo(&a.record);
        assert_eq!(files_in(&dir), vec![PROFILE_FILE.to_owned()]);
    }
}
