use std::ffi::{c_void, OsStr};
use std::mem::{size_of, transmute};
use std::os::windows::ffi::OsStrExt;
use std::thread::sleep;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW,
    MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows_sys::Win32::System::Threading::{
    CreateRemoteThread, OpenProcess, QueryFullProcessImageNameW, WaitForSingleObject,
    PROCESS_CREATE_THREAD, PROCESS_NAME_WIN32, PROCESS_QUERY_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

use crate::args::{self, Args, Target};
use crate::pick::{self, Waiter};

const INJECT_FIX: &str = "check that the dll exists; flat3d-inject setup fetch uevr fetches UEVR";

struct Handle(HANDLE);

impl Handle {
    fn new(handle: HANDLE) -> Option<Self> {
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            None
        } else {
            Some(Self(handle))
        }
    }

    fn get(&self) -> HANDLE {
        self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            // SAFETY: this wrapper owns a valid Win32 handle.
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn wide_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn nul_terminated_string(value: &[u16]) -> String {
    let end = value
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..end])
}

fn process_list() -> Result<Vec<(u32, String)>, u32> {
    // SAFETY: the snapshot API accepts these fixed flags and process id.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let snapshot = Handle::new(snapshot).ok_or_else(|| unsafe { GetLastError() })?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; 260],
    };
    let mut found = Vec::new();
    // SAFETY: entry has its required size and the snapshot remains open here.
    let mut more = unsafe { Process32FirstW(snapshot.get(), &mut entry) } != 0;
    while more {
        let fallback = nul_terminated_string(&entry.szExeFile);
        let path = process_path(entry.th32ProcessID).unwrap_or(fallback);
        found.push((entry.th32ProcessID, path));
        // SAFETY: entry and snapshot are valid for this enumeration.
        more = unsafe { Process32NextW(snapshot.get(), &mut entry) } != 0;
    }
    Ok(found)
}

fn process_path(pid: u32) -> Option<String> {
    // SAFETY: requests query-only access to the specified process.
    let process = Handle::new(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) })?;
    let mut buffer = vec![0u16; 32_768];
    let mut length = buffer.len() as u32;
    // SAFETY: buffer is writable and length describes its capacity.
    let ok = unsafe {
        QueryFullProcessImageNameW(
            process.get(),
            PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &mut length,
        )
    };
    (ok != 0).then(|| String::from_utf16_lossy(&buffer[..length as usize]))
}

fn module_list(pid: u32) -> Result<Vec<String>, u32> {
    let mut last_error = 0;
    for attempt in 0..=5 {
        // SAFETY: the module snapshot is scoped to the selected process id.
        let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) };
        let Some(snapshot) = Handle::new(raw) else {
            // SAFETY: GetLastError reports why the snapshot call failed.
            last_error = unsafe { GetLastError() };
            if last_error == 24 && attempt < 5 {
                sleep(Duration::from_millis(100));
                continue;
            }
            return Err(last_error);
        };
        let mut entry = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            th32ModuleID: 0,
            th32ProcessID: 0,
            GlblcntUsage: 0,
            ProccntUsage: 0,
            modBaseAddr: std::ptr::null_mut(),
            modBaseSize: 0,
            hModule: std::ptr::null_mut(),
            szModule: [0; 256],
            szExePath: [0; 260],
        };
        // SAFETY: entry has its required size and the snapshot remains open here.
        let mut more = unsafe { Module32FirstW(snapshot.get(), &mut entry) } != 0;
        let mut modules = Vec::new();
        while more {
            modules.push(nul_terminated_string(&entry.szModule));
            // SAFETY: entry and snapshot are valid for this enumeration.
            more = unsafe { Module32NextW(snapshot.get(), &mut entry) } != 0;
        }
        return Ok(modules);
    }
    Err(last_error)
}

fn inject_one(pid: u32, dll: &str) -> Result<(), u32> {
    let path = wide_string(dll);
    // SAFETY: the requested rights are needed for remote LoadLibraryW injection.
    let process = Handle::new(unsafe {
        OpenProcess(
            PROCESS_CREATE_THREAD
                | PROCESS_QUERY_INFORMATION
                | PROCESS_VM_OPERATION
                | PROCESS_VM_WRITE
                | PROCESS_VM_READ,
            0,
            pid,
        )
    })
    .ok_or_else(|| unsafe { GetLastError() })?;
    // SAFETY: allocates writable storage in the target process for the UTF-16 path.
    let remote_path = unsafe {
        VirtualAllocEx(
            process.get(),
            std::ptr::null(),
            path.len() * size_of::<u16>(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };
    if remote_path.is_null() {
        return Err(unsafe { GetLastError() });
    }
    let remote_memory = RemoteMemory {
        process: process.get(),
        address: remote_path,
    };
    let mut written = 0usize;
    // SAFETY: local path is readable and remote allocation is large enough.
    let wrote = unsafe {
        WriteProcessMemory(
            process.get(),
            remote_path,
            path.as_ptr().cast(),
            path.len() * size_of::<u16>(),
            &mut written,
        )
    };
    if wrote == 0 || written != path.len() * size_of::<u16>() {
        return Err(unsafe { GetLastError() });
    }

    // SAFETY: kernel32.dll is loaded in this process and the name is NUL-terminated.
    let kernel = unsafe { GetModuleHandleW(windows_sys::w!("kernel32.dll")) };
    if kernel.is_null() {
        return Err(unsafe { GetLastError() });
    }
    // SAFETY: the function name is NUL-terminated and its symbol is requested from kernel32.
    let Some(load_library) = (unsafe { GetProcAddress(kernel, c"LoadLibraryW".as_ptr().cast()) })
    else {
        return Err(unsafe { GetLastError() });
    };
    // SAFETY: LoadLibraryW is an extern-system function taking one UTF-16 path pointer.
    let start: unsafe extern "system" fn(*mut c_void) -> u32 = unsafe {
        transmute::<
            unsafe extern "system" fn() -> isize,
            unsafe extern "system" fn(*mut c_void) -> u32,
        >(load_library)
    };
    // SAFETY: the function and parameter use the Win32 remote-thread ABI.
    let thread = Handle::new(unsafe {
        CreateRemoteThread(
            process.get(),
            std::ptr::null(),
            0,
            Some(start),
            remote_path.cast_const(),
            0,
            std::ptr::null_mut(),
        )
    })
    .ok_or_else(|| unsafe { GetLastError() })?;
    // SAFETY: thread is a valid handle owned by this scope.
    if unsafe { WaitForSingleObject(thread.get(), 60_000) } != WAIT_OBJECT_0 {
        return Err(unsafe { GetLastError() });
    }
    drop(remote_memory);
    Ok(())
}

struct RemoteMemory {
    process: HANDLE,
    address: *mut c_void,
}

impl Drop for RemoteMemory {
    fn drop(&mut self) {
        // SAFETY: address is an allocation from VirtualAllocEx in process.
        unsafe { VirtualFreeEx(self.process, self.address, 0, MEM_RELEASE) };
    }
}

fn fail(token: &str, msg: &str, fix: &str, code: u8) -> std::process::ExitCode {
    eprintln!("{}", args::error_line(token, msg, fix));
    std::process::ExitCode::from(code)
}

pub fn run(args: &Args) -> std::process::ExitCode {
    let pid = match &args.target {
        Target::Pid(pid) => *pid,
        Target::Exe(stem) => {
            let started = Instant::now();
            let mut waiter = Waiter::new(stem);
            loop {
                let processes = match process_list() {
                    Ok(processes) => processes,
                    Err(error) => {
                        return fail(
                            "inject",
                            &format!("process snapshot GetLastError={error}"),
                            INJECT_FIX,
                            args::EXIT_INJECT,
                        );
                    }
                };
                if let Some(pid) = waiter.step(&processes, started.elapsed().as_millis() as u64) {
                    break pid;
                }
                if started.elapsed() >= Duration::from_secs(args.wait_s) {
                    return fail(
                        "no-game",
                        &format!("no {stem}.exe after {}s", args.wait_s),
                        "start the game from Steam, or raise --wait",
                        args::EXIT_NO_GAME,
                    );
                }
                sleep(Duration::from_millis(500));
            }
        }
    };

    sleep(Duration::from_secs(args.delay_s));
    let mut modules = match module_list(pid) {
        Ok(modules) => modules,
        Err(error) => {
            return fail(
                "inject",
                &format!("module snapshot GetLastError={error}"),
                INJECT_FIX,
                args::EXIT_INJECT,
            );
        }
    };
    for dll in &args.dlls {
        if pick::has_module(&modules, dll) {
            continue;
        }
        if let Err(error) = inject_one(pid, dll) {
            return fail(
                "inject",
                &format!("{dll}: GetLastError={error}"),
                INJECT_FIX,
                args::EXIT_INJECT,
            );
        }
        modules = match module_list(pid) {
            Ok(modules) => modules,
            Err(error) => {
                return fail(
                    "inject",
                    &format!("module snapshot GetLastError={error}"),
                    INJECT_FIX,
                    args::EXIT_INJECT,
                );
            }
        };
        if !pick::has_module(&modules, dll) {
            return fail(
                "inject",
                &format!("{dll}: GetLastError=0"),
                INJECT_FIX,
                args::EXIT_INJECT,
            );
        }
    }
    std::process::ExitCode::SUCCESS
}
