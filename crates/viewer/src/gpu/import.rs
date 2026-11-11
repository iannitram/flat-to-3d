#![cfg(feature = "xr")]
use super::super::xr::session::XrContext;
use ash::vk;
use std::ffi::CStr;

pub const DEVICE_EXTENSIONS: [&CStr; 4] = [
    c"VK_KHR_external_memory_fd",
    c"VK_EXT_external_memory_dma_buf",
    c"VK_EXT_image_drm_format_modifier",
    c"VK_EXT_queue_family_foreign",
];
#[derive(Debug)]
pub enum ImportError {
    Unsupported(String),
    Vk(vk::Result),
}
impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(s) => write!(f, "unsupported frame import: {s}"),
            Self::Vk(e) => write!(f, "Vulkan import error: {e:?}"),
        }
    }
}
impl From<vk::Result> for ImportError {
    fn from(e: vk::Result) -> Self {
        Self::Vk(e)
    }
}
pub fn drm_fourcc_to_vk(f: u32) -> Option<vk::Format> {
    match f {
        0x34325258 | 0x34325241 => Some(vk::Format::B8G8R8A8_SRGB),
        0x34324258 | 0x34324241 => Some(vk::Format::R8G8B8A8_SRGB),
        _ => None,
    }
}
pub fn modifiers(ctx: &XrContext, f: vk::Format) -> Vec<u64> {
    let mut list = vk::DrmFormatModifierPropertiesListEXT::default();
    let mut props = vk::FormatProperties2::default().push_next(&mut list);
    unsafe {
        ctx.vk_instance
            .get_physical_device_format_properties2(ctx.physical, f, &mut props)
    };
    let mut values = vec![
        vk::DrmFormatModifierPropertiesEXT::default();
        list.drm_format_modifier_count as usize
    ];
    list = list.drm_format_modifier_properties(&mut values);
    let mut props = vk::FormatProperties2::default().push_next(&mut list);
    unsafe {
        ctx.vk_instance
            .get_physical_device_format_properties2(ctx.physical, f, &mut props)
    };
    values
        .into_iter()
        .filter(|p| {
            p.drm_format_modifier_tiling_features
                .contains(vk::FormatFeatureFlags::TRANSFER_SRC | vk::FormatFeatureFlags::BLIT_SRC)
        })
        .map(|p| p.drm_format_modifier)
        .collect()
}
pub struct Importer {
    device: ash::Device,
    pool: vk::CommandPool,
    command: vk::CommandBuffer,
    fence: vk::Fence,
    staging: vk::Buffer,
    staging_mem: vk::DeviceMemory,
    staging_size: u64,
    upload: vk::Image,
    upload_mem: vk::DeviceMemory,
    upload_shape: (u32, u32, vk::Format),
    cache: Vec<((u64, u64, u64), vk::Image, vk::DeviceMemory)>,
}
impl Importer {
    pub fn new(ctx: &XrContext) -> Result<Self, ImportError> {
        let pool = unsafe {
            ctx.device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(ctx.queue_family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        }?;
        let command = unsafe {
            ctx.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )
        }?[0];
        let fence = unsafe {
            ctx.device
                .create_fence(&vk::FenceCreateInfo::default(), None)
        }?;
        let (staging, staging_mem) = create_buffer(ctx, 4)?;
        let (upload, upload_mem) = create_image(ctx, 1, 1, vk::Format::B8G8R8A8_SRGB)?;
        Ok(Self {
            device: ctx.device.clone(),
            pool,
            command,
            fence,
            staging,
            staging_mem,
            staging_size: 4,
            upload,
            upload_mem,
            upload_shape: (1, 1, vk::Format::B8G8R8A8_SRGB),
            cache: Vec::new(),
        })
    }
    pub fn copy_to(
        &mut self,
        ctx: &XrContext,
        frame: &crate::frame::Frame,
        dst: vk::Image,
    ) -> Result<(), ImportError> {
        let format = drm_fourcc_to_vk(frame.fourcc)
            .ok_or_else(|| ImportError::Unsupported(format!("DRM format {:#x}", frame.fourcc)))?;
        match &frame.data {
            crate::frame::FrameData::Mem { stride, bytes } => {
                let row = frame.width as usize * 4;
                let needed = (*stride as usize)
                    .checked_mul(frame.height as usize)
                    .ok_or_else(|| ImportError::Unsupported("frame size overflow".into()))?;
                if (*stride as usize) < row || bytes.len() < needed {
                    return Err(ImportError::Unsupported(
                        "short or invalid memory frame".into(),
                    ));
                }
                ensure_buffer(
                    ctx,
                    &mut self.staging,
                    &mut self.staging_mem,
                    &mut self.staging_size,
                    needed as u64,
                )?;
                let ptr = unsafe {
                    ctx.device.map_memory(
                        self.staging_mem,
                        0,
                        needed as u64,
                        vk::MemoryMapFlags::empty(),
                    )
                }? as *mut u8;
                for y in 0..frame.height as usize {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            bytes.as_ptr().add(y * (*stride as usize)),
                            ptr.add(y * row),
                            row,
                        )
                    }
                }
                unsafe { ctx.device.unmap_memory(self.staging_mem) };
                ensure_image(
                    ctx,
                    &mut self.upload,
                    &mut self.upload_mem,
                    &mut self.upload_shape,
                    frame.width,
                    frame.height,
                    format,
                )?;
                self.record_copy(ctx, self.upload, self.staging, frame.width, frame.height)?;
                self.blit(ctx, self.upload, dst, frame.width, frame.height)?;
            }
            crate::frame::FrameData::DmaBuf { planes, modifier } => {
                if planes.len() != 1 {
                    return Err(ImportError::Unsupported(
                        "only single-plane DMA-BUF is supported".into(),
                    ));
                }
                #[cfg(unix)]
                let plane = &planes[0];
                #[cfg(not(unix))]
                let _ = modifier;
                #[cfg(unix)]
                let key = {
                    let plane = &planes[0];
                    let mut stat = unsafe { std::mem::zeroed::<libc::stat>() };
                    if unsafe { libc::fstat(std::os::fd::AsRawFd::as_raw_fd(&plane.fd), &mut stat) }
                        != 0
                    {
                        return Err(ImportError::Unsupported(
                            std::io::Error::last_os_error().to_string(),
                        ));
                    }
                    (stat.st_dev as u64, stat.st_ino as u64, *modifier)
                };
                #[cfg(unix)]
                {
                    let image = if let Some((_, image, _)) =
                        self.cache.iter().find(|(k, _, _)| *k == key)
                    {
                        *image
                    } else {
                        let (image, memory) = import_dmabuf(ctx, frame, plane, *modifier, format)?;
                        if self.cache.len() >= 8 {
                            let (_, old, mem) = self.cache.remove(0);
                            unsafe {
                                self.device.destroy_image(old, None);
                                self.device.free_memory(mem, None)
                            }
                        }
                        self.cache.push((key, image, memory));
                        image
                    };
                    self.blit_dmabuf(ctx, image, dst, frame.width, frame.height)?;
                }
                #[cfg(not(unix))]
                return Err(ImportError::Unsupported(
                    "DMA-BUF import requires Linux".into(),
                ));
            }
        }
        Ok(())
    }
    pub fn reset(&mut self) {
        for (_, image, memory) in self.cache.drain(..) {
            unsafe {
                self.device.destroy_image(image, None);
                self.device.free_memory(memory, None)
            }
        }
    }
    fn record_copy(
        &mut self,
        ctx: &XrContext,
        image: vk::Image,
        buffer: vk::Buffer,
        w: u32,
        h: u32,
    ) -> Result<(), ImportError> {
        self.begin(ctx)?;
        unsafe {
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .image(image)
                .subresource_range(color_range())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
            let copy = vk::BufferImageCopy::default()
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_extent(vk::Extent3D {
                    width: w,
                    height: h,
                    depth: 1,
                });
            ctx.device.cmd_copy_buffer_to_image(
                self.command,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy],
            );
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .image(image)
                    .subresource_range(color_range())
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)],
            );
        }
        self.end(ctx)
    }
    fn blit(
        &mut self,
        ctx: &XrContext,
        src: vk::Image,
        dst: vk::Image,
        w: u32,
        h: u32,
    ) -> Result<(), ImportError> {
        self.begin(ctx)?;
        unsafe {
            let barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .image(dst)
                .subresource_range(color_range())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
            let blit = vk::ImageBlit::default()
                .src_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: w as i32,
                        y: h as i32,
                        z: 1,
                    },
                ])
                .dst_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .dst_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: w as i32,
                        y: h as i32,
                        z: 1,
                    },
                ]);
            ctx.device.cmd_blit_image(
                self.command,
                src,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::NEAREST,
            );
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .image(dst)
                    .subresource_range(color_range())
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ)],
            );
        }
        self.end(ctx)
    }
    #[cfg(unix)]
    fn blit_dmabuf(
        &mut self,
        ctx: &XrContext,
        src: vk::Image,
        dst: vk::Image,
        w: u32,
        h: u32,
    ) -> Result<(), ImportError> {
        self.begin(ctx)?;
        unsafe {
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .src_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
                    .dst_queue_family_index(ctx.queue_family)
                    .old_layout(vk::ImageLayout::GENERAL)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .image(src)
                    .subresource_range(color_range())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)],
            );
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(dst)
                    .subresource_range(color_range())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)],
            );
            let blit = vk::ImageBlit::default()
                .src_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: w as i32,
                        y: h as i32,
                        z: 1,
                    },
                ])
                .dst_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .dst_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: w as i32,
                        y: h as i32,
                        z: 1,
                    },
                ]);
            ctx.device.cmd_blit_image(
                self.command,
                src,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::NEAREST,
            );
            ctx.device.cmd_pipeline_barrier(
                self.command,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[
                    vk::ImageMemoryBarrier::default()
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .image(dst)
                        .subresource_range(color_range())
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ),
                    vk::ImageMemoryBarrier::default()
                        .src_queue_family_index(ctx.queue_family)
                        .dst_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
                        .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                        .new_layout(vk::ImageLayout::GENERAL)
                        .image(src)
                        .subresource_range(color_range())
                        .src_access_mask(vk::AccessFlags::TRANSFER_READ),
                ],
            );
        }
        self.end(ctx)
    }
    fn begin(&self, ctx: &XrContext) -> Result<(), ImportError> {
        unsafe {
            ctx.device.wait_for_fences(&[self.fence], true, u64::MAX)?;
            ctx.device.reset_fences(&[self.fence])?;
            ctx.device
                .reset_command_buffer(self.command, vk::CommandBufferResetFlags::empty())?;
            ctx.device
                .begin_command_buffer(self.command, &vk::CommandBufferBeginInfo::default())?;
        }
        Ok(())
    }
    fn end(&self, ctx: &XrContext) -> Result<(), ImportError> {
        unsafe {
            ctx.device.end_command_buffer(self.command)?;
            ctx.device.queue_submit(
                ctx.queue,
                &[vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&self.command))],
                self.fence,
            )?;
            ctx.device.wait_for_fences(&[self.fence], true, u64::MAX)?;
        }
        Ok(())
    }
}
impl Drop for Importer {
    fn drop(&mut self) {
        self.reset();
        unsafe {
            self.device.destroy_buffer(self.staging, None);
            self.device.free_memory(self.staging_mem, None);
            self.device.destroy_image(self.upload, None);
            self.device.free_memory(self.upload_mem, None);
            self.device.destroy_fence(self.fence, None);
            self.device.destroy_command_pool(self.pool, None);
        }
    }
}
fn color_range() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    }
}
fn memory_type(
    ctx: &XrContext,
    bits: u32,
    flags: vk::MemoryPropertyFlags,
) -> Result<u32, ImportError> {
    let p = unsafe {
        ctx.vk_instance
            .get_physical_device_memory_properties(ctx.physical)
    };
    (0..p.memory_type_count)
        .find(|i| {
            bits & (1 << i) != 0 && p.memory_types[*i as usize].property_flags.contains(flags)
        })
        .ok_or_else(|| ImportError::Unsupported("no compatible Vulkan memory type".into()))
}
fn create_buffer(
    ctx: &XrContext,
    size: u64,
) -> Result<(vk::Buffer, vk::DeviceMemory), ImportError> {
    let b = unsafe {
        ctx.device.create_buffer(
            &vk::BufferCreateInfo::default()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            None,
        )
    }?;
    let req = unsafe { ctx.device.get_buffer_memory_requirements(b) };
    let idx = memory_type(
        ctx,
        req.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    let m = unsafe {
        ctx.device.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(req.size)
                .memory_type_index(idx),
            None,
        )
    }?;
    unsafe { ctx.device.bind_buffer_memory(b, m, 0) }?;
    Ok((b, m))
}
fn ensure_buffer(
    ctx: &XrContext,
    b: &mut vk::Buffer,
    m: &mut vk::DeviceMemory,
    capacity: &mut u64,
    size: u64,
) -> Result<(), ImportError> {
    if *capacity >= size {
        return Ok(());
    }
    unsafe {
        ctx.device.destroy_buffer(*b, None);
        ctx.device.free_memory(*m, None)
    };
    (*b, *m) = create_buffer(ctx, size)?;
    *capacity = size;
    Ok(())
}
fn create_image(
    ctx: &XrContext,
    w: u32,
    h: u32,
    f: vk::Format,
) -> Result<(vk::Image, vk::DeviceMemory), ImportError> {
    let i = unsafe {
        ctx.device.create_image(
            &vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(f)
                .extent(vk::Extent3D {
                    width: w,
                    height: h,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED),
            None,
        )
    }?;
    let req = unsafe { ctx.device.get_image_memory_requirements(i) };
    let idx = memory_type(
        ctx,
        req.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let m = unsafe {
        ctx.device.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(req.size)
                .memory_type_index(idx),
            None,
        )
    }?;
    unsafe { ctx.device.bind_image_memory(i, m, 0) }?;
    Ok((i, m))
}
fn ensure_image(
    ctx: &XrContext,
    i: &mut vk::Image,
    m: &mut vk::DeviceMemory,
    shape: &mut (u32, u32, vk::Format),
    w: u32,
    h: u32,
    f: vk::Format,
) -> Result<(), ImportError> {
    if *shape == (w, h, f) {
        return Ok(());
    }
    unsafe {
        ctx.device.destroy_image(*i, None);
        ctx.device.free_memory(*m, None)
    };
    (*i, *m) = create_image(ctx, w, h, f)?;
    *shape = (w, h, f);
    Ok(())
}
#[cfg(unix)]
fn import_dmabuf(
    ctx: &XrContext,
    frame: &crate::frame::Frame,
    plane: &crate::frame::Plane,
    modifier: u64,
    format: vk::Format,
) -> Result<(vk::Image, vk::DeviceMemory), ImportError> {
    use std::os::fd::AsRawFd;
    if !modifiers(ctx, format).contains(&modifier) {
        return Err(ImportError::Unsupported(format!(
            "modifier {modifier:#x} is not supported for {format:?}"
        )));
    }
    let layout = [vk::SubresourceLayout {
        offset: plane.offset as u64,
        size: 0,
        row_pitch: plane.stride as u64,
        array_pitch: 0,
        depth_pitch: 0,
    }];
    let mut ext = vk::ExternalMemoryImageCreateInfo::default()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
    let mut modinfo = vk::ImageDrmFormatModifierExplicitCreateInfoEXT::default()
        .drm_format_modifier(modifier)
        .plane_layouts(&layout);
    let info = vk::ImageCreateInfo::default()
        .push_next(&mut ext)
        .push_next(&mut modinfo)
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: frame.width,
            height: frame.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
        .usage(vk::ImageUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = unsafe { ctx.device.create_image(&info, None) }?;
    let req = unsafe { ctx.device.get_image_memory_requirements(image) };
    let fd = unsafe { libc::dup(plane.fd.as_raw_fd()) };
    if fd < 0 {
        unsafe { ctx.device.destroy_image(image, None) };
        return Err(ImportError::Unsupported(
            std::io::Error::last_os_error().to_string(),
        ));
    }
    let ext_fd = ash::khr::external_memory_fd::Device::new(&ctx.vk_instance, &ctx.device);
    let mut fd_props = vk::MemoryFdPropertiesKHR::default();
    if let Err(e) = unsafe {
        ext_fd.get_memory_fd_properties(
            vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
            fd,
            &mut fd_props,
        )
    } {
        unsafe {
            libc::close(fd);
            ctx.device.destroy_image(image, None)
        };
        return Err(ImportError::Vk(e));
    }
    let bits = req.memory_type_bits & fd_props.memory_type_bits;
    let props = unsafe {
        ctx.vk_instance
            .get_physical_device_memory_properties(ctx.physical)
    };
    let index = (0..props.memory_type_count)
        .find(|i| bits & (1 << i) != 0)
        .ok_or_else(|| ImportError::Unsupported("dma-buf has no compatible memory type".into()))?;
    let mut import = vk::ImportMemoryFdInfoKHR::default()
        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
        .fd(fd);
    let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
    let allocate = vk::MemoryAllocateInfo::default()
        .push_next(&mut import)
        .push_next(&mut dedicated)
        .allocation_size(req.size)
        .memory_type_index(index);
    let memory = match unsafe { ctx.device.allocate_memory(&allocate, None) } {
        Ok(m) => m,
        Err(e) => {
            unsafe {
                libc::close(fd);
                ctx.device.destroy_image(image, None)
            };
            return Err(ImportError::Vk(e));
        }
    };
    if let Err(e) = unsafe { ctx.device.bind_image_memory(image, memory, 0) } {
        unsafe {
            ctx.device.free_memory(memory, None);
            ctx.device.destroy_image(image, None)
        };
        return Err(ImportError::Vk(e));
    }
    Ok((image, memory))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats() {
        assert_eq!(
            drm_fourcc_to_vk(0x34325258),
            Some(vk::Format::B8G8R8A8_SRGB)
        );
        assert_eq!(
            drm_fourcc_to_vk(0x34325241),
            Some(vk::Format::B8G8R8A8_SRGB)
        );
        assert_eq!(
            drm_fourcc_to_vk(0x34324258),
            Some(vk::Format::R8G8B8A8_SRGB)
        );
        assert_eq!(
            drm_fourcc_to_vk(0x34324241),
            Some(vk::Format::R8G8B8A8_SRGB)
        );
        assert_eq!(drm_fourcc_to_vk(0x3231564e), None);
    }
}
