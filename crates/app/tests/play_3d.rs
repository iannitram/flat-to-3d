//! Locked tests for #404: the per-game 3D button's state and `update`. Pure.

use flat3d_app::play_3d::{
    button, note, update, Action, Button, Mode, Msg, PlayState, Session, FLAT_QUALITY_NOTE,
};

fn state(appid: u32, mode: Mode) -> PlayState {
    PlayState {
        session: Some(Session { appid, mode }),
        error: None,
        headset_lost: false,
        awaiting: None,
    }
}

#[test]
fn a_new_state_has_no_session_and_no_error() {
    let s = PlayState::default();
    assert!(s.session.is_none() && s.error.is_none() && s.awaiting.is_none());
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

    // An outstanding start is Busy everywhere it could otherwise be played.
    let awaiting = PlayState {
        session: None,
        awaiting: Some(1),
        ..PlayState::default()
    };
    assert_eq!(button(&awaiting, 1), Button::Busy);
    assert_eq!(button(&awaiting, 2), Button::Busy);

    let mut s = state(1, Mode::OnLaptop { headset_back: true });
    s.awaiting = Some(1);
    assert_eq!(button(&s, 1), Button::Busy);
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
    assert_eq!(s.awaiting, Some(7));
    assert!(s.error.is_none());
}

#[test]
fn play_pressed_clears_an_old_error() {
    let mut s = PlayState {
        session: None,
        error: Some("old".into()),
        headset_lost: false,
        awaiting: None,
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
fn play_pressed_then_game_exited_issues_no_second_action() {
    let mut s = PlayState::default();
    assert_eq!(
        update(&mut s, Msg::PlayPressed(7)),
        Some(Action::StartLinkAndLaunch(7))
    );
    assert_eq!(update(&mut s, Msg::GameExited(7)), None);
    assert!(s.session.is_none());
    assert_eq!(s.awaiting, Some(7));

    // The start is still outstanding, so a new press issues nothing.
    assert_eq!(update(&mut s, Msg::PlayPressed(7)), None);
    assert!(s.session.is_none());

    // Its late answer stops the viewer the game no longer needs.
    assert_eq!(update(&mut s, Msg::Started(7)), Some(Action::StopViewer(7)));
    assert!(s.session.is_none());
    assert_eq!(s.awaiting, None);

    // Now a fresh play works.
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
}

#[test]
fn a_late_start_failed_after_game_exited_changes_nothing() {
    let mut s = PlayState::default();
    update(&mut s, Msg::PlayPressed(7));
    update(&mut s, Msg::GameExited(7));
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert!(s.session.is_none());
    assert!(s.error.is_none());
    assert_eq!(s.awaiting, None);
}

#[test]
fn a_reply_with_nothing_outstanding_is_ignored() {
    let mut s = state(7, Mode::Starting);
    assert_eq!(update(&mut s, Msg::Started(7)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );
    assert_eq!(s.awaiting, None);

    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
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
fn an_ignored_press_keeps_the_error() {
    let mut s = PlayState {
        session: Some(Session {
            appid: 7,
            mode: Mode::In3d,
        }),
        error: Some("old".into()),
        headset_lost: false,
        awaiting: None,
    };
    assert_eq!(update(&mut s, Msg::PlayPressed(9)), None);
    assert_eq!(s.error.as_deref(), Some("old"));
}

#[test]
fn started_moves_starting_and_resuming_into_3d() {
    for mode in [Mode::Starting, Mode::Resuming] {
        let mut s = state(7, mode);
        s.awaiting = Some(7);
        assert_eq!(update(&mut s, Msg::Started(7)), None);
        assert_eq!(s.session.unwrap().mode, Mode::In3d);
        assert_eq!(s.awaiting, None);
    }
}

#[test]
fn a_started_reply_after_the_session_moved_on_stops_the_late_viewer() {
    // An already-running session stays as it is; the answer is just consumed.
    let mut s = state(7, Mode::In3d);
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::Started(7)), None);
    assert_eq!(s.session.unwrap().mode, Mode::In3d);
    assert_eq!(s.awaiting, None);

    for mode in [
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        s.awaiting = Some(7);
        assert_eq!(update(&mut s, Msg::Started(7)), Some(Action::StopViewer(7)));
        assert_eq!(s.session, Some(Session { appid: 7, mode }));
        assert_eq!(s.awaiting, None);
    }

    let mut s = PlayState::default();
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::Started(7)), Some(Action::StopViewer(7)));
    assert!(s.session.is_none() && s.awaiting.is_none());
}

#[test]
fn start_failed_from_starting_ends_the_session_with_the_error() {
    let mut s = state(7, Mode::Starting);
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert!(s.session.is_none());
    assert_eq!(s.error.as_deref(), Some("no link"));
    assert_eq!(s.awaiting, None);
}

#[test]
fn start_failed_from_resuming_leaves_the_game_on_the_laptop() {
    let mut s = state(7, Mode::Resuming);
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop { headset_back: true }
        })
    );
    assert_eq!(s.error.as_deref(), Some("no link"));
    assert_eq!(s.awaiting, None);
}

#[test]
fn start_failed_after_the_session_moved_on_sets_the_error() {
    for mode in [
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        s.awaiting = Some(7);
        assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
        assert_eq!(s.session, Some(Session { appid: 7, mode }));
        assert_eq!(s.error.as_deref(), Some("no link"));
        assert_eq!(s.awaiting, None);
    }

    // In 3D the session is untouched; a start answer cannot arrive there.
    let mut s = state(7, Mode::In3d);
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::In3d
        })
    );
    assert!(s.error.is_none());
    assert_eq!(s.awaiting, None);
}

#[test]
fn start_failed_with_no_session_leaves_the_error_as_it_was() {
    let mut s = PlayState {
        session: None,
        error: Some("old".into()),
        headset_lost: false,
        awaiting: Some(7),
    };
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert!(s.session.is_none());
    assert_eq!(s.error.as_deref(), Some("old"));
    assert_eq!(s.awaiting, None);
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
    assert!(s.headset_lost);
}

#[test]
fn grace_ended_is_remembered_outside_3d_without_changing_the_mode() {
    for mode in [
        Mode::Starting,
        Mode::OnLaptop { headset_back: true },
        Mode::OnLaptop {
            headset_back: false,
        },
    ] {
        let mut s = state(7, mode);
        assert_eq!(update(&mut s, Msg::GraceEnded), None);
        assert_eq!(s.session.unwrap().mode, mode);
        assert!(s.headset_lost);
    }
}

#[test]
fn a_resume_whose_grace_ends_stays_busy_until_the_reply() {
    let mut s = state(7, Mode::Resuming);
    s.awaiting = Some(7);
    update(&mut s, Msg::GraceEnded);
    update(&mut s, Msg::HeadsetReturned);

    let on_laptop = Some(Session {
        appid: 7,
        mode: Mode::OnLaptop { headset_back: true },
    });
    assert_eq!(s.session, on_laptop);
    assert_eq!(button(&s, 7), Button::Busy);
    assert_eq!(update(&mut s, Msg::BackTo3dPressed(7)), None);
    assert_eq!(s.session, on_laptop);
    assert_eq!(s.awaiting, Some(7));

    assert_eq!(update(&mut s, Msg::Started(7)), Some(Action::StopViewer(7)));
    assert_eq!(s.session, on_laptop);
    assert_eq!(s.awaiting, None);
    assert_eq!(button(&s, 7), Button::BackTo3d);
}

#[test]
fn a_loss_during_resuming_ends_on_the_laptop_without_a_headset() {
    let mut s = state(7, Mode::Resuming);
    s.awaiting = Some(7);
    update(&mut s, Msg::HeadsetLost);
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
    assert!(s.headset_lost);

    // The late answer to the resume stops the viewer and leaves the session alone.
    assert_eq!(update(&mut s, Msg::Started(7)), Some(Action::StopViewer(7)));
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop {
                headset_back: false
            }
        })
    );
    assert_eq!(s.awaiting, None);
}

#[test]
fn the_headset_returning_after_a_loss_during_resuming_offers_the_way_back() {
    let mut s = state(7, Mode::Resuming);
    s.awaiting = Some(7);
    update(&mut s, Msg::HeadsetLost);
    update(&mut s, Msg::GraceEnded);
    assert_eq!(update(&mut s, Msg::Started(7)), Some(Action::StopViewer(7)));
    update(&mut s, Msg::HeadsetReturned);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop { headset_back: true }
        })
    );
    assert!(!s.headset_lost);
    assert_eq!(button(&s, 7), Button::BackTo3d);
}

#[test]
fn grace_ended_during_starting_keeps_the_mode_and_remembers_the_loss() {
    let mut s = state(7, Mode::Starting);
    assert_eq!(update(&mut s, Msg::GraceEnded), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );
    assert!(s.headset_lost);
}

#[test]
fn a_loss_with_no_session_is_remembered() {
    let mut s = PlayState::default();
    assert_eq!(update(&mut s, Msg::HeadsetLost), None);
    assert!(s.headset_lost && s.session.is_none());

    assert_eq!(update(&mut s, Msg::HeadsetReturned), None);
    assert!(!s.headset_lost);
}

#[test]
fn grace_ended_with_no_session_is_remembered_for_the_next_bring_back() {
    let mut s = PlayState::default();
    assert_eq!(update(&mut s, Msg::GraceEnded), None);
    assert!(s.headset_lost && s.session.is_none());

    update(&mut s, Msg::PlayPressed(7));
    update(&mut s, Msg::Started(7));
    assert_eq!(
        update(&mut s, Msg::BringBackPressed(7)),
        Some(Action::BringBack(7))
    );
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop {
                headset_back: false
            }
        })
    );
    assert_eq!(button(&s, 7), Button::OnLaptop);
}

#[test]
fn headset_returned_and_lost_change_only_the_headset_back_offer() {
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
    assert!(!s.headset_lost);

    let mut s = state(7, Mode::OnLaptop { headset_back: true });
    assert_eq!(update(&mut s, Msg::HeadsetLost), None);
    assert_eq!(
        s.session.unwrap().mode,
        Mode::OnLaptop {
            headset_back: false
        }
    );
    assert!(s.headset_lost);

    // In3d is covered by the replug grace, so neither message changes the mode,
    // but both are still remembered.
    for mode in [Mode::Starting, Mode::In3d, Mode::Resuming] {
        let mut s = state(7, mode);
        update(&mut s, Msg::HeadsetReturned);
        assert_eq!(s.session.unwrap().mode, mode);
        assert!(!s.headset_lost);
        update(&mut s, Msg::HeadsetLost);
        assert_eq!(s.session.unwrap().mode, mode);
        assert!(s.headset_lost);
    }
}

#[test]
fn bring_back_after_a_loss_offers_no_way_back() {
    let mut s = state(7, Mode::In3d);
    update(&mut s, Msg::HeadsetLost);
    assert_eq!(
        update(&mut s, Msg::BringBackPressed(7)),
        Some(Action::BringBack(7))
    );
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop {
                headset_back: false
            }
        })
    );
    assert_eq!(button(&s, 7), Button::OnLaptop);
}

#[test]
fn bring_back_after_the_headset_returns_offers_the_way_back() {
    let mut s = state(7, Mode::In3d);
    update(&mut s, Msg::HeadsetLost);
    update(&mut s, Msg::HeadsetReturned);
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
    assert_eq!(button(&s, 7), Button::BackTo3d);
}

#[test]
fn start_failed_from_resuming_with_the_headset_lost_stays_on_the_laptop() {
    let mut s = state(7, Mode::Resuming);
    s.awaiting = Some(7);
    update(&mut s, Msg::HeadsetLost);
    assert_eq!(update(&mut s, Msg::StartFailed(7, "no link".into())), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::OnLaptop {
                headset_back: false
            }
        })
    );
    assert_eq!(button(&s, 7), Button::OnLaptop);
    assert_eq!(s.error.as_deref(), Some("no link"));
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
    assert_eq!(s.awaiting, Some(7));

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
        headset_lost: false,
        awaiting: None,
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
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::Started(9)), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );
    assert_eq!(s.awaiting, Some(7));

    let mut s = state(7, Mode::Starting);
    s.awaiting = Some(7);
    assert_eq!(update(&mut s, Msg::StartFailed(9, "no link".into())), None);
    assert_eq!(
        s.session,
        Some(Session {
            appid: 7,
            mode: Mode::Starting
        })
    );
    assert!(s.error.is_none());
    assert_eq!(s.awaiting, Some(7));
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
