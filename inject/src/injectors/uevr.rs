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
