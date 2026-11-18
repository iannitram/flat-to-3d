//! The geometry that turns a window point into a game pixel for an eye view.
//!
//! An eye view shows one crop of a stereo stream in a window. The crop keeps its aspect inside the
//! window with letterbox bars, and a point in the fitted picture maps back to the game it came
//! from. This is pure geometry: no I/O and no game-specific constants.

use std::fmt;

/// A rectangle in pixels, with its top-left corner at `(x, y)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Why an [`EyeView`] could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeViewError {
    /// The stream, the crop or the game had a zero width or height.
    ZeroSize,
    /// The crop did not fit inside the stream.
    CropOutsideStream,
}

impl fmt::Display for EyeViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EyeViewError::ZeroSize => "an eye-view size is zero",
            EyeViewError::CropOutsideStream => "the crop is outside the stream",
        })
    }
}

impl std::error::Error for EyeViewError {}

/// One eye's crop of a stereo stream and the game it is shown from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EyeView {
    pub stream_w: u32,
    pub stream_h: u32,
    pub crop: Rect,
    pub game_w: u32,
    pub game_h: u32,
}

impl EyeView {
    /// Build a view, rejecting zero sizes and a crop outside the stream.
    pub fn new(
        stream_w: u32,
        stream_h: u32,
        crop: Rect,
        game_w: u32,
        game_h: u32,
    ) -> Result<EyeView, EyeViewError> {
        if stream_w == 0
            || stream_h == 0
            || crop.w == 0
            || crop.h == 0
            || game_w == 0
            || game_h == 0
        {
            return Err(EyeViewError::ZeroSize);
        }
        if crop.x > stream_w
            || crop.y > stream_h
            || crop.w > stream_w - crop.x
            || crop.h > stream_h - crop.y
        {
            return Err(EyeViewError::CropOutsideStream);
        }
        Ok(EyeView {
            stream_w,
            stream_h,
            crop,
            game_w,
            game_h,
        })
    }

    /// The crop rectangle in stream pixels.
    pub fn crop(&self) -> Rect {
        self.crop
    }

    /// The largest centred rectangle with the crop's aspect that fits the window.
    ///
    /// Returns `None` when either window side is zero.
    pub fn fit(&self, window_w: u32, window_h: u32) -> Option<Rect> {
        if window_w == 0 || window_h == 0 {
            return None;
        }
        let window_w = u64::from(window_w);
        let window_h = u64::from(window_h);
        let crop_w = u64::from(self.crop.w);
        let crop_h = u64::from(self.crop.h);

        let (w, h) = if crop_w * window_h <= window_w * crop_h {
            // The window is wider than the crop, so its height limits the picture.
            (window_h * crop_w / crop_h, window_h)
        } else {
            // The window is taller than the crop, so its width limits the picture.
            (window_w, window_w * crop_h / crop_w)
        };
        let w = w.clamp(1, window_w);
        let h = h.clamp(1, window_h);
        let w = w as u32;
        let h = h as u32;
        Some(Rect {
            x: (window_w as u32 - w) / 2,
            y: (window_h as u32 - h) / 2,
            w,
            h,
        })
    }

    /// Map a window point to a game pixel.
    ///
    /// Returns `None` outside the fitted picture (in the letterbox bars). The point becomes a
    /// crop-relative point, then a game pixel, clamped to the last valid pixel.
    pub fn window_to_game(
        &self,
        window_w: u32,
        window_h: u32,
        px: u32,
        py: u32,
    ) -> Option<(u32, u32)> {
        let fitted = self.fit(window_w, window_h)?;
        if px < fitted.x || py < fitted.y {
            return None;
        }
        let rx = px - fitted.x;
        let ry = py - fitted.y;
        if rx >= fitted.w || ry >= fitted.h {
            return None;
        }

        let crop_x = u64::from(rx) * u64::from(self.crop.w) / u64::from(fitted.w);
        let crop_y = u64::from(ry) * u64::from(self.crop.h) / u64::from(fitted.h);
        let game_x = crop_x * u64::from(self.game_w) / u64::from(self.crop.w);
        let game_y = crop_y * u64::from(self.game_h) / u64::from(self.crop.h);
        Some((
            game_x.min(u64::from(self.game_w) - 1) as u32,
            game_y.min(u64::from(self.game_h) - 1) as u32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route() -> EyeView {
        EyeView::new(
            2880,
            1440,
            Rect {
                x: 0,
                y: 360,
                w: 1440,
                h: 720,
            },
            2880,
            1440,
        )
        .unwrap()
    }

    fn second_route() -> EyeView {
        EyeView::new(
            1920,
            1080,
            Rect {
                x: 320,
                y: 0,
                w: 1280,
                h: 720,
            },
            2560,
            1440,
        )
        .unwrap()
    }

    #[test]
    fn crop_is_in_stream_pixels() {
        assert_eq!(
            route().crop(),
            Rect {
                x: 0,
                y: 360,
                w: 1440,
                h: 720
            }
        );
    }

    #[test]
    fn fit_keeps_the_crop_aspect() {
        // Exact aspect fills the window.
        assert_eq!(
            route().fit(1440, 720),
            Some(Rect {
                x: 0,
                y: 0,
                w: 1440,
                h: 720
            })
        );
        // A wider window gets bars on the left and right.
        assert_eq!(
            route().fit(2000, 720),
            Some(Rect {
                x: 280,
                y: 0,
                w: 1440,
                h: 720
            })
        );
        // A taller window gets bars at the top and bottom.
        assert_eq!(
            route().fit(1440, 1000),
            Some(Rect {
                x: 0,
                y: 140,
                w: 1440,
                h: 720
            })
        );
        // A 1x1 window still gets a picture.
        assert_eq!(
            route().fit(1, 1),
            Some(Rect {
                x: 0,
                y: 0,
                w: 1,
                h: 1
            })
        );
        // An empty window has no picture.
        assert_eq!(route().fit(0, 720), None);
        assert_eq!(route().fit(1440, 0), None);
        assert_eq!(route().fit(0, 0), None);
    }

    #[test]
    fn corners_and_centre_map_to_the_game() {
        let view = route();
        assert_eq!(view.window_to_game(1440, 720, 0, 0), Some((0, 0)));
        assert_eq!(view.window_to_game(1440, 720, 1439, 0), Some((2878, 0)));
        assert_eq!(view.window_to_game(1440, 720, 0, 719), Some((0, 1438)));
        assert_eq!(
            view.window_to_game(1440, 720, 1439, 719),
            Some((2878, 1438))
        );
        assert_eq!(view.window_to_game(1440, 720, 720, 360), Some((1440, 720)));
        assert_eq!(view.window_to_game(1, 1, 0, 0), Some((0, 0)));
        assert_eq!(view.window_to_game(1, 1, 1, 0), None);
    }

    #[test]
    fn points_in_letterbox_bars_are_outside() {
        let view = route();
        // Side bars.
        assert_eq!(view.window_to_game(2000, 720, 279, 360), None);
        assert_eq!(view.window_to_game(2000, 720, 280, 360), Some((0, 720)));
        assert_eq!(view.window_to_game(2000, 720, 1719, 360), Some((2878, 720)));
        assert_eq!(view.window_to_game(2000, 720, 1720, 360), None);
        // Top and bottom bars.
        assert_eq!(view.window_to_game(1440, 1000, 720, 139), None);
        assert_eq!(view.window_to_game(1440, 1000, 720, 140), Some((1440, 0)));
        assert_eq!(
            view.window_to_game(1440, 1000, 720, 859),
            Some((1440, 1438))
        );
        assert_eq!(view.window_to_game(1440, 1000, 720, 860), None);
    }

    #[test]
    fn worked_example_maps_stream_eye_points_to_game() {
        let view = route();
        // The window shows the crop at 1:1, so a stream-eye point is a window point with the
        // crop's top-left corner taken off.
        for (eye_x, eye_y) in [(0, 360), (100, 400), (1439, 1079)] {
            assert_eq!(
                view.window_to_game(1440, 720, eye_x, eye_y - 360),
                Some((eye_x * 2, (eye_y - 360) * 2))
            );
        }
    }

    #[test]
    fn second_route_with_a_non_zero_crop_x() {
        let view = second_route();
        assert_eq!(view.crop().x, 320);
        assert_eq!(
            view.fit(640, 360),
            Some(Rect {
                x: 0,
                y: 0,
                w: 640,
                h: 360
            })
        );
        assert_eq!(view.window_to_game(640, 360, 0, 0), Some((0, 0)));
        assert_eq!(view.window_to_game(640, 360, 639, 0), Some((2556, 0)));
        assert_eq!(view.window_to_game(640, 360, 0, 359), Some((0, 1436)));
        assert_eq!(view.window_to_game(640, 360, 639, 359), Some((2556, 1436)));
        assert_eq!(view.window_to_game(640, 360, 320, 180), Some((1280, 720)));
        // The crop x only shifts the crop, not the game scale.
        assert_eq!(view.window_to_game(128, 72, 127, 71), Some((2540, 1420)));
    }

    #[test]
    fn new_rejects_zero_sizes_and_crops_outside_the_stream() {
        let crop = Rect {
            x: 0,
            y: 0,
            w: 1440,
            h: 720,
        };
        assert_eq!(
            EyeView::new(0, 1440, crop, 2880, 1440),
            Err(EyeViewError::ZeroSize)
        );
        assert_eq!(
            EyeView::new(2880, 0, crop, 2880, 1440),
            Err(EyeViewError::ZeroSize)
        );
        assert_eq!(
            EyeView::new(2880, 1440, Rect { w: 0, ..crop }, 2880, 1440),
            Err(EyeViewError::ZeroSize)
        );
        assert_eq!(
            EyeView::new(2880, 1440, Rect { h: 0, ..crop }, 2880, 1440),
            Err(EyeViewError::ZeroSize)
        );
        assert_eq!(
            EyeView::new(2880, 1440, crop, 0, 1440),
            Err(EyeViewError::ZeroSize)
        );
        assert_eq!(
            EyeView::new(2880, 1440, crop, 2880, 0),
            Err(EyeViewError::ZeroSize)
        );
        // The crop runs past the right or bottom edge.
        assert_eq!(
            EyeView::new(
                2880,
                1440,
                Rect {
                    x: 1441,
                    y: 0,
                    w: 1440,
                    h: 720
                },
                2880,
                1440
            ),
            Err(EyeViewError::CropOutsideStream)
        );
        assert_eq!(
            EyeView::new(
                2880,
                1440,
                Rect {
                    x: 0,
                    y: 721,
                    w: 1440,
                    h: 720
                },
                2880,
                1440
            ),
            Err(EyeViewError::CropOutsideStream)
        );
        // A crop exactly against the far edges is allowed.
        assert!(EyeView::new(
            2880,
            1440,
            Rect {
                x: 1440,
                y: 720,
                w: 1440,
                h: 720
            },
            2880,
            1440
        )
        .is_ok());
    }
}
