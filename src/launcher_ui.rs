use crate::{
    app_init::get_game_directory,
    downloader::download_file,
    gui::*,
    java_finder::find_compatible_java,
    launcher::launch_minecraft,
    models::{self, Profile},
    profiles::{read_profiles, write_profiles},
    version_manager::{fetch_version_data, get_version_link},
};
use fltk::{app, button::Button, frame::Frame, menu::Choice, prelude::*};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub fn initialize_profiles(
    profiles_path: &PathBuf,
    font: fltk::enums::Font,
) -> (Arc<Mutex<Vec<Profile>>>, Vec<String>) {
    let profiles = Arc::new(Mutex::new(match read_profiles(&profiles_path) {
        Ok(profiles) => profiles,
        Err(e) => {
            show_error_dialog(&format!("Failed to read profiles: {}", e), font);
            Vec::new()
        }
    }));

    let profile_names: Vec<String> = profiles
        .lock()
        .unwrap()
        .iter()
        .map(|p| p.username.clone())
        .collect();

    (profiles, profile_names)
}

pub fn setup_profile_callbacks(
    profiles: Arc<Mutex<Vec<Profile>>>,
    profiles_path: Arc<PathBuf>,
    profile_choice: Choice,
    mut new_profile_button: Button,
    mut edit_profile_button: Button,
    font: fltk::enums::Font,
) {
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
}

pub fn setup_play_button_callback(
    mut play_button: Button,
    profiles: Arc<Mutex<Vec<Profile>>>,
    version_choice: Choice,
    profile_choice: Choice,
    java_choice: Choice,
    status_label: Frame,
    progress_bar: Frame,
    java_installations: Vec<(std::path::PathBuf, String)>,
    error_message: Arc<Mutex<Option<String>>>,
    font: fltk::enums::Font,
) {
    let profiles_clone = profiles.clone();

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

        let mut status_label_clone = status_label.clone();
        let java_path_to_use = match selected_java_path {
            Some(path) => Some(path),
            None => {
                if let Some(required_version) = version_data.get_required_java_version() {
                    match find_compatible_java(required_version, false) {
                        Some(java_path) => {
                            status_label_clone.set_label(&format!(
                                "Found compatible Java version {}",
                                required_version
                            ));
                            Some(java_path)
                        }
                        None => {
                            status_label_clone.set_label(&format!(
                                "Warning: Required Java {} not found",
                                required_version
                            ));
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };

        let mut progress_bar_clone = progress_bar.clone();

        status_label_clone.set_label("Preparing to launch Minecraft...");
        progress_bar_clone.show();
        progress_bar_clone.set_size(0, progress_bar_clone.h());
        app::redraw();
        app::flush();

        let (sender, receiver) = std::sync::mpsc::channel::<models::LaunchProgress>();

        setup_progress_monitoring(
            receiver,
            progress_bar.clone(),
            status_label.clone(),
            error_message.clone(),
        );

        launch_minecraft_process(
            version_id,
            username,
            version_data,
            java_path_to_use,
            profiles_clone.clone(),
            sender,
            error_message.clone(),
        );
    });
}

fn setup_progress_monitoring(
    receiver: std::sync::mpsc::Receiver<models::LaunchProgress>,
    progress_bar: Frame,
    status_label: Frame,
    error_msg: Arc<Mutex<Option<String>>>,
) {
    app::add_timeout3(0.05, {
        let mut progress_bar_clone = progress_bar.clone();
        let mut status_label_clone = status_label.clone();
        let error_msg_clone = error_msg.clone();

        move |handle| {
            match receiver.try_recv() {
                Ok(progress) => {
                    let percentage = progress.percentage();

                    update_win98_progress_bar(&mut progress_bar_clone, percentage);

                    let status_text = progress.message.clone();

                    if progress.stage == models::LaunchStage::Complete {
                        let error_keywords =
                            ["error", "failed", "crashed", "exited", "unexpectedly"];

                        if error_keywords
                            .iter()
                            .any(|&keyword| status_text.to_lowercase().contains(keyword))
                        {
                            let mut error = error_msg_clone.lock().unwrap();
                            *error = Some(status_text.clone());

                            app::awake();
                        }

                        progress_bar_clone.hide();
                    }

                    status_label_clone.set_label(&status_text);
                    app::redraw();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    progress_bar_clone.hide();
                    app::redraw();
                    return;
                }
            };

            app::repeat_timeout3(0.01, handle);
        }
    });
}

fn launch_minecraft_process(
    version_id: String,
    username: String,
    version_data: crate::models::VersionData,
    java_path: Option<std::path::PathBuf>,
    profiles: Arc<Mutex<Vec<Profile>>>,
    sender: std::sync::mpsc::Sender<models::LaunchProgress>,
    error_msg: Arc<Mutex<Option<String>>>,
) {
    let error_msg_clone = error_msg.clone();
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
            java_path,
            jvm_args,
            Some(sender),
        ) {
            let error_text = format!("{}", e);

            let mut error = error_msg_clone.lock().unwrap();
            *error = Some(error_text);

            app::awake();
        }
    });
}