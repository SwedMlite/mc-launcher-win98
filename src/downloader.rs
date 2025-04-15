use crate::models::{AssetIndexData, Extract, Library, VersionData};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::error::Error;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub fn download_file(url: &str, dest_path: &Path) -> Result<(), Box<dyn Error>> {
    if dest_path.exists() {
        return Ok(());
    }
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Error downloading file: {}", response.status()),
        )));
    }

    let mut dest_file = File::create(dest_path)?;
    let bytes = response.bytes()?;
    io::copy(&mut &bytes[..], &mut dest_file)?;
    Ok(())
}

pub fn download_libraries(
    version_data: &VersionData,
    libraries_dir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

pub fn download_and_extract_natives(
    version_data: &VersionData,
    natives_dir: &Path,
) -> Result<(), Box<dyn Error>> {
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
                            extract_natives_from_jar(
                                &natives_jar_path,
                                natives_dir,
                                &library.extract,
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_natives_from_jar(
    jar_path: &Path,
    dest_dir: &Path,
    extract: &Option<Extract>,
) -> Result<(), Box<dyn Error>> {
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

pub fn download_and_extract_assets(
    version_data: &VersionData,
    game_dir: &Path,
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

    let assets_to_download: Vec<(String, PathBuf, String)> = asset_index_data
        .objects
        .iter()
        .map(|(virtual_path, asset)| {
            let hash = &asset.hash;
            let first_two = &hash[0..2];
            let asset_url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                first_two, hash
            );
            let asset_dest = objects_dir.join(first_two).join(hash);
            (asset_url, asset_dest, virtual_path.clone())
        })
        .collect();

    assets_to_download
        .par_iter()
        .for_each(|(url, dest, virtual_path)| {
            if let Some(parent) = dest.parent() {
                if let Err(_) = fs::create_dir_all(parent) {
                    return;
                }
            }

            if !dest.exists() {
                if let Err(_) = download_file(url, dest) {
                    return;
                }
            }

            if is_legacy {
                let legacy_file_path = legacy_dir.join(virtual_path);

                if let Some(parent) = legacy_file_path.parent() {
                    if let Err(_) = fs::create_dir_all(parent) {
                        return;
                    }
                }

                if !legacy_file_path.exists() {
                    let _ = fs::copy(dest, &legacy_file_path);
                }
            } else if asset_index_id == "1.7.10"
                || asset_index_id.parse::<f32>().unwrap_or(0.0) <= 1.8
            {
                let resource_file_path = resource_dir.join(virtual_path);

                if let Some(parent) = resource_file_path.parent() {
                    if let Err(_) = fs::create_dir_all(parent) {
                        return;
                    }
                }

                if !resource_file_path.exists() {
                    let _ = fs::copy(dest, &resource_file_path);
                }
            }
        });

    Ok(())
}

fn should_use_library(library: &Library) -> bool {
    if library.rules.is_empty() {
        return true;
    }

    let mut allow = false;

    for rule in &library.rules {
        let applies = match &rule.os {
            Some(os) => match &os.name {
                Some(name) => {
                    #[cfg(target_os = "windows")]
                    let current_os = "windows";
                    #[cfg(target_os = "linux")]
                    let current_os = "linux";
                    #[cfg(target_os = "macos")]
                    let current_os = "osx";
                    name == current_os
                }
                None => true,
            },
            None => true,
        };

        if applies {
            allow = rule.action == "allow";
        }
    }

    allow
}

fn get_current_os() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(target_os = "macos")]
    return "osx";
}