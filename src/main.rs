#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use fltk::{app, prelude::*};
use std::sync::Arc;

mod app_init;
mod downloader;
mod gui;
mod java_finder;
mod launcher;
mod launcher_ui;
mod models;
mod profiles;
mod version_manager;
mod window_manager;

use app_init::{get_game_directory, setup_error_handler};
use java_finder::find_all_java_installations;
use launcher_ui::{initialize_profiles, setup_play_button_callback, setup_profile_callbacks};
use version_manager::get_version_ids;
use window_manager::{finalize_window, setup_window};

#[cfg(target_os = "windows")]
mod windows;

fn main() {
    let version_ids = get_version_ids();

    let (error_message, _font_for_error) = setup_error_handler();

    let app = app::App::default();
    let font = gui::setup_font(app, "");

    let win = setup_window(font);

    let game_dir = get_game_directory();
    let profiles_path = game_dir.join("profiles.json");

    let (profiles, profile_names) = initialize_profiles(&profiles_path, font);
    let profiles_path = Arc::new(profiles_path);

    let java_installations = find_all_java_installations();

    let (
        version_choice,
        mut profile_choice,
        play_button,
        new_profile_button,
        edit_profile_button,
        _welcome_frame,
        java_choice,
        status_label,
        progress_bar,
    ) = gui::setup_main_controls(font, version_ids, &profile_names);

    if !profiles.lock().unwrap().is_empty() {
        profile_choice.set_value(0);
    }

    setup_profile_callbacks(
        profiles.clone(),
        profiles_path.clone(),
        profile_choice.clone(),
        new_profile_button,
        edit_profile_button,
        font,
    );

    setup_play_button_callback(
        play_button,
        profiles.clone(),
        version_choice,
        profile_choice,
        java_choice,
        status_label,
        progress_bar,
        java_installations,
        error_message.clone(),
        font,
    );

    finalize_window(win);

    if let Err(e) = app.run() {
        gui::show_error_dialog(&format!("Failed to launch app: {}", e), font);
    }
}
