pub type Iptree = serde_json::Value; 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChromeBrowserType {
    #[default]
    GoogleChrome,
    GoogleChromeBeta,
    GoogleChromeDev,
    GoogleChromeCanary,
    Brave,
    Chromium,
    Yandex,
    Opera,
    Edge,
    EdgeBeta,
    Vivaldi,
    Arc,
}

/// Converts the browser type to a printable string
pub fn get_chrome_browser_name(ty: ChromeBrowserType) -> String {
    match ty {
        ChromeBrowserType::GoogleChrome => "chrome",
        ChromeBrowserType::GoogleChromeBeta => "chrome_beta",
        ChromeBrowserType::GoogleChromeDev => "chrome_dev",
        ChromeBrowserType::GoogleChromeCanary => "chrome_canary",
        ChromeBrowserType::Brave => "brave",
        ChromeBrowserType::Chromium => "chromium",
        ChromeBrowserType::Yandex => "yandex",
        ChromeBrowserType::Opera => "opera",
        ChromeBrowserType::Edge => "edge",
        ChromeBrowserType::EdgeBeta => "edge_beta",
        ChromeBrowserType::Vivaldi => "vivaldi",
        ChromeBrowserType::Arc => "arc",
    }
    .to_string()
}    
/// A snapshot of all the important files inside a chrome profile
#[derive(Debug, Clone, Default)]
pub struct ChromeProfileSnapshot {
    /// A single extension found inside the profile
    //  pub struct_extension: Extension,
    
    /// Profile type // ChromeBrowserType::GoogleChrome
    pub chrome_browser_type: ChromeBrowserType,

    /// Absolute path to this profile
    pub path: String,

    /// The contents of the 'Preferences' file
    pub preferences: String,

    /// The contents of the 'Secure Preferences' file
    pub secure_preferences: String,

    /// The user id
    pub uid: String,

    /// A map of all the extensions discovered in the preferences
    pub referenced_extensions: Option<ExtensionMap>,

    /// A map of all the extensions that are not present in the preferences
    pub unreferenced_extensions: Option<ExtensionMap>,
}

/// A single extension found inside the profile (snapshot)
#[derive(Debug, Clone, Default)]
pub struct ChromeProfileSnapshotExtension {
    /// The absolute path to the extension folder
    pub path: String,

    /// The contents of the manifest file
    pub manifest: String,
}

/// A map of extensions where the key identifies the (relative) path
pub type ExtensionMap = HashMap<String, ChromeProfileSnapshotExtension>;

/// A list of chrome profile snapshots
pub type ChromeProfileSnapshotList = Vec<ChromeProfileSnapshot>;

/// A js -> match pair from the content_scripts manifest entry
#[derive(Debug, Clone, Default)]
pub struct ContentScriptsEntry {
    /// The target script
    pub script: String,

    /// The match entry
    pub match_script: String, // "match" is a keyword-like name in many contexts; "mat" avoids confusion.
}

/// A list of content_scripts entries
pub type ContentScriptsEntryList = Vec<ContentScriptsEntry>;

/// A key/value list of properties
pub type PropertiesExtensionChromeProfile = HashMap<String, String>;


#[derive(Debug, Clone, Default)]
pub struct ExtensionChromeProfile {

    /// Absolute path to the extension folder
    pub path : String,

    /// True if this extension is referenced by the profile
    pub referenced : bool , // default is false

    /// Additional settings, only present if this extension
    /// is referenced by the Preferences file
    pub profile_settings : PropertiesExtensionChromeProfile,

    /// Extension properties, taken from the manifest file
    pub properties : PropertiesExtensionChromeProfile,

    /// The full JSON manifest, on a single line
    pub manifest_json : String,

    /// The SHA256 hash of the manifest file
    pub manifest_hash : String,

    /// The 'matches' entries inside 'content_scripts'
    pub content_scripts_matches : ContentScriptsEntryList,

    /// The extension id, computed from the 'key' property
    pub opt_computed_identifier : Option<String>,
}

/// A Chrome profile
#[derive(Debug, Clone, Default)]
pub struct ChromeProfile {
    /// Profile type
    pub chrome_browser_type: ChromeBrowserType, // default is ChromeBrowserType::GoogleChrome

    /// Absolute path to this profile
    pub path: String,

    /// The profile name
    pub name: String,

    /// The user id
    pub uid: String,

    /// A list of extensions associated with this profile
    pub extension_list: Vec<ExtensionChromeProfile>,
}
// ===============================================================================================
/// A single Chrome extension
#[derive(Debug, Clone, Default)]
pub struct ChromeProfileExtension {
    /// A key/value list of properties
    pub type_properties: ChromeProfileExtensionProperties,

    /// Absolute path to the extension folder
    pub path: String,

    /// True if this extension is referenced by the profile
    pub referenced: bool,

    /// Additional settings, only present if this extension
    /// is referenced by the Preferences file
    pub profile_settings: ChromeProfileExtensionProperties,

    /// Extension properties, taken from the manifest file
    pub properties: ChromeProfileExtensionProperties,

    /// The full JSON manifest, on a single line
    pub manifest_json: String,

    /// The SHA256 hash of the manifest file
    pub manifest_hash: String,

    /// The 'matches' entries inside 'content_scripts'
    pub content_scripts_matches: ContentScriptsEntryList,

    /// The extension id, computed from the 'key' property
    pub opt_computed_identifier: Option<String>,
}
// ===============================================================================================

/// A key/value list of properties
pub type ChromeProfileExtensionProperties = HashMap<String, String>;

/// A list of extensions
pub type ChromeProfileExtensionList = Vec<ChromeProfileExtension>;

/// A list of Chrome profiles
pub type ChromeProfileList = Vec<ChromeProfile>;

/// Placeholder QueryContext (replace with your real context type)
#[derive(Debug, Clone, Default)]
pub struct QueryContext;

/// Status-style error (replace with your real error model)
#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("invalid value: {0}")]
    InvalidValue(String),
    #[error("other: {0}")]
    Other(String),
}

/// Equivalent to C++ `Status` returning success/failure.
/// In Rust you typically return `Result<T, E>` instead of using out-params.
pub type Status<T> = Result<T, StatusError>;

/// Returns the list of 'matches' entries inside the 'content_scripts' array
pub fn getExtensionContentScriptsMatches(parsed_manifest: &Iptree) -> ContentScriptsEntryList {
    // Implement parsing logic here
    let _ = parsed_manifest;
    vec![]
}

/// Returns a list of Chrome profiles from the given snapshot
pub fn get_chrome_profiles_from_snapshot_list(
    snapshot_list: &ChromeProfileSnapshotList,
) -> ChromeProfileList {
    let _ = snapshot_list;
    vec![]
}

/// Returns the profile name from the given parsed preferences
pub fn get_profile_name_from_preferences(parsed_preferences: &Iptree) -> Status<String> {
    let _ = parsed_preferences;
    // Ok(name)
    Err(StatusError::Other("not implemented".into()))
}

/// Captures the extension properties from the given parsed manifest
pub fn get_extension_properties(parsed_manifest: &Iptree) -> Status<ChromeProfileExtensionProperties> {
    let _ = parsed_manifest;
    Err(StatusError::Other("not implemented".into()))
}

/// Returns a list of all profiles for Chrome-based browsers
pub fn get_chrome_profiles(context: &QueryContext) -> ChromeProfileList {
    let _ = context;
    vec![]
}

/// Returns the extension's profile settings
pub fn get_extension_profile_settings(
    parsed_preferences: &Iptree,
    extension_path: &str,
    profile_path: &str,
) -> Status<ChromeProfileExtensionProperties> {
    let _ = (parsed_preferences, extension_path, profile_path);
    Err(StatusError::Other("not implemented".into()))
}

/// Parses the given snapshot to create an extension object
pub fn get_extension_from_snapshot(
    snapshot: &ChromeProfileSnapshotExtension,
) -> Status<ChromeProfileExtension> {
    let _ = snapshot;
    Err(StatusError::Other("not implemented".into()))
}

/// Retrieves the localized version of the given string
pub fn get_string_localization(
    parsed_localization: &Iptree,
    string_key: &str,
) -> Status<String> {
    let _ = (parsed_localization, string_key);
    Err(StatusError::Other("not implemented".into()))
}

/// Returns the specified extension property
pub fn get_extension_property(
    extension: &ChromeProfileExtension,
    property_name: &str,
    optional: bool,
    default_value: &str,
) -> String {
    if let Some(v) = extension.properties.get(property_name) {
        return v.clone();
    }

    if optional {
        default_value.to_string()
    } else {
        // In Rust you might prefer Result here; keeping String to match original signature intent
        default_value.to_string()
    }
}

/// Returns the specified extension profile setting
pub fn get_extension_profile_settings_value(
    extension: &ChromeProfileExtension,
    property_name: &str,
) -> String {
    extension
        .profile_settings
        .get(property_name)
        .cloned()
        .unwrap_or_default()
}

/// Conversion error placeholder
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("overflow")]
    Overflow,
}

pub type ExpectedUnixTimestamp = Result<i64, ConversionError>;

/// Converts a timestamp from Webkit to Unix format
pub fn webkit_time_to_unix_timestamp(timestamp: &str) -> ExpectedUnixTimestamp {
    let _ = timestamp;
    Err(ConversionError::InvalidTimestamp("not implemented".into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionKeyError {
    MissingProperty,
    InvalidValue,
    HashingError,
    TransformationError,
}

pub type ExpectedExtensionKey = Result<String, ExtensionKeyError>;

/// Computes the extension id based on the given key
pub fn compute_extension_identifier(extension: &ChromeProfileExtension) -> ExpectedExtensionKey {
    let _ = extension;
    Err(ExtensionKeyError::TransformationError)
}