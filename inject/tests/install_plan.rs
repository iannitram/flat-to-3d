//! Locked tests for I1: the install and uninstall plans. Pure.

use flat3d_inject::install::*;

fn c(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}
fn p(a: &str, b: &str) -> (String, String) {
    (a.to_string(), b.to_string())
}
const H: &str = "/home/deck";

#[test]
fn the_unit_text_is_exactly_this() {
    assert_eq!(
        unit_text("%h/.local/bin/flat3d-inject watch"),
        "[Unit]\nDescription=Flat to 3D headset link\n\n[Service]\nType=simple\nExecStart=%h/.local/bin/flat3d-inject watch\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n"
    );
}

#[test]
fn the_paths() {
    assert_eq!(UNIT_NAME, "flat3d-link.service");
    assert_eq!(unit_path(H), "/home/deck/.config/systemd/user/flat3d-link.service");
    assert_eq!(bin_dir(H), "/home/deck/.local/bin");
}

#[test]
fn install_copies_the_binaries_then_the_tools() {
    let plan = install_plan(H, "/opt/f3d");
    let t = "/home/deck/.local/share/flat3d/bin";
    assert_eq!(
        plan.copies,
        vec![
            p("/opt/f3d/flat3d-inject", "/home/deck/.local/bin/flat3d-inject"),
            p("/opt/f3d/flat3d-run", "/home/deck/.local/bin/flat3d-run"),
            p("/opt/f3d/tools/adb", &format!("{t}/adb")),
            p("/opt/f3d/tools/gnirehtet", &format!("{t}/gnirehtet")),
            p("/opt/f3d/tools/gnirehtet.apk", &format!("{t}/gnirehtet.apk")),
        ]
    );
}

#[test]
fn install_writes_the_unit_and_enables_it() {
    let plan = install_plan(H, "/opt/f3d");
    assert_eq!(plan.unit_path, unit_path(H));
    assert_eq!(plan.unit_text, unit_text("%h/.local/bin/flat3d-inject watch"));
    assert_eq!(
        plan.commands,
        vec![
            c(&["systemctl", "--user", "daemon-reload"]),
            c(&["systemctl", "--user", "enable", "--now", "flat3d-link.service"]),
        ]
    );
}

#[test]
fn uninstall_disables_removes_and_reloads() {
    let u = uninstall_plan(H);
    assert_eq!(u.before, vec![c(&["systemctl", "--user", "disable", "--now", "flat3d-link.service"])]);
    assert_eq!(u.after, vec![c(&["systemctl", "--user", "daemon-reload"])]);
    assert_eq!(u.removes.len(), 6);
    assert_eq!(u.removes[0], "/home/deck/.config/systemd/user/flat3d-link.service");
    assert!(u.removes.contains(&"/home/deck/.local/bin/flat3d-run".to_string()));
    assert!(u.removes.contains(&"/home/deck/.local/share/flat3d/bin/gnirehtet.apk".to_string()));
}

#[test]
fn uninstall_keeps_settings_and_logs() {
    for r in uninstall_plan(H).removes {
        assert!(!r.contains(".config/flat3d") && !r.contains(".local/state"), "{r}");
    }
}
