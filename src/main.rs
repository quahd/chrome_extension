use std::fs;
use std::error::Error;
use serde::{Deserialize};
use serde_json::{Value as JSonValue, value};
extern crate windows_service;
use std::ffi::OsString;
use std::time::Duration;
use windows_service::{service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, 
    ServiceType, ServiceAccess}, service_manager::{ServiceManager, ServiceManagerAccess}
};
use tokio::{join, runtime::Builder};
use std::collections::HashMap;

use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
#[derive(Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub age: Option<u32>,
    pub product: Option<Product>,
    pub address: Address,
}

#[derive(Deserialize, Debug)]
pub struct Product {
    pub title: String,
    pub price: i64,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    pub city: String,
    pub street: String,
}
use tokio::runtime::Runtime;
use std::thread;
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
fn main() -> Result<(), Box<dyn Error>> {
    let _ = getExtensionProperties(); 
//     let result = get_status_services();
//     println!("result: {:#?}", result);
   //let rt = Runtime::new().unwrap();
//    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
//    let handle = thread::spawn(move || {
//         rt.block_on(async {
//             // Simulate a long blocking operation
//             println!("Blocking task started");
//             let _result = long_computation_7().await;
//             let _re = long_computation().await;
//             println!("Blocking task completed");
//             let (f,s) = tokio::join!(long_computation(), long_computation_7());
//             // chỉ dùng trong async context, nó để chờ các future hoàn thành
//         });
//     }); 
//     println!("Main thread continues to run while blocking task is in progress");
//     handle.join().unwrap();
    // let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    // rt.block_on(main_task());
    Ok(())
}
fn getExtensionProperties()  {
    let mut str = r#"{
        "name": "Demo Extension",
        "update_url": "https://example.com/updates.xml",
        "version": "1.2.3",
        "author": "Nguyen Van A",
        "default_locale": "en",
        "current_locale": "vi",
        "description": "This is a demo manifest for testing getExtensionProperties.",
        "key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtestkeyonlyfortesting",

        "background": {
            "persistent": "have"
        },

        "permissions": [
            "storage",
            "tabs",
            "https://*.example.com/*"
        ],

        "optional_permissions": [
            "bookmarks",
            "history"
        ]
    }"#;
    let parsed_manifest = match serde_json::from_str(str){
        Ok(_t) => _t,
        Err(_e) => {
            JSonValue::Null
        }
    };
    static LIST: &[ExtensionProperty] = &[
        ExtensionProperty { ty: PropertyType::String,      path: "name",                  name: "name" },
        ExtensionProperty { ty: PropertyType::String,      path: "update_url",            name: "update_url" },
        ExtensionProperty { ty: PropertyType::String,      path: "version",               name: "version" },
        ExtensionProperty { ty: PropertyType::String,      path: "author",                name: "author" },
        ExtensionProperty { ty: PropertyType::String,      path: "default_locale",        name: "default_locale" },
        ExtensionProperty { ty: PropertyType::String,      path: "current_locale",        name: "current_locale" },
        ExtensionProperty { ty: PropertyType::String,      path: "background.persistent", name: "persistent" },
        ExtensionProperty { ty: PropertyType::String,      path: "description",           name: "description" },
        ExtensionProperty { ty: PropertyType::StringArray, path: "permissions",           name: "permissions" },
        ExtensionProperty { ty: PropertyType::StringArray, path: "optional_permissions",  name: "optional_permissions" },
        ExtensionProperty { ty: PropertyType::String,      path: "key",                   name: "key" },
    ];
    let mut properties: HashMap<String, String> = HashMap::new();
    for property in LIST {
        let arr_path:Vec<&str> = property.path.split(".").collect();
        let mut opt_node:JSonValue = parsed_manifest.clone();
        for item in arr_path {
            opt_node = match opt_node.get(item){
                Some(value) => value.clone(),
                None => {
                    println!("không tồn tại {:?}", item);
                    JSonValue::Null
                }
            };
        }
        // println!("{:?} :: {:?}", &property.path, opt_node);
        if property.ty == PropertyType::String {
  // đoạn này cần test thêm case không là string vd false, 1001
            match opt_node.clone() {
                JSonValue::String(value) => {
                    if !value.is_empty() {
                        properties.insert(property.clone().name.to_string(), value);
                    }else{
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            }
        }else if property.ty == PropertyType::StringArray {
            let mut list_value = String::new();
            match &opt_node {
                JSonValue::Array(arr) => {
                    for p in arr {
                        match p {
                            JSonValue::String(child_node) => {
                                list_value.push_str(", ");
                                list_value.push_str(&child_node);
                            },
                            _ => {
                                continue;
                            }
                        }
                    }
                    properties.insert(property.clone().name.to_string(), list_value);
                         // Also provide the json-encoded value
                        let list_value_json = match serde_json::to_string(&opt_node){
                        Ok(vl_json) => vl_json,
                        Err(_e) => {
                            let err = format!("lỗi khi chuyển nodes thành json  {:?}", _e);
                            continue;
                        }
                    };
                    let name_arr = format!("{}{}", property.clone().name, "_json");
                    properties.insert(name_arr.to_string(), list_value_json);
                },
                _ => {}              
            }
        }else{
            let err = format!("Invalid property type specified");
            continue;
        }
    }
    println!("------------------------------------------------------------------------------");
    println!("{:#?}", properties);
    println!("------------------------------------------------------------------------------");
}
async fn main_task() {
    println!("Main task started");
    let result = tokio::join!(quick_task(), quick_task());
    println!("Main task result: {:?}", result);
}

async fn quick_task() -> &'static str {
    println!("Quick task running");
    // A short non-blocking delay
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    "Done"
}
async fn long_computation_7() -> &'static str {
    // Emulates a long computation
    tokio::time::sleep(std::time::Duration::from_secs(7)).await;
    println!("Long computation finished 7s");
    "7"
}
async fn long_computation() -> &'static str {
    // Emulates a long computation
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    println!("Long computation finished 5s");
    "5"
}
fn get_status_services() -> Vec<String> {
    let mut status_services: Vec<String> = Vec::new();
    let manager = match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT) {
            Ok(manager) => manager,
            Err(_err) => {
                return status_services
            }
        };
    let services = vec!["CMCManager".to_string(), 
    "CMCMngService".to_string(), "CMCEDRLogClr".to_string(), "CMCDrvMngService".to_string(), "CMCUpdaterService".to_string()];
    for service in services.iter(){
        let service_cur  = match manager.open_service(service,
                                         windows_service::service::ServiceAccess::QUERY_STATUS | windows_service::service::ServiceAccess::STOP | 
                                         windows_service::service::ServiceAccess::START) {
            Ok(service) => service,
            Err(_err) => {
                println!("open_service for {} error: {:?}", service , _err);
                continue;
            }
        };
        let status = match service_cur.query_status(){
            Ok(status) => status.current_state,
            Err(_err) => {
                println!("get_status_services for {} error: {:?}", service , _err);
                continue;
            }
        };
        match status {
            ServiceState::Running => {
                println!("Service {} is running", service);
                match service_cur.stop(){
                    Ok(_) => println!("Service {} stop command sent successfully", service),
                    Err(err) => println!("err to stop service {}: {:?}", service, err),
                };
                
            },
            _ => println!("Service {} is in state: {:?}", service, status),
        }
        status_services.push(format!("{:?}",status));
    }
    status_services
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    String,
    StringArray,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionProperty {
    pub ty: PropertyType,
    pub path: &'static str,
    pub name: &'static str,
}

pub type ExtensionPropertyMap = Vec<ExtensionProperty>;