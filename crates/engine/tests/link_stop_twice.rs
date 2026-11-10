use flat3d_inject::api::LinkState;
use flat3d_inject::link::{
    self, start_3d_commands_with, start_3d_with, stop_3d, FlatpakScope, LinkError, WivrnStart,
    SERVER_FLAGS,
};
use flat3d_inject::testing::FakeRunner;

const NO_DEV: &str = "error: no devices/emulators found";
const DEVICE: &str = "List of devices attached\nSER\tdevice\n";
#[test]
fn a_second_stop_with_the_unit_not_loaded_is_stopped() {
    let mut f = FakeRunner::new()
        .ok(&["adb"], "")
        .ok(&["adb"], "")
        .ok(&["systemctl"], "")
        .fail(&["adb"], NO_DEV)
        .fail(&["adb"], NO_DEV)
        .fail(
            &["systemctl"],
            "Failed to stop flat3d-wivrn.service: Unit flat3d-wivrn.service not loaded.\n",
        );
    assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
    assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
    let mut expected = link::stop_3d_commands();
    expected.extend(link::stop_3d_commands());
    assert_eq!(f.ran.len(), 6);
    assert_eq!(f.ran, expected);
}

#[test]
fn unit_not_found_is_stopped() {
    let mut f = FakeRunner::new().ok(&["adb"], "").ok(&["adb"], "").fail(
        &["systemctl"],
        "Failed to stop flat3d-wivrn.service: Unit flat3d-wivrn.service not found.\n",
    );
    assert_eq!(stop_3d(&mut f), Ok(LinkState::Stopped));
    assert_eq!(f.ran.len(), 3);
}

#[test]
fn access_denied_is_still_an_error() {
    let mut f = FakeRunner::new().ok(&["adb"], "").ok(&["adb"], "").fail(
        &["systemctl"],
        "Failed to stop flat3d-wivrn.service: Access denied\n",
    );
    assert_eq!(
        stop_3d(&mut f),
        Err(LinkError {
            command: "systemctl --user stop flat3d-wivrn.service".into(),
            message: "Failed to stop flat3d-wivrn.service: Access denied".into()
        })
    );
    assert_eq!(f.ran.len(), 3);
}

#[test]
fn a_bus_error_is_still_an_error() {
    let mut f = FakeRunner::new().ok(&["adb"], "").ok(&["adb"], "").fail(
        &["systemctl"],
        "Failed to connect to bus: No medium found\n",
    );
    assert_eq!(
        stop_3d(&mut f).unwrap_err().message,
        "Failed to connect to bus: No medium found"
    );
    assert_eq!(f.ran.len(), 3);
}

#[test]
fn start_with_default_adds_both_flags() {
    let c = start_3d_commands_with(&WivrnStart::default());
    assert!(c[1].ends_with(&[
        "io.github.wivrn.wivrn".into(),
        SERVER_FLAGS[0].into(),
        SERVER_FLAGS[1].into()
    ]));
}

#[test]
fn custom_flags_replace_the_defaults() {
    let s = WivrnStart {
        flags: vec![],
        ..WivrnStart::default()
    };
    assert!(start_3d_commands_with(&s)[1].ends_with(&["io.github.wivrn.wivrn".into()]));
}

#[test]
fn user_scope_puts_the_flag_after_run() {
    let s = WivrnStart {
        scope: Some(FlatpakScope::User),
        ..WivrnStart::default()
    };
    assert_eq!(
        &start_3d_commands_with(&s)[1][4..7],
        &["flatpak", "run", "--user"]
    );
}

#[test]
fn a_custom_app_is_the_last_word_of_am_start() {
    let s = WivrnStart {
        app: "org.custom.app".into(),
        ..WivrnStart::default()
    };
    assert_eq!(
        start_3d_commands_with(&s)[3].last().unwrap(),
        "org.custom.app"
    );
}

#[test]
fn start_with_runs_four_commands_and_is_ready() {
    let mut f = FakeRunner::new().ok(&["adb", "devices"], DEVICE);
    assert_eq!(
        start_3d_with(&mut f, &WivrnStart::default()),
        Ok(LinkState::Ready)
    );
    assert_eq!(f.ran, start_3d_commands_with(&WivrnStart::default()));
    assert_eq!(f.ran.len(), 4);
}

#[test]
fn with_lists_never_name_tether_or_dashboard() {
    for c in start_3d_commands_with(&WivrnStart::default())
        .iter()
        .chain(link::stop_3d_commands().iter())
    {
        let line = c.join(" ").to_lowercase();
        assert!(
            !line.contains("tether")
                && !line.contains("gnirehtet")
                && !line.contains("wivrn-dashboard")
        );
    }
}

#[test]
fn link_reexports_the_wivrn_names() {
    assert_eq!(link::SERVER_FLAGS, link::wivrn::SERVER_FLAGS);
}
