#![cfg(feature = "xr")]
use super::session::{Session, XrError};
use ash::vk::{self, Handle};
use openxr as xr;

pub const PREFERRED: [vk::Format; 2] = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];
pub fn pick_format(offered: &[u32]) -> Option<vk::Format> {
    PREFERRED
        .into_iter()
        .find(|want| offered.contains(&((*want).as_raw() as u32)))
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapKey {
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
}
pub fn needs_recreate(cur: Option<SwapKey>, want: SwapKey) -> bool {
    cur != Some(want)
}
pub struct QuadSwapchain {
    pub key: SwapKey,
    pub handle: xr::Swapchain<xr::Vulkan>,
    pub images: Vec<vk::Image>,
}
impl QuadSwapchain {
    pub fn create(s: &Session, key: SwapKey) -> Result<Self, XrError> {
        let handle = s.session.create_swapchain(&xr::SwapchainCreateInfo {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT
                | xr::SwapchainUsageFlags::TRANSFER_DST
                | xr::SwapchainUsageFlags::SAMPLED,
            format: key.format.as_raw() as u32,
            sample_count: 1,
            width: key.width,
            height: key.height,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        })?;
        let images = handle
            .enumerate_images()?
            .into_iter()
            .map(vk::Image::from_raw)
            .collect();
        Ok(Self {
            key,
            handle,
            images,
        })
    }
    pub fn acquire(&mut self) -> Result<u32, XrError> {
        let i = self.handle.acquire_image()?;
        self.handle.wait_image(xr::Duration::INFINITE)?;
        Ok(i)
    }
    pub fn release(&mut self) -> Result<(), XrError> {
        self.handle.release_image()?;
        Ok(())
    }
}
pub fn ensure(
    slot: &mut Option<QuadSwapchain>,
    s: &Session,
    want: SwapKey,
) -> Result<bool, XrError> {
    if !needs_recreate(slot.as_ref().map(|x| x.key), want) {
        return Ok(false);
    }
    *slot = Some(QuadSwapchain::create(s, want)?);
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn choose_and_recreate() {
        assert_eq!(
            pick_format(&[
                vk::Format::R8G8B8A8_SRGB.as_raw() as u32,
                vk::Format::B8G8R8A8_SRGB.as_raw() as u32
            ]),
            Some(vk::Format::B8G8R8A8_SRGB)
        );
        assert_eq!(
            pick_format(&[vk::Format::R8G8B8A8_SRGB.as_raw() as u32]),
            Some(vk::Format::R8G8B8A8_SRGB)
        );
        assert_eq!(
            pick_format(&[vk::Format::R8G8B8A8_UNORM.as_raw() as u32]),
            None
        );
        let a = SwapKey {
            width: 1,
            height: 2,
            format: vk::Format::R8G8B8A8_SRGB,
        };
        assert!(needs_recreate(None, a));
        assert!(!needs_recreate(Some(a), a));
        assert!(needs_recreate(Some(a), SwapKey { width: 3, ..a }));
    }
}
