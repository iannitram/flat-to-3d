# WP-18 recheck: edits to apply (plan mode blocked editing WP-18.md)

File: roadmap-v2.md (WP-18 section)

1. Delete the trailing "Review note: none rejected. All 5 corrections hold..." block (lines after the last `---`). It is about WP-15/WP-01/WP-00 and includes the stale "ApiError does not exist" claim.
2. Frame types: switch to bones §2 frozen frame.rs form.
   - `use crate::frame::{Frame, FrameData, PixelFormat};` (no `Plane`).
   - `Offer { pub formats: Vec<PixelFormat>, pub modifiers: Vec<u64>, pub max_w: u32, pub max_h: u32 }`.
   - Mapping: Bgra8 -> SPA BGRx + BGRA; Rgba8 -> RGBx + RGBA; Nv12 -> NV12; back: BGRx/BGRA -> Bgra8, RGBx/RGBA -> Rgba8, NV12 -> Nv12; else drop.
   - Modifier EnumFormat lists only Bgra8/Rgba8 formats (single-plane DmaBuf); NV12 only in the shm EnumFormat.
   - process: DmaBuf -> `FrameData::DmaBuf{fd: BorrowedFd::borrow_raw(datas[0].fd()).try_clone_to_owned()?, offset: chunk.offset(), stride: chunk.stride() as u32, modifier}`; if datas.len()!=1 drop the frame.
   - MemFd/MemPtr -> `FrameData::Cpu{stride, bytes}` (concatenate each data's [offset..offset+size] in plane order).
   - `Frame{width, height, format, data}`; no `seq`.
3. Name the trait method: `impl FrameSource for PwSource { fn next(&mut self, timeout: Duration) -> Result<Option<Frame>, String> }`.
4. gamescope.rs: WP-01's source.rs has `pub mod gamescope;` UNgated. So in gamescope.rs, `NODE_NAME` is ungated and `open` plus its `use` lines carry `#[cfg(feature = "capture")]`. Fix the line "All three files are behind feature = capture".
5. `pw::init()` is already idempotent (pipewire-0.10.1 lib.rs:180 OnceLock), so the extra `std::sync::Once` is optional.
6. Flags: NV12 DmaBuf is not representable in the frozen single-fd `DmaBuf`, so NV12 comes over shm only.
