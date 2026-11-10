//! One captured stereo input frame, either copied into memory or held by DMA-BUF handles.

#[cfg(unix)]
use std::os::fd::OwnedFd;
#[cfg(windows)]
use std::os::windows::io::OwnedHandle as OwnedFd;

#[derive(Debug)]
pub struct Plane {
    pub fd: OwnedFd,
    pub offset: u32,
    pub stride: u32,
}

#[derive(Debug)]
pub enum FrameData {
    Mem { stride: u32, bytes: Vec<u8> },
    DmaBuf { planes: Vec<Plane>, modifier: u64 },
}

#[derive(Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    /// DRM fourcc format identifier.
    pub fourcc: u32,
    pub seq: u64,
    pub data: FrameData,
}
