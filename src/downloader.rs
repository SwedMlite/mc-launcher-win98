use crate::models::{AssetIndexData, Extract, Library, VersionData};
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelRefIterator, ParallelIterator},
};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

pub fn download_file(url: &str, dest_path: &Path) -> Result<(), Box<dyn Error>> {
    if dest_path.exists() {
        return Ok(());
    }

    let client = reqwest::blocking::Client::builder()
        .pool_max_idle_per_host(10)
        .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
        .build()?;

    let response = client.get(url).send()?;
    if !response.status().is_success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Error downloading file: {}", response.status()),
        )));
    }

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut dest_file = File::create(dest_path)?;
    let bytes = response.bytes()?;
    dest_file.write_all(&bytes)?;

    Ok(())
}

pub fn download_libraries(
    version_data: &VersionData,
    libraries_dir: &Path,
    mut progress_callback: Option<&mut dyn FnMut(usize, usize, &str)>,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut classpath = Vec::new();
    fs::create_dir_all(libraries_dir)?;

    let libraries_to_download: Vec<&Library> = version_data
        .libraries
        .iter()
        .filter(|lib| should_use_library(lib))
        .collect();

    let total = libraries_to_download.len();

    for (i, library) in libraries_to_download.iter().enumerate() {
        let name = library
            .downloads
            .as_ref()
            .and_then(|d| d.artifact.as_ref())
            .map(|a| a.path.clone())
            .unwrap_or_else(|| "Unknown library".to_string());

        if let Some(ref mut callback) = progress_callback {
            callback(i, total, &name);
        }

        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                let library_path = libraries_dir.join(&artifact.path);
                if !library_path.exists() {
                    if let Ok(_) = download_file(&artifact.url, &library_path) {
                        classpath.push(library_path);
                    }
                } else {
                    classpath.push(library_path);
                }
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
    progress_callback: Option<impl Fn(usize, usize, &str) + Send + Sync>,
) -> Result<(), Box<dyn Error>> {
    let _ = ThreadPoolBuilder::new().num_threads(16).build_global();

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

    let _total_assets = assets_to_download.len();
    let counter = std::sync::atomic::AtomicUsize::new(0);

    let required_assets: Vec<_> = assets_to_download
        .iter()
        .filter(|(_, _, path)| {
            path.contains("minecraft/sounds/ui/")
                || path.contains("minecraft/sounds/random/click")
                || path.contains("minecraft/lang/")
                || path.contains("minecraft/textures/gui/")
                || path.contains("minecraft/font/")
        })
        .collect();

    if !required_assets.is_empty() {
        let required_total = required_assets.len();

        required_assets
            .par_iter()
            .for_each(|(url, dest, virtual_path)| {
                let current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

                if let Some(callback) = &progress_callback {
                    callback(
                        current.min(required_total),
                        required_total,
                        &format!("Required: {}", virtual_path),
                    );
                }

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
    }

    if let Some(callback) = &progress_callback {
        callback(100, 100, "Required assets downloaded. Launching game...");
    }

    let remaining_assets: Vec<(String, PathBuf, String)> = assets_to_download
        .into_iter()
        .filter(|(_, dest, _)| !dest.exists())
        .collect();

    if !remaining_assets.is_empty() {
        let _objects_dir_clone = objects_dir.to_path_buf();
        let legacy_dir_clone = legacy_dir.to_path_buf();
        let resource_dir_clone = resource_dir.to_path_buf();
        let asset_index_id_clone = asset_index_id.to_string();
        let is_legacy_clone = is_legacy;

        std::thread::spawn(move || {
            if remaining_assets.is_empty() {
                return;
            }

            let _remaining_total = remaining_assets.len();
            let remaining_counter = std::sync::atomic::AtomicUsize::new(0);

            remaining_assets
                .par_iter()
                .for_each(|(url, dest, virtual_path)| {
                    let _current =
                        remaining_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

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

                    if is_legacy_clone {
                        let legacy_file_path = legacy_dir_clone.join(virtual_path);

                        if let Some(parent) = legacy_file_path.parent() {
                            if let Err(_) = fs::create_dir_all(parent) {
                                return;
                            }
                        }

                        if !legacy_file_path.exists() {
                            let _ = fs::copy(dest, &legacy_file_path);
                        }
                    } else if asset_index_id_clone == "1.7.10"
                        || asset_index_id_clone.parse::<f32>().unwrap_or(0.0) <= 1.8
                    {
                        let resource_file_path = resource_dir_clone.join(virtual_path);

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
        });
    }

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
