//! Shipped headset profiles: what a route needs to know about a headset.
//! One file per headset under `headsets/`, in the recipes' `key: value` syntax
//! (`headsets/README.md`); Quest 2 ships first. Automatic detection stays after 2.0.

use std::collections::BTreeMap;
use std::path::Path;

/// Connection types a profile may name.
pub const CONNECTIONS: [&str; 2] = ["usb", "wifi"];
/// The per-eye stream width an unknown headset gets.
pub const DEFAULT_STREAM_WIDTH: u32 = 1440;
/// The per-eye stream height an unknown headset gets.
pub const DEFAULT_STREAM_HEIGHT: u32 = 1440;
/// The stream frame rate an unknown headset gets.
pub const DEFAULT_STREAM_FPS: u32 = 60;
/// The frame-generation multiplier an unknown headset gets; 1 means off.
pub const DEFAULT_FRAME_GENERATION: u32 = 1;
/// Every key a profile file may give; any other key is refused.
pub const KEYS: [&str; 17] = [
    "name",
    "usb-id",
    "adb-product",
    "adb-model",
    "panel-width",
    "panel-height",
    "refresh-rates",
    "field-of-view",
    "field-of-view-note",
    "connections",
    "client-app",
    "client-app-name",
    "stream-width",
    "stream-height",
    "stream-fps",
    "frame-generation",
    "stream-connection",
];
/// A width and a height in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// A USB vendor and product id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsbId {
    pub vendor: u16,
    pub product: u16,
}

impl UsbId {
    pub const fn new(vendor: u16, product: u16) -> Self {
        Self { vendor, product }
    }

    /// Parse `vendor:product`, four hex digits each; upper case is accepted.
    pub fn parse(text: &str) -> Option<Self> {
        let (vendor, product) = text.trim().split_once(':')?;
        if vendor.len() != 4
            || product.len() != 4
            || !vendor.bytes().all(|b| b.is_ascii_hexdigit())
            || !product.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return None;
        }
        Some(Self::new(
            u16::from_str_radix(vendor, 16).ok()?,
            u16::from_str_radix(product, 16).ok()?,
        ))
    }

    /// The lower-case `vendor:product` text used in files.
    pub fn text(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor, self.product)
    }
}

impl std::fmt::Display for UsbId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// What the engine knows about a headset when it looks one up.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Identification {
    /// `usb-id`, when the caller knows it.
    pub usb: Option<UsbId>,
    /// `adb-product`, when `adb devices` gave it.
    pub adb_product: Option<String>,
    /// `adb-model`, when `adb devices` gave it.
    pub adb_model: Option<String>,
}

impl Identification {
    pub fn new(usb: Option<UsbId>, adb_product: Option<&str>, adb_model: Option<&str>) -> Self {
        Self {
            usb,
            adb_product: adb_product.map(str::to_string),
            adb_model: adb_model.map(str::to_string),
        }
    }

    /// True when there is nothing to match on.
    pub fn is_empty(&self) -> bool {
        self.usb.is_none() && self.adb_product.is_none() && self.adb_model.is_none()
    }
}

/// One headset's shipped settings.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadsetProfile {
    /// `name`.
    pub name: String,
    /// `usb-id`.
    pub usb: UsbId,
    /// `adb-product`.
    pub adb_product: String,
    /// `adb-model`.
    pub adb_model: String,
    /// `panel-width` and `panel-height`, per eye.
    pub panel: Resolution,
    /// `refresh-rates`, in Hz, low to high.
    pub refresh_rates: Vec<u32>,
    /// `field-of-view` in degrees, when the maker publishes one.
    pub field_of_view: Option<f32>,
    /// `field-of-view-note`: the source, or why the figure is not given.
    pub field_of_view_note: Option<String>,
    /// `connections`: `usb` and/or `wifi`.
    pub connections: Vec<String>,
    /// `client-app`: the Android package.
    pub client_app: String,
    /// `client-app-name`: the app's name for people.
    pub client_app_name: String,
    /// `stream-width` and `stream-height`, per eye.
    pub stream: Resolution,
    /// `stream-fps`.
    pub stream_fps: u32,
    /// `frame-generation`; 1 means off.
    pub frame_generation: u32,
    /// `stream-connection`: the recommended connection.
    pub stream_connection: String,
    /// False for the safe defaults returned for a headset that is not in the folder.
    pub known: bool,
}

impl HeadsetProfile {
    /// The safe defaults for an unknown headset: 1440x1440 per eye at 60 fps, no frame generation.
    pub fn unknown() -> Self {
        Self {
            name: "Unknown headset".into(),
            usb: UsbId::new(0, 0),
            adb_product: String::new(),
            adb_model: String::new(),
            panel: Resolution::new(0, 0),
            refresh_rates: Vec::new(),
            field_of_view: None,
            field_of_view_note: None,
            connections: Vec::new(),
            client_app: String::new(),
            client_app_name: String::new(),
            stream: Resolution::new(DEFAULT_STREAM_WIDTH, DEFAULT_STREAM_HEIGHT),
            stream_fps: DEFAULT_STREAM_FPS,
            frame_generation: DEFAULT_FRAME_GENERATION,
            stream_connection: "usb".into(),
            known: false,
        }
    }

    pub fn is_known(&self) -> bool {
        self.known
    }

    /// The side-by-side stream size: twice the per-eye width, by the per-eye height.
    pub fn stream_size(&self) -> (u32, u32) {
        (self.stream.width.saturating_mul(2), self.stream.height)
    }
}

/// The side-by-side stream size a route should open: twice the per-eye width, by the per-eye height.
pub fn stream_size(profile: &HeadsetProfile) -> (u32, u32) {
    profile.stream_size()
}

/// Why a headset profile did not parse or load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadsetError {
    /// A line with no `:`.
    BadLine { line: usize, text: String },
    /// A key given twice; `line` is the second one.
    DuplicateKey { key: String, line: usize },
    /// A key the format does not know.
    UnknownKey { key: String, line: usize },
    /// A required key that never appeared.
    MissingKey(String),
    /// A value the key does not allow.
    BadValue { key: String, value: String },
    /// A file whose name is not a profile file name.
    BadFileName(String),
    /// Two files for one identification; file names, sorted.
    DuplicateIdentification {
        identification: String,
        first: String,
        second: String,
    },
    /// A folder or file could not be read.
    Io { path: String, message: String },
    /// A file's own error; `file` is the file name only.
    InFile {
        file: String,
        error: Box<HeadsetError>,
    },
}

impl std::fmt::Display for HeadsetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeadsetError::BadLine { line, text } => {
                write!(f, "line {line}: not `key: value`: {text}")
            }
            HeadsetError::DuplicateKey { key, line } => {
                write!(f, "line {line}: duplicate key `{key}`")
            }
            HeadsetError::UnknownKey { key, line } => {
                write!(f, "line {line}: unknown key `{key}`")
            }
            HeadsetError::MissingKey(key) => write!(f, "missing key `{key}`"),
            HeadsetError::BadValue { key, value } => write!(f, "bad value for `{key}`: `{value}`"),
            HeadsetError::BadFileName(name) => write!(f, "bad file name: {name}"),
            HeadsetError::DuplicateIdentification {
                identification,
                first,
                second,
            } => write!(
                f,
                "identification {identification} in both {first} and {second}"
            ),
            HeadsetError::Io { path, message } => write!(f, "{path}: {message}"),
            HeadsetError::InFile { file, error } => write!(f, "{file}: {error}"),
        }
    }
}

impl std::error::Error for HeadsetError {}

fn bad(key: &str, value: &str) -> HeadsetError {
    HeadsetError::BadValue {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn take(values: &mut BTreeMap<String, String>, key: &str) -> Result<String, HeadsetError> {
    values
        .remove(key)
        .ok_or_else(|| HeadsetError::MissingKey(key.to_string()))
}

fn take_number(values: &mut BTreeMap<String, String>, key: &str) -> Result<u32, HeadsetError> {
    let value = take(values, key)?;
    value.parse::<u32>().map_err(|_| bad(key, &value))
}

fn numbers(value: &str, key: &str) -> Result<Vec<u32>, HeadsetError> {
    let mut out = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        let number = part.parse::<u32>().map_err(|_| bad(key, value))?;
        out.push(number);
    }
    Ok(out)
}

fn words(value: &str, key: &str) -> Result<Vec<String>, HeadsetError> {
    let mut out = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err(bad(key, value));
        }
        out.push(part.to_string());
    }
    Ok(out)
}

/// Parse one profile file's `key: value` text and validate it.
pub fn parse_headset(text: &str) -> Result<HeadsetProfile, HeadsetError> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            return Err(HeadsetError::BadLine {
                line,
                text: trimmed.to_string(),
            });
        };
        let raw_key = raw_key.trim();
        let Some(key) = KEYS.iter().copied().find(|known| *known == raw_key) else {
            return Err(HeadsetError::UnknownKey {
                key: raw_key.to_string(),
                line,
            });
        };
        if values.contains_key(key) {
            return Err(HeadsetError::DuplicateKey {
                key: key.to_string(),
                line,
            });
        }
        let mut value = raw_value.trim();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = &value[1..value.len() - 1];
        }
        values.insert(key.to_string(), value.to_string());
    }

    let name = take(&mut values, "name")?;
    let usb_text = take(&mut values, "usb-id")?;
    let usb = UsbId::parse(&usb_text).ok_or_else(|| bad("usb-id", &usb_text))?;
    let adb_product = take(&mut values, "adb-product")?;
    let adb_model = take(&mut values, "adb-model")?;
    let panel = Resolution::new(
        take_number(&mut values, "panel-width")?,
        take_number(&mut values, "panel-height")?,
    );
    let refresh_rates = numbers(&take(&mut values, "refresh-rates")?, "refresh-rates")?;
    let connections = words(&take(&mut values, "connections")?, "connections")?;
    let client_app = take(&mut values, "client-app")?;
    let client_app_name = take(&mut values, "client-app-name")?;
    let stream = Resolution::new(
        take_number(&mut values, "stream-width")?,
        take_number(&mut values, "stream-height")?,
    );
    let stream_fps = take_number(&mut values, "stream-fps")?;
    let frame_generation = take_number(&mut values, "frame-generation")?;
    let stream_connection = take(&mut values, "stream-connection")?;

    let field_of_view = match values
        .remove("field-of-view")
        .filter(|value| !value.trim().is_empty())
    {
        Some(value) => Some(
            value
                .parse::<f32>()
                .map_err(|_| bad("field-of-view", &value))?,
        ),
        None => None,
    };
    let field_of_view_note = values
        .remove("field-of-view-note")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let profile = HeadsetProfile {
        name,
        usb,
        adb_product,
        adb_model,
        panel,
        refresh_rates,
        field_of_view,
        field_of_view_note,
        connections,
        client_app,
        client_app_name,
        stream,
        stream_fps,
        frame_generation,
        stream_connection,
        known: true,
    };
    check_headset(&profile)?;
    Ok(profile)
}

/// Check a profile's values; reports the first bad key.
pub fn check_headset(profile: &HeadsetProfile) -> Result<(), HeadsetError> {
    if profile.name.trim().is_empty() {
        return Err(bad("name", &profile.name));
    }
    if profile.usb.vendor == 0 || profile.usb.product == 0 {
        return Err(bad("usb-id", &profile.usb.text()));
    }
    for (key, value) in [
        ("adb-product", &profile.adb_product),
        ("adb-model", &profile.adb_model),
        ("client-app", &profile.client_app),
    ] {
        if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
            return Err(bad(key, value));
        }
    }
    if profile.client_app_name.trim().is_empty() {
        return Err(bad("client-app-name", &profile.client_app_name));
    }
    if profile.panel.width == 0 || profile.panel.height == 0 {
        return Err(bad(
            "panel-width",
            &format!("{}x{}", profile.panel.width, profile.panel.height),
        ));
    }
    let refresh_text = profile
        .refresh_rates
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    if profile.refresh_rates.is_empty()
        || profile
            .refresh_rates
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(bad("refresh-rates", &refresh_text));
    }
    if profile.connections.is_empty() {
        return Err(bad("connections", ""));
    }
    for connection in &profile.connections {
        if !CONNECTIONS.contains(&connection.as_str()) {
            return Err(bad("connections", connection));
        }
    }
    if profile
        .connections
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != profile.connections.len()
    {
        return Err(bad("connections", &profile.connections.join(", ")));
    }
    if profile.stream.width == 0 || profile.stream.height == 0 {
        return Err(bad(
            "stream-width",
            &format!("{}x{}", profile.stream.width, profile.stream.height),
        ));
    }
    if profile.stream_fps == 0 {
        return Err(bad("stream-fps", &profile.stream_fps.to_string()));
    }
    if !(1..=8).contains(&profile.frame_generation) {
        return Err(bad(
            "frame-generation",
            &profile.frame_generation.to_string(),
        ));
    }
    if !profile.connections.contains(&profile.stream_connection)
        || !CONNECTIONS.contains(&profile.stream_connection.as_str())
    {
        return Err(bad("stream-connection", &profile.stream_connection));
    }
    if let Some(fov) = profile.field_of_view {
        if !(fov > 0.0 && fov <= 180.0) {
            return Err(bad("field-of-view", &fov.to_string()));
        }
        if profile
            .field_of_view_note
            .as_deref()
            .is_none_or(|note| note.trim().is_empty())
        {
            return Err(HeadsetError::MissingKey("field-of-view-note".into()));
        }
    }
    Ok(())
}

/// Every `.headset` file under `folder`, parsed and checked. Other files are ignored.
/// A malformed file comes back as [`HeadsetError::InFile`] naming it.
pub fn load_headsets(folder: &Path) -> Result<Vec<HeadsetProfile>, HeadsetError> {
    let entries = std::fs::read_dir(folder).map_err(|error| HeadsetError::Io {
        path: folder.display().to_string(),
        message: error.to_string(),
    })?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| HeadsetError::Io {
            path: folder.display().to_string(),
            message: error.to_string(),
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".headset") {
            names.push(name);
        }
    }
    names.sort();
    let mut headsets = Vec::new();
    let mut files: BTreeMap<String, String> = BTreeMap::new();
    for name in names {
        if name == ".headset" {
            return Err(HeadsetError::BadFileName(name));
        }
        let path = folder.join(&name);
        let text = std::fs::read_to_string(&path).map_err(|error| HeadsetError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        let profile = parse_headset(&text).map_err(|error| HeadsetError::InFile {
            file: name.clone(),
            error: Box::new(error),
        })?;
        let key = format!(
            "{} {} {}",
            profile.usb.text(),
            profile.adb_product,
            profile.adb_model
        );
        if let Some(first) = files.get(&key) {
            return Err(HeadsetError::DuplicateIdentification {
                identification: key,
                first: first.clone(),
                second: name,
            });
        }
        files.insert(key, name);
        headsets.push(profile);
    }
    Ok(headsets)
}

/// The profile whose identification matches every field the caller gave.
pub fn find<'a>(
    headsets: &'a [HeadsetProfile],
    identification: &Identification,
) -> Option<&'a HeadsetProfile> {
    if identification.is_empty() {
        return None;
    }
    headsets.iter().find(|profile| {
        identification.usb.is_none_or(|usb| profile.usb == usb)
            && identification
                .adb_product
                .as_deref()
                .is_none_or(|product| profile.adb_product == product)
            && identification
                .adb_model
                .as_deref()
                .is_none_or(|model| profile.adb_model == model)
    })
}

/// Look a headset up; an unknown one comes back as safe defaults marked unknown.
pub fn lookup(headsets: &[HeadsetProfile], identification: &Identification) -> HeadsetProfile {
    if let Some(profile) = find(headsets, identification) {
        return profile.clone();
    }
    let mut profile = HeadsetProfile::unknown();
    if let Some(usb) = identification.usb {
        profile.usb = usb;
    }
    if let Some(product) = &identification.adb_product {
        profile.adb_product = product.clone();
    }
    if let Some(model) = &identification.adb_model {
        profile.adb_model = model.clone();
    }
    profile
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_profile_is_marked_unknown_and_uses_the_safe_defaults() {
        let profile = HeadsetProfile::unknown();
        assert!(!profile.is_known());
        assert_eq!(profile.stream, Resolution::new(1440, 1440));
        assert_eq!(profile.stream_fps, 60);
        assert_eq!(profile.frame_generation, 1);
        assert_eq!(profile.stream_size(), (2880, 1440));
    }

    #[test]
    fn usb_ids_parse_and_print_in_lower_case() {
        assert_eq!(UsbId::parse("2833:5013"), Some(UsbId::new(0x2833, 0x5013)));
        assert_eq!(UsbId::parse("2833:5013").unwrap().text(), "2833:5013");
        assert_eq!(UsbId::parse("283:5013"), None);
        assert_eq!(UsbId::parse("2833-5013"), None);
    }
}
