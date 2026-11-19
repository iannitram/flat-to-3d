use std::time::Duration;

use flat3d_viewer::{
    frame::FrameData,
    layout::{eye_rects, quad_size, screen_rect, unsqueeze, Rect},
    source::{
        test_pattern::{TestPattern, LEFT_MARK, RIGHT_MARK},
        FrameSource,
    },
    spec::{Layout, Source, ViewerSpec},
};

fn spec(layout: Layout) -> ViewerSpec {
    ViewerSpec {
        layout,
        ..ViewerSpec::default()
    }
}

fn tuples(rects: [Rect; 2]) -> [(u32, u32, u32, u32); 2] {
    rects.map(|r| (r.x, r.y, r.w, r.h))
}

#[test]
fn splits_and_reverses_eyes() {
    assert_eq!(
        tuples(eye_rects(1920, 1080, &spec(Layout::Sbs))),
        [(0, 0, 960, 1080), (960, 0, 960, 1080)]
    );
    assert_eq!(
        tuples(eye_rects(1920, 1080, &spec(Layout::Tab))),
        [(0, 0, 1920, 540), (0, 540, 1920, 540)]
    );
    assert_eq!(
        eye_rects(1920, 1080, &spec(Layout::SbsRev)),
        eye_rects(1920, 1080, &spec(Layout::Sbs)).map(|r| Rect {
            x: if r.x == 0 { 960 } else { 0 },
            ..r
        })
    );
    assert_eq!(
        tuples(eye_rects(1920, 1080, &spec(Layout::TabRev))),
        [(0, 540, 1920, 540), (0, 0, 1920, 540)]
    );
    let mut swapped = spec(Layout::Sbs);
    swapped.swap_eyes = true;
    assert_eq!(
        eye_rects(1920, 1080, &swapped),
        eye_rects(1920, 1080, &spec(Layout::SbsRev))
    );
    let mut swapped_rev = spec(Layout::SbsRev);
    swapped_rev.swap_eyes = true;
    assert_eq!(
        eye_rects(1920, 1080, &swapped_rev),
        eye_rects(1920, 1080, &spec(Layout::Sbs))
    );
    assert_eq!(
        tuples(eye_rects(1921, 1080, &spec(Layout::Sbs))),
        [(0, 0, 960, 1080), (960, 0, 960, 1080)]
    );
}

#[test]
fn crops_aspect_and_safely_handles_empty_frames() {
    let mut wide = spec(Layout::Sbs);
    wide.crop_aspect = Some(16.0 / 9.0);
    assert_eq!(
        tuples(eye_rects(10240, 1440, &wide)),
        [(1280, 0, 2560, 1440), (6400, 0, 2560, 1440)]
    );
    wide.swap_eyes = true;
    assert_eq!(
        tuples(eye_rects(10240, 1440, &wide)),
        [(6400, 0, 2560, 1440), (1280, 0, 2560, 1440)]
    );
    assert_eq!(
        tuples(eye_rects(
            1920,
            1080,
            &ViewerSpec {
                layout: Layout::Sbs,
                crop_aspect: Some(16.0 / 9.0),
                ..ViewerSpec::default()
            }
        )),
        [(0, 270, 960, 540), (960, 270, 960, 540)]
    );
    assert_eq!(
        eye_rects(1920, 1080, &spec(Layout::Sbs)),
        eye_rects(
            1920,
            1080,
            &ViewerSpec {
                crop_aspect: Some(0.0),
                ..spec(Layout::Sbs)
            }
        )
    );
    assert_eq!(eye_rects(0, 0, &spec(Layout::Sbs)), [Rect::default(); 2]);
}

#[test]
fn sizes_quads_and_restores_half_layout() {
    assert_eq!(
        quad_size(
            Rect {
                x: 0,
                y: 0,
                w: 2560,
                h: 1440
            },
            3.0
        ),
        (3.0, 1.6875)
    );
    assert_eq!(
        quad_size(
            Rect {
                w: 0,
                ..Rect::default()
            },
            3.0
        ),
        (3.0, 0.0)
    );
    assert_eq!(
        unsqueeze(
            Rect {
                x: 0,
                y: 0,
                w: 960,
                h: 1080
            },
            Layout::Sbs
        ),
        Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080
        }
    );
    assert_eq!(
        unsqueeze(
            Rect {
                x: 0,
                y: 0,
                w: 1920,
                h: 540
            },
            Layout::Tab
        ),
        Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080
        }
    );
}

fn pixel(bytes: &[u8], stride: u32, x: u32, y: u32) -> [u8; 4] {
    let i = y as usize * stride as usize + x as usize * 4;
    bytes[i..i + 4].try_into().unwrap()
}

#[test]
fn test_pattern_draws_stereo_marks_and_advances_sequence() {
    let mut pattern = TestPattern::new(64, 32, spec(Layout::Sbs), 0);
    for expected_seq in 0..2 {
        let frame = pattern.next(Duration::ZERO).unwrap().unwrap();
        assert_eq!(
            (frame.width, frame.height, frame.fourcc, frame.seq),
            (64, 32, u32::from_le_bytes(*b"XR24"), expected_seq)
        );
        let FrameData::Mem { stride, bytes } = frame.data else {
            panic!("expected memory frame")
        };
        assert_eq!((stride, bytes.len()), (256, 8192));
        assert_eq!(pixel(&bytes, stride, 2, 2), LEFT_MARK);
        assert_eq!(pixel(&bytes, stride, 34, 2), RIGHT_MARK);
    }
    let mut reversed = TestPattern::new(64, 32, spec(Layout::SbsRev), 0);
    let frame = reversed.next(Duration::ZERO).unwrap().unwrap();
    let FrameData::Mem { stride, bytes } = frame.data else {
        panic!("expected memory frame")
    };
    assert_eq!(pixel(&bytes, stride, 2, 2), RIGHT_MARK);
    assert_eq!(pixel(&bytes, stride, 34, 2), LEFT_MARK);
}

#[test]
fn test_pattern_handles_tiny_frames_and_pacing() {
    for (w, h) in [(1, 1), (0, 0)] {
        let mut pattern = TestPattern::new(w, h, spec(Layout::Sbs), 0);
        assert!(pattern.next(Duration::ZERO).unwrap().is_some());
    }
    let mut paced = TestPattern::new(64, 32, spec(Layout::Sbs), 1);
    assert_eq!(
        paced.next(Duration::from_millis(10)).unwrap().unwrap().seq,
        0
    );
    let start = std::time::Instant::now();
    assert!(paced.next(Duration::from_millis(10)).unwrap().is_none());
    assert!(start.elapsed() < Duration::from_millis(200));
    assert_eq!(paced.next(Duration::from_secs(2)).unwrap().unwrap().seq, 1);
}

#[test]
fn eye_shift_trims_opposite_edges_of_each_eye() {
    let mut shifted = spec(Layout::Sbs);
    shifted.eye_shift = 0.25;
    let [left, right] = eye_rects(1920, 1200, &shifted);
    assert_eq!((left.x, left.w), (240, 720));
    assert_eq!((right.x, right.w), (960, 720));
}

#[test]
fn swapping_eyes_keeps_the_eye_shift_direction() {
    for (plain, reversed) in [(Layout::Sbs, Layout::SbsRev), (Layout::Tab, Layout::TabRev)] {
        let mut swapped = spec(plain);
        swapped.swap_eyes = true;
        swapped.eye_shift = 0.25;
        let mut expected = spec(reversed);
        expected.eye_shift = 0.25;
        assert_eq!(
            eye_rects(1920, 1080, &swapped),
            eye_rects(1920, 1080, &expected)
        );
    }
    let mut swapped = spec(Layout::Sbs);
    swapped.swap_eyes = true;
    swapped.eye_shift = 0.25;
    assert_eq!(
        tuples(eye_rects(1920, 1080, &swapped)),
        [(1200, 0, 720, 1080), (0, 0, 720, 1080)]
    );
}

#[test]
fn full_width_eyes_keep_their_shape_on_the_screen() {
    let eye = Rect {
        x: 0,
        y: 0,
        w: 1440,
        h: 1440,
    };
    for (layout, stretched) in [
        (Layout::Sbs, (2880, 1440)),
        (Layout::SbsRev, (2880, 1440)),
        (Layout::Tab, (1440, 2880)),
        (Layout::TabRev, (1440, 2880)),
    ] {
        let mut viewer = spec(layout);
        viewer.source = Source::Gamescope;
        let rect = screen_rect(eye, &viewer);
        assert_eq!((rect.w, rect.h), stretched);
        viewer.full_width_eyes = true;
        assert_eq!(screen_rect(eye, &viewer), eye);
    }
}
