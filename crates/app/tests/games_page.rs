//! Locked tests for A2: the Games page's state and update. Pure.

use flat3d_app::games_page::{update, Action, GamesState, Msg};
use flat3d_inject::rows::GameRow;

fn row(appid: u32, on: bool) -> GameRow {
    GameRow {
        appid,
        name: format!("G{appid}"),
        status: "works-now".into(),
        status_label: "Works now".into(),
        status_colour: "#2e9e5b".into(),
        switch_on: on,
        plays_3d: on,
        flat_reason: None,
    }
}

#[test]
fn a_new_page_shows_all_games_and_nothing_else() {
    let s = GamesState::default();
    assert!(s.rows.is_empty() && s.error.is_none() && s.note.is_none());
    assert_eq!(s.filter, "all");
}

#[test]
fn rows_replace_the_old_ones_and_clear_the_error() {
    let mut s = GamesState {
        error: Some("x".into()),
        ..GamesState::default()
    };
    assert_eq!(
        update(&mut s, Msg::Rows(Ok(vec![row(1, true), row(2, true)]))),
        None
    );
    assert_eq!(s.rows.len(), 2);
    assert!(s.error.is_none());
}

#[test]
fn a_failed_load_keeps_the_rows() {
    let mut s = GamesState {
        rows: vec![row(1, true)],
        ..GamesState::default()
    };
    assert_eq!(update(&mut s, Msg::Rows(Err("bad".into()))), None);
    assert_eq!(s.rows.len(), 1);
    assert_eq!(s.error.as_deref(), Some("bad"));
}

#[test]
fn the_filter_asks_for_a_reload() {
    let mut s = GamesState::default();
    assert_eq!(
        update(&mut s, Msg::Filter("likely".into())),
        Some(Action::LoadRows {
            filter: "likely".into()
        })
    );
    assert_eq!(s.filter, "likely");
}

#[test]
fn a_switch_shows_at_once_and_is_asked_for() {
    let mut s = GamesState {
        rows: vec![row(1, true), row(2, true)],
        ..GamesState::default()
    };
    assert_eq!(
        update(
            &mut s,
            Msg::Switch {
                appid: 2,
                on: false
            }
        ),
        Some(Action::SetSwitch {
            appid: 2,
            on: false
        })
    );
    assert!(s.rows[0].switch_on);
    assert!(!s.rows[1].switch_on);
}

#[test]
fn a_switch_for_an_unknown_game_is_still_asked_for() {
    let mut s = GamesState::default();
    assert_eq!(
        update(&mut s, Msg::Switch { appid: 9, on: true }),
        Some(Action::SetSwitch { appid: 9, on: true })
    );
}

#[test]
fn undo_clears_old_messages_and_asks() {
    let mut s = GamesState {
        note: Some("n".into()),
        error: Some("e".into()),
        ..GamesState::default()
    };
    assert_eq!(update(&mut s, Msg::UndoPressed(7)), Some(Action::Undo(7)));
    assert!(s.note.is_none() && s.error.is_none());
}

#[test]
fn undo_done_shows_the_summary_and_reloads_with_the_filter() {
    let mut s = GamesState {
        filter: "likely".into(),
        ..GamesState::default()
    };
    let got = update(&mut s, Msg::UndoDone(Ok("Put 2 files back.".into())));
    assert_eq!(
        got,
        Some(Action::LoadRows {
            filter: "likely".into()
        })
    );
    assert_eq!(s.note.as_deref(), Some("Put 2 files back."));
}

#[test]
fn undo_failed_shows_the_error() {
    let mut s = GamesState::default();
    assert_eq!(update(&mut s, Msg::UndoDone(Err("no record".into()))), None);
    assert_eq!(s.error.as_deref(), Some("no record"));
}
