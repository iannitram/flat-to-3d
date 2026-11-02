//! Install and uninstall plans. docs/code-map.md, Packaging.

pub const UNIT_NAME: &str = "flat3d-link.service";

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

pub struct InstallPlan {
    pub copies: Vec<(String, String)>,
    pub unit_path: String,
    pub unit_text: String,
    pub commands: Vec<Vec<String>>,
}

pub fn install_plan(home: &str, src_dir: &str) -> InstallPlan {
    let bin_dir = bin_dir(home);
    let tools_dir = crate::tether::adb::tools_dir(home);
    let copies = [
        ("flat3d-inject", format!("{bin_dir}/flat3d-inject")),
        ("flat3d-run", format!("{bin_dir}/flat3d-run")),
    ]
    .into_iter()
    .map(|(name, destination)| (format!("{src_dir}/{name}"), destination))
    .chain(["adb", "gnirehtet", "gnirehtet.apk"].into_iter().map(|name| {
        (
            format!("{src_dir}/tools/{name}"),
            format!("{tools_dir}/{name}"),
        )
    }))
    .collect();

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
    }
}

pub struct UninstallPlan {
    pub before: Vec<Vec<String>>,
    pub removes: Vec<String>,
    pub after: Vec<Vec<String>>,
}

pub fn uninstall_plan(home: &str) -> UninstallPlan {
    let bin_dir = bin_dir(home);
    let tools_dir = crate::tether::adb::tools_dir(home);
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
            format!("{tools_dir}/gnirehtet"),
            format!("{tools_dir}/gnirehtet.apk"),
        ],
        after: vec![vec!["systemctl", "--user", "daemon-reload"]
            .into_iter()
            .map(str::to_string)
            .collect()],
    }
}
