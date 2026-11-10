//! Locked tests for E4b: the launch-option wrapper rule. Pure.

use flat3d_inject::launch_options::{add_wrapper, has_wrapper, remove_wrapper, WRAPPER};

const PAIRS: [(&str, &str); 7] = [
    ("", "flat3d-run %command%"),
    ("%command%", "flat3d-run %command%"),
    ("mangohud %command%", "flat3d-run mangohud %command%"),
    ("-novid", "flat3d-run %command% -novid"),
    ("FOO=1 BAR=2 %command%", "FOO=1 BAR=2 flat3d-run %command%"),
    (
        "FOO=1 mangohud %command% -x",
        "FOO=1 flat3d-run mangohud %command% -x",
    ),
    (
        "gamemoderun %command% -windowed",
        "flat3d-run gamemoderun %command% -windowed",
    ),
];

#[test]
fn add_puts_the_wrapper_in_front_of_the_command() {
    for (before, after) in PAIRS {
        assert_eq!(add_wrapper(before), after, "{before}");
    }
}

#[test]
fn add_does_nothing_when_the_wrapper_is_already_there() {
    assert_eq!(add_wrapper("flat3d-run %command%"), "flat3d-run %command%");
    let full = "/home/deck/.local/bin/flat3d-run mangohud %command%";
    assert_eq!(add_wrapper(full), full);
}

#[test]
fn add_collapses_extra_spaces() {
    assert_eq!(
        add_wrapper("  mangohud   %command%  "),
        "flat3d-run mangohud %command%"
    );
}

#[test]
fn remove_is_the_inverse_of_add() {
    for (before, after) in PAIRS {
        let want = if before == "%command%" { "" } else { before };
        assert_eq!(remove_wrapper(after), want, "{after}");
    }
}

#[test]
fn remove_leaves_options_without_the_wrapper_alone() {
    for o in [
        "",
        "mangohud %command%",
        "-novid",
        "flat3d-runner %command%",
    ] {
        assert_eq!(remove_wrapper(o), o);
    }
}

#[test]
fn remove_takes_out_the_wrapper_given_by_full_path() {
    assert_eq!(
        remove_wrapper("/home/deck/.local/bin/flat3d-run %command%"),
        ""
    );
}

#[test]
fn has_wrapper_looks_at_whole_words_before_the_command() {
    assert_eq!(WRAPPER, "flat3d-run");
    assert!(has_wrapper("flat3d-run %command%"));
    assert!(has_wrapper("FOO=1 flat3d-run %command% -x"));
    assert!(!has_wrapper("flat3d-runner %command%"));
    assert!(!has_wrapper("%command% flat3d-run"));
    assert!(!has_wrapper(""));
}
