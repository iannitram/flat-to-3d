use crate::spec::{Layout, ViewerSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn eye_rects(frame_w: u32, frame_h: u32, spec: &ViewerSpec) -> [Rect; 2] {
    if frame_w == 0 || frame_h == 0 {
        return [Rect::default(); 2];
    }
    let (mut left, mut right) = match spec.layout {
        Layout::Sbs | Layout::SbsRev => {
            let half = frame_w / 2;
            (
                Rect {
                    x: 0,
                    y: 0,
                    w: half,
                    h: frame_h,
                },
                Rect {
                    x: half,
                    y: 0,
                    w: half,
                    h: frame_h,
                },
            )
        }
        Layout::Tab | Layout::TabRev => {
            let half = frame_h / 2;
            (
                Rect {
                    x: 0,
                    y: 0,
                    w: frame_w,
                    h: half,
                },
                Rect {
                    x: 0,
                    y: half,
                    w: frame_w,
                    h: half,
                },
            )
        }
    };
    if matches!(spec.layout, Layout::SbsRev | Layout::TabRev) {
        std::mem::swap(&mut left, &mut right);
    }
    if spec.swap_eyes {
        std::mem::swap(&mut left, &mut right);
    }
    if let Some(aspect) = spec.crop_aspect.filter(|a| *a > 0.0) {
        crop(&mut left, aspect);
        crop(&mut right, aspect);
    }
    [left, right]
}

fn crop(rect: &mut Rect, aspect: f32) {
    let current = rect.w as f32 / rect.h as f32;
    if current > aspect {
        let width = (rect.h as f32 * aspect).round().clamp(0.0, rect.w as f32) as u32;
        rect.x += (rect.w - width) / 2;
        rect.w = width;
    } else {
        let height = (rect.w as f32 / aspect).round().clamp(0.0, rect.h as f32) as u32;
        rect.y += (rect.h - height) / 2;
        rect.h = height;
    }
}

pub fn unsqueeze(mut rect: Rect, layout: Layout) -> Rect {
    match layout {
        Layout::Sbs | Layout::SbsRev => rect.w = rect.w.saturating_mul(2),
        Layout::Tab | Layout::TabRev => rect.h = rect.h.saturating_mul(2),
    }
    rect
}

pub fn quad_size(rect: Rect, width_m: f32) -> (f32, f32) {
    if rect.w == 0 {
        (width_m, 0.0)
    } else {
        (width_m, width_m * rect.h as f32 / rect.w as f32)
    }
}
