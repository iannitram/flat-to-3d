#![cfg(feature = "xr")]
use super::{
    session::{Session, XrError},
    swapchain::QuadSwapchain,
};
use ash::vk;
use openxr as xr;

pub fn yaw_only(p: xr::Posef) -> xr::Posef {
    let q = p.orientation;
    // Extract the heading from the horizontal projection of the forward vector.
    let yaw = (2.0 * (q.w * q.y + q.x * q.z)).atan2(1.0 - 2.0 * (q.y * q.y + q.x * q.x));
    xr::Posef {
        orientation: xr::Quaternionf {
            x: 0.0,
            y: (yaw * 0.5).sin(),
            z: 0.0,
            w: (yaw * 0.5).cos(),
        },
        position: p.position,
    }
}
pub fn quad_pose(anchor: xr::Posef, distance_m: f32) -> xr::Posef {
    let q = anchor.orientation;
    let x = 2.0 * (q.x * q.z + q.w * q.y);
    let z = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
    xr::Posef {
        orientation: anchor.orientation,
        position: xr::Vector3f {
            x: anchor.position.x - x * distance_m,
            y: anchor.position.y,
            z: anchor.position.z - z * distance_m,
        },
    }
}
pub fn submit(
    s: &mut Session,
    state: &xr::FrameState,
    blend: xr::EnvironmentBlendMode,
    sc: Option<&QuadSwapchain>,
    eyes: [xr::Rect2Di; 2],
    size: xr::Extent2Df,
    pose: xr::Posef,
) -> Result<(), XrError> {
    if let Some(sc) = sc {
        let left = xr::CompositionLayerQuad::new()
            .space(&s.local)
            .eye_visibility(xr::EyeVisibility::LEFT)
            .sub_image(
                xr::SwapchainSubImage::new()
                    .swapchain(&sc.handle)
                    .image_rect(eyes[0])
                    .image_array_index(0),
            )
            .pose(pose)
            .size(size);
        let right = xr::CompositionLayerQuad::new()
            .space(&s.local)
            .eye_visibility(xr::EyeVisibility::RIGHT)
            .sub_image(
                xr::SwapchainSubImage::new()
                    .swapchain(&sc.handle)
                    .image_rect(eyes[1])
                    .image_array_index(0),
            )
            .pose(pose)
            .size(size);
        s.stream
            .end(state.predicted_display_time, blend, &[&left, &right])?;
    } else {
        s.stream.end(state.predicted_display_time, blend, &[])?;
    }
    let _ = vk::ImageLayout::UNDEFINED;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yaw_and_offset() {
        let p = xr::Posef {
            orientation: xr::Quaternionf {
                x: 0.3,
                y: 0.2,
                z: 0.4,
                w: 0.8,
            },
            position: xr::Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        let y = yaw_only(p);
        assert_eq!(y.orientation.x, 0.0);
        assert_eq!(y.orientation.z, 0.0);
        assert!((y.orientation.y - 0.319).abs() < 0.01);
        assert!((y.orientation.w - 0.948).abs() < 0.01);
        let q = quad_pose(xr::Posef::IDENTITY, 2.5);
        assert!((q.position.z + 2.5).abs() < f32::EPSILON);
    }
}
