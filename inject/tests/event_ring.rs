//! Locked tests for E5g: the event ring.

use flat3d_inject::api::{EventKind, LinkEvent};
use flat3d_inject::state::{EventRing, RING_MAX};

fn seqs(e: &[LinkEvent]) -> Vec<u64> {
    e.iter().map(|x| x.seq).collect()
}

#[test]
fn numbering_starts_at_one() {
    let mut r = EventRing::new();
    assert_eq!(r.push(EventKind::Ready, "a"), 1);
    assert_eq!(r.push(EventKind::Warm, "b"), 2);
    assert_eq!(seqs(r.events()), vec![1, 2]);
    assert_eq!(r.events()[1].kind, EventKind::Warm);
    assert_eq!(r.events()[1].text, "b");
}

#[test]
fn it_keeps_the_last_ten() {
    assert_eq!(RING_MAX, 10);
    let mut r = EventRing::new();
    for _ in 0..12 {
        r.push(EventKind::Ready, "x");
    }
    assert_eq!(r.events().len(), 10);
    assert_eq!(r.events()[0].seq, 3);
    assert_eq!(r.events()[9].seq, 12);
}

#[test]
fn since_returns_only_newer_events() {
    let mut r = EventRing::new();
    for _ in 0..12 {
        r.push(EventKind::Hot, "x");
    }
    assert_eq!(seqs(&r.since(10)), vec![11, 12]);
    assert!(r.since(12).is_empty());
    assert_eq!(r.since(0).len(), 10);
}

#[test]
fn numbering_resumes_from_a_saved_list() {
    let saved = vec![
        LinkEvent {
            seq: 7,
            kind: EventKind::Ready,
            text: "a".into(),
        },
        LinkEvent {
            seq: 8,
            kind: EventKind::Hot,
            text: "b".into(),
        },
    ];
    let mut r = EventRing::from_events(saved);
    assert_eq!(r.push(EventKind::Warm, "c"), 9);
    assert_eq!(seqs(r.events()), vec![7, 8, 9]);
    assert_eq!(
        EventRing::from_events(vec![]).push(EventKind::Ready, "z"),
        1
    );
}
