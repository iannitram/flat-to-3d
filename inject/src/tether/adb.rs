//! Choosing the adb executable shared by the tether and link service.

use crate::api::AdbSource;

pub fn listener_pid(ss_output: &str, port: u16) -> Option<u32> {
    let suffix = format!(":{port}");
    for line in ss_output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 || !fields[3].ends_with(&suffix) {
            continue;
        }
        if let Some(pid_start) = line.find("pid=") {
            let digits: String = line[pid_start + 4..]
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .collect();
            if let Ok(pid) = digits.parse() {
                return Some(pid);
            }
        }
    }
    None
}

pub fn proc_exe_path(pid: u32) -> String {
    format!("/proc/{pid}/exe")
}

pub fn tools_dir(home: &str) -> String {
    format!("{home}/.local/share/flat3d/bin")
}

pub fn bundled_adb(home: &str) -> String {
    format!("{}/adb", tools_dir(home))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdbChoice {
    pub path: String,
    pub source: AdbSource,
}

pub fn choose_adb(running_exe: Option<&str>, bundled: Option<&str>) -> AdbChoice {
    if let Some(path) = running_exe.filter(|path| !path.is_empty()) {
        return AdbChoice {
            path: path.to_string(),
            source: AdbSource::Running,
        };
    }
    if let Some(path) = bundled.filter(|path| !path.is_empty()) {
        return AdbChoice {
            path: path.to_string(),
            source: AdbSource::Bundled,
        };
    }
    AdbChoice {
        path: String::new(),
        source: AdbSource::Missing,
    }
}
