use super::files::{game_modes, read_settings, take_request, Paths};
use super::service::Service;
use super::service::TrackDevices;
use crate::api::LinkSnapshot;
use crate::tether::adb::{self as tether_adb, ProcessRunner};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The adb server's binary, when one is running (the one the tether and link should use).
const ADB_SERVER_PORT: u16 = 5037;

fn running_adb_exe() -> Option<String> {
    let output = Command::new("ss").arg("-tlnp").output().ok()?;
    let pid = tether_adb::listener_pid(&String::from_utf8_lossy(&output.stdout), ADB_SERVER_PORT)?;
    fs::read_link(tether_adb::proc_exe_path(pid))
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default()
}

/// The link service's loop: every second, read the settings, the request file, the run records
/// and the device list, and carry out what they decide. Runs until it is stopped.
pub fn run(paths: &Paths) -> ! {
    let bundled_path = tether_adb::bundled_adb(&paths.home);
    let bundled = if Path::new(&bundled_path).is_file() {
        Some(bundled_path)
    } else {
        None
    };
    let choice = tether_adb::choose_adb(running_adb_exe().as_deref(), bundled.as_deref());
    let feed = TrackDevices::new(&choice.path);
    let previous = fs::read_to_string(paths.snapshot())
        .ok()
        .and_then(|text| serde_json::from_str::<LinkSnapshot>(&text).ok());
    let mut service = Service::new(
        paths.clone(),
        Rc::new(RefCell::new(ProcessRunner::default())),
        feed,
        choice,
    );
    if let Some(previous) = previous {
        service.restore(previous.events);
    }
    loop {
        let now = unix_now();
        let settings = read_settings(&paths.settings());
        let request = take_request(&paths.request());
        let modes = game_modes(&paths.runs());
        service.tick(now, &settings, request, modes);
        thread::sleep(Duration::from_secs(1));
    }
}
