//! A do-nothing `version.dll` proxy for Alan Wake 2 (#293, SPEC M32).
//!
//! On `DLL_PROCESS_ATTACH` it appends one line to `flat3d-aw2-proxy.log` beside the
//! game exe. Every export of the system `version.dll` is forwarded, unchanged, to
//! the system copy, which is loaded once from the system directory on the first call.
//! No hooks, no patching, no threads.
#![cfg(windows)]
#![allow(non_snake_case)]

use std::ffi::c_void;
use std::io::Write;
use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

const DLL_PROCESS_ATTACH: u32 = 1;
const LOG_NAME: &str = "flat3d-aw2-proxy.log";

/// The real system `version.dll` as a module handle, loaded on first use (never in DllMain).
static REAL: OnceLock<usize> = OnceLock::new();

/// A forwarder cannot return an error to the game, so an unusable system directory aborts.
fn fail() -> ! {
    std::process::abort()
}

fn real_module() -> usize {
    *REAL.get_or_init(|| {
        let mut path = vec![0u16; 512];
        // SAFETY: the buffer is `path.len()` UTF-16 units long and the call writes at most that.
        let n = unsafe { GetSystemDirectoryW(path.as_mut_ptr(), path.len() as u32) } as usize;
        let suffix: Vec<u16> = "\\version.dll".encode_utf16().chain([0]).collect();
        if n == 0 || n + suffix.len() > path.len() {
            fail();
        }
        path.truncate(n);
        path.extend(suffix);
        // SAFETY: `path` is a nul-terminated UTF-16 string that outlives the call.
        let module = unsafe { LoadLibraryW(path.as_ptr()) };
        if module.is_null() {
            fail();
        }
        module as usize
    })
}

/// The address of `name` (nul-terminated) in the system `version.dll`.
fn resolve(name: &[u8]) -> usize {
    // SAFETY: `real_module` returns a live module handle; `name` is nul-terminated.
    match unsafe { GetProcAddress(real_module() as _, name.as_ptr()) } {
        Some(function) => function as usize,
        None => fail(),
    }
}

/// Declares each export with its system signature and forwards the call to the real one.
macro_rules! forward {
    ($($name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty;)*) => {
        $(
            #[doc = concat!("Forwards `", stringify!($name), "` to the system `version.dll`.")]
            ///
            /// # Safety
            ///
            /// The arguments are passed on unchanged, so the caller must meet the system export's contract.
            #[no_mangle]
            #[allow(clippy::too_many_arguments)]
            pub unsafe extern "system" fn $name($($arg: $ty),*) -> $ret {
                type Target = unsafe extern "system" fn($($ty),*) -> $ret;
                // SAFETY: the name matches the export, so the address has this signature.
                let target: Target = std::mem::transmute(resolve(concat!(stringify!($name), "\0").as_bytes()));
                target($($arg),*)
            }
        )*
    };
}

forward! {
    GetFileVersionInfoA(file: *const u8, handle: u32, len: u32, data: *mut c_void) -> i32;
    // Undocumented export: its signature is not public, so the eight argument words are
    // passed on as they are (the four register words and the stack words after them).
    GetFileVersionInfoByHandle(a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize, a7: usize) -> usize;
    GetFileVersionInfoExA(flags: u32, file: *const u8, handle: u32, len: u32, data: *mut c_void) -> i32;
    GetFileVersionInfoExW(flags: u32, file: *const u16, handle: u32, len: u32, data: *mut c_void) -> i32;
    GetFileVersionInfoSizeA(file: *const u8, handle: *mut u32) -> u32;
    GetFileVersionInfoSizeExA(flags: u32, file: *const u8, handle: *mut u32) -> u32;
    GetFileVersionInfoSizeExW(flags: u32, file: *const u16, handle: *mut u32) -> u32;
    GetFileVersionInfoSizeW(file: *const u16, handle: *mut u32) -> u32;
    GetFileVersionInfoW(file: *const u16, handle: u32, len: u32, data: *mut c_void) -> i32;
    VerFindFileA(flags: u32, file: *mut u8, win_dir: *mut u8, app_dir: *mut u8, cur_dir: *mut u8, cur_dir_len: *mut u32, dest_dir: *mut u8, dest_dir_len: *mut u32) -> u32;
    VerFindFileW(flags: u32, file: *mut u16, win_dir: *mut u16, app_dir: *mut u16, cur_dir: *mut u16, cur_dir_len: *mut u32, dest_dir: *mut u16, dest_dir_len: *mut u32) -> u32;
    VerInstallFileA(flags: u32, src_file: *const u8, dest_file: *const u8, src_dir: *const u8, dest_dir: *const u8, cur_dir: *const u8, tmp_file: *mut u8, out_flags: *mut u32) -> u32;
    VerInstallFileW(flags: u32, src_file: *const u16, dest_file: *const u16, src_dir: *const u16, dest_dir: *const u16, cur_dir: *const u16, tmp_file: *mut u16, out_flags: *mut u32) -> u32;
    VerLanguageNameA(lang: u32, name: *mut u8, len: u32) -> u32;
    VerLanguageNameW(lang: u32, name: *mut u16, len: u32) -> u32;
    VerQueryValueA(block: *const c_void, sub_block: *const u8, buffer: *mut *mut c_void, len: *mut u32) -> i32;
    VerQueryValueW(block: *const c_void, sub_block: *const u16, buffer: *mut *mut c_void, len: *mut u32) -> i32;
}

/// Appends the load line to the log beside the host exe. Errors are ignored: the game must not notice us.
fn log_load() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };
    // SAFETY: GetCurrentProcessId has no preconditions.
    let pid = unsafe { GetCurrentProcessId() };
    let line = format!(
        "flat3d-aw2-proxy: loaded into {} pid {}\n",
        exe.display(),
        pid
    );
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(LOG_NAME))
    {
        let _ = file.write_all(line.as_bytes());
    }
}

/// The loader calls this with the reason; only process attach does anything.
#[no_mangle]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, _reserved: *mut c_void) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        log_load();
    }
    1
}
