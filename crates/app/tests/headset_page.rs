//! Locked tests for A1: the Headset page's state and update. Pure.

use flat3d_app::headset_page::{update, Action, HeadsetState, Msg};
use flat3d_inject::api::{LinkSnapshot, LinkState};

fn snap(state: LinkState) -> LinkSnapshot {
    let mut s = LinkSnapshot::stopped(5);
    s.state = state;
    s
}
fn with(state: LinkState) -> HeadsetState {
    HeadsetState { snapshot: Some(snap(state)), busy: false, error: None }
}

#[test]
fn a_new_page_has_nothing() {
    let s = HeadsetState::default();
    assert!(s.snapshot.is_none() && !s.busy && s.error.is_none());
}

#[test]
fn a_snapshot_is_stored_and_clears_the_error() {
    let mut s = HeadsetState { error: Some("old".into()), ..HeadsetState::default() };
    assert_eq!(update(&mut s, Msg::Snapshot(Ok(snap(LinkState::Ready)))), None);
    assert_eq!(s.snapshot.as_ref().unwrap().state, LinkState::Ready);
    assert!(s.error.is_none());
}

#[test]
fn a_failed_read_keeps_the_old_snapshot() {
    let mut s = with(LinkState::Ready);
    assert_eq!(update(&mut s, Msg::Snapshot(Err("gone".into()))), None);
    assert_eq!(s.error.as_deref(), Some("gone"));
    assert_eq!(s.snapshot.as_ref().unwrap().state, LinkState::Ready);
}

#[test]
fn start_runs_once_when_the_link_is_down() {
    let mut s = with(LinkState::Stopped);
    assert_eq!(update(&mut s, Msg::StartPressed), Some(Action::Start3d));
    assert!(s.busy);
    assert_eq!(update(&mut s, Msg::StartPressed), None);
}

#[test]
fn start_with_no_snapshot_counts_as_stopped() {
    let mut s = HeadsetState::default();
    assert_eq!(update(&mut s, Msg::StartPressed), Some(Action::Start3d));
}

#[test]
fn start_is_ignored_while_the_link_is_up_and_stop_while_it_is_down() {
    for up in [LinkState::Starting, LinkState::Ready] {
        let mut s = with(up);
        assert_eq!(update(&mut s, Msg::StartPressed), None);
        assert!(!s.busy);
        assert_eq!(update(&mut s, Msg::StopPressed), Some(Action::Stop3d));
    }
    let mut s = with(LinkState::NoHeadset);
    assert_eq!(update(&mut s, Msg::StopPressed), None);
}

#[test]
fn done_frees_the_buttons_and_refreshes() {
    let mut s = with(LinkState::Stopped);
    update(&mut s, Msg::StartPressed);
    assert_eq!(update(&mut s, Msg::Done(Ok(()))), Some(Action::Refresh));
    assert!(!s.busy);
}

#[test]
fn a_failed_command_shows_the_error() {
    let mut s = with(LinkState::Stopped);
    update(&mut s, Msg::StartPressed);
    assert_eq!(update(&mut s, Msg::Done(Err("no adb".into()))), None);
    assert!(!s.busy);
    assert_eq!(s.error.as_deref(), Some("no adb"));
}

#[test]
fn the_switches_ask_for_a_setting() {
    let mut s = with(LinkState::Ready);
    assert_eq!(update(&mut s, Msg::SetAuto3d(true)), Some(Action::SetSetting { key: "auto-3d", on: true }));
    assert!(s.snapshot.as_ref().unwrap().auto_3d);
    assert_eq!(update(&mut s, Msg::SetTunnel(false)), Some(Action::SetSetting { key: "tunnel", on: false }));
}
