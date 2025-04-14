use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::process::Command;
use std::error::Error;
use std::collections::HashMap;
use std::io;
use std::io::{BufReader,BufWriter};
use zip::ZipArchive;
use regex;

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct VersionData {
    pub downloads: Downloads,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default, rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    #[serde(default)]
    pub javaVersion: Option<JavaVersion>,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(default, rename = "type")]
    pub _type: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: DownloadInfo,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndexData {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
pub struct AssetObject {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadInfo {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub extract: Option<Extract>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub url: String
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<Os>,
}

#[derive(Debug, Deserialize)]
pub struct Os {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)] 
pub struct Profile {
    pub username: String,
}

pub fn read_profiles(path: &Path) -> Result<Vec<Profile>, Box<dyn std::error::Error>> {
    if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let profiles: Vec<Profile> = serde_json::from_reader(reader)?;
        Ok(profiles)
    } else {
        Ok(Vec::new())
    }
}

pub fn write_profiles(path: &Path, profiles: &[Profile]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, profiles)?;
    Ok(())
}

pub fn fetch_version_manifest(manifest_url: &str) -> Result<VersionManifest, Box<dyn Error>> {
    let response = reqwest::blocking::get(manifest_url)?;
    let manifest_data: VersionManifest = response.json()?;
    Ok(manifest_data)
}

pub fn fetch_version_data(version_url: &str) -> Result<VersionData, Box<dyn Error>> {
    let response = reqwest::blocking::get(version_url)?;
    let version_data: VersionData = response.json()?;
    Ok(version_data)
}

pub fn get_version_ids() -> String {
    let mut versions = String::new();
    match fetch_version_manifest("https://launchermeta.mojang.com/mc/game/version_manifest.json") {
        Ok(manifest) => {
            for version in &manifest.versions { 
                versions.push_str(&format!("{}|{}|", version.id, version._type));
            }
        }
        Err(e) => eprintln!("Failed to fetch versions: {}", e),
    }
    return versions;
}

pub fn get_version_link(version_id: String) -> Option<String> {
    match fetch_version_manifest("https://launchermeta.mojang.com/mc/game/version_manifest.json") {
        Ok(manifest) => {
            for version in manifest.versions {
                if version.id == version_id {
                    return Some(version.url);
                }
            }
            None
        }
        Err(e) => {
            eprintln!("Failed to fetch versions: {}", e);
            None
        }
    }
}

pub fn download_file(url: &str, dest_path: &Path) -> Result<(), Box<dyn Error>> {
    if dest_path.exists() {
        println!("File {} already exists.", dest_path.display());
        return Ok(());
    }
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Error downloading file: {}", response.status())
        )));
    }
    
    let mut dest_file = File::create(dest_path)?;
    let bytes = response.bytes()?;
    io::copy(&mut &bytes[..], &mut dest_file)?;
    Ok(())
}

pub fn download_libraries(version_data: &VersionData, libraries_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut classpath = Vec::new();
    fs::create_dir_all(libraries_dir)?;
    
    for library in &version_data.libraries {
        if !should_use_library(library) {
            continue;
        }
        
        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                let library_path = libraries_dir.join(&artifact.path);
                if let Some(parent) = library_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                if !library_path.exists() {
                    download_file(&artifact.url, &library_path)?;
                }
                classpath.push(library_path);
            }
        }
    }
    
    Ok(classpath)
}

pub fn download_and_extract_natives(version_data: &VersionData, natives_dir: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(natives_dir)?;

    for library in &version_data.libraries {
        if !should_use_library(library) {
            continue;
        }

        if let Some(natives) = &library.natives {
            let os = get_current_os();
            if let Some(classifier) = natives.get(os) {
                if let Some(downloads) = &library.downloads {
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(artifact) = classifiers.get(classifier) {
                            let natives_jar_path = natives_dir.join(&artifact.path);
                            if let Some(parent) = natives_jar_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            if !natives_jar_path.exists() {
                                download_file(&artifact.url, &natives_jar_path)?;
                            }
                            extract_natives_from_jar(&natives_jar_path, natives_dir, &library.extract)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_natives_from_jar(jar_path: &Path, dest_dir: &Path, extract: &Option<Extract>) -> Result<(), Box<dyn Error>> {
    let file = File::open(jar_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();
        
        if let Some(extract_info) = extract {
            if let Some(exclude) = &extract_info.exclude {
                if exclude.iter().any(|e| file_name.starts_with(e)) {
                    continue;
                }
            }
        }

        let outpath = dest_dir.join(file_name);
        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn get_current_os() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(target_os = "macos")]
    return "osx";
}

fn should_use_library(library: &Library) -> bool {
    if library.rules.is_empty() {
        return true;
    }
    
    let mut allow = false;
    
    for rule in &library.rules {
        let applies = match &rule.os {
            Some(os) => {
                match &os.name {
                    Some(name) => {
                        #[cfg(target_os = "windows")]
                        let current_os = "windows";
                        #[cfg(target_os = "linux")]
                        let current_os = "linux";
                        #[cfg(target_os = "macos")]
                        let current_os = "osx";
                        name == current_os
                    },
                    None => true,
                }
            },
            None => true,
        };
        
        if applies {
            allow = rule.action == "allow";
        }
    }
    
    allow
}

pub fn download_and_extract_assets(
    version_data: &VersionData, 
    game_dir: &PathBuf
) -> Result<(), Box<dyn Error>> {
    let asset_index_url = &version_data.asset_index.url;
    let asset_index_id = &version_data.asset_index.id;
    let assets_dir = game_dir.join("assets");
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");
    let legacy_dir = assets_dir.join("legacy");
    let resource_dir = assets_dir.join("resources");
    
    fs::create_dir_all(&indexes_dir)?;
    fs::create_dir_all(&objects_dir)?;
    
    let asset_index_path = indexes_dir.join(format!("{}.json", asset_index_id));
    if !asset_index_path.exists() {
        println!("Downloading asset index: {}", asset_index_id);
        download_file(asset_index_url, &asset_index_path)?;
    }
    
    let asset_index_content = fs::read_to_string(&asset_index_path)?;
    let asset_index_data: AssetIndexData = serde_json::from_str(&asset_index_content)?;
    
    let is_legacy = asset_index_id == "legacy" || asset_index_id == "pre-1.6";
    
    if is_legacy {
        fs::create_dir_all(&legacy_dir)?;
    } else if asset_index_id == "1.7.10" || asset_index_id.parse::<f32>().unwrap_or(0.0) <= 1.8 {
        fs::create_dir_all(&resource_dir)?;
    }
    
    let assets_to_download: Vec<(String, PathBuf, String)> = asset_index_data.objects
        .iter()
        .map(|(virtual_path, asset)| {
            let hash = &asset.hash;
            let first_two = &hash[0..2];
            let asset_url = format!(
                "https://resources.download.minecraft.net/{}/{}", 
                first_two, hash
            );
            let asset_dest = objects_dir
                .join(first_two)
                .join(hash);
            (asset_url, asset_dest, virtual_path.clone())
        })
        .collect();
    
    assets_to_download.par_iter().for_each(|(url, dest, virtual_path)| {
        if let Some(parent) = dest.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Error creating directory for asset {}: {}", dest.display(), e);
                return;
            }
        }
        
        if !dest.exists() {
            if let Err(e) = download_file(url, dest) {
                eprintln!("Error downloading asset {}: {}", url, e);
                return;
            }
        }
        
        if is_legacy {
            let legacy_file_path = legacy_dir.join(virtual_path);
            
            if let Some(parent) = legacy_file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Error creating directory for legacy asset: {}", e);
                    return;
                }
            }
            
            if !legacy_file_path.exists() {
                if let Err(e) = fs::copy(dest, &legacy_file_path) {
                    eprintln!("Error copying asset to legacy directory: {}", e);
                }
            }
        } else if asset_index_id == "1.7.10" || asset_index_id.parse::<f32>().unwrap_or(0.0) <= 1.8 {
            let resource_file_path = resource_dir.join(virtual_path);
            
            if let Some(parent) = resource_file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Error creating directory for resource asset: {}", e);
                    return;
                }
            }
            
            if !resource_file_path.exists() {
                if let Err(e) = fs::copy(dest, &resource_file_path) {
                    eprintln!("Error copying asset to resources directory: {}", e);
                }
            }
        }
    });
    
    Ok(())
}

pub fn find_all_java_installations() -> Vec<(PathBuf, String)> {
    let mut java_installations = Vec::new();

    // Try system Java in PATH
    let default_java = if cfg!(windows) { "java.exe" } else { "java" };
    if let Some(version) = get_java_full_version(&PathBuf::from(default_java)) {
        java_installations.push((PathBuf::from(default_java), version));
    }

    // Check JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_exe = PathBuf::from(&java_home).join("bin").join(if cfg!(windows) { "java.exe" } else { "java" });
        if java_exe.exists() {
            if let Some(version) = get_java_full_version(&java_exe) {
                if !java_installations.iter().any(|(path, _)| path == &java_exe) {
                    java_installations.push((java_exe, version));
                }
            }
        }
    }

    // Define common Java installation directories
    #[cfg(target_os = "windows")]
    let java_dirs = vec![
        r"C:\Program Files\Java",
        r"C:\Program Files (x86)\Java",
        r"C:\Program Files\AdoptOpenJDK",
        r"C:\Program Files (x86)\AdoptOpenJDK",
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files (x86)\Eclipse Adoptium",
        r"C:\Program Files\Zulu",
        r"C:\Program Files (x86)\Zulu",
        r"C:\Program Files\BellSoft",
        r"C:\Program Files (x86)\BellSoft",
    ];
    
    #[cfg(target_os = "linux")]
    let java_dirs = vec![
        "/usr/lib/jvm",
        "/usr/java",
        "/opt/java",
    ];
    
    #[cfg(target_os = "macos")]
    let java_dirs = vec![
        "/Library/Java/JavaVirtualMachines",
        "/System/Library/Java/JavaVirtualMachines",
        "/Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home",
    ];

    // Search in standard locations
    for dir in java_dirs {
        if !Path::new(dir).exists() {
            continue;
        }
        
        let java_paths = find_java_executables(dir);
        
        for java_path in java_paths {
            if let Some(version) = get_java_full_version(&java_path) {
                if !java_installations.iter().any(|(path, _)| path == &java_path) {
                    java_installations.push((java_path, version));
                }
            }
        }
    }

    // Sort by major version for consistency
    java_installations.sort_by(|(_, ver_a), (_, ver_b)| {
        let major_a = ver_a.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        let major_b = ver_b.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        major_a.cmp(&major_b)
    });
    
    java_installations
}

pub fn launch_minecraft(
    version_id: &String, 
    username: &String, 
    version_data: &VersionData,
    custom_java_path: Option<PathBuf>
) -> Result<(), Box<dyn Error>> {
    let game_dir = PathBuf::from("game");
    let minecraft_dir = game_dir.join(".minecraft");
    fs::create_dir_all(&minecraft_dir)?;

    let jar_path = game_dir.join("versions").join(version_id).join("client.jar");
    let libraries_dir = game_dir.join("libraries");
    fs::create_dir_all(&libraries_dir)?;

    let library_paths = download_libraries(version_data, &libraries_dir)?;

    let mut classpath = String::new();
    classpath.push_str(&jar_path.to_string_lossy());
    for path in &library_paths {
        #[cfg(target_os = "windows")]
        classpath.push_str(";");
        #[cfg(not(target_os = "windows"))]
        classpath.push_str(":");
        classpath.push_str(&path.to_string_lossy());
    }

    let natives_dir = game_dir.join("natives").join(version_id);
    fs::create_dir_all(&natives_dir)?;
    download_and_extract_natives(version_data, &natives_dir)?;

    download_and_extract_assets(version_data, &game_dir)?;

    let is_alpha_or_beta = version_id.starts_with("a") || version_id.starts_with("b");
    
    let (required_java_version, strict_match) = match &version_data.javaVersion {
        Some(java_version) => (java_version.major_version, false),
        None => {
            if is_alpha_or_beta || 
               version_id.starts_with("1.8") || 
               version_id.starts_with("1.7") ||
               version_id.starts_with("1.6") || 
               version_id.starts_with("1.5") ||
               version_id.starts_with("1.4") ||
               version_id.starts_with("1.3") ||
               version_id.starts_with("1.2") ||
               version_id.starts_with("1.1") ||
               version_id.starts_with("1.0") {
                (8, true)  // Legacy versions need exactly Java 8
            } else {
                (17, false) // Default to Java 17 for newer versions
            }
        }
    };
    
    // Use custom Java path if provided, otherwise find compatible Java
    let java_path = match custom_java_path {
        Some(path) => path,
        None => match find_compatible_java(required_java_version, strict_match) {
            Some(path) => path,
            None => {
                println!("Warning: Java {} not found for Minecraft version {}.", required_java_version, version_id);
                println!("Attempting to use system Java, but this may cause errors.");
                PathBuf::from(if cfg!(windows) { "java.exe" } else { "java" })
            }
        }
    };
    
    println!("Minecraft version: {}, Required Java: {}, Using Java: {}", 
             version_id, required_java_version, java_path.display());
    
    let asset_index_id = &version_data.asset_index.id;
    let (assets_dir_arg, assets_dir_path) = if asset_index_id == "legacy" || asset_index_id == "pre-1.6" {
        ("--assetsDir", game_dir.join("assets").join("legacy"))
    } else {
        ("--assetsDir", game_dir.join("assets"))
    };

    let mut launch_cmd = Command::new(&java_path);
    
    launch_cmd
        .arg("-Djava.library.path=".to_string() + &natives_dir.to_string_lossy())
        .arg("-Dminecraft.launcher.brand=CustomLauncher")
        .arg("-Dminecraft.launcher.version=1.0")
        .arg("-cp")
        .arg(&classpath);
    
    if is_alpha_or_beta {
        launch_cmd.arg("net.minecraft.client.Minecraft");
        launch_cmd.arg(username).arg("token:0:0");
    } else {
        let main_class = if !version_data.main_class.is_empty() {
            &version_data.main_class
        } else {
            "net.minecraft.client.main.Main"
        };
        launch_cmd.arg(main_class);
        
        launch_cmd
            .arg("--username")
            .arg(username)
            .arg("--version")
            .arg(version_id)
            .arg("--gameDir")
            .arg(minecraft_dir.to_string_lossy().to_string())
            .arg(assets_dir_arg)
            .arg(assets_dir_path.to_string_lossy().to_string());

        if asset_index_id != "legacy" && asset_index_id != "pre-1.6" {
            launch_cmd
                .arg("--assetIndex")
                .arg(asset_index_id);
        }

        launch_cmd
            .arg("--accessToken")
            .arg("0")
            .arg("--uuid")
            .arg("00000000-0000-0000-0000-000000000000");
    }

    println!("Launching Minecraft with command: {:?}", launch_cmd);
    let status = launch_cmd.status()?;

    if !status.success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Process exited with code: {}", status)
        )));
    }
    Ok(())
}

fn find_compatible_java(required_version: u32, strict_match: bool) -> Option<PathBuf> {
    // Define common Java installation directories with more specific paths for Java 8
    #[cfg(target_os = "windows")]
    let java_dirs = vec![
        r"C:\Program Files\Java",
        r"C:\Program Files (x86)\Java",
        r"C:\Program Files\AdoptOpenJDK",
        r"C:\Program Files (x86)\AdoptOpenJDK",
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files (x86)\Eclipse Adoptium",
        r"C:\Program Files\Zulu",
        r"C:\Program Files (x86)\Zulu",
    ];
    
    #[cfg(target_os = "linux")]
    let java_dirs = vec![
        "/usr/lib/jvm",
        "/usr/java",
        "/opt/java",
    ];
    
    #[cfg(target_os = "macos")]
    let java_dirs = vec![
        "/Library/Java/JavaVirtualMachines",
        "/System/Library/Java/JavaVirtualMachines",
        "/Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home",
    ];
    
    // For legacy Minecraft (requiring Java 8), try to find exact Java 8
    if strict_match && required_version == 8 {
        // First check hardcoded common Java 8 paths
        #[cfg(target_os = "windows")]
        let java8_specific_paths = vec![
            r"C:\Program Files\Java\jre1.8.0_",
            r"C:\Program Files\Java\jdk1.8.0_",
            r"C:\Program Files (x86)\Java\jre1.8.0_",
            r"C:\Program Files (x86)\Java\jdk1.8.0_",
        ];
        
        #[cfg(target_os = "windows")]
        for base_path in java8_specific_paths {
            // Try common update numbers
            for update in &[401, 361, 351, 333, 321, 311, 301, 291, 281, 271, 261, 251, 241, 231, 221, 211, 202, 201, 191, 181, 171, 161, 151, 141, 131, 121, 111, 101, 91, 81, 71, 65, 60, 51, 45, 40, 31, 25, 20, 11, 5] {
                let potential_path = format!("{}{}", base_path, update);
                if PathBuf::from(&potential_path).exists() {
                    let java_exe = PathBuf::from(&potential_path).join("bin").join("java.exe");
                    if java_exe.exists() {
                        if let Some(version) = get_java_version(&java_exe) {
                            if version == 8 {
                                println!("Found exact Java 8 match at: {}", java_exe.display());
                                return Some(java_exe);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Try JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_exe = PathBuf::from(&java_home).join("bin").join(if cfg!(windows) { "java.exe" } else { "java" });
        if java_exe.exists() {
            if let Some(version) = get_java_version(&java_exe) {
                if version == required_version {
                    println!("Found exact Java {} match in JAVA_HOME: {}", required_version, java_exe.display());
                    return Some(java_exe);
                } else if !strict_match && version > required_version {
                    println!("Found compatible Java {} in JAVA_HOME: {}", version, java_exe.display());
                    // Only use if we're not strict matching
                    if !strict_match {
                        return Some(java_exe);
                    }
                }
            }
        }
    }
    
    let mut exact_matches = Vec::new();
    let mut compatible_matches = Vec::new();
    
    // Search for Java in standard locations
    for dir in java_dirs {
        if !Path::new(dir).exists() {
            continue;
        }
        
        let java_paths = find_java_executables(dir);
        
        for java_path in java_paths {
            if let Some(version) = get_java_version(&java_path) {
                if version == required_version {
                    println!("Found exact Java {} match: {}", required_version, java_path.display());
                    exact_matches.push(java_path);
                } else if !strict_match && version > required_version {
                    println!("Found compatible Java {}: {}", version, java_path.display());
                    compatible_matches.push((java_path, version));
                }
            }
        }
    }
    
    // For strict matching (like legacy Minecraft versions), we only want exact matches
    if !exact_matches.is_empty() {
        return Some(exact_matches[0].clone());
    }
    
    // For legacy Minecraft, try a last resort search on Windows
    #[cfg(target_os = "windows")]
    if strict_match && required_version == 8 {
        // Try to directly check disk drives for Program Files/Java folders
        for drive in &["C:", "D:", "E:", "F:"] {
            let patterns = vec![
                format!(r"{}\Program Files\Java", drive),
                format!(r"{}\Program Files (x86)\Java", drive),
            ];
            
            for pattern in patterns {
                if PathBuf::from(&pattern).exists() {
                    let subdirs = match std::fs::read_dir(&pattern) {
                        Ok(dir) => dir,
                        Err(_) => continue,
                    };
                    
                    for entry in subdirs.filter_map(Result::ok) {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = path.file_name().unwrap_or_default().to_string_lossy();
                            if name.contains("jre1.8") || name.contains("jdk1.8") || name.contains("jre8") || name.contains("jdk8") {
                                let java_exe = path.join("bin").join("java.exe");
                                if java_exe.exists() {
                                    if let Some(version) = get_java_version(&java_exe) {
                                        if version == 8 {
                                            println!("Last resort: Found Java 8 at: {}", java_exe.display());
                                            return Some(java_exe);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // For non-strict matching, we prefer the closest higher version
    if !strict_match && !compatible_matches.is_empty() {
        // Sort by version to get the closest higher version
        compatible_matches.sort_by_key(|(_, version)| *version);
        return Some(compatible_matches[0].0.clone());
    }
    
    // Try system java as a last resort
    let default_java = if cfg!(windows) { "java.exe" } else { "java" };
    if let Some(version) = get_java_version(&PathBuf::from(default_java)) {
        println!("System Java version: {}", version);
        if version == required_version {
            return Some(PathBuf::from(default_java));
        } else if !strict_match && version > required_version {
            return Some(PathBuf::from(default_java));
        }
    }
    
    None
}

fn find_java_executables(dir: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let java_exe_name = if cfg!(windows) { "java.exe" } else { "java" };
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let mut sub_results = find_java_executables(&path.to_string_lossy());
                result.append(&mut sub_results);
            } else if path.file_name().map_or(false, |name| name == java_exe_name) {
                result.push(path);
            }
        }
    }
    
    result
}

fn get_java_full_version(java_path: &Path) -> Option<String> {
    if let Ok(output) = Command::new(java_path).arg("-version").output() {
        let version_str = String::from_utf8_lossy(&output.stderr);
        
        // Modern Java versions (9+): java version "11.0.12" 2021-07-20
        if let Some(cap) = regex::Regex::new(r#"version\s+"(\d+(?:\.\d+)*(?:_\d+)?(?:-[a-zA-Z0-9]+)?)"#).ok()
            .and_then(|re| re.captures(&version_str)) {
            if let Some(version) = cap.get(1) {
                return Some(version.as_str().to_string());
            }
        }
        
        // Legacy Java versions (1.8, 1.7): java version "1.8.0_XXX"
        if let Some(cap) = regex::Regex::new(r#"version\s+"(1\.\d+\.\d+(?:_\d+)?(?:-[a-zA-Z0-9]+)?)"#).ok()
            .and_then(|re| re.captures(&version_str)) {
            if let Some(version) = cap.get(1) {
                return Some(version.as_str().to_string());
            }
        }
    }
    None
}

fn get_java_version(java_path: &Path) -> Option<u32> {
    if let Some(full_version) = get_java_full_version(java_path) {
        // Check if it's a new format (Java 9+) or old format (1.X)
        if full_version.starts_with("1.") {
            if let Some(minor) = full_version.split('.').nth(1) {
                if let Ok(version) = minor.parse::<u32>() {
                    return Some(version);
                }
            }
        } else {
            // For Java 9+, the major version is the first part
            if let Some(major) = full_version.split('.').next() {
                if let Ok(version) = major.parse::<u32>() {
                    return Some(version);
                }
            }
        }
    }
    None
}