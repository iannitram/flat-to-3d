//! Locked tests for E9b: which toasts are new. Pure.

use flat3d_inject::api::{EventKind, LinkEvent};
use flat3d_inject::toasts::toasts_since;

fn e(seq: u64) -> LinkEvent {
    LinkEvent {
        seq,
        kind: EventKind::Ready,
        text: format!("event {seq}"),
    }
}
fn seqs(events: &[LinkEvent]) -> Vec<u64> {
    events.iter().map(|x| x.seq).collect()
}

#[test]
fn only_the_newer_events_oldest_first_and_the_mark_moves() {
    let got = toasts_since(&[e(7), e(5), e(6)], 5);
    assert_eq!(seqs(&got.toasts), vec![6, 7]);
    assert_eq!(got.last_seq, 7);
}

#[test]
fn nothing_new_when_everything_was_shown() {
    let got = toasts_since(&[e(1), e(2)], 2);
    assert!(got.toasts.is_empty());
    assert_eq!(got.last_seq, 2);
}

#[test]
fn no_events_keeps_the_mark() {
    let got = toasts_since(&[], 9);
    assert!(got.toasts.is_empty());
    assert_eq!(got.last_seq, 9);
}

#[test]
fn starting_from_zero_shows_everything() {
    let got = toasts_since(&[e(1), e(2)], 0);
    assert_eq!(seqs(&got.toasts), vec![1, 2]);
    assert_eq!(got.last_seq, 2);
}

#[test]
fn a_counter_that_went_backwards_shows_everything_again() {
    let got = toasts_since(&[e(2), e(1)], 40);
    assert_eq!(seqs(&got.toasts), vec![1, 2]);
    assert_eq!(got.last_seq, 2);
}

#[test]
fn the_text_travels_with_the_event() {
    let got = toasts_since(&[e(3)], 0);
    assert_eq!(got.toasts[0].text, "event 3");
}
