//! Edge test for the event ring: a saved list at the top of the sequence range must not panic.

use flat3d_inject::api::{EventKind, LinkEvent};
use flat3d_inject::state::EventRing;

#[test]
fn a_saved_list_at_the_top_of_the_range_does_not_panic() {
    let saved = vec![LinkEvent {
        seq: u64::MAX,
        kind: EventKind::Ready,
        text: "a".into(),
    }];
    let mut r = EventRing::from_events(saved);
    assert_eq!(
        r.events()[0].seq,
        1,
        "the retained event is renumbered from one"
    );
    assert_eq!(r.push(EventKind::Warm, "b"), 2);
    assert_eq!(r.events().len(), 2);
}

#[test]
fn push_at_the_top_of_the_range_renumbers_and_goes_on() {
    // A ring restored at the top of the range: the next push would be seq u64::MAX.
    let saved = vec![LinkEvent {
        seq: u64::MAX - 1,
        kind: EventKind::Ready,
        text: "x".into(),
    }];
    let mut r = EventRing::from_events(saved);
    assert_eq!(
        r.push(EventKind::Warm, "y"),
        2,
        "reaching the top renumbers the retained events"
    );
    assert_eq!(r.push(EventKind::Warm, "z"), 3);
    let seqs: Vec<u64> = r.events().iter().map(|e| e.seq).collect();
    assert_eq!(seqs, vec![1, 2, 3]);
}
