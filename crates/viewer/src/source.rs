pub use crate::frame::Frame;

pub trait FrameSource {
    fn next(&mut self, timeout: std::time::Duration) -> Result<Option<Frame>, String>;
}

#[cfg(feature = "capture")]
pub mod gamescope;
#[cfg(feature = "capture")]
pub mod pipewire;
#[cfg(feature = "capture")]
pub mod portal;
pub mod test_pattern;
