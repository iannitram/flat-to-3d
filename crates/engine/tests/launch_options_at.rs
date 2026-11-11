use flat3d_inject::launch_options::{add_wrapper_at, remove_wrapper};

const A: &str = "~/.local/bin/flat3d-run";

#[test]
fn adds_absolute_wrapper_and_is_idempotent() {
    for (before, expected) in [
        ("", format!("{A} %command%")),
        ("%command%", format!("{A} %command%")),
        (
            "mangohud %command% -novid",
            format!("{A} mangohud %command% -novid"),
        ),
        (
            "FOO=1 BAR=2 %command%",
            format!("FOO=1 BAR=2 {A} %command%"),
        ),
        ("-novid", format!("{A} %command% -novid")),
        ("flat3d-run %command%", format!("{A} %command%")),
        ("FOO=1 flat3d-run %command%", format!("FOO=1 {A} %command%")),
        ("/opt/x/flat3d-run %command%", format!("{A} %command%")),
        ("flat3d-run flat3d-run %command%", format!("{A} %command%")),
    ] {
        let got = add_wrapper_at(before, A);
        assert_eq!(got, expected, "{before}");
        assert_eq!(add_wrapper_at(&got, A), got, "{before}");
    }
}

#[test]
fn wrapper_can_be_removed_and_quotes_survive() {
    for before in [
        "",
        "%command%",
        "mangohud %command%",
        "-novid",
        "FOO=1 BAR=2 %command%",
        "FOO=1 mangohud %command% -x",
        "gamemoderun %command% -windowed",
    ] {
        let expected = if before == "%command%" { "" } else { before };
        assert_eq!(remove_wrapper(&add_wrapper_at(before, A)), expected);
    }
    assert_eq!(
        add_wrapper_at("%command% -arg \"a b\"", A),
        format!("{A} %command% -arg \"a b\"")
    );
    assert_eq!(
        add_wrapper_at("%command% flat3d-run", A),
        format!("{A} %command% flat3d-run")
    );
}

#[test]
fn invalid_paths_fall_back_to_bare_wrapper() {
    assert_eq!(
        add_wrapper_at("%command%", "/home/a b/.local/bin/flat3d-run"),
        "flat3d-run %command%"
    );
    assert_eq!(
        add_wrapper_at("%command%", "/usr/bin/other"),
        "flat3d-run %command%"
    );
}

#[test]
fn user_quotes_are_kept_and_round_trip() {
    let x = "sh -c \"%command%\"";
    let added = add_wrapper_at(x, A);
    assert_eq!(added, format!("{A} %command% sh -c \"%command%\""));
    assert_eq!(remove_wrapper(&added), x);
}

#[test]
fn shell_special_paths_fall_back_to_bare_wrapper() {
    for bad in [
        "/h/a&b/flat3d-run",
        "/h/a'b/flat3d-run",
        "/h/$x/flat3d-run",
        "/h/a;b/flat3d-run",
        "/h/a(b)/flat3d-run",
        "/h/a*/flat3d-run",
        "/h/a#b/flat3d-run",
        "/h/a\\b/flat3d-run",
        "/h/~x/flat3d-run",
        "~x/flat3d-run",
    ] {
        assert_eq!(
            add_wrapper_at("%command%", bad),
            "flat3d-run %command%",
            "{bad}"
        );
    }
    assert_eq!(add_wrapper_at("%command%", A), format!("{A} %command%"));
}
