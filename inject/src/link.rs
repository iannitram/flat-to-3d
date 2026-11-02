//! Start 3D and Stop 3D over WiVRn's wired mode (`adb reverse tcp:9757`).

use crate::NotBuilt;

/// WiVRn's port, reversed over USB.
pub const WIVRN_PORT: u16 = 9757;

/// WiVRn's Flatpak package.
pub const WIVRN_FLATPAK: &str = "io.github.wivrn.wivrn";

/// The WiVRn app on the headset.
pub const WIVRN_APP: &str = "org.meumeu.wivrn.github";

/// The user unit that runs the WiVRn server.
pub const WIVRN_UNIT: &str = "flat3d-wivrn";

/// Where the headset link stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headset {
    /// adb does not list the headset as ready.
    Missing,
    /// adb lists the headset, but it has not authorised this machine.
    Unauthorised,
    /// adb lists the headset and it is ready.
    Present,
}

/// Read `adb devices` output into a [`Headset`].
pub fn headset_from_adb(stdout: &str) -> Headset {
    let mut seen_unauthorised = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
            continue;
        }
        match line.split_whitespace().nth(1) {
            Some("device") => return Headset::Present,
            Some("unauthorized") => seen_unauthorised = true,
            _ => {}
        }
    }
    if seen_unauthorised {
        Headset::Unauthorised
    } else {
        Headset::Missing
    }
}

/// Where the headset link stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkState {
    /// adb lists the Quest.
    pub headset_seen: bool,
    /// Quest Tether reports its link.
    pub tether_up: bool,
    /// adb reverse is set and the WiVRn server is running.
    pub wivrn_ready: bool,
}

/// The commands Start 3D runs, in order (pure: tests assert on this list).
pub fn start_3d_commands() -> Vec<Vec<String>> {
    let port = format!("tcp:{WIVRN_PORT}");
    let unit = format!("--unit={WIVRN_UNIT}");
    vec![
        vec!["adb".into(), "devices".into()],
        vec![
            "systemd-run".into(),
            "--user".into(),
            "--collect".into(),
            unit,
            "flatpak".into(),
            "run".into(),
            "--command=wivrn-server".into(),
            WIVRN_FLATPAK.into(),
        ],
        vec!["adb".into(), "reverse".into(), port.clone(), port],
        vec![
            "adb".into(),
            "shell".into(),
            "am".into(),
            "start".into(),
            "-a".into(),
            "android.intent.action.VIEW".into(),
            "-d".into(),
            "wivrn+tcp://localhost".into(),
            WIVRN_APP.into(),
        ],
    ]
}

/// The commands Stop 3D runs, in order (pure: tests assert on this list).
pub fn stop_3d_commands() -> Vec<Vec<String>> {
    let port = format!("tcp:{WIVRN_PORT}");
    let unit = format!("{WIVRN_UNIT}.service");
    vec![
        vec![
            "adb".into(),
            "shell".into(),
            "am".into(),
            "force-stop".into(),
            WIVRN_APP.into(),
        ],
        vec!["adb".into(), "reverse".into(), "--remove".into(), port],
        vec!["systemctl".into(), "--user".into(), "stop".into(), unit],
    ]
}

/// Run [`start_3d_commands`] and report where the link stands.
pub fn start_3d() -> Result<LinkState, NotBuilt> {
    Err(NotBuilt("link::start_3d: see the Start 3D issue"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_exists() {
        assert!(start_3d().is_err());
        assert_eq!(WIVRN_PORT, 9757);
    }
}
