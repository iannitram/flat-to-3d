//! Locked tests for E7b: the toast gate and its wording.

use flat3d_inject::api::{EventKind, Zone};
use flat3d_inject::health::{toast_text, ToastGate};

#[test]
fn yellow_toasts_once_while_playing() {
    let mut g = ToastGate::new();
    assert_eq!(g.observe(Zone::Yellow, true), Some(EventKind::Warm));
    assert_eq!(g.observe(Zone::Yellow, true), None);
}

#[test]
fn red_toasts_once_after_yellow() {
    let mut g = ToastGate::new();
    g.observe(Zone::Yellow, true);
    assert_eq!(g.observe(Zone::Red, true), Some(EventKind::Hot));
    assert_eq!(g.observe(Zone::Red, true), None);
    assert_eq!(g.observe(Zone::Yellow, true), None);
}

#[test]
fn red_first_skips_the_yellow_toast() {
    let mut g = ToastGate::new();
    assert_eq!(g.observe(Zone::Red, true), Some(EventKind::Hot));
    assert_eq!(g.observe(Zone::Yellow, true), None);
}

#[test]
fn nothing_toasts_while_not_playing() {
    let mut g = ToastGate::new();
    assert_eq!(g.observe(Zone::Red, false), None);
    assert_eq!(g.observe(Zone::Yellow, false), None);
    assert_eq!(g.observe(Zone::Yellow, true), Some(EventKind::Warm));
}

#[test]
fn green_re_arms_both() {
    let mut g = ToastGate::new();
    g.observe(Zone::Red, true);
    assert_eq!(g.observe(Zone::Green, true), None);
    assert_eq!(g.observe(Zone::Yellow, true), Some(EventKind::Warm));
    assert_eq!(g.observe(Zone::Red, true), Some(EventKind::Hot));
}

#[test]
fn green_re_arms_even_when_idle() {
    let mut g = ToastGate::new();
    g.observe(Zone::Yellow, true);
    g.observe(Zone::Green, false);
    assert_eq!(g.observe(Zone::Yellow, true), Some(EventKind::Warm));
}

#[test]
fn reset_re_arms_and_unknown_changes_nothing() {
    let mut g = ToastGate::new();
    g.observe(Zone::Red, true);
    assert_eq!(g.observe(Zone::Unknown, true), None);
    assert_eq!(g.observe(Zone::Red, true), None);
    g.reset();
    assert_eq!(g.observe(Zone::Red, true), Some(EventKind::Hot));
}

#[test]
fn the_wording() {
    assert_eq!(
        toast_text(EventKind::Warm, 38.5),
        "Headset is warm: 38.5 C. It may slow down."
    );
    assert_eq!(
        toast_text(EventKind::Hot, 42.46),
        "Headset is hot: 42.5 C. Take a break."
    );
    assert_eq!(toast_text(EventKind::Ready, 30.0), "");
}
