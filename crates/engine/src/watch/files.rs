use crate::api::LinkSnapshot;
use crate::runs::{parse_run_record, running_modes, RunRecord};
use crate::state::{self, Request};
use crate::switches::{parse_settings, Settings};
use std::fs;
use std::io::Write;
use std::path::Path;

/// The files the link service reads and writes, under the user's home and the state directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    /// The user's home directory.
    pub home: String,
    /// The shared state directory (`state::state_dir`).
    pub state_dir: String,
}

impl Paths {
    /// The state file, `link.json`.
    pub fn snapshot(&self) -> String {
        state::snapshot_path(&self.state_dir)
    }

    /// The one-line request file.
    pub fn request(&self) -> String {
        state::request_path(&self.state_dir)
    }

    /// The directory of run records.
    pub fn runs(&self) -> String {
        state::runs_dir(&self.state_dir)
    }

    /// The settings file (`setting set`, `switch set`).
    pub fn settings(&self) -> String {
        format!("{}/.config/flat3d/settings", self.home)
    }

    /// The service's own log, `link.log`.
    pub fn log(&self) -> String {
        format!("{}/.local/state/flat3d/link.log", self.home)
    }
}

/// Where the device list comes from: the newest `adb track-devices` frame, when one arrived since
/// the last call.
pub fn write_snapshot(path: &str, snapshot: &LinkSnapshot) -> Result<(), String> {
    let text = serde_json::to_string(snapshot).map_err(|error| error.to_string())?;
    let target = Path::new(path);
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    }
    let tmp = format!("{path}.tmp");
    fs::write(&tmp, text.as_bytes()).map_err(|error| format!("{tmp}: {error}"))?;
    fs::rename(&tmp, target).map_err(|error| format!("{path}: {error}"))
}

pub(super) fn append_log(path: &str, now: u64, line: &str) {
    if let Some(dir) = Path::new(path).parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{now} {line}");
    }
}

/// Read the request file and remove it. Anything but `start-3d` or `stop-3d` is dropped.
pub(super) fn take_request(path: &str) -> Option<Request> {
    let text = fs::read_to_string(path).ok()?;
    let _ = fs::remove_file(path);
    state::parse_request(&text)
}

/// The settings, or the defaults when the file is missing.
pub(super) fn read_settings(path: &str) -> Settings {
    fs::read_to_string(path)
        .map(|text| parse_settings(&text))
        .unwrap_or_default()
}

/// Whether a live game runs in 3D and whether one runs flat, from the run records.
pub(super) fn game_modes(runs_dir: &str) -> (bool, bool) {
    let records: Vec<RunRecord> = fs::read_dir(runs_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|text| parse_run_record(&text))
        .collect();
    let alive = |pid: u32| Path::new(&format!("/proc/{pid}")).exists();
    running_modes(&records, &alive)
}
