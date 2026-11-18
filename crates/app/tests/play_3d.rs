//! Locked tests for #404: the per-game 3D button's state and `update`. Pure.

use flat3d_app::play_3d::{
    button, note, update, Action, Button, Mode, Msg, PlayState, Session, FLAT_QUALITY_NOTE,
};

fn state(appid: u32, mode: Mode) -> PlayState {
    PlayState {
        session: Some(Session { appid, mode }),
        error: None,
    }
}

#[test]
fn a_new_state_has_no_session_and_no_error() {
    let s = PlayState::default();
    assert!(s.session.is_none() && s.error.is_none());
    assert_eq!(button(&s, 1), Button::PlayIn3d);
    assert_eq!(note(&s, 1), None);
}

#[test]
fn every_button_value() {
    assert_eq!(button(&PlayState::default(), 1), Button::PlayIn3d);
    assert_eq!(button(&state(1, Mode::Starting), 1), Button::Busy);
    assert_eq!(button(&state(1, Mode::Resuming), 1), Button::Busy);
    assert_eq!(button(&state(1, Mode::In3d), 1), Button::BringBackToLaptop);
    assert_eq!(
        button(&state(1, Mode::OnLaptop { headset_back: true }), 1),
        Button::BackTo3d
    );
    assert_eq!(
        button(
            &state(
                1,
                Mode::OnLaptop {
                    headset_back: false
                }
            ),
            1
        ),
        Button::OnLaptop
    );
    assert_eq!(button(&state(1, Mode::In3d), 2), Button::OtherGameRunning);
}

#[test]
fn play_pressed_starts_a_session_and_asks_for_the_link() {
    let mut s = PlayState::default();
    assert_eq!(
        update(&mut s, Msg::PlayPressed(7)),
        Some(Action::StartLinkAndLaunch(7))
    );
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );
    assert!(s.error.is_none());
}

#[test]
fn play_pressed_clears_an_old_error() {
    let mut s = PlayState {
        session: None,
        error: Some("old".into()),
    };
    update(&mut s, Msg::PlayPressed(7));
    assert!(s.error.is_none());
}

#[test]
fn a_second_play_pressed_during_a_session_does_nothing() {
    let mut s = state(7, Mode::In3d);
    assert_eq!(update(&mut s, Msg::PlayPressed(7)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::In3d
        })
    );

    // Even for another game: only one 3D session at a time.
    let mut s = state(7, Mode::In3d);
    assert_eq!(update(&mut s, Msg::PlayPressed(9)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::In3d
        })
    );
}

#[test]
fn an_ignored_press_keeps_the_error() {
    let mut s = PlayState {
        session: Some(Session {
            appid: 7,
            mode: Mode::In3d,
        }),
        error: Some("old".into()),
    };
    assert_eq!(update(&mut s, Msg::PlayPressed(9)), None);
    assert_eq!(s.error.as_deref(), Some("old"));
}

#[test]
fn started_moves_starting_and_resuming_into_3d() {
    let mut s = state(7, Mode::Starting);
    assert_eq!(update(&mut s, Msg::Started(7)), None);
    assert_eq!(s.session.unwrap().mode, Mode::In3d);

    let mut s = state(7, Mode::Resuming);
    assert_eq!(update(&mut s, Msg::Started(7)), None);
    assert_eq!(s.session.unwrap().mode, Mode::In3d);
}

#[test]
fn started_elsewhere_changes_nothing() {
    for mode in [
        Mode::In3d,
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::Started(7)), None);
        assert_eq!(s.session.unwrap().mode, mode);
    }
}

#[test]
fn start_failed_from_starting_ends_the_session_with_the_error() {
    let mut s = state(7, Mode::Starting);
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert!(s.session.is_none());
    assert_eq!(s.error.as_deref(), Some("no link"));
}

#[test]
fn start_failed_from_resuming_leaves_the_game_on_the_laptop() {
    let mut s = state(7, Mode::Resuming);
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop { headset_back: true }
        })
    );
    assert_eq!(s.error.as_deref(), Some("no link"));
}

#[test]
fn start_failed_elsewhere_changes_nothing() {
    for mode in [
        Mode::In3d,
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
        assert_eq!(s.session, Some(Session { appid: 7, mode }));
        assert!(s.error.is_none());
    }

    let mut s = PlayState::default();
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert!(s.session.is_none() && s.error.is_none());
}

#[test]
fn bring_back_pressed_keeps_the_headset_on_offer() {
    let mut s = state(7, Mode::In3d);
    assert_eq!(
        update(&mut s, Msg::BringBackPressed(7)),
        Some(Action::BringBack(7))
    );
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop { headset_back: true }
        })
    );
}

#[test]
fn bring_back_pressed_is_ignored_outside_3d() {
    for mode in [
        Mode::Starting,
        Mode::Resuming,
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::BringBackPressed(7)), None);
        assert_eq!(s.session, Some(Session { appid: 7, mode }));
    }
}

#[test]
fn grace_ended_moves_in_3d_to_the_laptop_without_a_headset() {
    let mut s = state(7, Mode::In3d);
    assert_eq!(update(&mut s, Msg::GraceEnded), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop {
                headset_back: false
            }
        })
    );
}

#[test]
fn grace_ended_changes_only_in_3d() {
    for mode in [
        Mode::Starting,
        Mode::Resuming,
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::GraceEnded), None);
        assert_eq!(s.session.unwrap().mode, mode);
    }
}

#[test]
fn headset_returned_and_lost_only_change_an_on_laptop_session() {
    let mut s = state(
        7,
        Mode::OnLaptop {
            headset_back: false,
        },
    );
    assert_eq!(update(&mut s, Msg::HeadsetReturned), None);
    assert_eq!(
        s.session.unwrap().mode,
        Mode::OnLaptop { headset_back: true }
    );

    let mut s = state(7, Mode::OnLaptop { headset_back: true });
    assert_eq!(update(&mut s, Msg::HeadsetLost), None);
    assert_eq!(
        s.session.unwrap().mode,
        Mode::OnLaptop {
            headset_back: false
        }
    );

    // In3d is covered by the replug grace, so neither message changes it.
    for mode in [Mode::Starting, Mode::In3d, Mode::Resuming] {
        let mut s = state(7, mode);
        update(&mut s, Msg::HeadsetReturned);
        assert_eq!(s.session.unwrap().mode, mode);
        update(&mut s, Msg::HeadsetLost);
        assert_eq!(s.session.unwrap().mode, mode);
    }
}

#[test]
fn back_to_3d_pressed_resumes_only_with_the_headset_back() {
    let mut s = state(7, Mode::OnLaptop { headset_back: true });
    assert_eq!(
        update(&mut s, Msg::BackTo3dPressed(7)),
        Some(Action::ResumeIn3d(7))
    );
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Resuming
        })
    );

    for mode in [
        Mode::Starting,
        Mode::In3d,
        Mode::Resuming,
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::BackTo3dPressed(7)), None);
        assert_eq!(s.session, Some(Session { appid: 7, mode }));
    }
}

#[test]
fn back_to_3d_pressed_clears_an_old_error() {
    let mut s = PlayState {
        session: Some(Session {
            appid: 7,
            mode: Mode::OnLaptop { headset_back: true },
        }),
        error: Some("old".into()),
    };
    update(&mut s, Msg::BackTo3dPressed(7));
    assert!(s.error.is_none());
}

#[test]
fn game_exited_ends_any_session() {
    for mode in [
        Mode::Starting,
        Mode::In3d,
        Mode::Resuming,
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::GameExited(7)), None);
        assert!(s.session.is_none());
    }

    let mut s = PlayState::default();
    assert_eq!(update(&mut s, Msg::GameExited(7)), None);
    assert!(s.session.is_none());
}

#[test]
fn a_message_for_another_appid_is_ignored() {
    let mut s = state(7, Mode::In3d);
    assert_eq!(update(&mut s, Msg::BringBackPressed(9)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::In3d
        })
    );
    assert_eq!(update(&mut s, Msg::GameExited(9)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::In3d
        })
    );

    let mut s = state(7, Mode::OnLaptop { headset_back: true });
    assert_eq!(update(&mut s, Msg::BackTo3dPressed(9)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop { headset_back: true }
        })
    );

    let mut s = state(7, Mode::Starting);
    assert_eq!(update(&mut s, Msg::Started(9)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );

    let mut s = state(7, Mode::Starting);
    assert_eq!(update(&mut s, Msg::StartFailed(9, "no link".into())), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );
    assert!(s.error.is_none());
}

#[test]
fn the_note_appears_only_on_the_laptop() {
    assert_eq!(
        note(&state(7, Mode::OnLaptop { headset_back: true }), 7),
        Some(FLAT_QUALITY_NOTE)
    );
    assert_eq!(
        note(
            &state(
                7,
                Mode::OnLaptop {
                    headset_back: false
                }
            ),
            7
        ),
        Some(FLAT_QUALITY_NOTE)
    );

    for mode in [Mode::Starting, Mode::In3d, Mode::Resuming] {
        assert_eq!(note(&state(7, mode), 7), None);
    }

    // Another game gets no note, even while one sits on the laptop.
    let on_laptop = state(7, Mode::OnLaptop { headset_back: true });
    assert_eq!(note(&on_laptop, 8), None);
    assert_eq!(note(&PlayState::default(), 7), None);
}
