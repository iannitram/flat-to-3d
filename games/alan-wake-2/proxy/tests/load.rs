//! Loads the built `version.dll` and checks that it forwards and logs (#293).
#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::time::SystemTime;

use windows_sys::Win32::Foundation::{FreeLibrary, HMODULE};
use windows_sys::Win32::Storage::FileSystem::GetFileVersionInfoSizeW;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain([0]).collect()
}

/// The `version.dll` that this build produced. Searches the test exe's ancestors for
/// `version.dll`, and cargo's build directory (`build/flat3d-aw2-proxy/<hash>/out/`);
/// the newest copy wins.
fn built_dll() -> PathBuf {
    let exe = std::env::current_exe().expect("test exe path");
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    for dir in exe.ancestors() {
        let direct = dir.join("version.dll");
        if direct.is_file() {
            found.push((SystemTime::UNIX_EPOCH, direct));
        }
        if let Ok(entries) = std::fs::read_dir(dir.join("build").join("flat3d-aw2-proxy")) {
            for entry in entries.flatten() {
                let dll = entry.path().join("out").join("version.dll");
                if let Some(modified) = std::fs::metadata(&dll).ok().and_then(|m| m.modified().ok())
                {
                    found.push((modified, dll));
                }
            }
        }
    }
    found
        .into_iter()
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
        .unwrap_or_else(|| PathBuf::from("version.dll"))
}

#[test]
fn proxy_forwards_size_query_and_logs_load() {
    let dll = built_dll();
    assert!(dll.exists(), "build the proxy first: {}", dll.display());
    let kernel32 = wide(r"C:\Windows\System32\kernel32.dll");

    // SAFETY: the path is nul-terminated; the handle is checked before use.
    let module: HMODULE = unsafe { LoadLibraryW(wide(dll.to_str().unwrap()).as_ptr()) };
    assert!(
        !module.is_null(),
        "LoadLibraryW failed for {}",
        dll.display()
    );

    // SAFETY: the name is nul-terminated and the export is looked up by name.
    let proc = unsafe { GetProcAddress(module, c"GetFileVersionInfoSizeW".as_ptr().cast()) };
    let proc = proc.expect("proxy exports GetFileVersionInfoSizeW");
    type SizeW = unsafe extern "system" fn(*const u16, *mut u32) -> u32;
    // SAFETY: the export has the documented signature of GetFileVersionInfoSizeW.
    let proxied: SizeW = unsafe { std::mem::transmute(proc) };

    let mut handle_a = 0u32;
    let mut handle_b = 0u32;
    // SAFETY: the path is nul-terminated and both handles are valid out-pointers.
    let (via_proxy, direct) = unsafe {
        (
            proxied(kernel32.as_ptr(), &mut handle_a),
            GetFileVersionInfoSizeW(kernel32.as_ptr(), &mut handle_b),
        )
    };
    assert_ne!(via_proxy, 0, "proxied size query returned 0");
    assert_eq!(
        via_proxy, direct,
        "proxy must answer like the system version.dll"
    );

    // The log sits beside this test exe, and DllMain wrote a line for this process.
    // SAFETY: GetCurrentProcessId has no preconditions.
    let pid = unsafe { GetCurrentProcessId() };
    let exe = std::env::current_exe().unwrap();
    let log = exe.parent().unwrap().join("flat3d-aw2-proxy.log");
    let text = std::fs::read_to_string(&log).expect("log file written beside the test exe");
    let expected = format!(
        "flat3d-aw2-proxy: loaded into {} pid {}",
        exe.display(),
        pid
    );
    assert!(
        text.lines().any(|line| line == expected),
        "no load line for this pid in {}",
        log.display()
    );

    // SAFETY: `module` came from LoadLibraryW above and is freed once.
    unsafe { FreeLibrary(module) };
}
