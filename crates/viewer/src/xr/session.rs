#![cfg(feature = "xr")]

use ash::vk::{self, Handle};
use openxr as xr;
use std::time::Duration;

pub struct XrContext {
    pub entry: xr::Entry,
    pub instance: xr::Instance,
    pub system: xr::SystemId,
    pub blend: xr::EnvironmentBlendMode,
    pub vk_entry: ash::Entry,
    pub vk_instance: ash::Instance,
    pub physical: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue: vk::Queue,
    pub queue_family: u32,
}
impl Drop for XrContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
            self.vk_instance.destroy_instance(None);
        }
    }
}

#[derive(Debug)]
pub enum XrError {
    NoLoader(String),
    Retry(String),
    Missing(&'static str),
    Vk(vk::Result),
    Xr(xr::sys::Result),
}

impl std::fmt::Display for XrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLoader(e) => write!(f, "OpenXR loader unavailable: {e}"),
            Self::Retry(e) => write!(f, "OpenXR runtime or headset unavailable: {e}"),
            Self::Missing(e) => write!(f, "required OpenXR/Vulkan capability missing: {e}"),
            Self::Vk(e) => write!(f, "Vulkan error: {e:?}"),
            Self::Xr(e) => write!(f, "OpenXR error: {e:?}"),
        }
    }
}
impl From<xr::sys::Result> for XrError {
    fn from(value: xr::sys::Result) -> Self {
        Self::Xr(value)
    }
}
impl From<vk::Result> for XrError {
    fn from(value: vk::Result) -> Self {
        Self::Vk(value)
    }
}

pub fn classify_system_err(r: xr::sys::Result) -> XrError {
    if r == xr::sys::Result::ERROR_FORM_FACTOR_UNAVAILABLE
        || r == xr::sys::Result::ERROR_RUNTIME_UNAVAILABLE
        || r == xr::sys::Result::ERROR_RUNTIME_FAILURE
    {
        XrError::Retry(r.to_string())
    } else {
        XrError::Xr(r)
    }
}

pub fn connect(app: &str) -> Result<XrContext, XrError> {
    let entry = unsafe { xr::Entry::load(&()) }.map_err(|e| XrError::NoLoader(e.to_string()))?;
    let available = entry.enumerate_extensions()?;
    if !available.khr_vulkan_enable2 {
        return Err(XrError::Missing("XR_KHR_vulkan_enable2"));
    }
    let mut exts = xr::ExtensionSet::default();
    exts.khr_vulkan_enable2 = true;
    let instance = entry.create_instance(
        &xr::ApplicationInfo {
            application_name: app,
            application_version: 0,
            engine_name: "flat3d",
            engine_version: 0,
            api_version: xr::Version::new(1, 0, 0),
        },
        &exts,
        &[],
        &(),
    )?;
    let system = instance
        .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
        .map_err(classify_system_err)?;
    let requirements = instance.graphics_requirements::<xr::Vulkan>(system)?;
    let vk_entry = unsafe { ash::Entry::load() }.map_err(|e| XrError::NoLoader(e.to_string()))?;
    let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_2);
    let ci = vk::InstanceCreateInfo::default().application_info(&app_info);
    let raw_instance = unsafe {
        instance.create_vulkan_instance(
            system,
            std::mem::transmute::<
                vk::PFN_vkGetInstanceProcAddr,
                xr::sys::platform::VkGetInstanceProcAddr,
            >(vk_entry.static_fn().get_instance_proc_addr),
            &ci as *const _ as _,
        )
    }
    .map_err(XrError::Xr)?
    .map_err(|r| XrError::Vk(vk::Result::from_raw(r)))?;
    let vk_instance = unsafe {
        ash::Instance::load(
            vk_entry.static_fn(),
            vk::Instance::from_raw(raw_instance as usize as u64),
        )
    };
    let physical_raw = unsafe { instance.vulkan_graphics_device(system, raw_instance) }?;
    let physical = vk::PhysicalDevice::from_raw(physical_raw as usize as u64);
    let props = unsafe { vk_instance.get_physical_device_properties(physical) };
    if props.api_version < requirements.min_api_version_supported.into_raw() as u32 {
        return Err(XrError::Missing(
            "Vulkan version required by OpenXR runtime",
        ));
    }
    let qprops = unsafe { vk_instance.get_physical_device_queue_family_properties(physical) };
    let queue_family = qprops
        .iter()
        .position(|q| q.queue_count > 0 && q.queue_flags.contains(vk::QueueFlags::GRAPHICS))
        .ok_or(XrError::Missing("graphics queue"))? as u32;
    let priorities = [1.0];
    let queues = [vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family)
        .queue_priorities(&priorities)];
    let ext_names = super::super::gpu::import::DEVICE_EXTENSIONS
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();
    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queues)
        .enabled_extension_names(&ext_names);
    let raw_device = unsafe {
        instance.create_vulkan_device(
            system,
            std::mem::transmute::<
                vk::PFN_vkGetInstanceProcAddr,
                xr::sys::platform::VkGetInstanceProcAddr,
            >(vk_entry.static_fn().get_instance_proc_addr),
            physical_raw,
            &device_info as *const _ as _,
        )
    }
    .map_err(XrError::Xr)?
    .map_err(|r| XrError::Vk(vk::Result::from_raw(r)))?;
    let device = unsafe {
        ash::Device::load(
            vk_instance.fp_v1_0(),
            vk::Device::from_raw(raw_device as usize as u64),
        )
    };
    let queue = unsafe { device.get_device_queue(queue_family, 0) };
    let blend = instance
        .enumerate_environment_blend_modes(system, xr::ViewConfigurationType::PRIMARY_STEREO)?
        .into_iter()
        .next()
        .ok_or(XrError::Missing("environment blend mode"))?;
    Ok(XrContext {
        entry,
        instance,
        system,
        blend,
        vk_entry,
        vk_instance,
        physical,
        device,
        queue,
        queue_family,
    })
}

pub struct Session {
    pub session: xr::Session<xr::Vulkan>,
    pub waiter: xr::FrameWaiter,
    pub stream: xr::FrameStream<xr::Vulkan>,
    pub local: xr::Space,
    pub view: xr::Space,
    pub running: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pump {
    Idle,
    Running,
    Recenter,
    Exit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnState {
    Begin,
    End,
    Exit,
    Nothing,
}
pub fn on_state(st: xr::SessionState) -> OnState {
    match st {
        xr::SessionState::READY => OnState::Begin,
        xr::SessionState::STOPPING => OnState::End,
        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => OnState::Exit,
        _ => OnState::Nothing,
    }
}
pub fn start(ctx: &XrContext) -> Result<Session, XrError> {
    let (session, waiter, stream) = unsafe {
        ctx.instance.create_session::<xr::Vulkan>(
            ctx.system,
            &xr::vulkan::SessionCreateInfo {
                instance: ctx.vk_instance.handle().as_raw() as usize as _,
                physical_device: ctx.physical.as_raw() as usize as _,
                device: ctx.device.handle().as_raw() as usize as _,
                queue_family_index: ctx.queue_family,
                queue_index: 0,
            },
        )?
    };
    let local =
        session.create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)?;
    let view = session.create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)?;
    Ok(Session {
        session,
        waiter,
        stream,
        local,
        view,
        running: false,
    })
}
pub fn pump_events(
    ctx: &XrContext,
    s: &mut Session,
    buf: &mut xr::EventDataBuffer,
) -> Result<Pump, XrError> {
    let mut result = Pump::Idle;
    while let Some(event) = ctx.instance.poll_event(buf)? {
        match event {
            xr::Event::SessionStateChanged(e) => match on_state(e.state()) {
                OnState::Begin => {
                    s.session.begin(xr::ViewConfigurationType::PRIMARY_STEREO)?;
                    s.running = true;
                    result = Pump::Running;
                }
                OnState::End => {
                    s.session.end()?;
                    s.running = false;
                    result = Pump::Idle;
                }
                OnState::Exit => return Ok(Pump::Exit),
                OnState::Nothing => {}
            },
            xr::Event::ReferenceSpaceChangePending(_) => result = Pump::Recenter,
            _ => {}
        }
        *buf = xr::EventDataBuffer::new();
    }
    Ok(result)
}
pub fn split_until_pid(args: Vec<String>) -> Result<(Vec<String>, Option<i32>), String> {
    let mut kept = Vec::with_capacity(args.len());
    let mut pid = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--until-pid" {
            if pid.is_some() {
                return Err("--until-pid may only be specified once".into());
            }
            let raw = args
                .get(i + 1)
                .ok_or_else(|| "missing value for --until-pid".to_owned())?;
            let value = raw
                .parse::<i32>()
                .map_err(|_| format!("invalid value for --until-pid: {raw}"))?;
            if value <= 0 {
                return Err("--until-pid must be a positive pid".into());
            }
            pid = Some(value);
            i += 2;
        } else {
            kept.push(args[i].clone());
            i += 1;
        }
    }
    Ok((kept, pid))
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    PidGone,
    NoHeadset,
    NoSource,
    Shown { w: u32, h: u32 },
    Fatal,
    SessionExit,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Do {
    Status(crate::status::Status),
    Wait(Duration),
    Exit(u8),
}
pub struct Supervisor {
    last: Option<crate::status::Status>,
    sent_at: Duration,
    shown: u64,
    interval_start: Duration,
}
impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}
impl Supervisor {
    pub fn new() -> Self {
        Self {
            last: None,
            sent_at: Duration::ZERO,
            shown: 0,
            interval_start: Duration::ZERO,
        }
    }
    pub fn step(&mut self, now: Duration, t: Tick) -> Vec<Do> {
        use crate::status::{State, Status};
        let status = match t {
            Tick::PidGone => return vec![Do::Exit(0)],
            Tick::NoHeadset => Some(Status {
                state: State::WaitingHeadset,
                fps: 0.0,
                w: 0,
                h: 0,
            }),
            Tick::NoSource => Some(Status {
                state: State::WaitingSource,
                fps: 0.0,
                w: 0,
                h: 0,
            }),
            Tick::Shown { w, h } => {
                self.shown += 1;
                Some(Status {
                    state: State::Showing,
                    fps: 0.0,
                    w,
                    h,
                })
            }
            Tick::Fatal => Some(Status {
                state: State::Error,
                fps: 0.0,
                w: 0,
                h: 0,
            }),
            Tick::SessionExit => return vec![Do::Exit(0)],
        };
        let mut out = Vec::new();
        if let Some(s) = status {
            let changed = self
                .last
                .map(|p| {
                    p.state != s.state || (s.state == State::Showing && (p.w != s.w || p.h != s.h))
                })
                .unwrap_or(true);
            let due = s.state == State::Showing
                && now.saturating_sub(self.sent_at) >= Duration::from_secs(5);
            if changed || due {
                let fps = if s.state == State::Showing {
                    self.shown as f32
                        / now
                            .saturating_sub(self.interval_start)
                            .as_secs_f32()
                            .max(0.001)
                } else {
                    0.0
                };
                let s = Status { fps, ..s };
                out.push(Do::Status(s));
                self.last = Some(s);
                self.sent_at = now;
                self.interval_start = now;
                self.shown = 0;
            }
        }
        if t == Tick::NoHeadset {
            out.push(Do::Wait(Duration::from_secs(2)));
        }
        if t == Tick::Fatal {
            out.push(Do::Exit(3));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_pid_and_errors() {
        assert_eq!(
            split_until_pid(vec![
                "--source".into(),
                "auto".into(),
                "--until-pid".into(),
                "42".into()
            ])
            .unwrap(),
            (vec!["--source".into(), "auto".into()], Some(42))
        );
        assert_eq!(
            split_until_pid(vec!["--until-pid".into(), "42".into()])
                .unwrap()
                .1,
            Some(42)
        );
        assert!(split_until_pid(vec!["--until-pid".into()]).is_err());
        assert!(split_until_pid(vec!["--until-pid".into(), "x".into()]).is_err());
    }
    #[test]
    fn sidecar_argv_roundtrips_viewer_specs() {
        use crate::spec::{Layout, Source, ViewerSpec};
        for spec in [
            ViewerSpec {
                layout: Layout::TabRev,
                source: Source::Portal,
                swap_eyes: true,
                crop_aspect: Some(16.0 / 9.0),
                screen_width_m: 3.5,
                distance_m: 2.0,
            },
            ViewerSpec::default(),
        ] {
            let mut argv = spec.to_args();
            argv.extend(["--until-pid".into(), "4242".into()]);
            let (args, pid) = split_until_pid(argv).unwrap();
            assert_eq!(pid, Some(4242));
            assert_eq!(ViewerSpec::from_args(&args), Ok(spec));
        }
    }
    #[test]
    fn system_errors_are_classified_for_retry() {
        for r in [
            xr::sys::Result::ERROR_FORM_FACTOR_UNAVAILABLE,
            xr::sys::Result::ERROR_RUNTIME_UNAVAILABLE,
            xr::sys::Result::ERROR_RUNTIME_FAILURE,
        ] {
            assert!(matches!(classify_system_err(r), XrError::Retry(_)));
        }
        assert!(matches!(
            classify_system_err(xr::sys::Result::ERROR_VALIDATION_FAILURE),
            XrError::Xr(_)
        ));
    }
    #[test]
    fn state_mapping() {
        assert_eq!(on_state(xr::SessionState::READY), OnState::Begin);
        assert_eq!(on_state(xr::SessionState::STOPPING), OnState::End);
        assert_eq!(on_state(xr::SessionState::EXITING), OnState::Exit);
        assert_eq!(on_state(xr::SessionState::FOCUSED), OnState::Nothing);
    }
    #[test]
    fn supervisor_reports_changes_and_exits() {
        let mut s = Supervisor::new();
        assert!(matches!(
            s.step(Duration::ZERO, Tick::PidGone).as_slice(),
            [Do::Exit(0)]
        ));
        let a = s.step(Duration::ZERO, Tick::NoHeadset);
        assert!(
            matches!(a.as_slice(),[Do::Status(st),Do::Wait(d)] if st.state==crate::status::State::WaitingHeadset&&*d==Duration::from_secs(2))
        );
        let b = s.step(Duration::from_secs(2), Tick::NoHeadset);
        assert!(matches!(b.as_slice(),[Do::Wait(d)] if *d==Duration::from_secs(2)));
        assert!(
            matches!(s.step(Duration::from_secs(3),Tick::Shown{w:100,h:50}).as_slice(),[Do::Status(st)] if st.state==crate::status::State::Showing&&st.w==100)
        );
        assert!(s
            .step(Duration::from_secs(4), Tick::Shown { w: 100, h: 50 })
            .is_empty());
        assert!(
            matches!(s.step(Duration::from_secs(4),Tick::Shown{w:200,h:100}).as_slice(),[Do::Status(st)] if st.w==200)
        );
        assert!(
            matches!(s.step(Duration::from_secs(10),Tick::Shown{w:200,h:100}).as_slice(),[Do::Status(st)] if st.state==crate::status::State::Showing)
        );
        assert!(
            matches!(s.step(Duration::from_secs(11),Tick::Fatal).as_slice(),[Do::Status(st),Do::Exit(3)] if st.state==crate::status::State::Error)
        );
        assert!(matches!(
            s.step(Duration::from_secs(11), Tick::SessionExit)
                .as_slice(),
            [Do::Exit(0)]
        ));
    }
}
