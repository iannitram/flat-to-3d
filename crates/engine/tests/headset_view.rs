//! Locked tests for E10d: which Headset-page buttons are live, and the Logs lines. Pure.

use flat3d_inject::api::LinkState::{self, *};
use flat3d_inject::headset_view::{
    headset_buttons, log_copy_text, log_lines_newest_first, DEFAULT_LOG_LINES,
};

const ALL: [LinkState; 5] = [NoHeadset, Unauthorised, Starting, Ready, Stopped];

fn s(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

#[test]
fn start_is_live_unless_the_link_is_starting_or_ready() {
    let got: Vec<bool> = ALL.iter().map(|x| headset_buttons(*x).start).collect();
    assert_eq!(got, vec![true, true, false, false, true]);
}

#[test]
fn stop_is_live_only_while_the_link_is_starting_or_ready() {
    let got: Vec<bool> = ALL.iter().map(|x| headset_buttons(*x).stop).collect();
    assert_eq!(got, vec![false, false, true, true, false]);
}

#[test]
fn the_newest_line_comes_first() {
    assert_eq!(
        log_lines_newest_first("a\nb\nc\n", 500),
        s(&["c", "b", "a"])
    );
}

#[test]
fn blank_lines_and_carriage_returns_are_dropped() {
    assert_eq!(
        log_lines_newest_first("a\r\n\r\n  \nb\r\n", 500),
        s(&["b", "a"])
    );
    assert!(log_lines_newest_first("", 500).is_empty());
    assert!(log_lines_newest_first("\n\n", 500).is_empty());
}

#[test]
fn only_the_newest_limit_lines_are_kept() {
    assert_eq!(log_lines_newest_first("1\n2\n3\n4", 2), s(&["4", "3"]));
    let big: Vec<String> = (0..600).map(|i| i.to_string()).collect();
    let got = log_lines_newest_first(&big.join("\n"), DEFAULT_LOG_LINES);
    assert_eq!(got.len(), 500);
    assert_eq!(got[0], "599");
    assert_eq!(got[499], "100");
}

#[test]
fn the_copy_text_has_no_trailing_newline() {
    assert_eq!(log_copy_text(&s(&["c", "b", "a"])), "c\nb\na");
    assert_eq!(log_copy_text(&[]), "");
}
