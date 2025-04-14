//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use fltk::image::PngImage;
use fltk::{app, prelude::*, window::Window};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
mod gui;
use gui::*;
mod get_versions;
use get_versions::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::adjust_window;
fn main() {
    let version_ids = get_version_ids();

    let app = app::App::default();
    
    // Загружаем шрифт для всего приложения
    let font = setup_font(app, "ms-sans-serif-1.ttf");

    let mut win = Window::new(100, 100, WIN_WIDTH, WIN_HEIGHT, "Minecraft Launcher");
    win.set_border(false);

    setup_tiled_background();
    setup_frame(WIN_WIDTH, WIN_HEIGHT);
    setup_title_bar(
        "Minecraft Launcher",
        &mut PngImage::from_data(include_bytes!("minecraft_icon.png")).unwrap(),
        font,
        &win,
    );
    let profiles_path = PathBuf::from("game").join("profiles.json");
    
    if let Err(e) = fs::create_dir_all("game") {
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
    
    // Get all Java installations for the dropdown
    let java_installations = find_all_java_installations();
    
let (
    version_choice,
    mut profile_choice,
    mut play_button,
    mut new_profile_button,
    mut edit_profile_button,
    _welcome_frame,
    java_choice
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
                show_error_dialog(&format!("Failed to save profiles: {}", e), font);
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
        if let Some(selected) = profile_choice.choice() {
            if let Some(index) = profiles
                .lock()
                .unwrap()
                .iter()
                .position(|p| p.username == selected)
            {
                let profile_clone = profiles.lock().unwrap()[index].clone();
                if let Some(updated_profile) = edit_profile_dialog(font, &profile_clone) {
                    profiles.lock().unwrap()[index] = updated_profile.clone();
                    if let Err(e) = write_profiles(&profiles_path, &profiles.lock().unwrap()) {
                        show_error_dialog(&format!("Failed to save profiles: {}", e), font);
                    }
                    profile_choice.clear();
                    for profile in profiles.lock().unwrap().iter() {
                        profile_choice.add_choice(&profile.username);
                    }
                    profile_choice.set_value(index as i32);
                }
            }
        }
    });
}

handle_drag(&mut win);

win.end();
win.show();

#[cfg(target_os = "windows")]
adjust_window(&mut win);

    {
        let _profiles = profiles.clone();
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

            let mut jar_path = PathBuf::from("game");
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
            
            // Get selected Java path from dropdown
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

            std::thread::spawn(move || {
                if let Err(e) = launch_minecraft(&version_id, &username, &version_data, selected_java_path) {
                    eprintln!("Failed to launch Minecraft: {}", e);
                }
            });
        });
    }
    if let Err(e) = app.run() {
        show_error_dialog(&format!("Failed to launch app: {}", e), font);
    }
}
