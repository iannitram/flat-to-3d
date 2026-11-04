//! The files the link service and its clients share: docs/code-map.md, API shape, State, IPC and files.

use crate::api::{EventKind, LinkEvent};

/// The maximum number of link events kept in memory.
pub const RING_MAX: usize = 10;

/// A bounded list of link events with monotonically increasing sequence numbers.
#[derive(Debug, Clone, Default)]
pub struct EventRing {
    events: Vec<LinkEvent>,
    next: u64,
}

impl EventRing {
    /// Create an empty event ring, numbered from one.
    pub fn new() -> EventRing {
        EventRing {
            events: Vec::new(),
            next: 1,
        }
    }

    /// Restore a ring from saved events, keeping the newest events and resuming numbering.
    pub fn from_events(mut events: Vec<LinkEvent>) -> EventRing {
        let max = events.iter().map(|event| event.seq).max().unwrap_or(0);
        if events.len() > RING_MAX {
            events.drain(..events.len() - RING_MAX);
        }
        let mut ring = EventRing {
            events,
            next: max.saturating_add(1),
        };
        if max == u64::MAX {
            ring.renumber();
        }
        ring
    }

    /// Number the retained events 1, 2, … again, so numbering can go on past the top of the range.
    fn renumber(&mut self) {
        for (i, event) in self.events.iter_mut().enumerate() {
            event.seq = i as u64 + 1;
        }
        self.next = self.events.len() as u64 + 1;
    }

    /// Add an event and return its sequence number. At the top of the range the retained events are
    /// renumbered from one first (see [`EventRing::renumber`]).
    pub fn push(&mut self, kind: EventKind, text: &str) -> u64 {
        if self.next == u64::MAX {
            self.renumber();
        }
        let seq = self.next;
        self.events.push(LinkEvent {
            seq,
            kind,
            text: text.to_owned(),
        });
        if self.events.len() > RING_MAX {
            self.events.remove(0);
        }
        self.next = self.next.saturating_add(1);
        seq
    }

    /// The retained events in oldest-first order.
    pub fn events(&self) -> &[LinkEvent] {
        &self.events
    }

    /// Clone the retained events newer than `seq`.
    pub fn since(&self, seq: u64) -> Vec<LinkEvent> {
        self.events
            .iter()
            .filter(|event| event.seq > seq)
            .cloned()
            .collect()
    }
}

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
