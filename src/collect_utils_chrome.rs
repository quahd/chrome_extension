fn collect_chrome_extensions() -> Result<(), Error> {
    let snapshot_list = getChromeProfileSnapshotList();
    let _chrome_profiles = getChromeProfilesFromSnapshotList(snapshot_list);
    Ok(())
}
// /ChromeProfileList
fn getChromeProfilesFromSnapshotList(snapshot_list: Vec<ChromeProfileSnapshot>) {
    let mut profile_list = Vec::new();
    for snapshot in snapshot_list {
        let mut type_profile = snapshot.chrome_browser_type;
        let mut path = snapshot.path;
        let mut uid = snapshot.uid;

        let mut parse_preference = match serde_json::from_str(&snapshot.preferences){
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

        let mut parsed_secure_preferences = match serde_json::from_str(&snapshot.secure_preferences){
            Ok(value) => value,
            Err(_e) => {
                println!("err while parse to JSonValue {:?}", _e);
                JSonValue::Null
            }
        };

        if parsed_secure_preferences.is_null() && !snapshot.secure_preferences.is_empty() {
            println!("Failed to parse the secure_preferences file of the following profile:  {:?}", &path);
            continue;
        }
        
        // Try to get the profile name; the Opera browser does not have it
        // getProfileNameFromPreferences
        // getExtensionFromSnapshot
        // getExtensionProfileSettings

    }

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
fn getExtensionFromSnapshot(snapshot: ChromeProfileSnapshotExtension)  {
    //let output ; //ChromeProfileExtension
    let path = snapshot.path;
    let manifest_hash = hashFromBuffer(snapshot.clone().manifest);

    let mut parsed_manifest = match serde_json::from_str(&snapshot.manifest) {
        Ok(manifest) => manifest,
        Err(_e) => {
            let err = format!("Failed to parse the Manifest file for the following extension:  {:?}", _e);
            // tạo return sau
        }
    };
    let properties = match getExtensionProperties(parsed_manifest.clone()) {
        Ok(properties) => properties,
        Err(_e) => {
            let err = format!("Failed to parse the Manifest file for the following extension:  {:?}", _e);
            // retrun về status là lỗi => result error output
        }
    };

}

fn getExtensionProperties(parsed_manifest: JSonValue) {
    let ExtensionPropertyMap = vec![
        ExtensionProperty { ty: PropertyType::String, path: "name".to_string(), name: "name".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "update_url".to_string(), name: "update_url".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "version".to_string(), name: "version".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "author".to_string(), name: "author".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "default_locale".to_string(), name: "default_locale".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "current_locale".to_string(), name: "current_locale".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "background.persistent".to_string(), name: "persistent".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "description".to_string(), name: "description".to_string() },
        ExtensionProperty { ty: PropertyType::StringArray, path: "permissions".to_string(), name: "permissions".to_string() },
        ExtensionProperty { ty: PropertyType::StringArray, path: "optional_permissions".to_string(), name: "optional_permissions".to_string() },
        ExtensionProperty { ty: PropertyType::String, path: "key".to_string(), name: "key".to_string() },
    ];
    let mut properties = HashMap::new();
    for property in ExtensionPropertyMap {
        let mut opt_node = match parsed_manifest.get(&property.path){
            Some(value) => value,
            None => {
                continue;
            }
        };
        if property.ty == PropertyType::String {
            match opt_node.clone() {
                JSonValue::String(value) => {
                    if !value.is_empty() {
                        properties.insert(property.clone().name, value);
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
            match opt_node {
                JSonValue::Array(arr) => {
                    for p in arr {
                        match p {
                            JSonValue::String(child_node) => {
                                list_value.push_str(", ");
                                list_value.push_str(child_node);
                            },
                            _ => {
                                continue;
                            }
                        }
                    }
                    properties.insert(property.clone().name, list_value);
                         // Also provide the json-encoded value
                    let list_value_json = match serde_json::to_string(opt_node){
                        Ok(vl_json) => vl_json,
                        Err(_e) => {
                            let err = format!("lỗi khi chuyển nodes thành json  {:?}", _e);
                            continue;
                        }
                    };
                    let name_arr = format!("{}{}", property.clone().name, "_json");
                    properties.insert(name_arr, list_value_json);
                },
                _ => {}              
            }
        }else{
            let err = format!("Invalid property type specified");
        }
    }
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

    // Now get a list of all the extensions referenced by the Preferences file.
    let mut referenced_ext_path_list:Vec<String> = Vec::new();
    // println!("file  {:#?} ", snapshot_clone.path);
    println!("profile value {:?}", profile_path.value);

    let (mut result, mut path_list_vec_refer) = getExtensionPathListFromPreferences(&mut profile_path.value, preferences.clone());
    if result == false {

        let err = format!("Failed to parse the following profile:  {:?}", profile_path.value);
        return Err(Error::Other(err));
    }
    {
        let (mut result, mut additional_ref_ext_path_list) = getExtensionPathListFromPreferences(&mut profile_path.value, secure_preferences.clone());
        if result == false {
            let err = format!("Failed to parse the following profile:  {:?}", profile_path.value);
            return Err(Error::Other(err));
        }

        referenced_ext_path_list.extend(path_list_vec_refer);
        referenced_ext_path_list.extend(additional_ref_ext_path_list);

    }
    for referenced_ext_path in referenced_ext_path_list {
        let mut unreferenced_extensions_temp = unreferenced_extensions.clone();
        for (name, profiles_napshot_extension) in &unreferenced_extensions {
            if *name == referenced_ext_path.clone() {
                // Move this extension to the referenced group
                referenced_extensions.insert(referenced_ext_path.clone(), profiles_napshot_extension.clone());
                unreferenced_extensions_temp.remove(&referenced_ext_path.clone());
            }else{
                let extension_path = referenced_ext_path.clone();
                let manifest_path = std::path::Path::new(&extension_path).join("manifest.json");
                let manifest = match fs::read_to_string(&manifest_path){
                    Ok(content) => content,
                    Err(_e) => {
                        println!("err {:?} while read manifest_path {:?}", _e, &manifest_path);
                        continue;
                    }
                };
                let extensions = ChromeProfileSnapshotExtension {
                    manifest : manifest,
                    path : extension_path
                };
                referenced_extensions.insert(referenced_ext_path.clone(), extensions);

            }
        }
        unreferenced_extensions = unreferenced_extensions_temp;
    }
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
        println!("opt_extensions_node has extensions");
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
                    Ok(_t) => _t.to_str().unwrap().to_string(),
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