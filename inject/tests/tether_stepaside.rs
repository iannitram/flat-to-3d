//! Locked tests for E6j: step-aside. Pure.

use flat3d_inject::api::TetherMode;
use flat3d_inject::tether::stepaside::{foreign_relay, parse_ps, tether_mode};

fn p(pid: u32, args: &str) -> (u32, String) {
    (pid, args.to_string())
}

#[test]
fn ps_lines_become_pid_and_command() {
    let out = "    1 /sbin/init\n 4242 java -jar /x/gnirehtet.jar run S1\nheader line\n\n";
    assert_eq!(parse_ps(out), vec![p(1, "/sbin/init"), p(4242, "java -jar /x/gnirehtet.jar run S1")]);
}

#[test]
fn a_gnirehtet_run_is_a_relay() {
    assert!(foreign_relay(&[p(10, "/usr/bin/gnirehtet run S1")], None));
    assert!(foreign_relay(&[p(10, "java -jar /x/gnirehtet.jar relay")], None));
}

#[test]
fn our_own_relay_does_not_count() {
    assert!(!foreign_relay(&[p(10, "/h/bin/gnirehtet run S1")], Some(10)));
    assert!(foreign_relay(&[p(10, "/h/bin/gnirehtet run S1"), p(11, "/u/gnirehtet run S1")], Some(10)));
}

#[test]
fn adb_commands_that_mention_gnirehtet_do_not_count() {
    let adb = p(5, "adb -s S1 shell am start -n com.genymobile.gnirehtet/.GnirehtetActivity -a com.genymobile.gnirehtet.START");
    assert!(!foreign_relay(&[adb], None));
    assert!(!foreign_relay(&[p(6, "/h/bin/adb reverse localabstract:gnirehtet tcp:31416")], None));
}

#[test]
fn other_programs_do_not_count() {
    assert!(!foreign_relay(&[p(1, "/sbin/init"), p(2, "vim gnirehtet.txt")], None));
    assert!(!foreign_relay(&[], None));
}

#[test]
fn the_mode_follows_a_foreign_relay_first() {
    assert_eq!(tether_mode(true, true), TetherMode::QuestTether);
    assert_eq!(tether_mode(false, true), TetherMode::QuestTether);
    assert_eq!(tether_mode(true, false), TetherMode::Own);
    assert_eq!(tether_mode(false, false), TetherMode::Off);
}
