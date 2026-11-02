//! Edge test for the event ring: a saved list at the top of the sequence range must not panic.

use flat3d_inject::api::{EventKind, LinkEvent};
use flat3d_inject::state::EventRing;

#[test]
fn a_saved_list_at_the_top_of_the_range_does_not_panic() {
    let saved = vec![LinkEvent { seq: u64::MAX, kind: EventKind::Ready, text: "a".into() }];
    let mut r = EventRing::from_events(saved);
    assert_eq!(r.push(EventKind::Warm, "b"), u64::MAX);
    assert_eq!(r.events().len(), 2);
}
