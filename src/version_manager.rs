use std::error::Error;
use crate::models::{VersionManifest, VersionData};

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
    versions
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