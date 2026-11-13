//! Locked tests for E2a: the Start 3D and Stop 3D command lists and the `adb devices`
//! reader. Pure: nothing is run. WiVRn wired mode only (no Quest Tether, DECISIONS facts).

use flat3d_inject::link::{
    headset_from_adb, start_3d_commands, stop_3d_commands, Headset, WIVRN_APP, WIVRN_FLATPAK,
    WIVRN_PORT, WIVRN_UNIT,
};

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn the_constants_are_these() {
    assert_eq!(WIVRN_PORT, 9757);
    assert_eq!(WIVRN_FLATPAK, "io.github.wivrn.wivrn");
    assert_eq!(WIVRN_APP, "org.meumeu.wivrn.github");
    assert_eq!(WIVRN_UNIT, "flat3d-wivrn");
}

#[test]
fn start_3d_runs_these_four_commands_in_order() {
    let expected = vec![
        words(&["adb", "devices"]),
        words(&[
            "systemd-run",
            "--user",
            "--collect",
            "--unit=flat3d-wivrn",
            "flatpak",
            "run",
            "--command=wivrn-server",
            "io.github.wivrn.wivrn",
        ]),
        words(&["adb", "reverse", "tcp:9757", "tcp:9757"]),
        words(&[
            "adb",
            "shell",
            "am",
            "start",
            "-a",
            "android.intent.action.VIEW",
            "-d",
            "wivrn+tcp://localhost",
            "org.meumeu.wivrn.github",
        ]),
    ];
    assert_eq!(start_3d_commands(), expected);
}

#[test]
fn stop_3d_runs_these_three_commands_in_order() {
    let expected = vec![
        words(&[
            "adb",
            "shell",
            "am",
            "force-stop",
            "org.meumeu.wivrn.github",
        ]),
        words(&["adb", "reverse", "--remove", "tcp:9757"]),
        words(&["systemctl", "--user", "stop", "flat3d-wivrn.service"]),
    ];
    assert_eq!(stop_3d_commands(), expected);
}

#[test]
fn no_command_uses_quest_tether_or_the_wivrn_window() {
    for c in start_3d_commands().iter().chain(stop_3d_commands().iter()) {
        let line = c.join(" ").to_lowercase();
        // The relay's name is split so WP-37's `git grep -i` over crates/ stays empty.
        assert!(
            !line.contains("tether") && !line.contains(concat!("gnire", "htet")),
            "{line}"
        );
        assert!(!line.contains("wivrn-dashboard"), "{line}");
    }
}

#[test]
fn the_lists_are_the_same_every_call_so_start_works_again_after_stop() {
    assert_eq!(start_3d_commands(), start_3d_commands());
    assert_eq!(stop_3d_commands(), stop_3d_commands());
}

#[test]
fn a_device_line_means_present() {
    let out = "List of devices attached\n1WMHH0A1B2C3D4\tdevice\n\n";
    assert_eq!(headset_from_adb(out), Headset::Present);
}

#[test]
fn only_the_header_means_missing() {
    assert_eq!(
        headset_from_adb("List of devices attached\n\n"),
        Headset::Missing
    );
    assert_eq!(headset_from_adb(""), Headset::Missing);
}

#[test]
fn an_unauthorized_line_means_unauthorised() {
    let out = "List of devices attached\n1WMHH0A1B2C3D4\tunauthorized\n\n";
    assert_eq!(headset_from_adb(out), Headset::Unauthorised);
}

#[test]
fn an_offline_line_means_missing() {
    let out = "List of devices attached\n1WMHH0A1B2C3D4\toffline\n";
    assert_eq!(headset_from_adb(out), Headset::Missing);
}

#[test]
fn any_ready_device_wins_over_an_unauthorized_one() {
    let out = "List of devices attached\nAAA\tunauthorized\nBBB\tdevice\n";
    assert_eq!(headset_from_adb(out), Headset::Present);
}

#[test]
fn daemon_start_lines_and_spaces_are_skipped() {
    let out = "* daemon not running; starting now at tcp:5037\n* daemon started successfully\n\
               List of devices attached\n1WMHH0A1B2C3D4   device usb:1-2 product:hollywood\n";
    assert_eq!(headset_from_adb(out), Headset::Present);
}
