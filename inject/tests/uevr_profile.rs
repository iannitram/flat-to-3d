//! Locked tests for E3a: UEVR's per-game profile text and where it lives inside the game's
//! Proton prefix. UEVR reads `config.txt` from `%APPDATA%\UnrealVRMod\<exe name>\` (its
//! `Framework::get_persistent_dir`); keys are `VR_<setting>` (praydog/UEVR, src/mods/VR.hpp).

use flat3d_inject::injectors::uevr::{profile_dir, profile_text, Rendering, UevrSettings};
use std::path::{Path, PathBuf};

const COMPAT: &str = "/home/deck/.local/share/Steam/steamapps/compatdata";
const TAIL: &str = "pfx/drive_c/users/steamuser/AppData/Roaming/UnrealVRMod";

#[test]
fn native_stereo_with_the_2d_screen_on() {
    let s = UevrSettings { rendering: Rendering::NativeStereo, screen_2d: true };
    assert_eq!(profile_text(&s), "VR_RenderingMethod=0\nVR_2DScreenMode=true\n");
}

#[test]
fn synchronized_sequential_adds_its_method_line() {
    let s = UevrSettings { rendering: Rendering::SynchronizedSequential, screen_2d: false };
    let want = "VR_RenderingMethod=1\nVR_SyncedSequentialMethod=1\nVR_2DScreenMode=false\n";
    assert_eq!(profile_text(&s), want);
}

#[test]
fn native_stereo_never_writes_the_synced_line() {
    let s = UevrSettings { rendering: Rendering::NativeStereo, screen_2d: false };
    assert!(!profile_text(&s).contains("Synced"));
}

#[test]
fn the_profile_dir_is_inside_the_games_prefix_under_the_exe_name() {
    let got = profile_dir(Path::new(COMPAT), 2075800, "ZeroCompany-Win64-Shipping.exe");
    let want = PathBuf::from(format!("{COMPAT}/2075800/{TAIL}/ZeroCompany-Win64-Shipping"));
    assert_eq!(got, want);
}

#[test]
fn only_a_trailing_exe_is_dropped_whatever_its_case() {
    let up = profile_dir(Path::new(COMPAT), 1, "Game.EXE");
    assert_eq!(up, PathBuf::from(format!("{COMPAT}/1/{TAIL}/Game")));
    let dotted = profile_dir(Path::new(COMPAT), 1, "Game.v2.exe");
    assert_eq!(dotted, PathBuf::from(format!("{COMPAT}/1/{TAIL}/Game.v2")));
    let bare = profile_dir(Path::new(COMPAT), 1, "Game");
    assert_eq!(bare, PathBuf::from(format!("{COMPAT}/1/{TAIL}/Game")));
}
