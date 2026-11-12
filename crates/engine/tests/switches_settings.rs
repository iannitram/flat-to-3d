//! Locked tests for E5i: the settings file.

use flat3d_inject::switches::{parse_settings, settings_text, Settings};

#[test]
fn everything_is_on_by_default() {
    let s = Settings::default();
    assert!(s.auto_3d && s.off.is_empty());
    assert!(s.game_on(2075800));
}

#[test]
fn the_default_text_is_exactly_this() {
    assert_eq!(
        settings_text(&Settings::default()),
        "flat3d-settings 1\nauto-3d\ton\n"
    );
}

#[test]
fn off_games_are_listed_in_order() {
    let mut s = Settings::default();
    s.auto_3d = false;
    s.set_game(502500, false);
    s.set_game(100, false);
    assert_eq!(
        settings_text(&s),
        "flat3d-settings 1\nauto-3d\toff\ngame-off\t100\ngame-off\t502500\n"
    );
}

#[test]
fn it_reads_back_equal() {
    let mut s = Settings::default();
    s.auto_3d = false;
    s.set_game(7, false);
    assert_eq!(parse_settings(&settings_text(&s)), s);
}

#[test]
fn a_missing_or_wrong_header_gives_the_defaults() {
    assert_eq!(parse_settings(""), Settings::default());
    assert_eq!(parse_settings("auto-3d\toff\n"), Settings::default());
    assert_eq!(
        parse_settings("flat3d-settings 2\nauto-3d\toff\n"),
        Settings::default()
    );
}

#[test]
fn reading_is_tolerant() {
    let t =
        "flat3d-settings 1\nauto-3d\tmaybe\ntunnel\toff\ngame-off\tx\ngame-off\t9\nwhat\tever\n";
    let s = parse_settings(t);
    assert!(s.auto_3d);
    // The old tunnel line is ignored, and the next write drops it.
    assert_eq!(
        settings_text(&s),
        "flat3d-settings 1\nauto-3d\ton\ngame-off\t9\n"
    );
    assert_eq!(s.off.iter().copied().collect::<Vec<_>>(), vec![9]);
}

#[test]
fn a_game_can_be_switched_off_and_on_again() {
    let mut s = Settings::default();
    s.set_game(5, false);
    assert!(!s.game_on(5));
    assert!(s.game_on(6));
    s.set_game(5, true);
    assert!(s.game_on(5));
    assert!(s.off.is_empty());
}

#[test]
fn an_old_file_with_a_tunnel_line_still_loads() {
    let old = "flat3d-settings 1\nauto-3d\toff\ntunnel\ton\ngame-off\t42\n";
    let s = parse_settings(old);
    assert!(!s.auto_3d);
    assert!(!s.game_on(42));
    assert_eq!(
        settings_text(&s),
        "flat3d-settings 1\nauto-3d\toff\ngame-off\t42\n"
    );
}
