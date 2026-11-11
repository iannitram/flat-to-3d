//! Locked tests for I2: which Quest apps are installed. Pure.

use flat3d_inject::setup::{missing, packages_command, parse_packages, Packages};

#[test]
fn the_command() {
    assert_eq!(
        packages_command(),
        vec!["adb", "shell", "pm", "list", "packages"]
    );
}

#[test]
fn both_installed() {
    let out = "package:com.oculus.vrshell\npackage:org.meumeu.wivrn.github\npackage:com.genymobile.gnirehtet\n";
    let p = parse_packages(out);
    assert_eq!(
        p,
        Packages {
            wivrn: true,
            gnirehtet: true
        }
    );
    assert!(missing(&p).is_empty());
}

#[test]
fn look_alikes_do_not_count() {
    let out = "package:org.meumeu.wivrn.github.debug\npackage:com.genymobile.gnirehtet2\n";
    assert_eq!(
        parse_packages(out),
        Packages {
            wivrn: false,
            gnirehtet: false
        }
    );
}

#[test]
fn windows_line_endings_are_fine() {
    let out = "package:org.meumeu.wivrn.github\r\n";
    assert!(parse_packages(out).wivrn);
}

#[test]
fn missing_names_in_order() {
    assert_eq!(
        missing(&Packages {
            wivrn: false,
            gnirehtet: false
        }),
        vec!["WiVRn", "gnirehtet"]
    );
    assert_eq!(
        missing(&Packages {
            wivrn: true,
            gnirehtet: false
        }),
        vec!["gnirehtet"]
    );
    assert_eq!(
        missing(&Packages {
            wivrn: false,
            gnirehtet: true
        }),
        vec!["WiVRn"]
    );
}

#[test]
fn nothing_installed() {
    assert_eq!(
        parse_packages(""),
        Packages {
            wivrn: false,
            gnirehtet: false
        }
    );
}
