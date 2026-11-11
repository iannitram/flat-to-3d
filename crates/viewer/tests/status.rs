use flat3d_viewer::status::{State, Status};

#[test]
fn emits_exact_line_for_each_state() {
    let cases = [
        (
            State::WaitingHeadset,
            "{\"state\":\"waiting-headset\",\"fps\":0.0,\"w\":0,\"h\":0}",
        ),
        (
            State::WaitingSource,
            "{\"state\":\"waiting-source\",\"fps\":0.0,\"w\":0,\"h\":0}",
        ),
        (
            State::Showing,
            "{\"state\":\"showing\",\"fps\":59.9,\"w\":1920,\"h\":1080}",
        ),
        (
            State::Error,
            "{\"state\":\"error\",\"fps\":0.0,\"w\":0,\"h\":0}",
        ),
    ];
    for (state, expected) in cases {
        let status = Status {
            state,
            fps: if state == State::Showing { 59.9 } else { 0.0 },
            w: if state == State::Showing { 1920 } else { 0 },
            h: if state == State::Showing { 1080 } else { 0 },
        };
        assert_eq!(status.to_line(), expected);
    }
}

#[test]
fn formats_one_decimal_and_normalizes_non_finite_fps() {
    let status = Status {
        state: State::Showing,
        fps: 59.94,
        w: 1,
        h: 2,
    };
    assert_eq!(
        status.to_line(),
        "{\"state\":\"showing\",\"fps\":59.9,\"w\":1,\"h\":2}"
    );
    let nan = Status {
        fps: f32::NAN,
        ..status
    };
    assert_eq!(
        nan.to_line(),
        "{\"state\":\"showing\",\"fps\":0.0,\"w\":1,\"h\":2}"
    );
}

#[test]
fn parses_fixed_shape_with_whitespace() {
    let line = " { \"state\" : \"showing\", \"fps\" : 59.9, \"w\" : 1920, \"h\" : 1080 } ";
    let parsed = Status::from_line(line).unwrap();
    assert_eq!(
        parsed,
        Status {
            state: State::Showing,
            fps: 59.9,
            w: 1920,
            h: 1080
        }
    );
    for fps in [59.9, 0.0] {
        let original = Status {
            state: State::Showing,
            fps,
            w: 1920,
            h: 1080,
        };
        assert_eq!(Status::from_line(&original.to_line()).unwrap(), original);
    }
}

#[test]
fn rejects_unknown_and_malformed_lines() {
    assert!(Status::from_line("{\"state\":\"bogus\",\"fps\":0.0,\"w\":0,\"h\":0}").is_err());
    assert!(Status::from_line("").is_err());
    assert!(Status::from_line("{\"fps\":0.0,\"state\":\"showing\",\"w\":0,\"h\":0}").is_err());
}
