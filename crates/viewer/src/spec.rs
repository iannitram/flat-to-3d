//! Stable command-line and serialized viewer settings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ViewerSpec {
    pub layout: Layout,
    pub source: Source,
    pub swap_eyes: bool,
    pub crop_aspect: Option<f32>,
    pub screen_width_m: f32,
    pub distance_m: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    #[default]
    Sbs,
    Tab,
    #[serde(rename = "sbs_reversed")]
    SbsRev,
    #[serde(rename = "tab_reversed")]
    TabRev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    #[default]
    Auto,
    Gamescope,
    Portal,
    TestPattern,
}

impl ViewerSpec {
    pub fn from_args(args: &[String]) -> Result<ViewerSpec, String> {
        let mut spec = Self::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--swap-eyes" => {
                    spec.swap_eyes = true;
                    i += 1;
                }
                "--layout" => {
                    let value = value(args, i, "--layout")?;
                    spec.layout = match value {
                        "sbs" => Layout::Sbs,
                        "tab" => Layout::Tab,
                        "sbs_reversed" => Layout::SbsRev,
                        "tab_reversed" => Layout::TabRev,
                        _ => return Err(format!("unknown layout: {value}")),
                    };
                    i += 2;
                }
                "--source" => {
                    let value = value(args, i, "--source")?;
                    spec.source = match value {
                        "auto" => Source::Auto,
                        "gamescope" => Source::Gamescope,
                        "portal" => Source::Portal,
                        "test_pattern" => Source::TestPattern,
                        _ => return Err(format!("unknown source: {value}")),
                    };
                    i += 2;
                }
                "--crop-aspect" => {
                    spec.crop_aspect = Some(parse_float(
                        value(args, i, "--crop-aspect")?,
                        "--crop-aspect",
                    )?);
                    i += 2;
                }
                "--screen-width-m" => {
                    spec.screen_width_m =
                        parse_float(value(args, i, "--screen-width-m")?, "--screen-width-m")?;
                    i += 2;
                }
                "--distance-m" => {
                    spec.distance_m = parse_float(value(args, i, "--distance-m")?, "--distance-m")?;
                    i += 2;
                }
                flag => return Err(format!("unknown flag: {flag}")),
            }
        }
        Ok(spec)
    }

    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "--layout".into(),
            layout_name(self.layout).into(),
            "--source".into(),
            source_name(self.source).into(),
        ];
        if self.swap_eyes {
            args.push("--swap-eyes".into());
        }
        if let Some(aspect) = self.crop_aspect {
            args.push("--crop-aspect".into());
            args.push(aspect.to_string());
        }
        if self.screen_width_m > 0.0 {
            args.push("--screen-width-m".into());
            args.push(self.screen_width_m.to_string());
        }
        if self.distance_m > 0.0 {
            args.push("--distance-m".into());
            args.push(self.distance_m.to_string());
        }
        args
    }
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn parse_float(value: &str, flag: &str) -> Result<f32, String> {
    value
        .parse::<f32>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn layout_name(layout: Layout) -> &'static str {
    match layout {
        Layout::Sbs => "sbs",
        Layout::Tab => "tab",
        Layout::SbsRev => "sbs_reversed",
        Layout::TabRev => "tab_reversed",
    }
}

fn source_name(source: Source) -> &'static str {
    match source {
        Source::Auto => "auto",
        Source::Gamescope => "gamescope",
        Source::Portal => "portal",
        Source::TestPattern => "test_pattern",
    }
}
