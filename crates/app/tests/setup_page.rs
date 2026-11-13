//! Locked tests for A3: the Setup page's checklist. Pure.

use flat3d_app::setup_page::{all_done, steps};
use flat3d_inject::api::{AdbSource, LinkSnapshot, LinkState};
use flat3d_inject::setup::Packages;

fn snap(state: LinkState, adb: AdbSource) -> LinkSnapshot {
    let mut s = LinkSnapshot::stopped(5);
    s.state = state;
    s.adb_source = adb;
    s
}
fn done(v: &[flat3d_app::setup_page::Step]) -> Vec<bool> {
    v.iter().map(|s| s.done).collect()
}
const WIVRN: Packages = Packages { wivrn: true };

#[test]
fn five_steps_in_order() {
    let titles: Vec<&str> = steps(None, None).iter().map(|s| s.title).collect();
    assert_eq!(
        titles,
        vec![
            "adb found",
            "Headset plugged in",
            "USB debugging allowed",
            "WiVRn on the headset",
            "Seated boundary"
        ]
    );
}

#[test]
fn with_nothing_known_nothing_is_done() {
    assert_eq!(done(&steps(None, None)), vec![false; 5]);
}

#[test]
fn a_ready_headset_with_wivrn_is_all_done_but_the_boundary() {
    let v = steps(
        Some(&WIVRN),
        Some(&snap(LinkState::Ready, AdbSource::Bundled)),
    );
    assert_eq!(done(&v), vec![true, true, true, true, false]);
    assert!(all_done(&v));
}

#[test]
fn an_unauthorised_headset_is_plugged_in_but_not_allowed() {
    let v = steps(
        Some(&WIVRN),
        Some(&snap(LinkState::Unauthorised, AdbSource::Running)),
    );
    assert_eq!(done(&v)[..3], [true, true, false]);
    assert!(!all_done(&v));
}

#[test]
fn starting_counts_as_allowed() {
    let v = steps(None, Some(&snap(LinkState::Starting, AdbSource::Running)));
    assert_eq!(done(&v)[..3], [true, true, true]);
}

#[test]
fn no_headset_and_a_stopped_service_are_not_plugged_in() {
    for st in [LinkState::NoHeadset, LinkState::Stopped] {
        let v = steps(None, Some(&snap(st, AdbSource::Bundled)));
        assert_eq!(done(&v)[..3], [true, false, false]);
    }
}

#[test]
fn a_missing_adb_is_not_found() {
    let v = steps(None, Some(&snap(LinkState::Ready, AdbSource::Missing)));
    assert!(!v[0].done);
}

#[test]
fn wivrn_is_checked_alone() {
    let v = steps(Some(&Packages { wivrn: true }), None);
    assert_eq!(done(&v)[..4], [false, false, false, true]);
    let v = steps(Some(&Packages { wivrn: false }), None);
    assert!(!v[3].done);
}

#[test]
fn only_the_boundary_is_manual() {
    let v = steps(None, None);
    assert_eq!(
        v.iter().map(|s| s.manual).collect::<Vec<_>>(),
        vec![false, false, false, false, true]
    );
    assert!(!v[4].hint.is_empty());
}
