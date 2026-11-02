//! The link service's command words. docs/code-map.md, API shape.
//!
//! Parsing only. `parse_link` is called first by the command line; it answers `None` for
//! any word that is not ours, so the caller can try its own commands.

/// A setting the link service can switch on or off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingKey {
    /// Automatic stereo injection (`auto-3d`).
    Auto3d,
    /// The headset tunnel (`tunnel`).
    Tunnel,
}

impl SettingKey {
    /// The word used on the command line and in the Decky panel.
    pub fn as_str(&self) -> &'static str {
        match self {
            SettingKey::Auto3d => "auto-3d",
            SettingKey::Tunnel => "tunnel",
        }
    }
}

/// A parsed link command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkCommand {
    /// Watch for games starting and stopping.
    Watch,
    /// Take a snapshot of the link state.
    Snapshot,
    /// Install the link service.
    Install,
    /// Remove the link service.
    Uninstall,
    /// Read whether a game's stereo switch is on.
    SwitchGet { appid: u32 },
    /// Turn a game's stereo switch on or off.
    SwitchSet { appid: u32, on: bool },
    /// Read a setting.
    SettingGet { key: SettingKey },
    /// Turn a setting on or off.
    SettingSet { key: SettingKey, on: bool },
    /// Run a game's command words; `argv` is kept as given.
    Run { appid: u32, argv: Vec<String> },
}

/// Parse the arguments after the program name. `None` when the first word is not one of
/// ours (or there are no words); `Some(Err)` carries a usage message for a bad form.
pub fn parse_link(args: &[String]) -> Option<Result<LinkCommand, String>> {
    match args.first().map(String::as_str) {
        Some("watch") => Some(Ok(LinkCommand::Watch)),
        Some("snapshot") => Some(Ok(LinkCommand::Snapshot)),
        Some("install") => Some(Ok(LinkCommand::Install)),
        Some("uninstall") => Some(Ok(LinkCommand::Uninstall)),
        Some("switch") => Some(parse_switch(&args[1..])),
        Some("setting") => Some(parse_setting(&args[1..])),
        Some("run") => Some(parse_run(&args[1..])),
        _ => None,
    }
}

/// A usage message for the given form, e.g. `switch set <appid> on|off`.
fn usage(form: &str) -> String {
    format!("usage: flat3d-inject {form}")
}

fn parse_appid(word: &str) -> Option<u32> {
    word.parse().ok()
}

fn parse_on_off(word: &str) -> Option<bool> {
    match word {
        "on" => Some(true),
        "off" => Some(false),
        _ => None,
    }
}

fn parse_key(word: &str) -> Option<SettingKey> {
    match word {
        "auto-3d" => Some(SettingKey::Auto3d),
        "tunnel" => Some(SettingKey::Tunnel),
        _ => None,
    }
}

fn parse_switch(rest: &[String]) -> Result<LinkCommand, String> {
    match rest {
        [verb, appid] if verb == "get" => parse_appid(appid)
            .map(|appid| LinkCommand::SwitchGet { appid })
            .ok_or_else(|| usage("switch get <appid>")),
        [verb, appid, state] if verb == "set" => match (parse_appid(appid), parse_on_off(state)) {
            (Some(appid), Some(on)) => Ok(LinkCommand::SwitchSet { appid, on }),
            _ => Err(usage("switch set <appid> on|off")),
        },
        _ => Err(usage("switch get <appid> | switch set <appid> on|off")),
    }
}

fn parse_setting(rest: &[String]) -> Result<LinkCommand, String> {
    match rest {
        [verb, key] if verb == "get" => parse_key(key)
            .map(|key| LinkCommand::SettingGet { key })
            .ok_or_else(|| usage("setting get auto-3d|tunnel")),
        [verb, key, state] if verb == "set" => match (parse_key(key), parse_on_off(state)) {
            (Some(key), Some(on)) => Ok(LinkCommand::SettingSet { key, on }),
            _ => Err(usage("setting set auto-3d|tunnel on|off")),
        },
        _ => Err(usage(
            "setting get auto-3d|tunnel | setting set auto-3d|tunnel on|off",
        )),
    }
}

fn parse_run(rest: &[String]) -> Result<LinkCommand, String> {
    const FORM: &str = "run <appid> -- <command words...>";
    match rest {
        [appid, dashes, words @ ..] if dashes == "--" && !words.is_empty() => match parse_appid(appid) {
            Some(appid) => Ok(LinkCommand::Run {
                appid,
                argv: words.to_vec(),
            }),
            None => Err(usage(FORM)),
        },
        _ => Err(usage(FORM)),
    }
}
