use std::time::{Duration, Instant};

use crate::{
    frame::{Frame, FrameData},
    layout::eye_rects,
    source::FrameSource,
    spec::ViewerSpec,
};

pub const LEFT_MARK: [u8; 4] = [0, 255, 0, 255];
pub const RIGHT_MARK: [u8; 4] = [255, 0, 255, 255];

pub struct TestPattern {
    w: u32,
    h: u32,
    spec: ViewerSpec,
    fps: u32,
    seq: u64,
    next_at: Instant,
}

impl TestPattern {
    pub fn new(w: u32, h: u32, spec: ViewerSpec, fps: u32) -> Self {
        Self {
            w,
            h,
            spec,
            fps,
            seq: 0,
            next_at: Instant::now(),
        }
    }
}

impl FrameSource for TestPattern {
    fn next(&mut self, timeout: Duration) -> Result<Option<Frame>, String> {
        if self.fps > 0 {
            let now = Instant::now();
            if self.next_at > now {
                let until = self.next_at.duration_since(now);
                if until > timeout {
                    std::thread::sleep(timeout);
                    return Ok(None);
                }
                std::thread::sleep(until);
            }
        }

        let stride = self.w.saturating_mul(4);
        let len = (stride as usize)
            .checked_mul(self.h as usize)
            .ok_or_else(|| "test pattern dimensions are too large".to_string())?;
        let mut bytes = vec![64; len];
        for px in bytes.chunks_exact_mut(4) {
            px.copy_from_slice(&[64, 64, 64, 255]);
        }

        {
            let mut canvas = Canvas {
                bytes: &mut bytes,
                stride,
                width: self.w,
                height: self.h,
            };
            let rects = eye_rects(self.w, self.h, &self.spec);
            for (index, rect) in rects.iter().copied().enumerate() {
                if rect.w == 0 || rect.h == 0 {
                    continue;
                }
                let row = (self.seq % u64::from(rect.h)) as u32;
                canvas.draw(rect, rect.x, rect.y + row, rect.w, 2, [255; 4]);
                let bar_w = (rect.w / 32).max(1);
                let offset = rect.w / 64;
                let center = rect.x + rect.w / 2;
                let bar_x = if index == 0 {
                    center.saturating_add(offset)
                } else {
                    center.saturating_sub(offset)
                };
                canvas.draw(rect, bar_x, rect.y, bar_w, rect.h, [255; 4]);
                let mark = if index == 0 { LEFT_MARK } else { RIGHT_MARK };
                canvas.draw(
                    rect,
                    rect.x.saturating_add(2),
                    rect.y.saturating_add(2),
                    8,
                    8,
                    mark,
                );
            }
        }

        let frame = Frame {
            width: self.w,
            height: self.h,
            fourcc: u32::from_le_bytes(*b"XR24"),
            seq: self.seq,
            data: FrameData::Mem { stride, bytes },
        };
        self.seq += 1;
        if self.fps > 0 {
            self.next_at += Duration::from_secs(1) / self.fps;
        }
        Ok(Some(frame))
    }
}

struct Canvas<'a> {
    bytes: &'a mut [u8],
    stride: u32,
    width: u32,
    height: u32,
}

impl Canvas<'_> {
    fn draw(&mut self, clip: crate::layout::Rect, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        let x0 = x.max(clip.x);
        let y0 = y.max(clip.y);
        let x1 = x
            .saturating_add(w)
            .min(clip.x.saturating_add(clip.w))
            .min(self.width);
        let y1 = y
            .saturating_add(h)
            .min(clip.y.saturating_add(clip.h))
            .min(self.height);
        for py in y0..y1 {
            for px in x0..x1 {
                let offset = py as usize * self.stride as usize + px as usize * 4;
                if let Some(pixel) = self.bytes.get_mut(offset..offset + 4) {
                    pixel.copy_from_slice(&color);
                }
            }
        }
    }
}
