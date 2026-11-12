use super::files::{game_modes, take_request, Paths};
use super::service::Service;
use super::service::TrackDevices;
use crate::api::LinkSnapshot;
use crate::exec::ProcessRunner;
use crate::link::adb;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default()
}

/// The link service's loop: every second, read the settings, the request file, the run records
/// and the device list, and carry out what they decide. Runs until it is stopped.
pub fn run(paths: &Paths) -> ! {
    let bundled_path = adb::bundled_adb(&paths.home);
    let bundled = if Path::new(&bundled_path).is_file() {
        Some(bundled_path)
    } else {
        None
    };
    let choice = adb::choose_adb(adb::running_adb_exe().as_deref(), bundled.as_deref());
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
        let settings =
            crate::switches::read_settings(Path::new(&paths.settings())).unwrap_or_default();
        let request = take_request(&paths.request());
        let modes = game_modes(&paths.runs());
        service.tick(now, &settings, request, modes);
        thread::sleep(Duration::from_secs(1));
    }
}
