use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

pub fn find_all_java_installations() -> Vec<(PathBuf, String)> {
    let mut java_installations = Vec::new();

    let default_java = if cfg!(windows) { "java.exe" } else { "java" };
    if let Some(version) = get_java_full_version(&PathBuf::from(default_java)) {
        java_installations.push((PathBuf::from(default_java), version));
    }

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

    java_installations.sort_by(|(_, ver_a), (_, ver_b)| {
        let major_a = ver_a.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        let major_b = ver_b.split('.').next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        major_a.cmp(&major_b)
    });
    
    java_installations
}

pub fn find_compatible_java(required_version: u32, strict_match: bool) -> Option<PathBuf> {
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
    
    if strict_match && required_version == 8 {
        #[cfg(target_os = "windows")]
        let java8_specific_paths = vec![
            r"C:\Program Files\Java\jre1.8.0_",
            r"C:\Program Files\Java\jdk1.8.0_",
            r"C:\Program Files (x86)\Java\jre1.8.0_",
            r"C:\Program Files (x86)\Java\jdk1.8.0_",
        ];
        
        #[cfg(target_os = "windows")]
        for base_path in java8_specific_paths {
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
    
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_exe = PathBuf::from(&java_home).join("bin").join(if cfg!(windows) { "java.exe" } else { "java" });
        if java_exe.exists() {
            if let Some(version) = get_java_version(&java_exe) {
                if version == required_version {
                    println!("Found exact Java {} match in JAVA_HOME: {}", required_version, java_exe.display());
                    return Some(java_exe);
                } else if !strict_match && version > required_version {
                    println!("Found compatible Java {} in JAVA_HOME: {}", version, java_exe.display());
                    if !strict_match {
                        return Some(java_exe);
                    }
                }
            }
        }
    }
    
    let mut exact_matches = Vec::new();
    let mut compatible_matches = Vec::new();
    
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
    
    if !exact_matches.is_empty() {
        return Some(exact_matches[0].clone());
    }
    
    #[cfg(target_os = "windows")]
    if strict_match && required_version == 8 {
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
    
    if !strict_match && !compatible_matches.is_empty() {
        compatible_matches.sort_by_key(|(_, version)| *version);
        return Some(compatible_matches[0].0.clone());
    }
    
    let default_java = if cfg!(windows) { "java.exe" } else { "java" };
    if let Some(version) = get_java_version(&PathBuf::from(default_java)) {
        println!("System Java version: {}", version);
        if version == required_version || (!strict_match && version > required_version) {
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
            } else if path.file_name().is_some_and(|name| name == java_exe_name) {
                result.push(path);
            }
        }
    }
    
    result
}

fn get_java_full_version(java_path: &Path) -> Option<String> {
    if let Ok(output) = Command::new(java_path).arg("-version").output() {
        let version_str = String::from_utf8_lossy(&output.stderr);
        
        if let Some(cap) = regex::Regex::new(r#"version\s+"(\d+(?:\.\d+)*(?:_\d+)?(?:-[a-zA-Z0-9]+)?)"#).ok()
            .and_then(|re| re.captures(&version_str)) {
            if let Some(version) = cap.get(1) {
                return Some(version.as_str().to_string());
            }
        }
        
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
        if full_version.starts_with("1.") {
            if let Some(minor) = full_version.split('.').nth(1) {
                if let Ok(version) = minor.parse::<u32>() {
                    return Some(version);
                }
            }
        } else if let Some(major) = full_version.split('.').next() {
            if let Ok(version) = major.parse::<u32>() {
                return Some(version);
            }
        }
    }
    None
} 