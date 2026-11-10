//! Locked tests for E6d: tun0 and tunnel state. Pure.

use flat3d_inject::tether::relay::*;

const UP: &str = "5: tun0: <POINTOPOINT,UP,LOWER_UP> mtu 1500\n    inet 10.0.0.2/32 scope global tun0\n       valid_lft forever\n";
const DEV: &str = "Inter-|   Receive                                                |  Transmit\n \
face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n\
    lo: 100 1 0 0 0 0 0 0 100 1 0 0 0 0 0 0\n\
  tun0: 123456 10 0 0 0 0 0 0 7890 9 0 0 0 0 0 0\n";

#[test]
fn tun0_is_present_when_it_has_an_inet_address() {
    assert!(tun0_present(UP));
}

#[test]
fn tun0_without_an_address_is_not_present() {
    assert!(!tun0_present(
        "5: tun0: <POINTOPOINT> mtu 1500\n    inet6 fe80::1/64 scope link\n"
    ));
    assert!(!tun0_present(""));
}

#[test]
fn tun0_is_gone_when_the_device_does_not_exist() {
    assert!(tun0_gone("Device \"tun0\" does not exist.\n"));
    assert!(tun0_gone("  \n"));
    assert!(!tun0_gone(UP));
}

#[test]
fn the_counters_are_receive_and_transmit_bytes() {
    assert_eq!(parse_tun0_bytes(DEV), Some((123456, 7890)));
}

#[test]
fn the_counters_work_with_no_space_after_the_colon() {
    let dev = "tun0:55 1 0 0 0 0 0 0 66 1 0 0 0 0 0 0\n";
    assert_eq!(parse_tun0_bytes(dev), Some((55, 66)));
}

#[test]
fn no_tun0_or_a_short_line_is_none() {
    assert_eq!(parse_tun0_bytes("    lo: 100 1 0 0 0 0 0 0 100 1\n"), None);
    assert_eq!(parse_tun0_bytes("tun0: 1 2 3\n"), None);
    assert_eq!(parse_tun0_bytes("tun0: a 1 0 0 0 0 0 0 b 1\n"), None);
    assert_eq!(parse_tun0_bytes(""), None);
}

#[test]
fn the_tunnel_is_connected_only_with_a_relay_and_tun0() {
    assert_eq!(tunnel_state(false, false), TunnelState::Off);
    assert_eq!(tunnel_state(false, true), TunnelState::Off);
    assert_eq!(tunnel_state(true, false), TunnelState::Starting);
    assert_eq!(tunnel_state(true, true), TunnelState::Connected);
}
