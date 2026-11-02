//! Locked tests for E5a: reading `adb track-devices` frames. Pure: no adb is run.

use flat3d_inject::watch::{parse_track_frames, Device};

fn d(serial: &str, state: &str) -> Device {
    Device { serial: serial.to_string(), state: state.to_string() }
}

#[test]
fn an_empty_frame_is_an_empty_list() {
    assert_eq!(parse_track_frames("0000"), (vec![vec![]], String::new()));
}

#[test]
fn one_frame_with_one_device() {
    let got = parse_track_frames("000eABC123\tdevice\n");
    assert_eq!(got, (vec![vec![d("ABC123", "device")]], String::new()));
}

#[test]
fn one_frame_with_two_devices() {
    let got = parse_track_frames("001cAAA\tunauthorized\nBBB\tdevice\n");
    assert_eq!(
        got,
        (vec![vec![d("AAA", "unauthorized"), d("BBB", "device")]], String::new())
    );
}

#[test]
fn two_frames_back_to_back() {
    let got = parse_track_frames("0000000eABC123\tdevice\n");
    assert_eq!(got, (vec![vec![], vec![d("ABC123", "device")]], String::new()));
}

#[test]
fn a_cut_frame_is_kept_for_the_next_read() {
    let got = parse_track_frames("000eABC123\tdevice\n000eABC");
    assert_eq!(got, (vec![vec![d("ABC123", "device")]], "000eABC".to_string()));
    let whole = parse_track_frames("000eABC123\tdev");
    assert_eq!(whole, (vec![], "000eABC123\tdev".to_string()));
}

#[test]
fn a_cut_header_is_kept() {
    assert_eq!(parse_track_frames("00"), (vec![], "00".to_string()));
    assert_eq!(parse_track_frames(""), (vec![], String::new()));
}

#[test]
fn a_bad_header_drops_the_buffer() {
    assert_eq!(parse_track_frames("zzzzfoo"), (vec![], String::new()));
    assert_eq!(parse_track_frames("zz"), (vec![], String::new()));
}

#[test]
fn frames_before_a_bad_header_are_kept() {
    let got = parse_track_frames("000eABC123\tdevice\nzzzz");
    assert_eq!(got, (vec![vec![d("ABC123", "device")]], String::new()));
}

#[test]
fn lines_without_a_tab_are_skipped() {
    // payload "junk\nXYZ\tdevice\n" is 5 + 11 = 16 bytes = 0x10
    let got = parse_track_frames("0010junk\nXYZ\tdevice\n");
    assert_eq!(got, (vec![vec![d("XYZ", "device")]], String::new()));
}
