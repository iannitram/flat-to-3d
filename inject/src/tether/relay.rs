//! Command lists for the gnirehtet relay and Quest-side client.

use super::adb::tools_dir;

pub const RELAY_PORT: u16 = 31416;
pub const CLIENT_PACKAGE: &str = "com.genymobile.gnirehtet";

pub fn relay_path(home: &str) -> String {
    format!("{}/gnirehtet", tools_dir(home))
}

pub fn apk_path(home: &str) -> String {
    format!("{}/gnirehtet.apk", tools_dir(home))
}

pub fn relay_command(relay: &str, serial: &str) -> Vec<String> {
    vec![relay.to_string(), "run".to_string(), serial.to_string()]
}

pub fn client_start_commands() -> Vec<Vec<String>> {
    vec![
        vec![
            "adb".to_string(),
            "reverse".to_string(),
            "localabstract:gnirehtet".to_string(),
            format!("tcp:{RELAY_PORT}"),
        ],
        vec![
            "adb".to_string(),
            "shell".to_string(),
            "am".to_string(),
            "start".to_string(),
            "-n".to_string(),
            format!("{CLIENT_PACKAGE}/.GnirehtetActivity"),
            "-a".to_string(),
            format!("{CLIENT_PACKAGE}.START"),
        ],
    ]
}

pub fn client_stop_command() -> Vec<String> {
    vec![
        "adb".to_string(),
        "shell".to_string(),
        "am".to_string(),
        "start".to_string(),
        "-n".to_string(),
        format!("{CLIENT_PACKAGE}/.GnirehtetActivity"),
        "-a".to_string(),
        format!("{CLIENT_PACKAGE}.STOP"),
    ]
}

pub fn force_stop_command() -> Vec<String> {
    vec![
        "adb".to_string(),
        "shell".to_string(),
        "am".to_string(),
        "force-stop".to_string(),
        CLIENT_PACKAGE.to_string(),
    ]
}

pub fn remove_reverse_command() -> Vec<String> {
    vec![
        "adb".to_string(),
        "reverse".to_string(),
        "--remove".to_string(),
        "localabstract:gnirehtet".to_string(),
    ]
}
