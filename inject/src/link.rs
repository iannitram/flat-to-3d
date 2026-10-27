//! Start 3D (SPEC M10): Quest Tether's USB link plus WiVRn's wired mode (`adb reverse tcp:9757`).

use crate::NotBuilt;

/// WiVRn's port, reversed over USB.
pub const WIVRN_PORT: u16 = 9757;

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
pub fn start_3d_commands() -> Result<Vec<Vec<String>>, NotBuilt> {
    Err(NotBuilt("link::start_3d_commands: see the Start 3D issue"))
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
        assert!(start_3d_commands().is_err());
        assert!(start_3d().is_err());
        assert_eq!(WIVRN_PORT, 9757);
    }
}
