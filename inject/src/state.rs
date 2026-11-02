//! The files the link service and its clients share: docs/code-map.md, API shape, State, IPC and files.

/// How old a snapshot may be before it is stale.
pub const STALE_SECS: u64 = 10;

/// The directory the link service and its clients share: under the runtime dir, else `/tmp`.
pub fn state_dir(runtime_dir: Option<&str>, uid: u32) -> String {
    let dir = runtime_dir
        .map(|d| d.strip_suffix('/').unwrap_or(d))
        .filter(|d| !d.is_empty());
    match dir {
        Some(d) => format!("{}/flat3d", d),
        None => format!("/tmp/flat3d-{}", uid),
    }
}

/// The link service's state file, `link.json`.
pub fn snapshot_path(dir: &str) -> String {
    format!("{}/link.json", dir)
}

/// The one-line request file, `start-3d` / `stop-3d`.
pub fn request_path(dir: &str) -> String {
    format!("{}/request", dir)
}

/// The directory of per-run records.
pub fn runs_dir(dir: &str) -> String {
    format!("{}/runs", dir)
}

/// One run's record, `runs/run-<pid>.txt`.
pub fn run_record_path(dir: &str, pid: u32) -> String {
    format!("{}/runs/run-{}.txt", dir, pid)
}

/// What a client asks the link service to do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// Bring the link up.
    Start3d,
    /// Take the link down.
    Stop3d,
}

/// Read a request from the request file's text.
pub fn parse_request(text: &str) -> Option<Request> {
    match text.trim() {
        "start-3d" => Some(Request::Start3d),
        "stop-3d" => Some(Request::Stop3d),
        _ => None,
    }
}

/// The request file's text for `r`.
pub fn request_text(r: Request) -> &'static str {
    match r {
        Request::Start3d => "start-3d",
        Request::Stop3d => "stop-3d",
    }
}

/// Whether `updated` is more than [`STALE_SECS`] behind `now`; a future `updated` is not stale.
pub fn is_stale(updated: u64, now: u64) -> bool {
    now.saturating_sub(updated) > STALE_SECS
}
