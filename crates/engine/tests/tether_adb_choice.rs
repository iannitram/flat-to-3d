//! Locked tests for E6a: choosing the adb to use. Pure: nothing is run.

use flat3d_inject::api::AdbSource;
use flat3d_inject::tether::adb::{bundled_adb, choose_adb, listener_pid, proc_exe_path, tools_dir};

const SS: &str = "State  Recv-Q Send-Q Local Address:Port Peer Address:Port Process\n\
LISTEN 0      128        127.0.0.1:5037      0.0.0.0:*     users:((\"adb\",pid=1234,fd=7))\n\
LISTEN 0      4096       127.0.0.1:631       0.0.0.0:*\n\
LISTEN 0      128        127.0.0.1:15037     0.0.0.0:*     users:((\"other\",pid=99,fd=3))\n\
LISTEN 0      128            [::1]:7000          [::]:*     users:((\"x\",pid=55,fd=3))\n\
LISTEN 0      128                *:8000              *:*     users:((\"y\",pid=66,fd=3))\n";

#[test]
fn it_finds_the_listener_on_the_port() {
    assert_eq!(listener_pid(SS, 5037), Some(1234));
}

#[test]
fn a_longer_port_ending_the_same_is_not_a_match() {
    assert_eq!(listener_pid(SS, 15037), Some(99));
    assert_eq!(listener_pid(SS, 37), None);
}

#[test]
fn ipv6_and_wildcard_addresses_match() {
    assert_eq!(listener_pid(SS, 7000), Some(55));
    assert_eq!(listener_pid(SS, 8000), Some(66));
}

#[test]
fn no_listener_or_no_pid_is_none() {
    assert_eq!(listener_pid(SS, 631), None);
    assert_eq!(listener_pid(SS, 9999), None);
    assert_eq!(listener_pid("", 5037), None);
}

#[test]
fn the_exe_path_of_a_pid() {
    assert_eq!(proc_exe_path(1234), "/proc/1234/exe");
}

#[test]
fn the_tool_paths() {
    assert_eq!(
        tools_dir("/home/deck"),
        "/home/deck/.local/share/flat3d/bin"
    );
    assert_eq!(
        bundled_adb("/home/deck"),
        "/home/deck/.local/share/flat3d/bin/adb"
    );
}

#[test]
fn a_running_adb_wins_then_the_bundled_one() {
    let c = choose_adb(Some("/usr/bin/adb"), Some("/h/bin/adb"));
    assert_eq!(
        (c.path.as_str(), c.source),
        ("/usr/bin/adb", AdbSource::Running)
    );
    let c = choose_adb(None, Some("/h/bin/adb"));
    assert_eq!(
        (c.path.as_str(), c.source),
        ("/h/bin/adb", AdbSource::Bundled)
    );
    let c = choose_adb(Some(""), Some("/h/bin/adb"));
    assert_eq!(c.source, AdbSource::Bundled);
}

#[test]
fn with_neither_it_is_missing() {
    let c = choose_adb(None, None);
    assert_eq!((c.path.as_str(), c.source), ("", AdbSource::Missing));
}
