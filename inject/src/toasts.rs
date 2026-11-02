//! Which toasts are new. docs/code-map.md, API shape, LinkEvent.

use crate::api::LinkEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct ToastBatch {
    pub toasts: Vec<LinkEvent>,
    pub last_seq: u64,
}

pub fn toasts_since(events: &[LinkEvent], last_seq: u64) -> ToastBatch {
    let Some(max_seq) = events.iter().map(|event| event.seq).max() else {
        return ToastBatch {
            toasts: Vec::new(),
            last_seq,
        };
    };

    let restarted = max_seq < last_seq;
    let mut toasts: Vec<LinkEvent> = events
        .iter()
        .filter(|event| restarted || event.seq > last_seq)
        .cloned()
        .collect();
    toasts.sort_by_key(|event| event.seq);

    ToastBatch {
        toasts,
        last_seq: if max_seq > last_seq || restarted {
            max_seq
        } else {
            last_seq
        },
    }
}
