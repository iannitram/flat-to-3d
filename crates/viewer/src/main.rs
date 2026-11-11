use ash::vk;
use flat3d_viewer::gpu::import::{self, Importer};
use flat3d_viewer::xr::{
    self,
    session::{Do, Pump, Supervisor, Tick, XrError},
    swapchain::{self, SwapKey},
};
use flat3d_viewer::{
    source::FrameSource,
    spec::{Source, ViewerSpec},
};
use openxr as oxr;
use std::{
    io::Write,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

static RECENTER: AtomicBool = AtomicBool::new(false);
extern "C" fn signal_recenter(_: libc::c_int) {
    RECENTER.store(true, Ordering::Relaxed)
}
struct PidWatch(Option<OwnedFd>);
impl PidWatch {
    fn open(pid: i32) -> Self {
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
        if fd >= 0 {
            return Self(Some(unsafe { OwnedFd::from_raw_fd(fd) }));
        }
        Self(None)
    }
    fn gone(&self, pid: Option<i32>) -> bool {
        if let Some(fd) = &self.0 {
            let mut p = libc::pollfd {
                fd: fd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            return unsafe { libc::poll(&mut p, 1, 0) } > 0 && p.revents & libc::POLLIN != 0;
        }
        if let Some(pid) = pid {
            let r = unsafe { libc::kill(pid, 0) };
            return r != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
        }
        false
    }
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(code) => std::process::ExitCode::from(code),
        Err((code, e)) => {
            eprintln!("flat3d-viewer: {e}");
            std::process::ExitCode::from(code)
        }
    }
}
fn run() -> Result<u8, (u8, String)> {
    let (args, pid) =
        xr::session::split_until_pid(std::env::args().skip(1).collect()).map_err(|e| (2, e))?;
    let mut spec = ViewerSpec::from_args(&args).map_err(|e| (2, e))?;
    if spec.screen_width_m <= 0.0 {
        spec.screen_width_m = 3.0
    }
    if spec.distance_m <= 0.0 {
        spec.distance_m = 2.5
    }
    let pidwatch = pid.map(PidWatch::open);
    if pidwatch.as_ref().is_some_and(|p| p.gone(pid)) {
        return Ok(0);
    }
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = signal_recenter as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = 0;
        libc::sigaction(libc::SIGUSR1, &action, std::ptr::null_mut());
    }
    let mut supervisor = Supervisor::new();
    let began = Instant::now();
    loop {
        let now = began.elapsed();
        if pidwatch.as_ref().is_some_and(|p| p.gone(pid)) {
            return run_actions(&mut supervisor, now, Tick::PidGone);
        }
        let ctx = match xr::session::connect("flat3d-viewer") {
            Ok(c) => c,
            Err(XrError::Retry(_)) => {
                let _ = run_actions(&mut supervisor, now, Tick::NoHeadset)?;
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
            Err(e) => {
                let _ = run_actions(&mut supervisor, now, Tick::Fatal);
                return Err((3, e.to_string()));
            }
        };
        let mut importer =
            Importer::new(&ctx).map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
        let mut source: Box<dyn FrameSource> = loop {
            if pidwatch.as_ref().is_some_and(|p| p.gone(pid)) {
                return run_actions(&mut supervisor, began.elapsed(), Tick::PidGone);
            }
            match open_source(&spec, &ctx) {
                Ok(Some(s)) => break s,
                Ok(None) => {
                    let _ = run_actions(&mut supervisor, began.elapsed(), Tick::NoSource)?;
                    std::thread::sleep(Duration::from_secs(2));
                }
                Err(e) => {
                    let _ = run_actions(&mut supervisor, began.elapsed(), Tick::Fatal);
                    return Err((3, e));
                }
            }
        };
        let mut session = xr::session::start(&ctx)
            .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
        let mut swap = None;
        let mut last_pose = None;
        let mut last_size = (0, 0);
        let mut events = oxr::EventDataBuffer::new();
        let mut first = true;
        loop {
            if pidwatch.as_ref().is_some_and(|p| p.gone(pid)) {
                let _ = session.session.request_exit();
                return run_actions(&mut supervisor, began.elapsed(), Tick::PidGone);
            }
            let event = match xr::session::pump_events(&ctx, &mut session, &mut events) {
                Ok(v) => v,
                Err(e) => return Err(fatal_error(&mut supervisor, began.elapsed(), e)),
            };
            if event == Pump::Exit {
                return run_actions(&mut supervisor, began.elapsed(), Tick::SessionExit);
            }
            if !session.running {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
            if event == Pump::Recenter || first || RECENTER.swap(false, Ordering::Relaxed) {
                let frame_state = session
                    .waiter
                    .wait()
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                let loc = session
                    .view
                    .locate(&session.local, frame_state.predicted_display_time)
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                if loc.location_flags.contains(
                    oxr::SpaceLocationFlags::ORIENTATION_VALID
                        | oxr::SpaceLocationFlags::POSITION_VALID,
                ) {
                    last_pose = Some(xr::frame::quad_pose(
                        xr::frame::yaw_only(loc.pose),
                        spec.distance_m,
                    ));
                }
                first = false;
                session
                    .stream
                    .begin()
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                let (eyes, size) = overlay(&spec, last_size.0, last_size.1);
                xr::frame::submit(
                    &mut session,
                    &frame_state,
                    ctx.blend,
                    if last_size == (0, 0) {
                        None
                    } else {
                        swap.as_ref()
                    },
                    eyes,
                    size,
                    last_pose.unwrap_or(oxr::Posef::IDENTITY),
                )
                .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                continue;
            }
            let frame_state = session
                .waiter
                .wait()
                .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
            session
                .stream
                .begin()
                .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
            let frame = source
                .next(Duration::ZERO)
                .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
            if let Some(frame) = frame {
                let formats = session
                    .session
                    .enumerate_swapchain_formats()
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                let format = swapchain::pick_format(&formats).ok_or_else(|| {
                    fatal_error(
                        &mut supervisor,
                        began.elapsed(),
                        "runtime offers no supported sRGB quad format",
                    )
                })?;
                let key = SwapKey {
                    width: frame.width,
                    height: frame.height,
                    format,
                };
                if swapchain::ensure(&mut swap, &session, key)
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?
                {
                    importer.reset()
                }
                let sc = swap.as_mut().unwrap();
                let index = sc
                    .acquire()
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                importer
                    .copy_to(&ctx, &frame, sc.images[index as usize])
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                sc.release()
                    .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                last_size = (frame.width, frame.height);
                let (eyes, size) = overlay(&spec, frame.width, frame.height);
                xr::frame::submit(
                    &mut session,
                    &frame_state,
                    ctx.blend,
                    swap.as_ref(),
                    eyes,
                    size,
                    last_pose.unwrap_or(oxr::Posef::IDENTITY),
                )
                .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                run_actions(
                    &mut supervisor,
                    began.elapsed(),
                    Tick::Shown {
                        w: frame.width,
                        h: frame.height,
                    },
                )?;
            } else {
                let (eyes, size) = overlay(&spec, last_size.0, last_size.1);
                xr::frame::submit(
                    &mut session,
                    &frame_state,
                    ctx.blend,
                    swap.as_ref(),
                    eyes,
                    size,
                    last_pose.unwrap_or(oxr::Posef::IDENTITY),
                )
                .map_err(|e| fatal_error(&mut supervisor, began.elapsed(), e))?;
                if last_size == (0, 0) {
                    run_actions(&mut supervisor, began.elapsed(), Tick::NoSource)?;
                }
            }
        }
    }
}
fn open_source(
    spec: &ViewerSpec,
    ctx: &xr::session::XrContext,
) -> Result<Option<Box<dyn FrameSource>>, String> {
    if spec.source == Source::TestPattern {
        return Ok(Some(Box::new(
            flat3d_viewer::source::test_pattern::TestPattern::new(3840, 1080, spec.clone(), 60),
        )));
    }
    let offer = flat3d_viewer::source::pipewire::Offer {
        fourccs: vec![0x34325258, 0x34325241, 0x34324258, 0x34324241],
        modifiers: import::modifiers(ctx, vk::Format::B8G8R8A8_SRGB),
        max_w: 7680,
        max_h: 4320,
    };
    let games = || flat3d_viewer::source::gamescope::open(&offer, Duration::from_secs(2));
    let portal = || {
        let base = std::env::var_os("XDG_STATE_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/state"))
            })
            .ok_or_else(|| "HOME is not set".to_owned())?;
        let path = base.join("flat3d/portal-token");
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?
        }
        flat3d_viewer::source::portal::open(&path, &offer)
    };
    match spec.source {
        Source::Gamescope => Ok(games()?.map(|s| Box::new(s) as Box<dyn FrameSource>)),
        Source::Portal => Ok(Some(Box::new(portal()?))),
        Source::Auto => match games()? {
            Some(s) => Ok(Some(Box::new(s))),
            None => Ok(Some(Box::new(portal()?))),
        },
        Source::TestPattern => unreachable!(),
    }
}
fn overlay(spec: &ViewerSpec, w: u32, h: u32) -> ([oxr::Rect2Di; 2], oxr::Extent2Df) {
    let rects = flat3d_viewer::layout::eye_rects(w, h, spec);
    let eyes = rects.map(|r| oxr::Rect2Di {
        offset: oxr::Offset2Di {
            x: r.x as i32,
            y: r.y as i32,
        },
        extent: oxr::Extent2Di {
            width: r.w as i32,
            height: r.h as i32,
        },
    });
    let rect = if matches!(spec.source, Source::TestPattern) {
        rects[0]
    } else {
        flat3d_viewer::layout::unsqueeze(rects[0], spec.layout)
    };
    let (width, height) = flat3d_viewer::layout::quad_size(rect, spec.screen_width_m);
    (eyes, oxr::Extent2Df { width, height })
}
fn run_actions(s: &mut Supervisor, now: Duration, t: Tick) -> Result<u8, (u8, String)> {
    for action in s.step(now, t) {
        match action {
            Do::Status(st) => {
                println!("{}", st.to_line());
                std::io::stdout().flush().map_err(|e| (3, e.to_string()))?
            }
            Do::Wait(d) => std::thread::sleep(d),
            Do::Exit(c) => return Ok(c),
        }
    }
    Ok(1)
}
fn fatal_error(s: &mut Supervisor, now: Duration, error: impl ToString) -> (u8, String) {
    let message = error.to_string();
    let _ = run_actions(s, now, Tick::Fatal);
    (3, message)
}
