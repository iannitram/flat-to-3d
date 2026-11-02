//! Locked tests for E6h: the Wi-Fi sync decision. Pure.

use flat3d_inject::tether::wifi::{next_attempts, sync_decision, SyncDecision, MAX_WIFI_ATTEMPTS};

#[test]
fn three_failures_is_the_limit() {
    assert_eq!(MAX_WIFI_ATTEMPTS, 3);
}

#[test]
fn the_pc_must_be_on_wifi() {
    assert_eq!(sync_decision(None, Some("A"), 0), SyncDecision::NoPcWifi);
    assert_eq!(sync_decision(None, None, 5), SyncDecision::NoPcWifi);
}

#[test]
fn the_headset_must_be_on_wifi() {
    assert_eq!(sync_decision(Some("A"), None, 0), SyncDecision::NoQuestWifi);
}

#[test]
fn the_same_network_needs_nothing_even_after_failures() {
    assert_eq!(sync_decision(Some("A"), Some("A"), 0), SyncDecision::Same);
    assert_eq!(sync_decision(Some("A"), Some("A"), 9), SyncDecision::Same);
}

#[test]
fn a_different_network_is_switched_until_three_tries_failed() {
    assert_eq!(sync_decision(Some("A"), Some("B"), 0), SyncDecision::Switch);
    assert_eq!(sync_decision(Some("A"), Some("B"), 2), SyncDecision::Switch);
    assert_eq!(sync_decision(Some("A"), Some("B"), 3), SyncDecision::GiveUp);
}

#[test]
fn names_compare_exactly() {
    assert_eq!(sync_decision(Some("Home"), Some("home"), 0), SyncDecision::Switch);
}

#[test]
fn the_counter_resets_on_success_and_grows_on_failure() {
    assert_eq!(next_attempts(2, true), 0);
    assert_eq!(next_attempts(0, false), 1);
    assert_eq!(next_attempts(2, false), 3);
}
