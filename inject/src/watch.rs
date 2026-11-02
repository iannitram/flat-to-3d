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
        let header = &buf[header_start..present];
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
