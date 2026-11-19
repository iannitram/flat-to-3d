#![allow(non_snake_case, clippy::missing_safety_doc)]
use ash::vk::{self, Handle};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::io::Write;
use std::mem::transmute;
use std::sync::{Mutex, OnceLock, RwLock};

#[repr(C)]
struct InstLink {
    next: *mut InstLink,
    gipa: vk::PFN_vkGetInstanceProcAddr,
    gpdpa: *const c_void,
}
#[repr(C)]
struct DevLink {
    next: *mut DevLink,
    gipa: vk::PFN_vkGetInstanceProcAddr,
    gdpa: vk::PFN_vkGetDeviceProcAddr,
}
#[repr(C)]
struct LoaderInfo {
    s_type: i32,
    p_next: *mut LoaderInfo,
    function: i32,
    link: *mut c_void,
}

type Void = vk::PFN_vkVoidFunction;
struct Dev {
    begin: Void,
    begin_khr: Void,
    vp: Void,
    vpc: Void,
    vpc_ext: Void,
    draw: Void,
    drawi: Void,
    present: Void,
    view: Void,
    sc: Void,
    scc: Void,
    scc_ext: Void,
}
unsafe impl Send for Dev {}
unsafe impl Sync for Dev {}
#[derive(Clone, Copy, Default)]
struct Cb {
    area: (u32, u32),
    colors: u32,
    fmt: i32,
    vp: (i32, i32, i32, i32),
    fresh: bool,
    vpv: vk::Viewport,
    rect: vk::Rect2D,
    vp_count: bool,
    sc_count: bool,
}
#[derive(Clone, Copy, Default)]
struct Val {
    passes: u64,
    draws: u64,
    verts: u64,
    last: u64,
}
type Key = ((u32, u32), u32, i32, (i32, i32, i32, i32));
#[derive(Default)]
struct St {
    cbs: HashMap<u64, Cb>,
    hist: HashMap<Key, Val>,
    views: HashMap<u64, i32>,
    seq: u64,
    frames: u64,
}

fn insts() -> &'static RwLock<HashMap<usize, usize>> {
    static I: OnceLock<RwLock<HashMap<usize, usize>>> = OnceLock::new();
    I.get_or_init(Default::default)
}
fn devs() -> &'static RwLock<HashMap<usize, &'static Dev>> {
    static I: OnceLock<RwLock<HashMap<usize, &'static Dev>>> = OnceLock::new();
    I.get_or_init(Default::default)
}
fn gdpas() -> &'static RwLock<HashMap<usize, usize>> {
    static I: OnceLock<RwLock<HashMap<usize, usize>>> = OnceLock::new();
    I.get_or_init(Default::default)
}
fn st() -> &'static Mutex<St> {
    static I: OnceLock<Mutex<St>> = OnceLock::new();
    I.get_or_init(Default::default)
}
unsafe fn key(raw: u64) -> usize {
    *(raw as *const usize)
}
unsafe fn dev(raw: u64) -> &'static Dev {
    devs().read().unwrap()[&key(raw)]
}
fn log_path() -> String {
    format!(
        "{}/.local/state/flat3d/ui-probe.log",
        std::env::var("HOME").unwrap_or_default()
    )
}
fn log(s: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = f.write_all(s.as_bytes());
    }
}

macro_rules! hook {
    ($f:expr) => {
        Some(transmute::<*const (), unsafe extern "system" fn()>(
            $f as *const (),
        ))
    };
}

unsafe extern "system" fn create_instance(
    ci: *const vk::InstanceCreateInfo<'_>,
    a: *const vk::AllocationCallbacks<'_>,
    out: *mut vk::Instance,
) -> vk::Result {
    let mut p = (*ci).p_next as *mut LoaderInfo;
    while !(p.is_null() || ((*p).s_type == 47 && (*p).function == 0)) {
        p = (*p).p_next;
    }
    if p.is_null() {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    }
    let l = (*p).link as *mut InstLink;
    let gipa = (*l).gipa;
    (*p).link = (*l).next as *mut c_void;
    let f: vk::PFN_vkCreateInstance =
        transmute(gipa(vk::Instance::null(), c"vkCreateInstance".as_ptr()));
    let r = f(ci, a, out);
    if r == vk::Result::SUCCESS {
        insts()
            .write()
            .unwrap()
            .insert(key((*out).as_raw()), gipa as usize);
        log("instance created\n");
    }
    r
}
unsafe extern "system" fn create_device(
    pd: vk::PhysicalDevice,
    ci: *const vk::DeviceCreateInfo<'_>,
    a: *const vk::AllocationCallbacks<'_>,
    out: *mut vk::Device,
) -> vk::Result {
    let mut p = (*ci).p_next as *mut LoaderInfo;
    while !(p.is_null() || ((*p).s_type == 48 && (*p).function == 0)) {
        p = (*p).p_next;
    }
    if p.is_null() {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    }
    let l = (*p).link as *mut DevLink;
    let (gipa, gdpa) = ((*l).gipa, (*l).gdpa);
    (*p).link = (*l).next as *mut c_void;
    let f: vk::PFN_vkCreateDevice =
        transmute(gipa(vk::Instance::null(), c"vkCreateDevice".as_ptr()));
    let r = f(pd, ci, a, out);
    if r == vk::Result::SUCCESS {
        let d = *out;
        let g = |n: &CStr| gdpa(d, n.as_ptr());
        let t = Dev {
            begin: g(c"vkCmdBeginRendering"),
            begin_khr: g(c"vkCmdBeginRenderingKHR"),
            vp: g(c"vkCmdSetViewport"),
            vpc: g(c"vkCmdSetViewportWithCount"),
            vpc_ext: g(c"vkCmdSetViewportWithCountEXT"),
            draw: g(c"vkCmdDraw"),
            drawi: g(c"vkCmdDrawIndexed"),
            present: g(c"vkQueuePresentKHR"),
            view: g(c"vkCreateImageView"),
            sc: g(c"vkCmdSetScissor"),
            scc: g(c"vkCmdSetScissorWithCount"),
            scc_ext: g(c"vkCmdSetScissorWithCountEXT"),
        };
        log(&format!(
            "device created begin={} khr={} vpc={}\n",
            t.begin.is_some(),
            t.begin_khr.is_some(),
            t.vpc.is_some()
        ));
        gdpas()
            .write()
            .unwrap()
            .insert(key(d.as_raw()), gdpa as usize);
        devs()
            .write()
            .unwrap()
            .insert(key(d.as_raw()), Box::leak(Box::new(t)));
    }
    r
}

unsafe fn begin_common(cb: vk::CommandBuffer, i: *const vk::RenderingInfo<'_>) {
    let i = &*i;
    let mut s = st().lock().unwrap();
    let (mut fmt, colors) = (0, i.color_attachment_count);
    if colors > 0 {
        let v = (*i.p_color_attachments).image_view.as_raw();
        fmt = s.views.get(&v).copied().unwrap_or(-1);
    }
    let c = s.cbs.entry(cb.as_raw()).or_default();
    c.area = (i.render_area.extent.width, i.render_area.extent.height);
    c.colors = colors;
    c.fmt = fmt;
    c.fresh = true;
}
unsafe extern "system" fn begin(cb: vk::CommandBuffer, i: *const vk::RenderingInfo<'_>) {
    begin_common(cb, i);
    let f: vk::PFN_vkCmdBeginRendering = transmute(dev(cb.as_raw()).begin);
    f(cb, i)
}
unsafe extern "system" fn begin_khr(cb: vk::CommandBuffer, i: *const vk::RenderingInfo<'_>) {
    begin_common(cb, i);
    let f: vk::PFN_vkCmdBeginRendering = transmute(dev(cb.as_raw()).begin_khr);
    f(cb, i)
}
unsafe fn set_vp(cb: vk::CommandBuffer, n: u32, v: *const vk::Viewport, count: bool) {
    if n == 0 {
        return;
    }
    let v = *v;
    let mut g = st().lock().unwrap();
    let c = g.cbs.entry(cb.as_raw()).or_default();
    c.vp = (v.x as i32, v.y as i32, v.width as i32, v.height as i32);
    c.vpv = v;
    c.vp_count = count;
}
unsafe fn set_sc(cb: vk::CommandBuffer, n: u32, r: *const vk::Rect2D, count: bool) {
    if n == 0 {
        return;
    }
    let r = *r;
    let mut g = st().lock().unwrap();
    let c = g.cbs.entry(cb.as_raw()).or_default();
    c.rect = r;
    c.sc_count = count;
}
unsafe extern "system" fn vp(cb: vk::CommandBuffer, first: u32, n: u32, v: *const vk::Viewport) {
    if first == 0 {
        set_vp(cb, n, v, false);
    }
    let f: vk::PFN_vkCmdSetViewport = transmute(dev(cb.as_raw()).vp);
    f(cb, first, n, v)
}
unsafe extern "system" fn vpc(cb: vk::CommandBuffer, n: u32, v: *const vk::Viewport) {
    set_vp(cb, n, v, true);
    let f: vk::PFN_vkCmdSetViewportWithCount = transmute(dev(cb.as_raw()).vpc);
    f(cb, n, v)
}
unsafe extern "system" fn vpc_ext(cb: vk::CommandBuffer, n: u32, v: *const vk::Viewport) {
    set_vp(cb, n, v, true);
    let f: vk::PFN_vkCmdSetViewportWithCount = transmute(dev(cb.as_raw()).vpc_ext);
    f(cb, n, v)
}
unsafe extern "system" fn sc(cb: vk::CommandBuffer, first: u32, n: u32, r: *const vk::Rect2D) {
    if first == 0 {
        set_sc(cb, n, r, false);
    }
    let f: vk::PFN_vkCmdSetScissor = transmute(dev(cb.as_raw()).sc);
    f(cb, first, n, r)
}
unsafe extern "system" fn scc(cb: vk::CommandBuffer, n: u32, r: *const vk::Rect2D) {
    set_sc(cb, n, r, true);
    let f: vk::PFN_vkCmdSetScissorWithCount = transmute(dev(cb.as_raw()).scc);
    f(cb, n, r)
}
unsafe extern "system" fn scc_ext(cb: vk::CommandBuffer, n: u32, r: *const vk::Rect2D) {
    set_sc(cb, n, r, true);
    let f: vk::PFN_vkCmdSetScissorWithCount = transmute(dev(cb.as_raw()).scc_ext);
    f(cb, n, r)
}
/// The layer does nothing unless the route sets its enable variable.
fn enabled() -> bool {
    static E: OnceLock<bool> = OnceLock::new();
    *E.get_or_init(|| std::env::var("FLAT3D_UI_LAYER").is_ok_and(|v| v == "1"))
}
fn disparity() -> f32 {
    static D: OnceLock<f32> = OnceLock::new();
    *D.get_or_init(|| {
        std::env::var("FLAT3D_UI_DISPARITY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(24.0)
    })
}
unsafe fn apply(cb: vk::CommandBuffer, d: &Dev, c: &Cb, v: vk::Viewport, r: vk::Rect2D) {
    if c.vp_count {
        let f: vk::PFN_vkCmdSetViewportWithCount = transmute(d.vpc.or(d.vpc_ext));
        f(cb, 1, &v);
    } else {
        let f: vk::PFN_vkCmdSetViewport = transmute(d.vp);
        f(cb, 0, 1, &v);
    }
    if c.sc_count {
        let f: vk::PFN_vkCmdSetScissorWithCount = transmute(d.scc.or(d.scc_ext));
        f(cb, 1, &r);
    } else {
        let f: vk::PFN_vkCmdSetScissor = transmute(d.sc);
        f(cb, 0, 1, &r);
    }
}
/// Full-window single-target draws are the interface: repeat them once per eye.
unsafe fn per_eye(
    cb: vk::CommandBuffer,
    indexed: bool,
    count: u32,
    emit: &mut dyn FnMut(),
) -> bool {
    let c = match st().lock().unwrap().cbs.get(&cb.as_raw()) {
        Some(c) => *c,
        None => return false,
    };
    let (w, h) = c.area;
    if c.colors != 1 || c.fmt != 64 || w < 2 || c.vp != (0, h as i32, w as i32, -(h as i32)) {
        return false;
    }
    // Full-screen triangles (colour output, background dimming) stay full size, drawn once.
    if count == 3 || (!indexed && count <= 6) {
        return false;
    }
    let d = dev(cb.as_raw());
    let (ew, fh) = ((w / 2) as f32, h as f32);
    let half = disparity() / 2.0;
    for e in 0..2u32 {
        let lo = e as f32 * ew;
        let ox = lo + if e == 0 { half } else { -half };
        let oy = fh * 0.25;
        let v = vk::Viewport {
            x: ox,
            y: oy + fh * 0.5,
            width: ew,
            height: -fh * 0.5,
            min_depth: c.vpv.min_depth,
            max_depth: c.vpv.max_depth,
        };
        let (sx, sy, sw, sh) = (
            c.rect.offset.x as f32,
            c.rect.offset.y as f32,
            c.rect.extent.width as f32,
            c.rect.extent.height as f32,
        );
        let l = (ox + sx * 0.5).clamp(lo, lo + ew);
        let r = (ox + (sx + sw) * 0.5).clamp(lo, lo + ew);
        let t = (oy + sy * 0.5).clamp(0.0, fh);
        let b = (oy + (sy + sh) * 0.5).clamp(0.0, fh);
        let rect = vk::Rect2D {
            offset: vk::Offset2D {
                x: l.floor() as i32,
                y: t.floor() as i32,
            },
            extent: vk::Extent2D {
                width: (r.ceil() - l.floor()).max(0.0) as u32,
                height: (b.ceil() - t.floor()).max(0.0) as u32,
            },
        };
        apply(cb, d, &c, v, rect);
        emit();
    }
    apply(cb, d, &c, c.vpv, c.rect);
    true
}
fn logging() -> bool {
    static L: OnceLock<bool> = OnceLock::new();
    *L.get_or_init(|| std::env::var_os("FLAT3D_UI_PROBE_LOG").is_some())
}
fn note(cb: vk::CommandBuffer, verts: u64) {
    if !logging() {
        return;
    }
    let mut g = st().lock().unwrap();
    let s = &mut *g;
    s.seq += 1;
    let seq = s.seq;
    let c = s.cbs.entry(cb.as_raw()).or_default();
    let k: Key = (c.area, c.colors, c.fmt, c.vp);
    let fresh = c.fresh;
    c.fresh = false;
    let v = s.hist.entry(k).or_default();
    v.draws += 1;
    v.verts += verts;
    v.last = seq;
    if fresh {
        v.passes += 1;
    }
}
unsafe extern "system" fn draw(cb: vk::CommandBuffer, vc: u32, ic: u32, fv: u32, fi: u32) {
    note(cb, vc as u64 * ic as u64);
    let f: vk::PFN_vkCmdDraw = transmute(dev(cb.as_raw()).draw);
    if !per_eye(cb, false, vc, &mut || f(cb, vc, ic, fv, fi)) {
        f(cb, vc, ic, fv, fi)
    }
}
unsafe extern "system" fn drawi(
    cb: vk::CommandBuffer,
    ic: u32,
    inst: u32,
    fi: u32,
    vo: i32,
    fin: u32,
) {
    note(cb, ic as u64 * inst as u64);
    let f: vk::PFN_vkCmdDrawIndexed = transmute(dev(cb.as_raw()).drawi);
    if !per_eye(cb, true, ic, &mut || f(cb, ic, inst, fi, vo, fin)) {
        f(cb, ic, inst, fi, vo, fin)
    }
}
unsafe extern "system" fn view(
    d: vk::Device,
    ci: *const vk::ImageViewCreateInfo<'_>,
    a: *const vk::AllocationCallbacks<'_>,
    out: *mut vk::ImageView,
) -> vk::Result {
    let f: vk::PFN_vkCreateImageView = transmute(dev(d.as_raw()).view);
    let r = f(d, ci, a, out);
    if r == vk::Result::SUCCESS {
        st().lock()
            .unwrap()
            .views
            .insert((*out).as_raw(), (*ci).format.as_raw());
    }
    r
}
unsafe extern "system" fn present(q: vk::Queue, i: *const vk::PresentInfoKHR<'_>) -> vk::Result {
    {
        let mut g = st().lock().unwrap();
        let s = &mut *g;
        s.frames += 1;
        s.seq = 0;
        if logging() && s.frames.is_multiple_of(240) {
            let mut v: Vec<_> = s.hist.iter().map(|(k, x)| (*k, *x)).collect();
            v.sort_by_key(|e| std::cmp::Reverse(e.1.last));
            let mut o = format!(
                "== frame {} keys {}\narea colors fmt viewport(x,y,w,h) passes draws verts lastseq\n",
                s.frames,
                v.len()
            );
            for (k, x) in v.iter().take(45) {
                o += &format!(
                    "{:?} {} {} {:?} {} {} {} {}\n",
                    k.0, k.1, k.2, k.3, x.passes, x.draws, x.verts, x.last
                );
            }
            log(&o);
            s.hist.clear();
        }
    }
    let f: vk::PFN_vkQueuePresentKHR = transmute(dev(q.as_raw()).present);
    f(q, i)
}
unsafe fn hooked(n: &[u8]) -> Void {
    match n {
        b"vkCmdBeginRendering" => hook!(begin),
        b"vkCmdBeginRenderingKHR" => hook!(begin_khr),
        b"vkCmdSetViewport" => hook!(vp),
        b"vkCmdSetViewportWithCount" => hook!(vpc),
        b"vkCmdSetViewportWithCountEXT" => hook!(vpc_ext),
        b"vkCmdDraw" => hook!(draw),
        b"vkCmdDrawIndexed" => hook!(drawi),
        b"vkQueuePresentKHR" => hook!(present),
        b"vkCreateImageView" => hook!(view),
        b"vkCmdSetScissor" => hook!(sc),
        b"vkCmdSetScissorWithCount" => hook!(scc),
        b"vkCmdSetScissorWithCountEXT" => hook!(scc_ext),
        _ => None,
    }
}
#[no_mangle]
pub unsafe extern "system" fn vkGetDeviceProcAddr(d: vk::Device, name: *const c_char) -> Void {
    if !enabled() {
        return None;
    }
    let n = CStr::from_ptr(name).to_bytes();
    if n == b"vkGetDeviceProcAddr" {
        return hook!(vkGetDeviceProcAddr);
    }
    let &g = gdpas().read().unwrap().get(&key(d.as_raw()))?;
    let g: vk::PFN_vkGetDeviceProcAddr = transmute(g);
    let next = g(d, name);
    if next.is_some() {
        hooked(n).or(next)
    } else {
        None
    }
}
#[no_mangle]
pub unsafe extern "system" fn vkGetInstanceProcAddr(i: vk::Instance, name: *const c_char) -> Void {
    if !enabled() {
        return None;
    }
    let n = CStr::from_ptr(name).to_bytes();
    match n {
        b"vkGetInstanceProcAddr" => return hook!(vkGetInstanceProcAddr),
        b"vkGetDeviceProcAddr" => return hook!(vkGetDeviceProcAddr),
        b"vkCreateInstance" => return hook!(create_instance),
        b"vkCreateDevice" => return hook!(create_device),
        _ => {}
    }
    if i == vk::Instance::null() {
        return None;
    }
    let &g = insts().read().unwrap().get(&key(i.as_raw()))?;
    let g: vk::PFN_vkGetInstanceProcAddr = transmute(g);
    let next = g(i, name);
    if next.is_some() {
        hooked(n).or(next)
    } else {
        None
    }
}
