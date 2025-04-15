use dirs;
use fltk::app;
use fltk::enums::Font;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::gui::show_error_dialog;
use crate::gui::setup_font;

pub fn get_game_directory() -> PathBuf {
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

pub fn setup_error_handler() -> (Arc<Mutex<Option<String>>>, Font) {
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
    
    (error_message, font_for_error)
} 