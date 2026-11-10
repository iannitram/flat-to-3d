//! Locked tests for E6i: the heal ladder. Pure: `now` is passed in.

use flat3d_inject::tether::heal::{Heal, Healer};

fn miss(h: &mut Healer, t: u64) -> Heal {
    h.observe(t, true, false)
}

#[test]
fn a_good_tunnel_needs_nothing() {
    let mut h = Healer::new();
    assert_eq!(h.observe(0, true, true), Heal::None);
}

#[test]
fn the_third_miss_in_a_row_restarts_the_client() {
    let mut h = Healer::new();
    assert_eq!(miss(&mut h, 1), Heal::None);
    assert_eq!(miss(&mut h, 2), Heal::None);
    assert_eq!(miss(&mut h, 3), Heal::RestartClient);
}

#[test]
fn a_good_reading_resets_the_miss_count() {
    let mut h = Healer::new();
    miss(&mut h, 1);
    miss(&mut h, 2);
    assert_eq!(h.observe(3, true, true), Heal::None);
    assert_eq!(miss(&mut h, 4), Heal::None);
    assert_eq!(miss(&mut h, 5), Heal::None);
}

#[test]
fn heals_are_at_least_thirty_seconds_apart_and_climb_the_ladder() {
    let mut h = Healer::new();
    for t in 1..=3 {
        miss(&mut h, t);
    }
    assert_eq!(miss(&mut h, 10), Heal::None);
    assert_eq!(miss(&mut h, 11), Heal::None);
    assert_eq!(miss(&mut h, 12), Heal::None);
    assert_eq!(miss(&mut h, 32), Heal::None);
    assert_eq!(miss(&mut h, 33), Heal::RestartAdbThenClient);
    for t in 34..=36 {
        assert_eq!(miss(&mut h, t), Heal::None);
    }
    assert_eq!(miss(&mut h, 63), Heal::FullReconnect);
    assert_eq!(h.reconnects(), 1);
}

#[test]
fn a_recovery_puts_the_ladder_back_at_the_start() {
    let mut h = Healer::new();
    for t in 1..=3 {
        miss(&mut h, t);
    }
    assert_eq!(h.observe(10, true, true), Heal::None);
    for t in 40..=42 {
        let got = miss(&mut h, t);
        if t == 42 {
            assert_eq!(got, Heal::RestartClient);
        }
    }
}

#[test]
fn a_dead_relay_is_a_miss_even_with_tun0_present() {
    let mut h = Healer::new();
    assert_eq!(h.observe(1, false, true), Heal::None);
    assert_eq!(h.observe(2, false, false), Heal::None);
    assert_eq!(h.observe(3, false, true), Heal::RestartClient);
}

#[test]
fn more_than_six_heals_in_ten_minutes_gives_up_for_good() {
    let mut h = Healer::new();
    for k in 0..6u64 {
        let t = k * 40;
        miss(&mut h, t);
        miss(&mut h, t + 1);
        assert_eq!(miss(&mut h, t + 2), Heal::RestartClient);
        assert_eq!(h.observe(t + 3, true, true), Heal::None);
    }
    miss(&mut h, 240);
    miss(&mut h, 241);
    assert_eq!(miss(&mut h, 242), Heal::GiveUp);
    assert_eq!(h.observe(300, true, true), Heal::GiveUp);
}

#[test]
fn ten_full_reconnects_is_the_limit() {
    let mut h = Healer::new();
    let mut t = 0u64;
    let mut heals = 0;
    loop {
        miss(&mut h, t);
        miss(&mut h, t + 1);
        let got = miss(&mut h, t + 2);
        t += 120;
        if got == Heal::GiveUp {
            break;
        }
        assert_ne!(got, Heal::None);
        heals += 1;
        assert!(heals < 100, "never gave up");
    }
    assert_eq!(h.reconnects(), 10);
    assert_eq!(heals, 32);
}
