use crate::db;
use crate::define::*;
use crate::utils;
use chrono::{Datelike, Local, Utc, Timelike};
use hex::encode_to_slice;
use wmi::{COMLibrary, WMIConnection, Variant};
use windows::Win32::{
    NetworkManagement::{
        IpHelper::{GetIpNetTable, GetIfTable, MIB_IPNETTABLE, MIB_IFTABLE, MIB_IPNET_TYPE_STATIC},
        Dns::{DnsFree, DnsFreeRecordList, DNS_TYPE, DNS_TYPE_A, DNS_TYPE_AAAA, DNS_TYPE_PTR},
    },
    Foundation::{BOOL, ERROR_INSUFFICIENT_BUFFER, NO_ERROR, ERROR_NO_DATA, LUID, CloseHandle},
    Security::Authentication::Identity::{LsaGetLogonSessionData, LsaEnumerateLogonSessions, SECURITY_LOGON_SESSION_DATA, SECURITY_LOGON_TYPE},
    System::SystemInformation::GetTickCount64,
    System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, GetPriorityClass},
};
use widestring::U16CString;
use windows_core::PCWSTR;
use sysinfo::System;
use base64::prelude::*;

use std::path::PathBuf;
use std::{fs::File, io::{self, BufRead}, collections::HashMap, ffi::c_void};
use log_lib::Logger;
use sys_info::{processes::{self, ProcessInfo}, system::{CPUInfo,get_cpu_info}, user::{get_local_accounts}};
use sha2::{Sha256, Digest};
use hex::decode_to_slice;
use std::fs::{self, DirEntry};
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSonValue;
pub fn collect_table_data(table_name: &str) -> Result<(), Error> {
    if table_name == LIST_TABLES[0] {
        collect_arp_cache()
    } else if table_name == LIST_TABLES[1] {
        collect_disk_info()
    } else if table_name == LIST_TABLES[2] {
        collect_dns_cache()
    } else if table_name == LIST_TABLES[3] {
        collect_etc_hosts()
    } else if table_name == LIST_TABLES[4] {
        collect_groups()
    } else if table_name == LIST_TABLES[5] {
        collect_interface_addresses()
    } else if table_name == LIST_TABLES[6] {
        collect_interface_details()
    } else if table_name == LIST_TABLES[7] {
        collect_logical_drives()
    } else if table_name == LIST_TABLES[8] {
        collect_logon_sessions()
    } else if table_name == LIST_TABLES[9] {
        collect_os_version()
    } else if table_name == LIST_TABLES[10] {
        collect_process_open_sockets()
    } else if table_name == LIST_TABLES[11] {
        collect_processes()
    } else if table_name == LIST_TABLES[12] {
        collect_programs()
    } else if table_name == LIST_TABLES[13] {
        collect_scheduled_tasks()
    } else if table_name == LIST_TABLES[14] {
        collect_services()
    } else if table_name == LIST_TABLES[15] {
        collect_startup_items()
    } else if table_name == LIST_TABLES[16] {
        collect_system_info()
    } else if table_name == LIST_TABLES[17] {
        collect_time()
    } else if table_name == LIST_TABLES[18] {
        collect_uptime()
    } else if table_name == LIST_TABLES[19] {
        collect_user_groups()
    } else if table_name == LIST_TABLES[20] {
        collect_user_ssh_keys()
    } else if table_name == LIST_TABLES[21] {
        collect_users()
    } else if table_name == LIST_TABLES[22] {
        collect_cpu_info()
    } else if table_name == LIST_TABLES[23] {
        collect_firefox_addons()
    } else if table_name == LIST_TABLES[24] {
        collect_chrome_extensions()
    }else{
        Err(Error::Other(format!("Table {} not found", table_name)))
    }
}
fn collect_cpu_info() -> Result<(), Error> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;
    let cpu_info: Vec<Win32_Processor> = wmi_con.query()?;
    if cpu_info.len() > 0 {
        return db::update_cpu_info(&cpu_info);
    }
    Ok(())
}
fn collect_firefox_addons() -> Result<(), Error> {
    let kFirefoxPaths = vec!["AppData\\Roaming\\Mozilla\\Firefox\\Profiles"];
    let users = get_local_accounts();
    let mut result = Vec::new();
    for user in users {
        if user.user_id > 0 && !user.profile_image_path.is_none() {
            let mut profiles:Vec<String> = Vec::new();
            let path_dir = user.profile_image_path.unwrap();
            for path in kFirefoxPaths.iter() {
                let directory = format!("{}{}{}", path_dir ,  "\\"  , path);
                let directory = std::path::Path::new(&directory);
                if directory.is_dir() {
                    for entry in fs::read_dir(directory)? {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_dir() {
                            profiles.push(path.to_str().unwrap().to_string());
                        }
                    }
                }
            }
            profiles.retain( |path| {
                !path.ends_with("Crash Reports") ||  !path.ends_with("Pending Pings")
            });
            for profile in profiles {
                let result_temp =  genFirefoxAddonsFromExtensions(user.user_id, profile);
                result.extend(result_temp);
            }
        }

    }
    println!("==============================================================");
    println!("{:#?}", result);
    let mut firefox = Vec::new();
    for map in result {
        let mut row = FirefoxAddonRow::new();
        row.uid = map.get("uid").and_then(|s| Some(s.parse::<i64>().ok())).unwrap_or(None);

        row.name = map.get("name").cloned();
        row.identifier = map.get("identifier").cloned();
        row.creator = map.get("creator").cloned();
        row.r#type = map.get("type").cloned();
        row.version = map.get("version").cloned();
        row.description = map.get("description").cloned();
        row.source_url = map.get("source_url").cloned();
        row.location = map.get("location").cloned();
        row.path = map.get("path").cloned();

        row.visible = map.get("visible").and_then(|s| {
            if s == "true" {
                Some(1)
            }else{
                Some(0)
            }
        });
        row.active = map.get("active").and_then(|s| {
            if s == "true" {
                Some(1)
            }else{
                Some(0)
            }
        });
        row.disabled = map.get("disabled").and_then(|s| s.parse::<i32>().ok());
        row.autoupdate = map.get("autoupdate").and_then(|s| s.parse::<i32>().ok());

        firefox.push(row);
    }
    println!("{:#?}", firefox);

    println!("==============================================================");

    if firefox.len() > 0 {
        return db::update_firefox_addons(&firefox);
    }

    Ok(())
}
fn genFirefoxAddonsFromExtensions(uid: u32, profile: String) -> Vec<HashMap<String, String>> {
    let kFirefoxExtensionsFile = "\\extensions.json";
    let kFirefoxAddonKeys:Vec<(String, String)> = vec![
        ("defaultLocale.name".to_string(), "name".to_string()),
        ("id".to_string(), "identifier".to_string()),
        ("type".to_string(), "type".to_string()),
        ("version".to_string(), "version".to_string()),
        ("defaultLocale.creator".to_string(), "creator".to_string()),
        ("defaultLocale.description".to_string(), "description".to_string()),
        ("sourceURI".to_string(), "source_url".to_string()),
        ("visible".to_string(), "visible".to_string()),
        ("active".to_string(), "active".to_string()),
        ("applyBackgroundUpdates".to_string(), "autoupdate".to_string()),
        ("location".to_string(), "location".to_string()),
        ("path".to_string(), "path".to_string()),
    ];
    let extension_path = format!("{}{}", profile,kFirefoxExtensionsFile );
    let mut last_result:Vec<HashMap<String, String>> = Vec::new();
    {
        let content = match std::fs::read_to_string(&extension_path){
            Ok(_t) => { 
                _t 
            },
            Err(_e) => {
                println!("lỗi khi đọc file json {:?}", _e);
                String::new()
            }
        };
        let mut extensions:JSonValue = match serde_json::from_str(&content){
            Ok(_t) => _t,
            Err(_e ) => {
                println!("lỗi parse json {:?}", _e);
                serde_json::Value::Null
            }
        };
        let mut  addons_it:JSonValue = extensions["addons"].take();
        if addons_it.is_null() {
            println!("could not find the 'addons' JSON member");
        }
        match addons_it {
            JSonValue::Array(addons) => {
                for addon in addons {
                    let mut result = HashMap::new();
                    let uid = uid.to_string();
                    result.insert("uid".to_string(), uid.to_string());
                    for (first, second) in &kFirefoxAddonKeys {
                        let mut opt_membert_it = match findNestedMember(first, &addon){
                            Some(_t) => {
                               _t
                            },
                            None => {
                                println!("bị bỏ qua ở addons");
                                continue;
                            }
                        };
                        let s = if opt_membert_it.is_string() {
                            match opt_membert_it {
                                JSonValue::String(vl) => vl.to_string(),
                                _ => String::new(),
                            }
                        } else {
                            opt_membert_it.to_string()
                        };
                        result.insert(second.to_string(), s);
                    }
                    let softDisable = match addon.get("softDisable"){
                        Some(_t) => _t.to_string(),
                        None => String::new()
                    };
                    let appDisabled = match addon.get("appDisabled"){
                        Some(_t) => _t.to_string(),
                        None => String::new()
                    };
                    let userDisabled = match addon.get("userDisabled"){
                        Some(_t) => _t.to_string(),
                        None => String::new()
                    };
                    if userDisabled == "true" || appDisabled == "true" || softDisable == "true" {
                        result.insert("disabled".to_string(), "1".to_string());
                    }else{
                        result.insert("disabled".to_string(), "0".to_string());
                    }
                    last_result.push(result);
                }
            },
            Other => {
                println!("không phải vector");
            }
        }
    }
    last_result
}
fn findNestedMember<'a>(member_name: &str, value: &'a JSonValue) -> Option<&'a JSonValue> {
    let mut current = value;
    let members:Vec<&str> = member_name.split('.').collect();
    
    for key in members {
        if !current.is_object() {
            return None;
        }
        current = match current.get(key) {
            Some(v) => v,
            None => return None,
        };
    }
    Some(current)
}

fn collect_chrome_extensions() -> Result<(), Error> {
    let snapshot_list = getChromeProfileSnapshotList();
    let _chrome_profiles = getChromeProfilesFromSnapshotList(snapshot_list);
    Ok(())
}
// /ChromeProfileList
fn getChromeProfilesFromSnapshotList(snapshot_list: Vec<ChromeProfileSnapshot>) {
    //let mut profile_list = Vec::new();
    for snapshot in snapshot_list {
        println!("snapshot is  {:#?}", &snapshot);
        let mut type_profile = snapshot.chrome_browser_type;
        let mut path = snapshot.clone().path;
        let mut uid = snapshot.clone().uid;

        let mut parse_preference = match serde_json::from_str(&snapshot.clone().preferences){
            Ok(value) => value,
            Err(_e) => {
                println!("err while parse to JSonValue {:?}", _e);
                JSonValue::Null
            }
        };

        if parse_preference.is_null() && !snapshot.preferences.is_empty() {
            println!("Failed to parse the Preferences file of the following profile:  {:?}", &path);
            continue;
        }

        let mut parsed_secure_preferences = match serde_json::from_str(&snapshot.clone().secure_preferences){
            Ok(value) => value,
            Err(_e) => {
                println!("err while parse to JSonValue {:?}", _e);
                JSonValue::Null
            }
        };

        if parsed_secure_preferences.is_null() && !snapshot.clone().secure_preferences.is_empty() {
            println!("Failed to parse the secure_preferences file of the following profile:  {:?}", &path);
            continue;
        }
        
        // Try to get the profile name; the Opera browser does not have it
        // getProfileNameFromPreferences
        let temp = snapshot.clone().referenced_extensions.unwrap();
        for item in temp {
            let mut extension = match getExtensionFromSnapshot(item.1) {
                Ok(ext) => ext,
                Err(_e) => {
                    println!("{:?}", _e);
                    continue;
                }
            };
            let ext_snapshot_path = item.1.clone().path;
            let extension_new = match getExtensionProfileSettings(&mut extension, parse_preference.clone(), ext_snapshot_path, path.clone()){
                Ok(ext) => ext,
                Err(_e) => {
                    println!("{:?}", _e);
                    continue;
                }
            };
        }
//ExtensionChromeProfile
    }

}
fn getExtensionProfileSettings(extension: &mut ExtensionChromeProfile, parsed_preferences: JSonValue, extension_path: String, profile_path: String) -> Result<(), Error>{
    let kExtensionProfileSettingsList = vec!["from_webstore".to_string(), "state".to_string(), "install_time".to_string()];
    let mut profile_settings = HashMap::new();
    let string_name = "extensions.settings".to_string();
    let arr_str:Vec<&str> = string_name.split(".").collect();
    let mut ext_settings = parsed_preferences.clone();
    for item in arr_str {   
        ext_settings = match ext_settings.get(item) {
            Some(value) => value.clone(),
            None => {
                let err ="Failed to locate the extensions.settings node".to_string();
                return Err(Error::Other(err));
            }
        }
    }
    let mut extension_id = String::new();
    let mut extension_obj:JSonValue;

    let ext_settings = match ext_settings.as_object() {
        Some(value) => value.clone(),
        None => {
            let err ="Failed to locate the extensions.settings node".to_string();
            return Err(Error::Other(err));
        }
    };
    for (item, value) in ext_settings {
        let mut key_name = item;
        let mut obj = value.clone();
        let mut opt_ext_path: Option<&str> = obj.get("path").and_then(|v| {
            v.as_str()
        });
        if opt_ext_path.is_some(){
            continue;
        }
        let ext_path = opt_ext_path.unwrap();

        let mut ext_path = std::path::Path::new(&ext_path);
        if !ext_path.is_absolute(){
            let path_str = format!("{}{}{}{}{}", profile_path, "\\".to_string(), "Extensions".to_string(), "\\".to_string(), ext_path.clone());
            ext_path = std::path::Path::new(&path_str);
        }
        let mut canonical_path = match fs::canonicalize(&ext_path){
            Ok(path) => path.to_string_lossy(),
            Err(_e) => {
                ext_path.to_string_lossy()
            }
        };
        if extension_path == canonical_path {
            extension_id = key_name;
            extension_obj = obj;
            break;
        }
    }
    if extension_id.is_empty() {
        let err = format!("Failed to locate the following extension in the preferences:  {:?}", &extension_path );
        return Err(Error::Other(err));
    }
    profile_settings.insert("referenced_identifier".to_string(), extension_id);
    for property_name in kExtensionProfileSettingsList {
        let mut opt_property = match extension_obj.get(property_name) {
            Some(property) => property,
            None => {
                continue;
            }
        };
        if opt_property.is_null() {
            continue;
        }
        let mut property = match opt_property {
            JSonValue::String(property)=> {
                property.to_string()
            },
            _ => {
                continue;
            }
        };
        profile_settings.insert(property_name, property);
    }


    return Err(Error::Other("test".to_string()));
}
fn getProfileNameFromPreferences (parsed_preferences: JSonValue) -> Result<String, Error> {
    
    let opt_profile_node = match parsed_preferences.get("profile"){
        Some(profile) => profile,
        None => {
            let err = format!("not found profile or other err ");
            return Err(Error::Other(err))
        }
    };
    let mut profile_node = match opt_profile_node.get("name"){
        Some(node) => node,
        None => {
            let err = format!("not found name or other err ");
            return Err(Error::Other(err))
        }
    };
    let profile_node = profile_node.to_string();

    Ok(profile_node)
}
fn getExtensionFromSnapshot(snapshot: ChromeProfileSnapshotExtension) -> Result<ExtensionChromeProfile, Error> { // return result
    //let output ; //ChromeProfileExtension
    let path = snapshot.clone().path;
    let manifest_hash = hashFromBuffer(snapshot.clone().manifest);

    let mut parsed_manifest:JSonValue = match serde_json::from_str(&snapshot.manifest) 
    {
        Ok(manifest) => manifest,
        Err(_e) => {
            let err = format!("Failed to parse the Manifest file for the following extension:  {:?}", _e);
            // tạo return sau
            return Err(Error::Other(err));
        }
    };
    let mut properties = match getExtensionProperties(parsed_manifest.clone()) {
        Ok(properties) => properties,
        Err(_e) => {
            let err = format!("Failed to parse the Manifest file for the following extension:  {:?}", _e);
            // retrun về status là lỗi => result error output
            // let temp:PropertiesExtensionChromeProfile = HashMap::new();
            // temp
            return Err(Error::Other(err));
        }
    };
    let mut properties_new = match localizeExtensionProperties(properties.clone(), path.clone()){
        Ok(_t) => _t,
        Err(_e) => {
            println!("failed to process the following extension: {:?}", &path);
            properties.clone()
        }
    }; 
    let content_scripts_matches = getExtensionContentScriptsMatches(parsed_manifest.clone());
    let manifest_json = match serde_json::to_string(&parsed_manifest){
        Ok(_t) => _t,
        Err(_e) => {
            String::new()
        }
    };
    let identifier_exp = match computeExtensionIdentifier(properties.clone()){
        Ok(_t) => _t,
        Err(_e) => {
            println!("{_e:?}");
            String::new()
        }
    };
    let opt_computed_identifier = identifier_exp;
    return Ok(ExtensionChromeProfile {
        path,
        referenced: false,
        profile_settings: HashMap::new(),
        properties,
        manifest_json,
        manifest_hash,
        content_scripts_matches,
        opt_computed_identifier: Some(opt_computed_identifier),
    })
}

fn computeExtensionIdentifier(properties: PropertiesExtensionChromeProfile) -> Result<String, Error> {
    let mut err = String::new();
    let extension_key = match properties.get("key") {
        Some(key) => key,
        None => {
            err =  "The 'key' property is missing from the extension manifest".to_string();
            return Err(Error::Other(err));
        }
    };
    let mut decoded_key = BASE64_STANDARD.decode(extension_key.as_bytes()).map_err(  |_| {
        return Error::Other("The 'key' property of the extension manifest could not be properly base64 decoded".to_string());
    });
    let decode = match decoded_key {
        Ok(_key) => _key,
        Err(_e) => {
            return Err(_e)
        }
    };
     if decode.is_empty() {
        return Err(Error::Other(
            "The 'key' property of the extension manifest could not be properly base64 decoded"
                .to_string(),
        ));
    }
    let hash = Sha256::digest(&decode);

    let mut identifier = String::with_capacity(32);
    for &b in hash[..16].iter() {
        let hi = (b >> 4) & 0x0F;
        let lo = b & 0x0F;
        identifier.push((b'a' + hi) as char);
        identifier.push((b'a' + lo) as char);
    }
    Ok(identifier)
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

fn getExtensionProperties(parsed_manifest: JSonValue) -> Result<PropertiesExtensionChromeProfile, Error>{
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
    Ok(properties)
}
fn localizeExtensionProperties(extension_properties: PropertiesExtensionChromeProfile, path : String)  -> Result<PropertiesExtensionChromeProfile, Error>  {
    let mut locale = extension_properties.get("default_locale").or_else(|| {
        extension_properties.get("current_locale")
    }).cloned().unwrap_or_else(|| "en".to_string());
    let mut check = false;
    for (item, value) in extension_properties.clone() {
        if value.starts_with("__MSG_")  {
            check = true;
            break;
        }
    }
    if check == false {
        return Ok(extension_properties.clone())
    }
    let parsed_localization = getLocalizationData(locale.clone(), path)?; 
 
    let mut extension_properties_new:PropertiesExtensionChromeProfile = HashMap::new();
    for (name, value) in extension_properties.iter() {
        if !value.starts_with("__MSG_"){
            extension_properties_new.insert(name.to_string(), value.to_string());
            continue;
        }
        let mut localized_property_value = match getStringLocalization(&parsed_localization, value.to_string()){
            Ok(string) => string, 
            Err(_e) => {
                value.to_string()
            }
        };
        extension_properties_new.insert(name.to_string(), localized_property_value);
    } 
    return Ok(extension_properties_new.clone())
}   
fn getLocalizationData(locale: String, extension_path: String) -> Result <JSonValue,Error>{
    let mut messages_file_path = std::path::Path::new(&extension_path).join("_locales").join(locale).join("messages.json");
    let mut err = String::new();
    let mut messages_json = match fs::read_to_string(&messages_file_path) {
        Ok(_t)=> _t,
        Err(_e) => {
            err = format!("err while try to read {:?} in getLocalizationData {:?}", &messages_file_path, _e);
            return Err(Error::Other(err))
            //String::new()
        }
    };
    let output = match serde_json::from_str(&messages_json) {
        Ok(_t) => _t,
        Err(_e) => {
            err = format!("err while try to convert string to jsonvalue in getLocalizationData {:?}", _e);
            JSonValue::Null
        }
    };
    if !output.is_null() {
        Ok(output)
    }else{
        return Err(Error::Other(err))
    }
}
fn getStringLocalization(parsed_localization: &JSonValue, property_value: String) -> Result <String, Error>{
    let kLocalizedMessagePrefix = "__MSG_";
    let mut localized_string = property_value.clone();
    let mut name = property_value.clone();
    if !property_value.starts_with(kLocalizedMessagePrefix) {
        return Ok(localized_string)
    }
    let (mut prefix, mut string_name) = name.split_at(6);
    string_name = string_name.strip_suffix("__").unwrap_or(string_name);
    let mut opt_node = match parsed_localization.get(string_name){
        Some(value) => value.clone(),
        _ => {
            let err = format!("No localization found for the following key: {}" , &property_value);
            JSonValue::Null
            //return Err(Error::Other(err))
        }
    };
    if opt_node.is_null() {
        let err = format!("No localization found for the following key: {}" , &property_value);
        return Err(Error::Other(err))
    }
    opt_node = match opt_node.get("message"){
        Some(value) => value.clone(),
        _ => {
            JSonValue::Null
        }
    };
    if opt_node.is_null() {
        let err = format!("No localization message found for the following key: {}" , &property_value);
        return Err(Error::Other(err))
    }
    let localized_string = opt_node.as_str().unwrap();
    return Ok(localized_string.to_string())
}
fn getExtensionContentScriptsMatches(parsed_manifest: JSonValue) -> ContentScriptsEntryList {
    let mut entry_list = vec![];
    println!("parsed_manifest : {:#?}", &parsed_manifest);
    let Some(content_scripts_node) = parsed_manifest.get("content_scripts").and_then(|v| v.as_array()) else {
        return vec![]
    };
    for entry in content_scripts_node {
        let Some(matches_node) = entry.get("matches").and_then(|v| v.as_array()) else { continue;};
        let Some(js_node) = entry.get("js").and_then(|v| v.as_array()) else { continue;};
        for match_entry in matches_node {
            let Some(match_value_str) = match_entry.as_str() else  {continue;};

            for js_entry in js_node {
                let Some(js_entry_str) = js_entry.as_str() else  {continue;};
                entry_list.push(ContentScriptsEntry {
                    match_script: match_value_str.clone().to_string(),
                    script: js_entry_str.clone().to_string()
                });
            }     
        }
    }
    return entry_list
}
fn hashFromBuffer(data: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut result = hasher.finalize();
    let result = hex::encode(result);
    result
}
fn getChromeProfileSnapshotList() -> Vec<ChromeProfileSnapshot> {
    let mut output:Vec<ChromeProfileSnapshot> = Vec::new(); 
    for profile_path in getChromeProfilePathList() {
        println!("=========================================== profile_path ===================================================");
        println!("{:#?}", profile_path);
        println!("=========================================== profile_path ===================================================");
        let mut snapshot = match captureProfileSnapshotSettingsFromPath(profile_path.clone()) {
            Ok(snapshot) => snapshot,
            Err(_e) => {
                println!("err captureProfileSnapshotSettingsFromPath {_e}");
                continue;
            }
        };
        //println!("snapshot {:?} {:?} {:?}", snapshot.uid, snapshot.referenced_extensions, snapshot.unreferenced_extensions);
        snapshot = match captureProfileSnapshotExtensionsFromPath(  snapshot.clone(), profile_path.clone()) {
            Ok(snapshot) => snapshot,
            Err(_e) => {
                println!("err captureProfileSnapshotExtensionsFromPath {_e}");
                continue;
            }
        };
        //println!("snapshot {:?} {:?} {:?}", snapshot.uid, snapshot.referenced_extensions, snapshot.unreferenced_extensions);
        output.push(snapshot);
    }
    output
}

/// Captures a Chrome profile from the given path
fn captureProfileSnapshotSettingsFromPath(profile_path : ChromeProfilePath) -> Result<ChromeProfileSnapshot, Error> {
    // Save path and type, so we can add all the chrome-based
    // extensions in the same table
    
    let mut profile_path_clone = profile_path.clone();

    let chrome_browser_type = profile_path.r#type;
    let path = profile_path.value;
    let uid = profile_path.uid;

    // Save the contents of the configuration files
    let preferences_file_path = std::path::Path::new(&profile_path_clone.value).join("Preferences");
    let secure_prefs_file_path = std::path::Path::new(&profile_path_clone.value).join("Secure Preferences");

    let preferences = fs::read_to_string(&preferences_file_path)?;

    let secure_preferences = fs::read_to_string(&secure_prefs_file_path)?;

    if (preferences.is_empty() && secure_preferences.is_empty()) {
        let err = format!("Failed to read the Preferences file for the following profile snapshot  {}", preferences_file_path.to_string_lossy());
        return Err(Error::Other(err));  
    }

    let mut snapshot_clone = ChromeProfileSnapshot{
        chrome_browser_type : chrome_browser_type,
        path : path,
        preferences : preferences,
        secure_preferences : secure_preferences,
        uid : uid,
        referenced_extensions : None,
        unreferenced_extensions : None
    };
    // println!("{:#?}", snapshot_clone);
    Ok(snapshot_clone)
}
fn  captureProfileSnapshotExtensionsFromPath( snapshot: ChromeProfileSnapshot,profile_path : ChromeProfilePath) -> Result<ChromeProfileSnapshot, Error> {
    // Enumerate all the extensions that are present inside this
    // profile. Note that they may not be present in the config
    // file. For now, let's store them all as unreferenced
    let mut referenced_extensions:HashMap<String, ChromeProfileSnapshotExtension> = HashMap::new();
    let mut unreferenced_extensions :HashMap<String, ChromeProfileSnapshotExtension> = HashMap::new();

    let mut preferences = snapshot.clone().preferences;
    let mut secure_preferences = snapshot.clone().secure_preferences;

    let mut profile_path_clone = profile_path.clone();
    let mut extensions_folder_path = std::path::Path::new(&profile_path_clone.value).join("Extensions");
    let mut profile_path = profile_path.clone(); 
    
    let mut extension_path_list:Vec<String> = Vec::new();
    
    {
        let base_path_list = listDirectoriesInDirectory(extensions_folder_path.to_str().unwrap().to_string())?;

        for base_path in base_path_list {
            let mut new_path_list = Vec::new();
            new_path_list =  listDirectoriesInDirectory(base_path)?;
            let mut new_path_list_store = Vec::new();
            for new_path in new_path_list {
                let mut canonical_path = fs::canonicalize(new_path)?;
                let path = canonical_path.to_string_lossy().into_owned();
                new_path_list_store.push(path);
            }

            extension_path_list.extend(new_path_list_store);
        }
    }
    // extension_path_list [
    //     "\\\\?\\C:\\Users\\Admin\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Extensions\\ghbmnnjooekpmoecnnnilnnbdlolhkhi\\1.97.1_0",
    //     "\\\\?\\C:\\Users\\Admin\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Extensions\\nmmhkkegccagdldgiimedpiccmgmieda\\1.0.0.6_0",
    // ]
    for extension_path in  extension_path_list {
        let extension_path = extension_path.clone();
        let mut manifest_path = std::path::Path::new(&extension_path).join("manifest.json");
        let extension_manifest = match fs::read_to_string(manifest_path){
            Ok(_t) => _t,
            Err(_e) => {
                println!("err in captureProfileSnapshotExtensionsFromPath function while read_to_string manifest_path {:?}", _e);
                String::new()
            }
        };
        let mut extension = ChromeProfileSnapshotExtension {
            path: extension_path.clone(),
            manifest: extension_manifest
        };
        unreferenced_extensions.insert(extension_path, extension);
        // snapshot_clone.unreferenced_extensions = Some(unreferenced_extensions);
        // println!("{:#?} ", snapshot_clone.clone().preferences);
    }
    println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    println!("unreferenced_extensions map is {:#?}", unreferenced_extensions.clone());
    println!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    // Now get a list of all the extensions referenced by the Preferences file.
    let mut referenced_ext_path_list:Vec<String> = Vec::new();
    // println!("file  {:#?} ", snapshot_clone.path);
    println!("profile value {:?}", profile_path.value);

    let (mut result, mut path_list_vec_refer) = getExtensionPathListFromPreferences(&mut profile_path.value, preferences.clone());
    if result == false {

        let err = format!("Failed to parse the following profile:  {:?}", profile_path.value);
        return Err(Error::Other(err));
    }
    println!("path_list_vec_refer {:#?}", &path_list_vec_refer);

    {
        let (mut result, mut additional_ref_ext_path_list) = getExtensionPathListFromPreferences(&mut profile_path.value, secure_preferences.clone());
        if result == false {
            let err = format!("Failed to parse the following profile:  {:?}", profile_path.value);
            return Err(Error::Other(err));
        }
        println!("additional_ref_ext_path_list {:#?}", &additional_ref_ext_path_list);

        referenced_ext_path_list.extend(path_list_vec_refer);
        referenced_ext_path_list.extend(additional_ref_ext_path_list);

    }
    for referenced_ext_path in referenced_ext_path_list {
        if let Some(ext) = unreferenced_extensions.remove(&referenced_ext_path) {
            referenced_extensions.insert(referenced_ext_path, ext);
        }else{
            let manifest_path = std::path::Path::new(&referenced_ext_path).join("manifest.json");
            let manifest = match fs::read_to_string(&manifest_path) {
                Ok(content) => content,
                Err(e) => {
                    println!("err {:?} while read manifest_path {:?}", e, manifest_path);
                    continue;
                }
            };

            let ext = ChromeProfileSnapshotExtension {
                path: referenced_ext_path.clone(),
                manifest,
            };
            referenced_extensions.insert(referenced_ext_path, ext);
        }
    }
    println!(" referenced_extensions {:#?}", referenced_extensions.clone());

    let snapshot_final = ChromeProfileSnapshot {
        chrome_browser_type : snapshot.chrome_browser_type,
        path : snapshot.path ,
        preferences : preferences ,
        referenced_extensions : Some(referenced_extensions),
        uid : snapshot.uid ,
        secure_preferences : snapshot.secure_preferences ,
        unreferenced_extensions : Some(unreferenced_extensions),
    };
    return Ok(snapshot_final)
    // return  Err(Error::Other("test".to_string()));
}

/// Retrieves the list of referenced extensions from the given profile preferences
pub fn getExtensionPathListFromPreferences(profile_path : &mut String, preferences :  String) -> (bool,  Vec<String>) {
    let mut result = true;
    let mut path_list = vec![];
    //println!("getExtensionPathListFromPreferences has preferences file is {:?}", preferences);
    let tree = match serde_json::from_str(&preferences) {
        Ok(_t) => _t,
        Err(_e) => {
            println!("err while parse to jsonvalue in getExtensionPathListFromPreferences {:?}", _e);
            result = false;
            JSonValue::Null
        }
    };
    if result == false {
        return (result, path_list.clone());
    }

    let mut opt_extensions_node = tree.get("extensions");
    if !opt_extensions_node.is_some() {
        println!("opt_extensions_node has not extensions");
        return (true, path_list.clone());
    }

    let mut extensions_node = opt_extensions_node.take().unwrap();

    let mut opt_settings_node = extensions_node.get("settings");
    if !opt_settings_node.is_some() {
        opt_settings_node = extensions_node.get("opsettings");
    }

    if !opt_settings_node.is_some() {
        println!("opt_extensions_node hasn't extensions and has not settings");
        return (true, path_list.clone());
    }

    let settings_node = opt_settings_node.take().unwrap();
    
   // println!( "opensetting node {:#?}", settings_node);
    match settings_node {
        JSonValue::Object(vl) =>{
            for (key, value) in vl {
                let mut opt_path = match value.get("path"){
                    Some(JSonValue::String(path)) => {
                        let mut absolute_path = std::path::Path::new(&path);
                        let mut path_new = String::new();
                        if !absolute_path.is_absolute() {
                            path_new = format!("{}{}{}{}{}", profile_path, "\\", "Extensions", "\\", path);
                            path_new
                        }else{
                            absolute_path.to_str().unwrap().to_string()
                        }
                    },
                    _ => {
                        println!( "error while take path");
                        continue; 
                    }
                };
                let canonical_path = match fs::canonicalize(&opt_path){
                    Ok(_t) => _t.to_string_lossy().to_string(),
                    Err(_e) => {
                        opt_path
                    }
                };
                path_list.push(canonical_path);
            }
        },
        _Other => {}
    }

    return (true, path_list.clone());
}


fn getChromeProfilePathList() -> Vec<ChromeProfilePath>{
    let kWindowsPathList = vec![
    (ChromeBrowserType::GoogleChrome, "AppData\\Local\\Google\\Chrome\\User Data"),
    // (ChromeBrowserType::GoogleChromeBeta, "AppData\\Local\\Google\\Chrome Beta\\User Data"),
    // (ChromeBrowserType::GoogleChromeDev, "AppData\\Local\\Google\\Chrome Dev\\User Data"),
    // (ChromeBrowserType::GoogleChromeCanary, "AppData\\Local\\Google\\Chrome SxS\\User Data"),
    // (ChromeBrowserType::Brave, "AppData\\Roaming\\brave"),
    // (ChromeBrowserType::Chromium, "AppData\\Local\\Chromium"),
    // (ChromeBrowserType::Yandex, "AppData\\Local\\Yandex\\YandexBrowser\\User Data"),
    // (ChromeBrowserType::Edge, "AppData\\Local\\Microsoft\\Edge\\User Data"),
    // (ChromeBrowserType::EdgeBeta, "AppData\\Local\\Microsoft\\Edge Beta\\User Data"),
    // (ChromeBrowserType::Opera, "AppData\\Roaming\\Opera Software\\Opera Stable"),
    // (ChromeBrowserType::Vivaldi, "AppData\\Local\\Vivaldi\\User Data")
    ];
    let user_infors = getUserInformationList();
    let mut output = Vec::new();
    for user in user_infors {
        let uid = user.uid;
        for (chrome_browser_type, path) in &kWindowsPathList {
            let browser_type = *chrome_browser_type;
            let path_sufix = *path;
            let entry_path = format!("{}{}{}", user.path, "\\", path_sufix);
            let entry_path = PathBuf::from(entry_path);
            let path_real = fs::canonicalize(&entry_path).unwrap_or(entry_path.clone());
            let mut value = String::new();
            if isValidChromeProfile(&path_real) {
                value =  path_real.to_string_lossy().to_string();
                output.push(ChromeProfilePath {
                    uid : uid.clone(),
                    value : value,
                    r#type: browser_type
                });                
                continue;
            }
            let mut chrome_subfolder_list = Vec::new();
            match listDirectoriesInDirectory(path_real.to_str().unwrap().to_string()){
                Ok(dir) =>{
                    println!("list subdir ok {:?}", &path_real);
                    chrome_subfolder_list = dir
                },
                Err(_e) =>{
                    println!("Error while list subdir {_e}")
                }
            }
            for chrome_subfolder in chrome_subfolder_list {
                
                let entry_path_folder = PathBuf::from(chrome_subfolder);
                let path_real = fs::canonicalize(&entry_path_folder).unwrap_or(entry_path_folder.clone());
                if isValidChromeProfile(&path_real) {
                    value =  path_real.to_string_lossy().to_string();
                    output.push(ChromeProfilePath {
                        uid : uid.clone(),
                        value : value.clone(),
                        r#type: browser_type
                    });
                    continue;
                }
            }
        }
    }
    output
}
fn listDirectoriesInDirectory(path: String) -> Result<Vec<String>, Error> {
    let mut dir  = Vec::new();
    for item in fs::read_dir(path)? {
        let item = item?;
        let path = item.path();
        if path.is_dir() {
            dir.push(path.to_str().unwrap().to_string());
        }
    }
    Ok(dir)
}
#[derive(Clone, Debug)]
pub struct ChromeProfilePath {
    pub uid : String,
    pub value : String,
    pub r#type : ChromeBrowserType
}
fn isValidChromeProfile(path: &PathBuf) -> bool {
    let kPossibleConfigFileNames = vec!["Preferences".to_string(),                                
                                                     "Secure Preferences".to_string()];
    for config_file_name_ref in &kPossibleConfigFileNames {
        let preferences_file_path = format!("{}{}{}", path.to_string_lossy() , "\\" ,config_file_name_ref);
        let _result =  match File::open(&preferences_file_path){
            Ok(_file) => {
                return true;
            }
            Err(_e) => {
                println!("lỗi mở file {:?} valid {:#?}", preferences_file_path, _e);
                return false;
            }
        };
    }
    return false
}
pub struct UserInfor {
    pub uid : String,
    pub path : String,
}
fn getUserInformationList() -> Vec<UserInfor> {
    let users = get_local_accounts();
    let mut user_infor:Vec<UserInfor> = Vec::new();
    for user in users {
        if user.user_id == 0 || user.profile_image_path.is_none() {
            continue;
        }
        let uid_as_string = user.user_id.to_string();
        let path = user.profile_image_path.unwrap();
        user_infor.push( UserInfor { uid: uid_as_string, path: path });
    }
    return user_infor
}
fn collect_arp_cache() -> Result<(), Error> {
    // let mut size_arp: u32 = 0;
    // let ret = unsafe { GetIpNetTable(None, &mut size_arp, BOOL::from(false)) };
    // if ret != ERROR_INSUFFICIENT_BUFFER.0 {
    //     return Err(Error::Other(format!("GetIpNetTable failed with error code: {}", ret)));
    // }

    // let buf_ipnet = vec![0u8; size_arp as usize];
    // let ipnet_tbl: *mut MIB_IPNETTABLE = buf_ipnet.as_ptr() as *mut _;
    // let ret: u32 = unsafe { GetIpNetTable(Some(ipnet_tbl), &mut size_arp, BOOL::from(false)) };
    // if ret == ERROR_NO_DATA.0 {
    //     let empty_vec: Vec<ArpCache> = Vec::new();
    //     return db::update_arp_cache(&empty_vec)
    // }

    // if ret != NO_ERROR.0 {
    //     return Err(Error::Other(format!("GetIpNetTable failed with error code: {}", ret)));
    // }

    // let mut size_if: u32 = 0;
    // let ret = unsafe { GetIfTable(None, &mut size_if, BOOL::from(false)) };
    // if ret != ERROR_INSUFFICIENT_BUFFER.0 {
    //     return Err(Error::Other(format!("GetIfTable failed with error code: {}", ret)));
    // }

    // let buf_if = vec![0u8; size_if as usize];
    // let if_tbl: *mut MIB_IFTABLE = buf_if.as_ptr() as *mut _;
    // let ret = unsafe { GetIfTable(Some(if_tbl), &mut size_if, BOOL::from(false)) };
    // if ret != NO_ERROR.0 {
    //     return Err(Error::Other(format!("GetIfTable failed with error code: {}", ret)));
    // }

    // let mut arp_cache: Vec<ArpCache> = Vec::new();

    // unsafe {
    //     for i in 0..(*ipnet_tbl).dwNumEntries {
    //         let ipnet_row = (*ipnet_tbl).table.as_ptr().add(i as usize);

    //         if (*ipnet_row).dwPhysAddrLen == 0 {
    //             continue;
    //         }

    //         let arp_ip = std::net::Ipv4Addr::from((*ipnet_row).dwAddr.to_be());
    //         let arp_mac = &(&(*ipnet_row).bPhysAddr)[..(*ipnet_row).dwPhysAddrLen as usize];
    //         let arp_type = (*ipnet_row).Anonymous.Type;

    //         let mut adapter_mac = None;

    //         for j in 0..(*if_tbl).dwNumEntries {
    //             let if_row = (*if_tbl).table.as_ptr().add(j as usize);
    //             if (*if_row).dwIndex == (*ipnet_row).dwIndex {
    //                 adapter_mac = Some(&(*if_row).bPhysAddr[..(*if_row).dwPhysAddrLen as usize]);
    //                 break;
    //             }
    //         }

    //         if adapter_mac.is_none() {
    //             return Err(Error::Other(format!("No mac interface found for ARP index: {}", (*ipnet_row).dwIndex)));
    //         }

    //         arp_cache.push(ArpCache {
    //             address: arp_ip.to_string(),
    //             mac: utils::mac_to_str(arp_mac),
    //             interface: utils::mac_to_str(adapter_mac.unwrap()),
    //             permanent: if arp_type == MIB_IPNET_TYPE_STATIC {
    //                 "1".to_string()
    //             } else {
    //                 "0".to_string()
    //             },
    //         });
    //     }
    // }

    // if arp_cache.len() > 0 {
    //     return db::update_arp_cache(&arp_cache);
    // }

    Ok(())
}

fn collect_disk_info() -> Result<(), Error> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;
    let disk_info: Vec<Win32_DiskDrive> =  wmi_con.query()?;

    if disk_info.len() > 0 {
        return db::update_disk_info(&disk_info);
    }

    Ok(())
}

fn collect_dns_cache() -> Result<(), Error> {
    unsafe {
        let lib = libloading::Library::new("dnsapi.dll")?;
        let fn_dns_get_cache_data_table: libloading::Symbol<DNS_GET_CACHE_DATA_TABLE> = lib.get(b"DnsGetCacheDataTable")?;

        let mut list_dns_caches: Vec<DnsCache> = Vec::new();
        let mut p_cache_entry: *mut DNS_CACHE_ENTRY = std::ptr::null_mut();

        let status = fn_dns_get_cache_data_table(&mut p_cache_entry);
        if status != 1 {
            // I don't know, but it seems that 1 is the success code for DnsGetCacheDataTable
            return Err(Error::Other(format!("DnsGetCacheDataTable failed with status: {}", status)));
        }
        
        let mut current_entry = p_cache_entry;
        while !current_entry.is_null() {
            let entry = &*current_entry;

            if entry.pszName.is_null() || entry.wType == 0 {
                current_entry = entry.pNext;
                continue;
            }

            let record_name = entry.pszName.to_string()?;

            let record_type: String = match DNS_TYPE(entry.wType) {
                DNS_TYPE_A => "A".to_string(),
                DNS_TYPE_AAAA => "AAAA".to_string(),
                DNS_TYPE_PTR => "PTR".to_string(),
                _ => format!("DNS record type: {}", entry.wType),
            };

            list_dns_caches.push(DnsCache {
                name: record_name,
                r#type: record_type,
                flags: entry.dwFlags as u64,
            });

            current_entry = entry.pNext;
        }

        DnsFree(Some(p_cache_entry as *mut _), DnsFreeRecordList);

        if list_dns_caches.len() > 0 {
            return db::update_dns_cache(&list_dns_caches);
        }
    }

    Ok(())
}

fn collect_etc_hosts() -> Result<(), Error> {
    let mut first_line = true;
    let mut data: Vec<ETCHosts> = Vec::new();

    let file = File::open(r"C:\Windows\System32\drivers\etc\hosts")?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if first_line {
            first_line = false;
            let line_bytes = line.as_bytes();

            if line_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) && line_bytes[3] == b'#' {
                continue;
            }
        }

        let mut parts = line.split_whitespace();
        if let Some(ip) = parts.next() {
            if let Some(hostname) = parts.next() {
                data.push(ETCHosts {
                    address: ip.to_string(),
                    hostname: hostname.to_string(),
                    pid_with_namespace: None,
                });
            }
        }
    }

    if data.len() > 0 {
        return db::update_etc_hosts(&data)
    }

    Ok(())
}

fn collect_groups() -> Result<(), Error> {
    let list_groups = sys_info::group::get_local_groups();
    return db::update_groups(&list_groups)
}

fn collect_interface_addresses() -> Result<(), Error> {
    let list_interface_addr = sys_info::network::live_query_enum_interface()?;
    db::update_interface_address(&list_interface_addr)
}

fn collect_interface_details() -> Result<(), Error> {
    let list_interface_details = sys_info::network::live_query_interface_details()?;
    db::update_interface_details(&list_interface_details)
}

fn collect_logical_drives() -> Result<(), Error> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let os_info: Vec<Win32_OperatingSystem> = wmi_con.query()?;
    let boot_drive = if let Some(system_drive) = os_info.get(0) {
        system_drive.SystemDrive.as_str()
    }
    else {
        "C:"
    };

    let logical_drives: Vec<Win32_LogicalDisk> = wmi_con.query()?;

    if logical_drives.len() > 0 {
        return db::update_logical_drives(&logical_drives, boot_drive);
    }

    Ok(())
}

fn collect_logon_sessions() -> Result<(), Error> {
    let mut list_logon_sessions: Vec<LogonSessions> = Vec::new();
    let logger = Logger::new(MODULE_NAME.to_string());

    let mut count: u32 = 0;
    let mut luid_ptr: *mut LUID = std::ptr::null_mut();

    let status = unsafe { LsaEnumerateLogonSessions(&mut count, &mut luid_ptr) };
    if status.is_err() {
        return Err(Error::Other(format!("LsaEnumerateLogonSessions error: {}", status.0)));
    }

    let lsa_buffer_luid = LsaBuffer::new(luid_ptr as *mut c_void);
    let luids = unsafe { std::slice::from_raw_parts(lsa_buffer_luid.as_luid(), count as usize) };

    for luid in luids {
        let mut data_ptr: *mut SECURITY_LOGON_SESSION_DATA = std::ptr::null_mut();

        let status = unsafe { LsaGetLogonSessionData(luid, &mut data_ptr) };
        if status.is_err() {
            log_lib::log_error!(logger, &format!("LsaGetLogonSessionData error: {}", status.0));
            continue;
        }

        let lsa_buffer_slsd = LsaBuffer::new(data_ptr as *mut c_void);
        let logon_session_data = unsafe { *lsa_buffer_slsd.as_security_logon_session_data() };

        let logon_type = logon_session_data.LogonType as i32;
        if SECURITY_LOGON_TYPE(logon_type) == SECURITY_LOGON_TYPE::Interactive
            || SECURITY_LOGON_TYPE(logon_type) == SECURITY_LOGON_TYPE::RemoteInteractive
            || SECURITY_LOGON_TYPE(logon_type) == SECURITY_LOGON_TYPE::CachedInteractive
        {
            let logon_id = format!("{}{}", logon_session_data.LogonId.HighPart, logon_session_data.LogonId.LowPart)
                .parse::<u64>()
                .unwrap_or(0);

            let logon_type_str = match SECURITY_LOGON_TYPE(logon_type) {
                SECURITY_LOGON_TYPE::Interactive => "Interactive".to_string(),
                SECURITY_LOGON_TYPE::RemoteInteractive => "RemoteInteractive".to_string(),
                SECURITY_LOGON_TYPE::CachedInteractive => "CachedInteractive".to_string(),
                _ => "Unknown".to_string()
            };
            
            let windows_epoch = 116444736000000000i64;
            let logon_time = (logon_session_data.LogonTime - windows_epoch) / 10_000_000;

            list_logon_sessions.push(LogonSessions {
                logon_id,
                user: utils::lsa_unicode_string(&logon_session_data.UserName).unwrap_or_default(),
                logon_domain: utils::lsa_unicode_string(&logon_session_data.LogonDomain).unwrap_or_default(),
                authentication_package: utils::lsa_unicode_string(&logon_session_data.AuthenticationPackage).unwrap_or_default(),
                logon_type: logon_type_str,
                session_id: logon_session_data.Session as u64,
                logon_sid: utils::sid_to_string(logon_session_data.Sid).unwrap_or_default(),
                logon_server: utils::lsa_unicode_string(&logon_session_data.LogonServer).unwrap_or_default(),
                logon_time: logon_time as u64,
                upn: utils::lsa_unicode_string(&logon_session_data.Upn).ok(),
                dns_domain_name: utils::lsa_unicode_string(&logon_session_data.DnsDomainName).ok(),
                logon_script: utils::lsa_unicode_string(&logon_session_data.LogonScript).ok(),
                profile_path: utils::lsa_unicode_string(&logon_session_data.ProfilePath).ok(),
                home_directory: utils::lsa_unicode_string(&logon_session_data.HomeDirectory).ok(),
                home_directory_drive: utils::lsa_unicode_string(&logon_session_data.HomeDirectoryDrive).ok()
            });
        }
    }

    db::update_logon_sessions(&list_logon_sessions)
}

fn collect_os_version() -> Result<(), Error> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let list_os_info: Vec<Win32_OperatingSystem> =  wmi_con.query()?;

    let os_info = match list_os_info.first() {
        Some(res) => res,
        None => return Err(Error::Other("List os info is empty!".to_string()))
    };

    db::update_os_version(os_info)
}

fn collect_process_open_sockets() -> Result<(), Error> {
    let all_connections = sys_info::network::enum_all_network_connections()?;
    db::update_process_open_sockets(&all_connections)
}

fn collect_processes() -> Result<(), Error> {
    let mut list_processes: Vec<ProcessInfo> = Vec::new();
    let mut system = System::new_all();

    system.refresh_all();

    for (pid, process) in system.processes() {
        let process_id = pid.as_u32();

        let process_name = process.name()
            .to_string_lossy()
            .to_string();

        let process_path = process.exe()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();

        let cmd = process.cmd()
            .iter()
            .map(|str| str.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let state = if process.status() == sysinfo::ProcessStatus::Run {
            "STILL_ACTIVE".to_string()
        } else {
            String::new()
        };

        let cwd = process.cwd()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();

        let root = process.root()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();

        let uid = if let Some(user_id) = process.user_id() {
            match utils::extract_rid_from_sid(&user_id.to_string()) {
                Some(rid) => rid as i32,
                None => -1,
            }
        } else {
            -1
        };

        let on_disk: i32 = if let Some(path) = process.exe() {
            if path.exists() {
                1
            } else {
                0
            }
        } else {
            -1
        };

        let parent = process.parent()
            .map(|pid| pid.as_u32() as i32)
            .unwrap_or(-1);

        // ================================
        let wired_size: i64;
        let resident_size: i64;
        let total_size: i64;
        let user_time: i64;
        let system_time: i64;
        let disk_bytes_read: i64;
        let disk_bytes_written: i64;
        let threads: i32;
        let nice: i32;
        let elevated_token: i32;
        let secure_process: i32;
        let protection_type: String;
        let virtual_process: i32;
        let handle_count: i32;
        // ================================

        match unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, BOOL::from(false), process_id) } {
            Ok(process_handle) => {
                (wired_size, resident_size, total_size) = processes::get_process_memory_info(&process_handle);

                (user_time, system_time) = processes::get_process_time_info(&process_handle);// ham nay dang loi

                (disk_bytes_read, disk_bytes_written) = processes::get_io_counters(&process_handle);

                threads = processes::count_threads_in_process(process_id);

                let priority_class = unsafe { GetPriorityClass(process_handle) };
                nice = if priority_class == 0 { -1 } else { priority_class as i32 };

                elevated_token = processes::get_elevated_token(&process_handle);

                (secure_process, virtual_process) = processes::get_process_info(&process_handle);

                protection_type = processes::get_proc_protected_type(&process_handle);

                handle_count = processes::get_handle_count(&process_handle);

                let _ = unsafe { CloseHandle(process_handle) };
            },
            Err(_err) => {
                #[cfg(debug_assertions)]
                println!("OpenProcess pid {} error: {}", process_id, _err);

                wired_size = -1;
                resident_size = -1;
                total_size = -1;

                user_time = -1;
                system_time = -1;
                
                disk_bytes_read = -1;
                disk_bytes_written = -1;

                threads = -1;
                nice = -1;

                elevated_token = -1;

                secure_process = -1;
                virtual_process = -1;

                protection_type = String::new();

                handle_count = -1;
            }
        }

        list_processes.push(ProcessInfo {
            pid: process_id as i32,
            name: process_name,
            path: process_path,
            cmdline: cmd,
            state,
            cwd,
            root,
            uid,
            gid: -1,
            egid: -1,
            euid: -1,
            suid: -1,
            sgid: -1,
            on_disk,
            wired_size,
            resident_size,
            total_size,
            user_time,
            system_time,
            disk_bytes_read,
            disk_bytes_written,
            start_time: process.start_time() as i64,
            parent,
            pgroup: -1,
            threads,
            nice,
            elevated_token,
            secure_process,
            protection_type,
            elapsed_time: process.run_time() as i64,
            virtual_process,
            handle_count,
            percent_processor_time: -1,
        });
    }

    db::update_processes(&list_processes)
}

fn collect_programs() -> Result<(), Error> {
    let apps = sys_info::apps::get_installed_apps();
    if apps.len() == 0 {
        return Err(Error::Other("get_installed_apps error size = 0!".to_string()));
    }
    db::update_programs(&apps)    
}

fn collect_scheduled_tasks() -> Result<(), Error> {
    let scheduler = sys_info::schedule::get_scheduled_tasks()?;
    db::update_scheduled_tasks(&scheduler)
}

fn collect_services() -> Result<(), Error> {
    let mut services: Vec<sys_info::service::Service> = Vec::new();

    let win32_services = sys_info::service::get_started_services()?;
    #[cfg(debug_assertions)]
    println!("win32_services: {}", win32_services.len());

    let kernel_services = sys_info::service::get_system_kernel_drivers()?;
    #[cfg(debug_assertions)]
    println!("kernel_services: {}", kernel_services.len());

    let file_drivers = sys_info::service::get_file_system_drivers()?;
    #[cfg(debug_assertions)]
    println!("file_drivers: {}", file_drivers.len());

    services.extend(win32_services);
    services.extend(kernel_services);
    services.extend(file_drivers);

    db::update_services(&services)
}

fn collect_startup_items() -> Result<(), Error> {
    // Cần check thêm trong SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run để xem state đang là Enabled hay Disabled
    let startup_items = sys_info::apps::get_startup_apps();
    db::update_startup_items(&startup_items)
}

fn collect_system_info() -> Result<(), Error> {
    let mut uuid = String::new();
    let mut vendor = String::new();

    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;
    let result: Vec<HashMap<String, Variant>> = wmi_con.raw_query("Select UUID, Vendor from Win32_ComputerSystemProduct")?;
    
    let csproduct = match result.get(0) {
        Some(ret) => ret,
        None => return Err(Error::Other("Get Win32_ComputerSystemProduct error!".to_string()))
    };

    if let Some(variant_uuid) = csproduct.get("UUID") {
        if let Variant::String(uuid_str) = variant_uuid {
            uuid = uuid_str.clone();
        }
    }

    if let Some(variant_vendor) = csproduct.get("Vendor") {
        if let Variant::String(vendor_str) = variant_vendor {
            vendor = vendor_str.clone();
        }
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap_or("".to_string());
    let cpu_type = System::cpu_arch();
    let total_memory = sys.total_memory();
    let cpus = sys.cpus();
    let brand = cpus[0].brand();
    let cpu_physic_cores = System::physical_core_count().unwrap_or(0);
    let cpu_logical_cores = cpus.len();

    db::update_system_info(&SystemInfo {
        hostname,
        uuid,
        cpu_type,
        cpu_brand: brand.to_string(),
        cpu_physic_cores: cpu_physic_cores as u32,
        cpu_logical_cores: cpu_logical_cores as u32,
        physical_memory: total_memory,
        hardware_vendor: vendor,
    })
}

fn collect_time() -> Result<(), Error> {
    let utc_time = Utc::now();

    let local_timezone = 
    {
        let local_time = Local::now();
        let offset = local_time.offset();
        let seconds = offset.local_minus_utc();
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let sign = if seconds >= 0 { "+" } else { "-" };

        format!(
            "UTC{}{:02}:{:02}",
            sign,
            hours.abs(),
            minutes.abs()
        )
    };

    return db::update_time(&Time {
        weekday: utc_time.weekday().to_string(),
        year: utc_time.year() as u64,
        month: utc_time.month() as u64,
        day: utc_time.day() as u64,
        hour: utc_time.hour() as u64,
        minutes: utc_time.minute() as u64,
        seconds: utc_time.second() as u64,
        timezone: "UTC".to_string(),
        local_timezone,
        unix_time: utc_time.timestamp() as u64,
        timestamp: String::new(),
        datetime: String::new(),
        iso_8601: utc_time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        win_timestamp: None,
    });
}

fn collect_uptime() -> Result<(), Error> {
    let uptime_ms = unsafe { GetTickCount64() };
    let total_secs = uptime_ms / 1000;

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    return db::update_uptime(&Uptime {
        days,
        hours,
        minutes,
        seconds,
        total_seconds: total_secs,
    });
}

fn collect_user_groups() -> Result<(), Error> {
    let mut list_user_groups: Vec<(u32, u32)> = Vec::new();
    let list_local_users = sys_info::user::get_local_accounts();

    if list_local_users.is_empty() {
        return Err(Error::Other("get_local_accounts error!".to_string()))
    }

    for acccount in list_local_users {
        let list_groups = sys_info::user::get_user_local_groups(&acccount.username)?;
        
        for group_name in list_groups {
            let group_name_c = U16CString::from_str(&group_name)?;

            if let Some(sid) = sys_info::group::get_sid_from_name(PCWSTR(group_name_c.as_ptr())) {
                let sid_u32: u32 = sid.rsplit('-').next().unwrap().parse().unwrap();
                list_user_groups.push((acccount.user_id, sid_u32));
            }
            else {
                return Err(Error::Other(format!("get_sid_from_name {} error!", group_name)))
            }
        }
    }

    return db::update_user_groups(&list_user_groups)
}

fn collect_user_ssh_keys() -> Result<(), Error> {
    
    Ok(())
}

fn collect_users() -> Result<(), Error> {
    let list_local_users = sys_info::user::get_local_accounts();
    return db::update_users(&list_local_users)
}