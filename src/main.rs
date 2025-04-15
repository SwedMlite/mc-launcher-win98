use dirs;
use fltk::image::PngImage;
use fltk::{app, prelude::*, window::Window};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
mod gui;
use gui::*;
mod downloader;
mod java_finder;
mod launcher;
mod models;
mod profiles;
mod version_manager;

use downloader::download_file;
use java_finder::find_all_java_installations;
use launcher::launch_minecraft;
use profiles::{read_profiles, write_profiles};
use version_manager::{fetch_version_data, get_version_ids, get_version_link};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::{adjust_window, set_window_icon};

fn get_game_directory() -> PathBuf {
    let path = if cfg!(target_os = "windows") {
        match env::var("APPDATA") {
            Ok(appdata) => PathBuf::from(appdata).join("MinecraftLauncher"),
            Err(_) => PathBuf::from("game"),
        }
    } else if cfg!(target_os = "macos") {
        match dirs::home_dir() {
            Some(home) => home.join("Library/Application Support/MinecraftLauncher"),
            None => PathBuf::from("game"),
        }
    } else {
        match dirs::home_dir() {
            Some(home) => home.join(".minecraft_launcher"),
            None => PathBuf::from("game"),
        }
    };

    if let Err(e) = std::fs::create_dir_all(&path) {
        eprintln!("Warning: Failed to create game directory: {}", e);
    }

    path
}

fn main() {
    let version_ids = get_version_ids();

    let error_message = Arc::new(Mutex::new(None::<String>));

    let error_for_awake = error_message.clone();

    let font_for_error = setup_font(app::App::default(), "");
    app::add_idle3(move |_| {
        let mut error = error_for_awake.lock().unwrap();
        if let Some(msg) = error.take() {
            let font_clone = font_for_error;
            app::add_timeout3(0.1, move |_| {
                show_error_dialog(&msg, font_clone);
            });
        }
    });

    let app = app::App::default();

    let font = setup_font(app, "");

    let mut win = Window::new(100, 100, WIN_WIDTH, WIN_HEIGHT, "Minecraft Launcher");
    win.set_border(false);

    setup_tiled_background();
    setup_frame(WIN_WIDTH, WIN_HEIGHT);
    setup_title_bar(
        "Minecraft Launcher",
        &mut PngImage::from_data(include_bytes!("../themes/minecraft_icon.png")).unwrap(),
        font,
        &win,
    );
    let game_dir = get_game_directory();
    let profiles_path = game_dir.join("profiles.json");

    if let Err(e) = fs::create_dir_all(&game_dir) {
        show_error_dialog(&format!("Failed to create game directory: {}", e), font);
    }

    let profiles = Arc::new(Mutex::new(match read_profiles(&profiles_path) {
        Ok(profiles) => profiles,
        Err(e) => {
            show_error_dialog(&format!("Failed to read profiles: {}", e), font);
            Vec::new()
        }
    }));
    let profiles_path = Arc::new(profiles_path);
    let profile_names: Vec<String> = profiles
        .lock()
        .unwrap()
        .iter()
        .map(|p| p.username.clone())
        .collect();

    let java_installations = find_all_java_installations();

    let (
        version_choice,
        mut profile_choice,
        mut play_button,
        mut new_profile_button,
        mut edit_profile_button,
        _welcome_frame,
        java_choice,
    ) = setup_main_controls(font, version_ids, &profile_names);

    if !profiles.lock().unwrap().is_empty() {
        profile_choice.set_value(0);
    }

    {
        let profiles = profiles.clone();
        let profiles_path = profiles_path.clone();
        let mut profile_choice = profile_choice.clone();
        new_profile_button.set_callback(move |_| {
            if let Some(new_profile) = create_new_profile_dialog(font) {
                profiles.lock().unwrap().push(new_profile.clone());

                if let Err(e) = write_profiles(&profiles_path, &profiles.lock().unwrap()) {
                    let error_msg = format!("Failed to save profiles: {}", e);
                    show_error_dialog(&error_msg, font);
                }

                profile_choice.add_choice(&new_profile.username);
                profile_choice.set_value(profile_choice.size() - 1);
            }
        });
    }

    {
        let profiles = profiles.clone();
        let profiles_path = profiles_path.clone();
        let mut profile_choice = profile_choice.clone();
        edit_profile_button.set_callback(move |_| {
            let selected_idx = profile_choice.value() as usize;

            if selected_idx >= profiles.lock().unwrap().len() {
                show_error_dialog("No profile selected", font);
                return;
            }

            let selected_profile = profiles.lock().unwrap()[selected_idx].clone();

            if let Some(edited_profile) = edit_profile_dialog(font, &selected_profile) {
                let old_username = profiles.lock().unwrap()[selected_idx].username.clone();
                profiles.lock().unwrap()[selected_idx] = edited_profile.clone();

                if old_username != edited_profile.username {
                    profile_choice.clear();

                    for p in profiles.lock().unwrap().iter() {
                        profile_choice.add_choice(&p.username);
                    }

                    profile_choice.set_value(selected_idx as i32);
                }

                if let Err(e) = write_profiles(&profiles_path, &profiles.lock().unwrap()) {
                    let error_msg = format!("Failed to save profiles: {}", e);
                    show_error_dialog(&error_msg, font);
                }
            }
        });
    }

    handle_drag(&mut win);

    win.end();

    #[cfg(target_os = "windows")]
    {
        let icon_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("themes")
            .join("minecraft_icon.png");

        if icon_path.exists() {
            set_window_icon(
                &mut win,
                icon_path.to_str().unwrap_or("themes/minecraft_icon.png"),
            );
        } else {
            set_window_icon(&mut win, "themes/minecraft_icon.png");
        }
    }

    win.show();

    #[cfg(target_os = "windows")]
    adjust_window(&win);

    {
        let profiles = profiles.clone();
        let java_installations = java_installations.clone();

        play_button.set_callback(move |_| {
            let username = match profile_choice.choice() {
                Some(selected) => selected,
                None => {
                    show_error_dialog("Please select a profile!", font);
                    return;
                }
            };

            let version_id = match version_choice.choice() {
                Some(id) => id,
                None => {
                    show_error_dialog("Please, choose version Minecraft!", font);
                    return;
                }
            };

            let version_json_url = match get_version_link(version_id.clone()) {
                Some(url) => url,
                None => {
                    show_error_dialog(
                        &format!("Failed to get URL for version {}", version_id),
                        font,
                    );
                    return;
                }
            };
            let version_data = match fetch_version_data(&version_json_url) {
                Ok(data) => data,
                Err(e) => {
                    show_error_dialog(&format!("Failed to get data version: {}", e), font);
                    return;
                }
            };

            let mut jar_path = get_game_directory();
            jar_path.push("versions");
            jar_path.push(&version_id);

            if let Err(e) = fs::create_dir_all(&jar_path) {
                show_error_dialog(
                    &format!("Failed to create directory {}: {}", jar_path.display(), e),
                    font,
                );
                return;
            }

            jar_path.push("client.jar");
            if !jar_path.exists() {
                if let Err(e) = download_file(&version_data.downloads.client.url, &jar_path) {
                    show_error_dialog(&format!("Failed to loading client.jar: {}", e), font);
                    return;
                }
            }

            let selected_java_path = if !java_installations.is_empty() {
                let selected_index = java_choice.value() as usize;
                if selected_index < java_installations.len() {
                    Some(java_installations[selected_index].0.clone())
                } else {
                    None
                }
            } else {
                None
            };

            let error_msg_clone = error_message.clone();
            let profiles_clone = profiles.clone();
            std::thread::spawn(move || {
                let jvm_args = profiles_clone
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|p| p.username == username)
                    .and_then(|profile| profile.jvm_args.clone())
                    .map(|args_str| {
                        args_str
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                    });

                if let Err(e) = launch_minecraft(
                    &version_id,
                    &username,
                    &version_data,
                    selected_java_path,
                    jvm_args,
                ) {
                    let error_text = format!("{}", e);

                    let mut error = error_msg_clone.lock().unwrap();
                    *error = Some(error_text);

                    app::awake();
                }
            });
        });
    }
    if let Err(e) = app.run() {
        show_error_dialog(&format!("Failed to launch app: {}", e), font);
    }
}
