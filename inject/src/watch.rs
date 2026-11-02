//! The link service: reads adb track-devices, decides, and runs the link. docs/code-map.md, Parts.

/// One device listed in an `adb track-devices` frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// The device's serial.
    pub serial: String,
    /// adb's state word, e.g. `device` or `unauthorized`.
    pub state: String,
}

/// Read `adb track-devices` output into the devices of every complete frame, in order, plus the
/// unconsumed rest of the buffer.
///
/// A frame is four hex digits holding the payload length in bytes, then that many bytes of
/// `serial<TAB>state` lines. A frame that is not complete yet is left in the rest for the next
/// read; a bad header drops the whole buffer.
pub fn parse_track_frames(buf: &str) -> (Vec<Vec<Device>>, String) {
    let mut frames = Vec::new();
    let mut pos = 0;
    loop {
        let header_start = pos;
        let header_end = header_start + 4;
        let present = header_end.min(buf.len());
        // `get` is None when the header cut lands inside a multi-byte character: that is a
        // non-hex char, so the buffer is bad too.
        let Some(header) = buf.get(header_start..present) else {
            return (frames, String::new());
        };
        if !header.bytes().all(|b| b.is_ascii_hexdigit()) {
            return (frames, String::new());
        }
        if present - header_start < 4 {
            return (frames, buf[header_start..].to_string());
        }
        let Ok(len) = usize::from_str_radix(header, 16) else {
            return (frames, String::new());
        };
        let frame_end = header_end + len;
        if buf.len() < frame_end {
            return (frames, buf[header_start..].to_string());
        }
        let Some(payload) = buf.get(header_end..frame_end) else {
            return (frames, buf[header_start..].to_string());
        };
        frames.push(parse_payload(payload));
        pos = frame_end;
    }
}

/// The devices in one frame's payload, skipping blank lines and lines without a tab.
fn parse_payload(payload: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in payload.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((serial, state)) = line.split_once('\t') else {
            continue;
        };
        devices.push(Device {
            serial: serial.to_string(),
            state: state.to_string(),
        });
    }
    devices
}

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
    pub tunnel_on: bool,
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
    StartTether,
    StopTether,
    StartLink,
    Reattach,
    StopLink,
    Notify(Note),
}

/// Apply a headset plug or unplug event without running its effects.
pub fn step(w: &Watch, e: &Event, now: u64, i: &Inputs) -> (Watch, Vec<Effect>) {
    match (w.phase, e) {
        (Phase::Absent, Event::Plugged) if i.auto_3d => {
            let mut effects = Vec::new();
            if i.tunnel_on {
                effects.push(Effect::StartTether);
            }
            effects.push(Effect::StartLink);
            effects.push(Effect::Notify(if i.game_flat_running {
                Note::LateRestartFor3d
            } else {
                Note::Ready
            }));
            (Watch { phase: Phase::Up }, effects)
        }
        (Phase::Absent, Event::Plugged) => {
            let effects = if i.tunnel_on {
                vec![Effect::StartTether]
            } else {
                vec![]
            };
            (
                Watch {
                    phase: Phase::Plugged,
                },
                effects,
            )
        }
        (Phase::Grace { .. }, Event::Plugged) => {
            (Watch { phase: Phase::Up }, vec![Effect::Reattach])
        }
        (Phase::Up, Event::Unplugged) if i.game_3d_running => (
            Watch {
                phase: Phase::Grace { since: now },
            },
            vec![],
        ),
        (Phase::Up, Event::Unplugged) => {
            let mut effects = vec![Effect::StopLink];
            if i.tunnel_on {
                effects.push(Effect::StopTether);
            }
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Plugged, Event::Unplugged) => {
            let effects = if i.tunnel_on {
                vec![Effect::StopTether]
            } else {
                vec![]
            };
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Grace { .. }, Event::Tick) if !i.game_3d_running => {
            let mut effects = vec![Effect::StopLink];
            if i.tunnel_on {
                effects.push(Effect::StopTether);
            }
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        (Phase::Grace { since }, Event::Tick) if now.saturating_sub(since) >= GRACE_SECS => {
            let mut effects = vec![Effect::StopLink];
            if i.tunnel_on {
                effects.push(Effect::StopTether);
            }
            effects.push(Effect::Notify(Note::StoppedRestartForFlat));
            (
                Watch {
                    phase: Phase::Absent,
                },
                effects,
            )
        }
        _ => (*w, vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_track_frames;

    #[test]
    fn a_non_ascii_char_in_the_header_drops_the_buffer() {
        assert_eq!(parse_track_frames("zzz\u{e9}rest"), (vec![], String::new()));
        assert_eq!(parse_track_frames("abc\u{e9}"), (vec![], String::new()));
        assert_eq!(parse_track_frames("0\u{e9}"), (vec![], String::new()));
    }

    #[test]
    fn the_start_and_stop_requests_do_nothing_until_their_steps_land() {
        use super::{step, Effect, Event, Inputs, Phase, Watch};
        let i = Inputs {
            auto_3d: true,
            tunnel_on: true,
            game_3d_running: true,
            game_flat_running: true,
        };
        for phase in [
            Phase::Absent,
            Phase::Plugged,
            Phase::Up,
            Phase::Grace { since: 1 },
        ] {
            for e in [Event::StartRequested, Event::StopRequested] {
                let w = Watch { phase };
                let (n, fx) = step(&w, &e, 9, &i);
                assert_eq!((n, fx), (w, Vec::<Effect>::new()));
            }
        }
    }
}
