//! Review probes for E6d (#98): edge cases the locked tests do not cover.

use flat3d_inject::tether::relay::*;

#[test]
fn present_finds_inet_after_an_inet6_line() {
    assert!(tun0_present(
        "    inet6 fe80::1/64 scope link\n    inet 10.0.0.2/32 scope global tun0\n"
    ));
}

#[test]
fn present_rejects_bare_inet_word() {
    assert!(!tun0_present("    inet\n"));
    assert!(!tun0_present("    inetfoo 10.0.0.2\n"));
}

#[test]
fn gone_is_false_for_other_errors() {
    assert!(!tun0_gone("RTNETLINK answers: No such device\n"));
}

#[test]
fn bytes_exact_nine_fields_is_some() {
    assert_eq!(parse_tun0_bytes("tun0: 1 0 0 0 0 0 0 0 2\n"), Some((1, 2)));
}

#[test]
fn bytes_tabs_are_separators() {
    assert_eq!(
        parse_tun0_bytes("tun0:\t1\t0\t0\t0\t0\t0\t0\t0\t2\n"),
        Some((1, 2))
    );
}

#[test]
fn bytes_do_not_match_a_longer_interface_name() {
    assert_eq!(
        parse_tun0_bytes("  notun0: 1 0 0 0 0 0 0 0 2\n  tun0.1: 5 0 0 0 0 0 0 0 6\n"),
        None
    );
}

#[test]
fn bytes_negative_or_overflowing_is_none() {
    assert_eq!(parse_tun0_bytes("tun0: -1 0 0 0 0 0 0 0 2\n"), None);
    assert_eq!(
        parse_tun0_bytes("tun0: 1 0 0 0 0 0 0 0 99999999999999999999999\n"),
        None
    );
}

#[test]
fn bytes_non_numeric_counter_anywhere_is_none() {
    // Spec: "a short or non-numeric line: None". Only fields 0 and 8 are checked today.
    assert_eq!(parse_tun0_bytes("tun0: 1 x 0 0 0 0 0 0 2 1\n"), None);
}
