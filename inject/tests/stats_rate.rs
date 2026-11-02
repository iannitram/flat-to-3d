//! Locked tests for E7c: throughput from counters. Pure.

use flat3d_inject::stats::RateMeter;

#[test]
fn the_first_reading_has_no_rate() {
    let mut m = RateMeter::new();
    assert_eq!(m.sample(1000, Some((0, 0))), (0, 0));
}

#[test]
fn a_normal_rate() {
    let mut m = RateMeter::new();
    m.sample(1000, Some((0, 0)));
    assert_eq!(m.sample(2000, Some((1000, 500))), (1000, 500));
    assert_eq!(m.sample(2500, Some((1500, 500))), (1000, 0));
}

#[test]
fn a_missing_reading_forgets_the_last_one() {
    let mut m = RateMeter::new();
    m.sample(1000, Some((10, 10)));
    assert_eq!(m.sample(2000, None), (0, 0));
    assert_eq!(m.sample(3000, Some((99, 99))), (0, 0));
    assert_eq!(m.sample(4000, Some((199, 99))), (100, 0));
}

#[test]
fn a_counter_that_went_backwards_gives_zero_then_recovers() {
    let mut m = RateMeter::new();
    m.sample(1000, Some((1000, 1000)));
    assert_eq!(m.sample(2000, Some((5, 5))), (0, 0));
    assert_eq!(m.sample(3000, Some((105, 5))), (100, 0));
}

#[test]
fn a_zero_time_step_gives_zero_and_keeps_the_old_reading() {
    let mut m = RateMeter::new();
    m.sample(1000, Some((0, 0)));
    assert_eq!(m.sample(1000, Some((500, 500))), (0, 0));
    assert_eq!(m.sample(2000, Some((1000, 1000))), (1000, 1000));
}

#[test]
fn a_clock_step_back_gives_zero_and_keeps_the_old_reading() {
    let mut m = RateMeter::new();
    m.sample(2000, Some((0, 0)));
    assert_eq!(m.sample(1000, Some((500, 500))), (0, 0));
    assert_eq!(m.sample(3000, Some((1000, 1000))), (1000, 1000));
}

#[test]
fn a_huge_counter_step_saturates_instead_of_overflowing() {
    let mut m = RateMeter::new();
    m.sample(1000, Some((0, 0)));
    assert_eq!(m.sample(2000, Some((u64::MAX / 2, 0))).0, u64::MAX / 1000);
}
