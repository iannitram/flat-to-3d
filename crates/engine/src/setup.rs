//! First-time setup checks. docs/code-map.md, Parts.

/// The package that ships the WiVRn server on the Quest.
pub const WIVRN_PACKAGE: &str = "org.meumeu.wivrn.github";

/// The command that lists the headset's packages.
pub fn packages_command() -> Vec<String> {
    ["adb", "shell", "pm", "list", "packages"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Which of the setup apps are installed on the headset. WiVRn is the only one (ruling 42).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packages {
    pub wivrn: bool,
}

/// Read the output of [`packages_command`] and note the packages we need.
pub fn parse_packages(out: &str) -> Packages {
    Packages {
        wivrn: out
            .lines()
            .any(|line| line.trim().strip_prefix("package:") == Some(WIVRN_PACKAGE)),
    }
}

/// The setup apps that are not installed, in the order they are installed.
pub fn missing(p: &Packages) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !p.wivrn {
        missing.push("WiVRn");
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_command_is_adb() {
        assert_eq!(packages_command()[0], "adb");
    }
}
