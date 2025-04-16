use crate::{
    downloader,
    models::{LaunchProgress, LaunchStage, VersionData},
};
use std::{fs, io, path::PathBuf, process::Command, sync::mpsc, thread, time::Duration};

pub fn launch_minecraft(
    version_id: &str,
    username: &str,
    version_data: &VersionData,
    java_path: Option<PathBuf>,
    jvm_args: Option<Vec<String>>,
    progress: Option<mpsc::Sender<LaunchProgress>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let send_progress = |stage: LaunchStage, current: usize, total: usize, message: String| {
        if let Some(progress_sender) = &progress {
            let _ = progress_sender.send(LaunchProgress {
                stage,
                current,
                total,
                message,
            });
            thread::sleep(Duration::from_millis(15));
        }
    };

    send_progress(
        LaunchStage::PreparingLibraries,
        0,
        100,
        "Preparing environment...".to_string(),
    );

    let game_dir = crate::get_game_directory();
    let version_dir = game_dir.join("versions").join(version_id);

    let client_jar = version_dir.join("client.jar");

    let natives_dir = version_dir.join("natives");

    if !natives_dir.exists() {
        fs::create_dir_all(&natives_dir)?;
    }

    send_progress(
        LaunchStage::PreparingLibraries,
        10,
        100,
        "Reading version information...".to_string(),
    );

    let libraries = &version_data.libraries;
    let lib_total = libraries.len();

    send_progress(
        LaunchStage::DownloadingLibraries,
        20,
        100,
        format!("Preparing libraries (0/{})...", lib_total),
    );

    let cache_dir = dirs::config_dir()
        .unwrap()
        .join("minecraft_launcher")
        .join("cache");

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    let mut progress_index = 0;
    let mut progress_callback = |i: usize, total: usize, name: &str| {
        progress_index = i;
        send_progress(
            LaunchStage::DownloadingLibraries,
            20 + (i * 20 / total),
            100,
            format!("Preparing libraries ({}/{}): {}", i + 1, total, name),
        );
    };

    let classpath_paths = downloader::download_libraries(
        version_data,
        &cache_dir,
        Some(&mut progress_callback as &mut dyn FnMut(usize, usize, &str)),
    )?;

    let mut classpath = String::new();
    for (i, path) in classpath_paths.iter().enumerate() {
        if i > 0 {
            classpath.push_str(if cfg!(windows) { ";" } else { ":" });
        }
        classpath.push_str(&path.to_string_lossy());
    }

    downloader::download_and_extract_natives(version_data, &natives_dir)?;

    send_progress(
        LaunchStage::ExtractingNatives,
        50,
        100,
        "Native libraries extracted".to_string(),
    );

    send_progress(
        LaunchStage::PreparingAssets,
        50,
        100,
        "Preparing game assets...".to_string(),
    );

    let game_assets_dir = game_dir.clone();

    let progress_fn = move |current: usize, total: usize, message: &str| {
        let percentage = 60 + (current * 15 / total);
        send_progress(
            LaunchStage::DownloadingAssets,
            percentage,
            100,
            format!("Downloading assets: {}", message),
        );
    };

    downloader::download_and_extract_assets(version_data, &game_assets_dir, Some(progress_fn))?;

    send_progress(
        LaunchStage::AssetLoadComplete,
        75,
        100,
        "Essential assets ready".to_string(),
    );

    send_progress(
        LaunchStage::ValidatingJava,
        80,
        100,
        "Validating Java installation...".to_string(),
    );

    if !classpath.is_empty() {
        classpath.push_str(if cfg!(windows) { ";" } else { ":" });
    }
    classpath.push_str(&client_jar.to_string_lossy());

    send_progress(
        LaunchStage::BuildingArguments,
        85,
        100,
        "Building game arguments...".to_string(),
    );

    let java_executable = java_path.unwrap_or_else(|| "java".into());
    let mut command = Command::new(java_executable);

    if let Some(args) = jvm_args {
        for arg in args {
            command.arg(arg);
        }
    }

    command.arg("-Djava.library.path=".to_string() + &natives_dir.to_string_lossy());
    command.arg("-cp");
    command.arg(&classpath);
    command.arg(&version_data.main_class);

    command.arg("--username");
    command.arg(username);
    command.arg("--version");
    command.arg(version_id);
    command.arg("--gameDir");
    command.arg(game_assets_dir.to_string_lossy().to_string());
    command.arg("--accessToken");
    command.arg("0");
    command.arg("--assetsDir");
    command.arg(game_assets_dir.join("assets").to_string_lossy().to_string());
    command.arg("--assetIndex");
    command.arg(&version_data.asset_index.id);
    command.arg("--uuid");
    command.arg("00000000-0000-0000-0000-000000000000");
    command.arg("--userProperties");
    command.arg("{}");

    send_progress(
        LaunchStage::StartingProcess,
        90,
        100,
        "Starting game process...".to_string(),
    );

    match command.spawn() {
        Ok(mut child) => {
            send_progress(
                LaunchStage::ProcessStarted,
                95,
                100,
                "Process started successfully".to_string(),
            );

            thread::sleep(Duration::from_millis(2000));

            match child.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code().unwrap_or(-1);
                    let error_message =
                        format!("Game process exited immediately with code: {}", exit_code);

                    send_progress(LaunchStage::Complete, 100, 100, error_message.clone());

                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        error_message,
                    )));
                }
                Ok(None) => {
                    send_progress(
                        LaunchStage::LaunchingGame,
                        98,
                        100,
                        "Minecraft is starting...".to_string(),
                    );

                    for i in 0..5 {
                        thread::sleep(Duration::from_millis(1000));

                        match child.try_wait() {
                            Ok(Some(status)) => {
                                let exit_code = status.code().unwrap_or(-1);

                                if exit_code != 0 {
                                    let error_message = format!(
                                        "Game crashed during startup with code: {}",
                                        exit_code
                                    );
                                    send_progress(
                                        LaunchStage::Complete,
                                        100,
                                        100,
                                        error_message.clone(),
                                    );

                                    return Err(Box::new(io::Error::new(
                                        io::ErrorKind::Other,
                                        error_message,
                                    )));
                                } else {
                                    if i < 2 {
                                        let warning_message =
                                            "Minecraft process ended unexpectedly but cleanly";
                                        send_progress(
                                            LaunchStage::Complete,
                                            100,
                                            100,
                                            warning_message.to_string(),
                                        );

                                        return Err(Box::new(io::Error::new(
                                            io::ErrorKind::Other,
                                            warning_message,
                                        )));
                                    }
                                }
                                break;
                            }
                            Ok(None) => {
                                send_progress(
                                    LaunchStage::LaunchingGame,
                                    98 + i,
                                    100,
                                    format!("Minecraft is loading... ({}s)", i + 1).to_string(),
                                );
                            }
                            Err(e) => {
                                let error_message =
                                    format!("Failed to check game process status: {}", e);
                                send_progress(
                                    LaunchStage::Complete,
                                    100,
                                    100,
                                    error_message.clone(),
                                );

                                return Err(Box::new(io::Error::new(
                                    io::ErrorKind::Other,
                                    error_message,
                                )));
                            }
                        }
                    }

                    send_progress(
                        LaunchStage::Complete,
                        100,
                        100,
                        "Game launched successfully!".to_string(),
                    );

                    Ok(())
                }
                Err(e) => {
                    let error_message = format!("Failed to check game process status: {}", e);
                    send_progress(LaunchStage::Complete, 100, 100, error_message.clone());

                    Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        error_message,
                    )))
                }
            }
        }
        Err(e) => {
            let error_message = if e.to_string().contains("No such file or directory") {
                "Error: Java not found. Please install Java and try again."
            } else if e.to_string().contains("permission denied") {
                "Error: Permission denied. Try running as administrator."
            } else if e.to_string().contains("Too many open files") {
                "Error: System resource limit reached. Try closing other applications."
            } else if e.to_string().contains("Cannot allocate memory") {
                "Error: Not enough memory. Try closing other applications or allocate more memory."
            } else {
                &format!("Failed to start game: {}", e)
            };

            send_progress(LaunchStage::Complete, 100, 100, error_message.to_string());

            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                error_message,
            )))
        }
    }
}