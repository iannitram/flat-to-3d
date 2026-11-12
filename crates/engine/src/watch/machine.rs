use crate::api::EventKind;

pub const GRACE_SECS: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Absent,
    Plugged,
    Up,
    Grace { since: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watch {
    pub phase: Phase,
}

impl Watch {
    pub fn new() -> Watch {
        Watch {
            phase: Phase::Absent,
        }
    }
}

impl Default for Watch {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Plugged,
    Unplugged,
    Tick,
    StartRequested,
    StopRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inputs {
    pub auto_3d: bool,
    pub game_3d_running: bool,
    pub game_flat_running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    Ready,
    LateRestartFor3d,
    StoppedRestartForFlat,
    AllowUsbDebugging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    StartLink,
    Reattach,
    StopLink,
    Notify(Note),
}

/// Apply one event (plug, unplug, tick, or a hand-pressed Start/Stop) without running its effects.
pub fn step(w: &Watch, e: &Event, now: u64, i: &Inputs) -> (Watch, Vec<Effect>) {
    match (w.phase, e) {
        (Phase::Absent, Event::Plugged) if i.auto_3d => {
            let effects = vec![
                Effect::StartLink,
                Effect::Notify(if i.game_flat_running {
                    Note::LateRestartFor3d
                } else {
                    Note::Ready
                }),
            ];
            (Watch { phase: Phase::Up }, effects)
        }
        (Phase::Absent, Event::Plugged) => (
            Watch {
                phase: Phase::Plugged,
            },
            vec![],
        ),
        (Phase::Grace { .. }, Event::Plugged) => {
            (Watch { phase: Phase::Up }, vec![Effect::Reattach])
        }
        (Phase::Up, Event::Unplugged) if i.game_3d_running => (
            Watch {
                phase: Phase::Grace { since: now },
            },
            vec![],
        ),
        (Phase::Up, Event::Unplugged) => (
            Watch {
                phase: Phase::Absent,
            },
            vec![Effect::StopLink],
        ),
        (Phase::Plugged, Event::Unplugged) => (
            Watch {
                phase: Phase::Absent,
            },
            vec![],
        ),
        (Phase::Grace { .. }, Event::Tick) if !i.game_3d_running => (
            Watch {
                phase: Phase::Absent,
            },
            vec![Effect::StopLink],
        ),
        (Phase::Grace { since }, Event::Tick) if now.saturating_sub(since) >= GRACE_SECS => (
            Watch {
                phase: Phase::Absent,
            },
            vec![
                Effect::StopLink,
                Effect::Notify(Note::StoppedRestartForFlat),
            ],
        ),
        (Phase::Plugged, Event::StartRequested) => (
            Watch { phase: Phase::Up },
            vec![Effect::StartLink, Effect::Notify(Note::Ready)],
        ),
        (Phase::Grace { .. }, Event::StartRequested) => {
            (Watch { phase: Phase::Up }, vec![Effect::Reattach])
        }
        (Phase::Up | Phase::Grace { .. }, Event::StopRequested) => (
            Watch {
                phase: Phase::Plugged,
            },
            vec![Effect::StopLink],
        ),
        _ => (*w, vec![]),
    }
}

/// The link commands an effect runs: Start 3D's list, the reattach part of it (from `adb
/// reverse` on; the server unit is already running), or Stop 3D's list. The toast effect
/// runs no link commands.
pub fn link_commands(effect: &Effect) -> Vec<Vec<String>> {
    match effect {
        Effect::StartLink => crate::link::start_3d_commands(),
        Effect::Reattach => crate::link::start_3d_commands()[2..].to_vec(),
        Effect::StopLink => crate::link::stop_3d_commands(),
        Effect::Notify(_) => Vec::new(),
    }
}

/// The toast text for a note, the same on every face.
pub fn note_text(n: Note) -> &'static str {
    match n {
        Note::Ready => "3D is ready. Press Play on a 3D Ready game.",
        Note::LateRestartFor3d => "Headset connected. Restart the game to play it in 3D.",
        Note::StoppedRestartForFlat => "Headset link closed. Restart the game to play it flat.",
        Note::AllowUsbDebugging => "Put the headset on and allow USB debugging.",
    }
}

/// The note's kind word, for the state file.
pub fn note_kind(n: Note) -> &'static str {
    match n {
        Note::Ready => "ready",
        Note::LateRestartFor3d => "late-plug",
        Note::StoppedRestartForFlat => "grace-ended",
        Note::AllowUsbDebugging => "allow-debugging",
    }
}

/// The event kind a note is written under in the state file and the event ring.
pub fn note_event(n: Note) -> EventKind {
    match n {
        Note::Ready => EventKind::Ready,
        Note::LateRestartFor3d => EventKind::LatePlug,
        Note::StoppedRestartForFlat => EventKind::GraceEnded,
        Note::AllowUsbDebugging => EventKind::AllowDebugging,
    }
}
