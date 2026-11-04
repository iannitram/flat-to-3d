//! The wrapper rule reads words outside double quotes only: a space inside quotes is part of the value.

use flat3d_inject::launch_options::{add_wrapper, remove_wrapper};

#[test]
fn the_wrapper_goes_before_the_command_and_never_inside_a_quoted_value() {
    assert_eq!(
        add_wrapper(r#"PROTON_LOG_DIR="/a b" %command%"#),
        r#"PROTON_LOG_DIR="/a b" flat3d-run %command%"#
    );
    assert_eq!(
        add_wrapper(r#"DXVK_HUD="fps, memory" %command% -novid"#),
        r#"DXVK_HUD="fps, memory" flat3d-run %command% -novid"#
    );
}

#[test]
fn an_escaped_quote_inside_double_quotes_does_not_close_the_quote() {
    assert_eq!(
        add_wrapper(r#"A="x\"y z" %command%"#),
        r#"A="x\"y z" flat3d-run %command%"#
    );
}

#[test]
fn an_unclosed_quote_splits_as_before_and_keeps_one_command() {
    assert_eq!(
        add_wrapper(r#"WINEPATH="C:\" %command% -x"#),
        r#"WINEPATH="C:\" flat3d-run %command% -x"#
    );
    assert_eq!(
        add_wrapper(r#"A="x %command%"#),
        r#"A="x flat3d-run %command%"#
    );
}

#[test]
fn a_command_inside_quotes_is_still_the_one_command() {
    assert_eq!(
        add_wrapper(r#"sh -c "x %command% y""#),
        r#"flat3d-run sh -c "x %command% y""#
    );
    assert_eq!(
        remove_wrapper(r#"flat3d-run sh -c "x %command% y""#),
        r#"sh -c "x %command% y""#
    );
}

#[test]
fn spaces_inside_quotes_survive_add_and_remove() {
    assert_eq!(
        add_wrapper(r#"FOO=1 BAR="a  b" %command% -x"#),
        r#"FOO=1 BAR="a  b" flat3d-run %command% -x"#
    );
    assert_eq!(
        remove_wrapper(r#"FOO=1 BAR="a  b" flat3d-run %command% -x "c  d""#),
        r#"FOO=1 BAR="a  b" %command% -x "c  d""#
    );
}
