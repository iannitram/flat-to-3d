//! Install and uninstall plans. docs/code-map.md, Packaging.

pub const UNIT_NAME: &str = "flat3d-link.service";
const FIX_PLAN: &str = "build the plan with install::install_plan(home, src_dir)";
const FIX_MISSING_BINARY: &str =
    "run flat3d-inject install from the release folder that holds flat3d-inject and flat3d-run";
const FIX_INSTALL_COMMAND: &str =
    "read systemctl --user status flat3d-link.service, then re-run flat3d-inject install";
const FIX_UNINSTALL: &str = "remove that file by hand, then re-run flat3d-inject uninstall";
const FIX_UNINSTALL_COMMAND: &str = "run systemctl --user daemon-reload";
const FIX_UDEV: &str = "re-run flat3d-inject install in a terminal and answer the sudo prompt (SteamOS: set a password with passwd first)";
const FIX_UDEV_UNINSTALL: &str =
    "run sudo rm /etc/udev/rules.d/70-flat3d-quest.rules, then sudo udevadm control --reload-rules";

/// Flat to 3D's own udev rule (ruling 42): the logged-in user may open Meta USB
/// devices, so adb finds a Quest without Quest Tether's rule.
pub const UDEV_RULE_TEXT: &str = include_str!("../../../packaging/flat3d-quest.rules");
/// Where install puts the rule. It sorts before 73-seat-late.rules, which turns
/// the `uaccess` tag into the user's access.
pub const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/70-flat3d-quest.rules";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSpec {
    pub description: String,
    pub exec_start: String,
    pub restart_sec: u32,
    pub start_limit_interval_sec: u32,
    pub start_limit_burst: u32,
    pub wanted_by: String,
}

impl Default for UnitSpec {
    fn default() -> Self {
        Self {
            description: "Flat to 3D headset link".into(),
            exec_start: "%h/.local/bin/flat3d-inject watch".into(),
            restart_sec: 3,
            start_limit_interval_sec: 60,
            start_limit_burst: 10,
            wanted_by: "default.target".into(),
        }
    }
}

pub fn unit_text_with(spec: &UnitSpec) -> String {
    format!("[Unit]\nDescription={}\nStartLimitIntervalSec={}\nStartLimitBurst={}\n\n[Service]\nType=simple\nExecStart={}\nRestart=on-failure\nRestartSec={}\n\n[Install]\nWantedBy={}\n", spec.description, spec.start_limit_interval_sec, spec.start_limit_burst, spec.exec_start, spec.restart_sec, spec.wanted_by)
}

pub fn execute(
    plan: &InstallPlan,
    r: &mut dyn crate::exec::CommandRunner,
) -> Result<crate::api::InstallReport, crate::api::ApiError> {
    use std::{fs, path::Path};
    let roots = install_roots(&plan.home);
    let destinations = plan
        .copies
        .iter()
        .map(|(_, d)| d)
        .chain(plan.optional_bins.iter().map(|(_, d)| d))
        .chain(plan.trees.iter().map(|(_, d)| d))
        .chain(std::iter::once(&plan.udev_source))
        .chain(std::iter::once(&plan.unit_path))
        .chain(std::iter::once(&plan.manifest));
    for path in destinations {
        if !inside_roots(path, &roots) {
            return Err(crate::api::ApiError::new(
                "refused",
                &format!("{path} is outside the install folders; fix: {FIX_PLAN}"),
            ));
        }
    }
    for path in &plan.dirs {
        let probe = if path.ends_with('/') {
            path.clone()
        } else {
            format!("{path}/")
        };
        if !inside_roots(&probe, &roots) {
            return Err(crate::api::ApiError::new(
                "refused",
                &format!("{path} is outside the install folders; fix: {FIX_PLAN}"),
            ));
        }
    }
    for (src, _) in &plan.copies {
        if Path::new(src)
            .file_name()
            .is_some_and(|n| n == "flat3d-inject" || n == "flat3d-run")
            && !Path::new(src).is_file()
        {
            return Err(crate::api::ApiError::new(
                "io",
                &format!("missing {src}; fix: {FIX_MISSING_BINARY}"),
            ));
        }
    }
    let mut written = Vec::new();
    let mut notes = Vec::new();
    let operation: Result<(), crate::api::ApiError> = (|| {
        for dir in &plan.dirs {
            fs::create_dir_all(dir).map_err(|e| io_error(dir, &e, IO_INSTALL))?;
        }
        for (src, dst) in plan.copies.iter().take(2) {
            copy_chmod(src, dst, 0o755)?;
            written.push(dst.clone());
        }
        for (src, dst) in plan.copies.iter().skip(2).chain(plan.optional_bins.iter()) {
            let Some(name) = Path::new(src).file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !Path::new(src).is_file() {
                notes.push(name.to_string());
                continue;
            }
            copy_chmod(src, dst, 0o755)?;
            written.push(dst.clone());
        }
        for (src, dst) in &plan.trees {
            if !Path::new(src).is_dir() {
                continue;
            }
            fs::create_dir_all(dst).map_err(|e| io_error(dst, &e, IO_INSTALL))?;
            let entries = fs::read_dir(src).map_err(|e| io_error(src, &e, IO_INSTALL))?;
            for entry in entries {
                let entry = entry.map_err(|e| io_error(src, &e, IO_INSTALL))?;
                if !entry
                    .file_type()
                    .map_err(|e| io_error(&entry.path().display().to_string(), &e, IO_INSTALL))?
                    .is_file()
                {
                    continue;
                }
                if Path::new(src).ends_with("recipes")
                    && entry.path().extension().is_none_or(|e| e != "recipe")
                {
                    continue;
                }
                let dest = format!("{dst}/{}", entry.file_name().to_string_lossy());
                let mode = if Path::new(src).ends_with("recipes") {
                    0o644
                } else {
                    0o755
                };
                copy_chmod(&entry.path().display().to_string(), &dest, mode)?;
                written.push(dest);
            }
        }
        place(&plan.udev_source, 0o644, |tmp| fs::write(tmp, UDEV_RULE_TEXT))?;
        written.push(plan.udev_source.clone());
        let tmp = format!("{}.tmp", plan.unit_path);
        let placed = fs::write(&tmp, unit_text_with(&UnitSpec::default()))
            .map_err(|e| io_error(&tmp, &e, IO_INSTALL))
            .and_then(|()| {
                fs::rename(&tmp, &plan.unit_path)
                    .map_err(|e| io_error(&plan.unit_path, &e, IO_INSTALL))
            });
        if let Err(error) = placed {
            let _ = fs::remove_file(&tmp);
            return Err(error);
        }
        written.push(plan.unit_path.clone());
        Ok(())
    })();
    if let Err(error) = operation {
        if !written.is_empty() {
            let _ = write_manifest(&plan.manifest, &written);
        }
        return Err(error);
    }
    if !notes.is_empty() {
        notes = vec![format!("not bundled: {}", notes.join(", "))];
    }
    write_manifest(&plan.manifest, &written)?;
    written.push(plan.manifest.clone());
    let mut ran = Vec::new();
    for command in &plan.commands {
        let output = r.run(command);
        let argv = command.join(" ");
        ran.push(argv.clone());
        if !output.success {
            return Err(crate::api::ApiError::new(
                "command-failed",
                &format!(
                    "{argv}: {}; fix: {FIX_INSTALL_COMMAND}",
                    last_line(&output.stderr)
                ),
            ));
        }
    }
    // The udev step needs root, so a refused or failed sudo prompt is a Setup
    // warning: the link service is in place, only the rule is missing.
    if !Path::new(&plan.udev_rule).exists() {
        for command in &plan.udev_commands {
            let output = r.run(command);
            ran.push(command.join(" "));
            if !output.success {
                notes.push(format!(
                    "warning: the Quest udev rule is not installed: {}; fix: {FIX_UDEV}",
                    last_line(&output.stderr)
                ));
                break;
            }
        }
    }
    Ok(crate::api::InstallReport {
        written,
        ran,
        summary: notes.join("; "),
    })
}

pub fn execute_uninstall(
    plan: &UninstallPlan,
    r: &mut dyn crate::exec::CommandRunner,
) -> Result<crate::api::InstallReport, crate::api::ApiError> {
    use std::{fs, io::ErrorKind};
    let roots = install_roots(&plan.home);
    let mut written = Vec::new();
    let mut ran = Vec::new();
    let mut notes = Vec::new();
    for command in &plan.before {
        let out = r.run(command);
        let argv = command.join(" ");
        ran.push(argv.clone());
        if !out.success {
            notes.push(format!("tolerated {argv}: {}", last_line(&out.stderr)));
        }
    }
    let mut paths = plan.removes.clone();
    if let Ok(contents) = fs::read_to_string(&plan.manifest) {
        paths.extend(contents.lines().map(str::to_string));
    }
    paths.push(plan.manifest.clone());
    for path in paths {
        if !inside_roots(&path, &roots) {
            notes.push(format!("refused {path}"));
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => written.push(path),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => {
                return Err(crate::api::ApiError::new(
                    "io",
                    &format!("{path}: {e}; fix: {FIX_UNINSTALL}"),
                ))
            }
        }
    }
    if std::path::Path::new(&plan.udev_rule).exists() {
        for command in &plan.udev_commands {
            let out = r.run(command);
            let argv = command.join(" ");
            ran.push(argv.clone());
            if !out.success {
                notes.push(format!(
                    "tolerated {argv}: {}; fix: {FIX_UDEV_UNINSTALL}",
                    last_line(&out.stderr)
                ));
                break;
            }
        }
    }
    for command in &plan.after {
        let out = r.run(command);
        let argv = command.join(" ");
        ran.push(argv.clone());
        if !out.success {
            return Err(crate::api::ApiError::new(
                "command-failed",
                &format!(
                    "{argv}: {}; fix: {FIX_UNINSTALL_COMMAND}",
                    last_line(&out.stderr)
                ),
            ));
        }
    }
    Ok(crate::api::InstallReport {
        written,
        ran,
        summary: notes.join("; "),
    })
}

const IO_INSTALL: &str = "make that folder writable, then re-run flat3d-inject install";
fn io_error(path: &str, error: &std::io::Error, fix: &str) -> crate::api::ApiError {
    crate::api::ApiError::new("io", &format!("{path}: {error}; fix: {fix}"))
}
fn last_line(stderr: &str) -> &str {
    stderr.lines().rfind(|s| !s.trim().is_empty()).unwrap_or("")
}
fn install_roots(home: &str) -> Vec<String> {
    vec![
        format!("{home}/.local/bin/"),
        format!("{home}/.local/share/flat3d/"),
        format!("{home}/.config/systemd/user/"),
    ]
}
fn inside_roots(path: &str, roots: &[String]) -> bool {
    !std::path::Path::new(path)
        .components()
        .any(|c| c == std::path::Component::ParentDir)
        && roots.iter().any(|root| path.starts_with(root))
}
fn write_manifest(path: &str, files: &[String]) -> Result<(), crate::api::ApiError> {
    let mut all: Vec<String> = Vec::new();
    if let Ok(existing) = std::fs::read_to_string(path) {
        all.extend(
            existing
                .lines()
                .filter(|l| !l.is_empty())
                .map(str::to_string),
        );
    }
    for file in files {
        if !all.contains(file) {
            all.push(file.clone());
        }
    }
    let body = if all.is_empty() {
        String::new()
    } else {
        format!("{}\n", all.join("\n"))
    };
    std::fs::write(path, body).map_err(|e| io_error(path, &e, IO_INSTALL))
}
fn copy_chmod(src: &str, dst: &str, mode: u32) -> Result<(), crate::api::ApiError> {
    place(dst, mode, |tmp| std::fs::copy(src, tmp).map(|_| ()))
}
/// Fills `dst.tmp`, sets its mode, then renames it over `dst`.
fn place(
    dst: &str,
    mode: u32,
    fill: impl FnOnce(&str) -> std::io::Result<()>,
) -> Result<(), crate::api::ApiError> {
    let tmp = format!("{dst}.tmp");
    let result = (|| {
        fill(&tmp).map_err(|e| io_error(dst, &e, IO_INSTALL))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(mode))
                .map_err(|e| io_error(dst, &e, IO_INSTALL))?;
        }
        let _ = mode;
        std::fs::rename(&tmp, dst).map_err(|e| io_error(dst, &e, IO_INSTALL))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

pub fn unit_text(exec_start: &str) -> String {
    format!(
        "[Unit]\nDescription=Flat to 3D headset link\n\n[Service]\nType=simple\nExecStart={exec_start}\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n"
    )
}

pub fn unit_path(home: &str) -> String {
    format!("{home}/.config/systemd/user/{UNIT_NAME}")
}

pub fn bin_dir(home: &str) -> String {
    format!("{home}/.local/bin")
}

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub copies: Vec<(String, String)>,
    pub unit_path: String,
    pub unit_text: String,
    pub commands: Vec<Vec<String>>,
    pub home: String,
    pub dirs: Vec<String>,
    pub optional_bins: Vec<(String, String)>,
    pub trees: Vec<(String, String)>,
    pub manifest: String,
    /// The rule's copy inside the install folders, which the sudo command reads.
    pub udev_source: String,
    /// Where the rule goes; the udev step is skipped when this file is already there.
    pub udev_rule: String,
    /// One sudo prompt: put the rule in, reload udev, re-trigger Meta USB devices.
    pub udev_commands: Vec<Vec<String>>,
}

pub fn install_plan(home: &str, src_dir: &str) -> InstallPlan {
    let bin_dir = bin_dir(home);
    let tools_dir = crate::link::adb::tools_dir(home);
    let copies = vec![
        (
            format!("{src_dir}/flat3d-inject"),
            format!("{bin_dir}/flat3d-inject"),
        ),
        (
            format!("{src_dir}/flat3d-run"),
            format!("{bin_dir}/flat3d-run"),
        ),
        (format!("{src_dir}/tools/adb"), format!("{tools_dir}/adb")),
    ];
    let udev_source = udev_source(home);

    InstallPlan {
        copies,
        unit_path: unit_path(home),
        unit_text: unit_text("%h/.local/bin/flat3d-inject watch"),
        commands: vec![
            vec!["systemctl", "--user", "daemon-reload"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            vec!["systemctl", "--user", "enable", "--now", UNIT_NAME]
                .into_iter()
                .map(str::to_string)
                .collect(),
        ],
        home: home.to_string(),
        dirs: vec![
            bin_dir.clone(),
            format!("{tools_dir}/"),
            format!("{home}/.local/share/flat3d/recipes"),
            format!("{home}/.config/systemd/user"),
        ],
        optional_bins: vec![(
            format!("{src_dir}/flat3d-viewer"),
            format!("{bin_dir}/flat3d-viewer"),
        )],
        trees: vec![
            (format!("{src_dir}/tools/lib"), format!("{tools_dir}/lib")),
            (
                format!("{src_dir}/recipes"),
                format!("{home}/.local/share/flat3d/recipes"),
            ),
        ],
        manifest: format!("{home}/.local/share/flat3d/installed.txt"),
        udev_commands: vec![[
            "sudo",
            "sh",
            "-c",
            "install -m 0644 \"$1\" \"$2\" && udevadm control --reload-rules && udevadm trigger --subsystem-match=usb --attr-match=idVendor=2833",
            "sh",
            udev_source.as_str(),
            UDEV_RULE_PATH,
        ]
        .into_iter()
        .map(str::to_string)
        .collect()],
        udev_source,
        udev_rule: UDEV_RULE_PATH.to_string(),
    }
}

fn udev_source(home: &str) -> String {
    format!("{home}/.local/share/flat3d/flat3d-quest.rules")
}

#[derive(Debug, Clone)]
pub struct UninstallPlan {
    pub before: Vec<Vec<String>>,
    pub removes: Vec<String>,
    pub after: Vec<Vec<String>>,
    pub home: String,
    pub manifest: String,
    /// Flat to 3D's udev rule; removed (one sudo prompt) only when it is there.
    pub udev_rule: String,
    pub udev_commands: Vec<Vec<String>>,
}

pub fn uninstall_plan(home: &str) -> UninstallPlan {
    let bin_dir = bin_dir(home);
    let tools_dir = crate::link::adb::tools_dir(home);
    UninstallPlan {
        before: vec![vec!["systemctl", "--user", "disable", "--now", UNIT_NAME]
            .into_iter()
            .map(str::to_string)
            .collect()],
        removes: vec![
            unit_path(home),
            format!("{bin_dir}/flat3d-inject"),
            format!("{bin_dir}/flat3d-run"),
            format!("{tools_dir}/adb"),
            udev_source(home),
        ],
        after: vec![vec!["systemctl", "--user", "daemon-reload"]
            .into_iter()
            .map(str::to_string)
            .collect()],
        home: home.to_string(),
        manifest: format!("{home}/.local/share/flat3d/installed.txt"),
        udev_rule: UDEV_RULE_PATH.to_string(),
        udev_commands: vec![[
            "sudo",
            "sh",
            "-c",
            "rm -f \"$1\" && udevadm control --reload-rules",
            "sh",
            UDEV_RULE_PATH,
        ]
        .into_iter()
        .map(str::to_string)
        .collect()],
    }
}
