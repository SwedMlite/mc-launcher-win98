use crate::downloader::{
    download_and_extract_assets, download_and_extract_natives, download_libraries,
};
use crate::java_finder::find_compatible_java;
use crate::models::VersionData;
use crate::get_game_directory;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::Command;

pub fn launch_minecraft(
    version_id: &String,
    username: &String,
    version_data: &VersionData,
    custom_java_path: Option<PathBuf>,
    jvm_args: Option<Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    let game_dir = get_game_directory();
    let minecraft_dir = game_dir.join(".minecraft");
    fs::create_dir_all(&minecraft_dir)?;

    let jar_path = game_dir
        .join("versions")
        .join(version_id)
        .join("client.jar");
    let libraries_dir = game_dir.join("libraries");
    fs::create_dir_all(&libraries_dir)?;

    let library_paths = download_libraries(version_data, &libraries_dir)?;

    let mut classpath = String::new();
    classpath.push_str(&jar_path.to_string_lossy());
    for path in &library_paths {
        #[cfg(target_os = "windows")]
        classpath.push(';');
        #[cfg(not(target_os = "windows"))]
        classpath.push_str(":");
        classpath.push_str(&path.to_string_lossy());
    }

    let natives_dir = game_dir.join("natives").join(version_id);
    fs::create_dir_all(&natives_dir)?;
    download_and_extract_natives(version_data, &natives_dir)?;

    download_and_extract_assets(version_data, &game_dir)?;

    let is_alpha_or_beta = version_id.starts_with("a") || version_id.starts_with("b");

    let (required_java_version, strict_match) = match &version_data.java_version {
        Some(java_version) => (java_version.major_version, false),
        None => {
            if is_alpha_or_beta
                || version_id.starts_with("1.8")
                || version_id.starts_with("1.7")
                || version_id.starts_with("1.6")
                || version_id.starts_with("1.5")
                || version_id.starts_with("1.4")
                || version_id.starts_with("1.3")
                || version_id.starts_with("1.2")
                || version_id.starts_with("1.1")
                || version_id.starts_with("1.0")
            {
                (8, true)
            } else {
                (17, false)
            }
        }
    };

    let java_path = match custom_java_path {
        Some(path) => path,
        None => match find_compatible_java(required_java_version, strict_match) {
            Some(path) => path,
            None => PathBuf::from(if cfg!(windows) { "java.exe" } else { "java" }),
        },
    };

    let asset_index_id = &version_data.asset_index.id;
    let (assets_dir_arg, assets_dir_path) =
        if asset_index_id == "legacy" || asset_index_id == "pre-1.6" {
            ("--assetsDir", game_dir.join("assets").join("legacy"))
        } else {
            ("--assetsDir", game_dir.join("assets"))
        };

    let mut launch_cmd = Command::new(&java_path);

    launch_cmd
        .arg("-Djava.library.path=".to_string() + &natives_dir.to_string_lossy())
        .arg("-Dminecraft.launcher.brand=CustomLauncher")
        .arg("-Dminecraft.launcher.version=1.0");

    if let Some(args) = &jvm_args {
        for arg in args {
            launch_cmd.arg(arg);
        }
    } else {
        launch_cmd.arg("-Xmx2G");
        launch_cmd.arg("-Xms512M");
    }

    launch_cmd.arg("-cp").arg(&classpath);

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
            launch_cmd.arg("--assetIndex").arg(asset_index_id);
        }

        launch_cmd
            .arg("--accessToken")
            .arg("0")
            .arg("--uuid")
            .arg("00000000-0000-0000-0000-000000000000")
            .arg("--userProperties")
            .arg("{}");
    }

    launch_cmd.stdout(std::process::Stdio::piped());
    launch_cmd.stderr(std::process::Stdio::piped());

    let mut child = launch_cmd.spawn()?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to capture stderr"))?;

    let mut stderr_reader = std::io::BufReader::new(stderr);
    let mut error_output = String::new();

    let mut line = String::new();
    for _ in 0..10 {
        line.clear();
        if stderr_reader.read_line(&mut line)? == 0 {
            break;
        }
        error_output.push_str(&line);
    }

    if !error_output.is_empty() {
        if error_output.contains("java.lang.UnsupportedClassVersionError") {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Incompatible Java version. Minecraft requires a newer Java version.\n{}",
                    error_output
                ),
            )));
        }

        if error_output.contains("Could not create the Java Virtual Machine")
            || error_output.contains("Error occurred during initialization of VM")
        {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Could not create Java Virtual Machine. You may need to adjust memory settings or install Java.\n{}",
                    error_output
                ),
            )));
        }

        if error_output.contains("java.lang.OutOfMemoryError") {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Java ran out of memory. Try allocating more memory to Java or close other applications.\n{}",
                    error_output
                ),
            )));
        }

        if error_output.contains("natives")
            && (error_output.contains("failed to load") || error_output.contains("no such file"))
        {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to load native libraries. Try redownloading the game.\n{}",
                    error_output
                ),
            )));
        }

        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Java error: {}", error_output),
        )));
    }

    let status = child.wait()?;

    if !status.success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Process exited with code: {}", status),
        )));
    }
    Ok(())
}
