use fltk::{
    app,
    button::{Button, CheckButton},
    enums::{Align, Color, Event, Font, FrameType},
    frame::Frame,
    image::PngImage,
    input::Input,
    menu::Choice,
    prelude::*,
    window::Window,
};

macro_rules! load_image_from_data {
    ($path:expr) => {
        PngImage::from_data(include_bytes!($path))
    };
}

use crate::java_finder::find_all_java_installations;
use crate::models::Profile;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "windows")]
use crate::windows::adjust_window;

pub const WIN_WIDTH: i32 = 600;
pub const WIN_HEIGHT: i32 = 275;

static DIALOG_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn create_new_profile_dialog(text_font: Font) -> Option<Profile> {
    if DIALOG_RUNNING.swap(true, Ordering::SeqCst) {
        return None;
    }

    let mut win = Window::default()
        .with_size(300, 200)
        .with_label("New Profile");
    win.set_border(false);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    win.set_pos((screen_width - 300) / 2, (screen_height - 200) / 2);

    let mut bg = Frame::new(0, 0, 300, 200, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(Color::from_rgb(192, 192, 192));

    let mut username_label = Frame::new(20, 40, 100, 25, "Username:");
    username_label.set_label_font(text_font);
    username_label.set_label_size(12);
    username_label.set_align(Align::Left | Align::Inside);

    let mut username_input = Input::new(20, 65, 260, 25, "");
    username_input.set_text_font(text_font);
    username_input.set_text_size(12);

    let mut jvm_args_label = Frame::new(20, 95, 160, 25, "JVM Arguments (optional):");
    jvm_args_label.set_label_font(text_font);
    jvm_args_label.set_label_size(12);
    jvm_args_label.set_align(Align::Left | Align::Inside);

    let mut jvm_hint_label = Frame::new(165, 95, 135, 25, "Example: -Xmx2G -Xms512M");
    jvm_hint_label.set_label_font(text_font);
    jvm_hint_label.set_label_size(10);
    jvm_hint_label.set_label_color(Color::from_rgb(80, 80, 80));
    jvm_hint_label.set_align(Align::Left | Align::Inside);

    let mut jvm_args_input = Input::new(20, 120, 260, 25, "");
    jvm_args_input.set_text_font(text_font);
    jvm_args_input.set_text_size(12);

    let mut create_button = Button::new(100, 160, 100, 25, "Create");
    create_button.set_label_font(text_font);
    create_button.set_label_size(12);
    create_button.set_frame(FrameType::UpBox);
    create_button.set_color(Color::from_rgb(192, 192, 192));

    setup_frame(win.width(), win.height());
    setup_title_bar(
        "New Profile",
        &mut load_image_from_data!("../themes/minecraft_icon.png").unwrap(),
        text_font,
        &win,
    );

    win.end();

    let result = Arc::new(Mutex::new(None::<Profile>));
    let result_clone = result.clone();

    let username_clone = username_input.clone();
    let jvm_args_clone = jvm_args_input.clone();
    let mut win_clone = win.clone();

    create_button.set_callback(move |_| {
        let username = username_clone.value().trim().to_string();
        let jvm_args = jvm_args_clone.value().trim().to_string();

        if !username.is_empty() {
            let jvm_args = if jvm_args.is_empty() {
                None
            } else {
                Some(jvm_args)
            };
            *result_clone.lock().unwrap() = Some(Profile { username, jvm_args });
        }
        win_clone.hide();
        DIALOG_RUNNING.store(false, Ordering::SeqCst);
    });

    win.set_callback(move |w| {
        w.hide();
        DIALOG_RUNNING.store(false, Ordering::SeqCst);
    });

    win.show();
    handle_drag(&mut win);

    #[cfg(target_os = "windows")]
    adjust_window(&win);

    while win.shown() {
        app::wait();
    }

    DIALOG_RUNNING.store(false, Ordering::SeqCst);

    result.lock().unwrap().clone()
}

pub fn edit_profile_dialog(text_font: Font, profile: &Profile) -> Option<Profile> {
    if DIALOG_RUNNING.swap(true, Ordering::SeqCst) {
        return None;
    }

    let mut win = Window::default()
        .with_size(300, 200)
        .with_label("Edit Profile");
    win.set_border(false);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    win.set_pos((screen_width - 300) / 2, (screen_height - 200) / 2);

    let mut bg = Frame::new(0, 0, 300, 200, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(Color::from_rgb(192, 192, 192));

    let mut username_label = Frame::new(20, 40, 100, 25, "Username:");
    username_label.set_label_font(text_font);
    username_label.set_label_size(12);
    username_label.set_align(Align::Left | Align::Inside);

    let mut username_input = Input::new(20, 65, 260, 25, "");
    username_input.set_text_font(text_font);
    username_input.set_text_size(12);
    username_input.set_value(&profile.username);

    let mut jvm_args_label = Frame::new(20, 95, 160, 25, "JVM Arguments (optional):");
    jvm_args_label.set_label_font(text_font);
    jvm_args_label.set_label_size(12);
    jvm_args_label.set_align(Align::Left | Align::Inside);

    let mut jvm_hint_label = Frame::new(165, 95, 135, 25, "Example: -Xmx2G -Xms512M");
    jvm_hint_label.set_label_font(text_font);
    jvm_hint_label.set_label_size(10);
    jvm_hint_label.set_label_color(Color::from_rgb(80, 80, 80));
    jvm_hint_label.set_align(Align::Left | Align::Inside);

    let mut jvm_args_input = Input::new(20, 120, 260, 25, "");
    jvm_args_input.set_text_font(text_font);
    jvm_args_input.set_text_size(12);
    if let Some(args) = &profile.jvm_args {
        jvm_args_input.set_value(args);
    }

    let mut save_button = Button::new(100, 160, 100, 25, "Save");
    save_button.set_label_font(text_font);
    save_button.set_label_size(12);
    save_button.set_frame(FrameType::UpBox);
    save_button.set_color(Color::from_rgb(192, 192, 192));

    setup_frame(win.width(), win.height());
    setup_title_bar(
        "Edit Profile",
        &mut load_image_from_data!("../themes/minecraft_icon.png").unwrap(),
        text_font,
        &win,
    );

    win.end();

    let result = Arc::new(Mutex::new(None::<Profile>));
    let result_clone = result.clone();

    let username_clone = username_input.clone();
    let jvm_args_clone = jvm_args_input.clone();
    let mut win_clone = win.clone();

    save_button.set_callback(move |_| {
        let username = username_clone.value().trim().to_string();
        let jvm_args = jvm_args_clone.value().trim().to_string();

        if !username.is_empty() {
            let jvm_args = if jvm_args.is_empty() {
                None
            } else {
                Some(jvm_args)
            };
            *result_clone.lock().unwrap() = Some(Profile { username, jvm_args });
        }
        win_clone.hide();
        DIALOG_RUNNING.store(false, Ordering::SeqCst);
    });

    win.set_callback(move |w| {
        w.hide();
        DIALOG_RUNNING.store(false, Ordering::SeqCst);
    });

    win.show();
    handle_drag(&mut win);

    #[cfg(target_os = "windows")]
    adjust_window(&win);

    while win.shown() {
        app::wait();
    }

    DIALOG_RUNNING.store(false, Ordering::SeqCst);

    result.lock().unwrap().clone()
}

pub fn setup_font(app: app::App, _font_path: &str) -> Font {
    let font_data = include_bytes!("../themes/windows98/ms-sans-serif-1.ttf");

    #[cfg(target_os = "windows")]
    {
        let temp_dir = std::env::temp_dir();
        let random_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let font_file_path = temp_dir.join(format!("mcl_font_{}.ttf", random_suffix));

        if let Err(_) = std::fs::write(&font_file_path, font_data) {
            return Font::Helvetica;
        }

        let path_str = font_file_path.to_str().unwrap_or("");

        use std::ffi::CString;
        use winapi::um::wingdi::AddFontResourceExA;
        use winapi::um::wingdi::{FR_NOT_ENUM, FR_PRIVATE};

        if let Ok(path_cstr) = CString::new(path_str) {
            unsafe {
                AddFontResourceExA(
                    path_cstr.as_ptr(),
                    FR_PRIVATE | FR_NOT_ENUM,
                    std::ptr::null_mut(),
                );
            }
        }

        if let Ok(f) = app::App::load_font(app, path_str) {
            let path_clone = font_file_path.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let _ = std::fs::remove_file(&path_clone);
            });

            return Font::by_name(&f);
        }
    }

    if let Ok(f) = app::App::load_font(app, "themes/windows98/ms-sans-serif-1.ttf") {
        return Font::by_name(&f);
    }

    for font_name in &["MS Sans Serif", "Microsoft Sans Serif", "Arial", "Tahoma"] {
        let font = Font::by_name(font_name);
        if font != Font::Helvetica {
            return font;
        }
    }

    Font::Helvetica
}

pub fn show_error_dialog(message: &str, text_font: Font) {
    let display_message = if message.contains("JVM allocation heap")
        || message.contains("heap space")
    {
        "Not enough memory to start Minecraft. Try closing other programs or increasing the memory allocation in the profile settings."
    } else if message.contains("java.lang.ClassNotFoundException") {
        "Could not find the required Java class. The Java installation may be corrupted or incompatible."
    } else if message.contains("java.io.IOException") {
        "I/O error. Check if you have write permissions to the game directory and if there is enough space on the disk."
    } else if message.contains("java.lang.OutOfMemoryError") {
        "Not enough memory to start Minecraft. Try closing other programs or increasing the memory allocation."
    } else if message.contains("exited immediately") || message.contains("crashed during startup") {
        "Minecraft exited with an error when starting. This may be due to Java version incompatibility, insufficient system resources, or game file corruption."
    } else if message.contains("no valid OpenGL") || message.contains("OpenGL Error") {
        "OpenGL error. Update your video card drivers or make sure your computer supports the required OpenGL version."
    } else {
        message
    };

    let mut text_height = 30;
    let max_width = 190;

    let chars_per_line = max_width / 7;
    let line_count = (display_message.len() as f32 / chars_per_line as f32).ceil() as i32;
    if line_count > 1 {
        text_height = line_count * 15;
    }

    let window_height = 125 + (text_height - 30).max(0);

    let mut dialog = Window::default()
        .with_size(300, window_height)
        .with_label("Error");
    dialog.set_border(false);
    dialog.make_modal(true);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    dialog.set_pos(
        (screen_width - 300) / 2,
        (screen_height - window_height) / 2,
    );

    let mut bg = Frame::new(0, 0, 300, window_height, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(Color::from_rgb(192, 192, 192));

    let mut error_icon = Frame::new(30, 45, 48, 48, "");
    error_icon.set_label_font(text_font);
    error_icon.set_label_size(24);
    error_icon.set_label_color(Color::from_rgb(255, 0, 0));
    let mut icon_img = load_image_from_data!("../themes/windows98/error_icon.png").unwrap();
    icon_img.scale(48, 48, false, true);
    error_icon.set_image(Some(icon_img));

    let mut text = Frame::new(90, 45, max_width, text_height, display_message);
    text.set_align(Align::Left | Align::Inside | Align::Wrap);
    text.set_label_font(text_font);
    text.set_label_size(12);

    let show_details = display_message != message;

    let buttons_y = window_height - 35;
    let mut ok_btn = Button::new(
        if show_details { 60 } else { 110 },
        buttons_y,
        100,
        25,
        "OK",
    );
    ok_btn.set_label_font(text_font);
    ok_btn.set_label_size(12);
    ok_btn.set_frame(FrameType::UpBox);
    ok_btn.set_color(Color::from_rgb(192, 192, 192));

    let mut details_btn = Button::new(170, buttons_y, 100, 25, "Details");
    if show_details {
        details_btn.set_label_font(text_font);
        details_btn.set_label_size(12);
        details_btn.set_frame(FrameType::UpBox);
        details_btn.set_color(Color::from_rgb(192, 192, 192));
    } else {
        details_btn.hide();
    }

    setup_frame(dialog.width(), dialog.height());
    setup_title_bar(
        "Error",
        &mut load_image_from_data!("../themes/windows98/error_icon.png").unwrap(),
        text_font,
        &dialog,
    );

    dialog.end();

    let mut dialog_clone = dialog.clone();
    ok_btn.set_callback(move |_| {
        dialog_clone.hide();
    });

    if show_details {
        let original_message = message.to_string();
        let mut text_clone = text.clone();
        let mut dialog_clone = dialog.clone();
        details_btn.set_callback(move |_| {
            text_clone.set_label(&original_message);
            dialog_clone.redraw();
        });
    }

    let mut dialog_clone = dialog.clone();
    dialog.set_callback(move |_| {
        dialog_clone.hide();
    });

    handle_drag(&mut dialog);

    dialog.show();

    #[cfg(target_os = "windows")]
    adjust_window(&dialog);
}

pub fn setup_frame(width: i32, height: i32) {
    let top = load_image_from_data!("../themes/windows98/top_frame.png").unwrap();
    let bottom = load_image_from_data!("../themes/windows98/bottom_frame.png").unwrap();
    let left = load_image_from_data!("../themes/windows98/left_frame.png").unwrap();
    let right = load_image_from_data!("../themes/windows98/right_frame.png").unwrap();
    let lt_corner = load_image_from_data!("../themes/windows98/left_top_corner.png").unwrap();
    let rt_corner = load_image_from_data!("../themes/windows98/right_top_corner.png").unwrap();
    let lb_corner = load_image_from_data!("../themes/windows98/left_bottom_corner.png").unwrap();
    let rb_corner = load_image_from_data!("../themes/windows98/right_bottom_corner.png").unwrap();

    let add_frame = |x, y, w, h, img: &PngImage, scale: bool| {
        let mut frame = Frame::new(x, y, w, h, "");

        if scale {
            let mut scaled = img.clone();
            scaled.scale(w, h, false, true);
            frame.set_image(Some(scaled));
        } else {
            frame.set_image(Some(img.clone()));
        }
        frame.set_frame(FrameType::NoBox);
        frame.set_pos(x, y);
    };

    let offset = 8;
    let bottom_adjust = 5;

    let tc_w = lt_corner.width();
    let tc_h = lt_corner.height();
    let bc_w = lb_corner.width();
    let bc_h = lb_corner.height();
    let v_w = left.width();
    let h_h = top.height();
    let b_h = bottom.height();

    let v_h = height - bc_h - tc_h - offset + bottom_adjust;

    add_frame(0, offset, tc_w, tc_h, &lt_corner, false);
    add_frame(
        width - rt_corner.width(),
        offset,
        rt_corner.width(),
        rt_corner.height(),
        &rt_corner,
        false,
    );
    add_frame(
        tc_w,
        offset,
        width - tc_w - rt_corner.width(),
        h_h,
        &top,
        true,
    );

    add_frame(
        0,
        height - bc_h + bottom_adjust,
        bc_w,
        bc_h,
        &lb_corner,
        false,
    );
    add_frame(
        width - rb_corner.width(),
        height - bc_h + bottom_adjust,
        rb_corner.width(),
        bc_h,
        &rb_corner,
        false,
    );
    add_frame(
        lb_corner.width(),
        height - b_h + bottom_adjust,
        width - lb_corner.width() - rb_corner.width(),
        b_h,
        &bottom,
        true,
    );

    add_frame(0, tc_h + offset, v_w, v_h, &left, true);
    add_frame(width - v_w, tc_h + offset, v_w, v_h, &right, true);
}

pub fn setup_tiled_background() {
    let mut bg_tile = load_image_from_data!("../themes/background.png").unwrap();
    bg_tile.scale(64, 64, false, true);
    let tile_w = bg_tile.width();
    let tile_h = bg_tile.height();

    let mut tiled_frame = Frame::new(0, 8, WIN_WIDTH, WIN_HEIGHT, "");
    tiled_frame.set_frame(FrameType::NoBox);

    tiled_frame.draw(move |f| {
        let fw = f.w();
        let fh = f.h();
        let fx = f.x();
        let fy = f.y();

        let mut y = 0;
        while y < fh {
            let mut x = 0;
            while x < fw {
                bg_tile.draw(fx + x, fy + y, tile_w, tile_h);
                x += tile_w;
            }
            y += tile_h;
        }
    });
}

pub fn setup_main_controls(
    text_font: Font,
    versions: String,
    profile_names: &Vec<String>,
) -> (
    Choice,
    Choice,
    Button,
    Button,
    Button,
    Frame,
    Choice,
    Frame,
    Frame,
) {
    let button_width = 110;
    let button_height = 30;

    let mut ver_label = Frame::new(20, 40, 100, 20, "Version:");
    ver_label.set_label_font(text_font);
    ver_label.set_label_size(12);
    ver_label.set_label_color(Color::White);
    ver_label.set_frame(FrameType::NoBox);
    ver_label.set_align(Align::Left | Align::Inside);

    let versions_data: Vec<(String, String)> = versions
        .split('|')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .chunks(2)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                Some((chunk[0].to_string(), chunk[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    let mut version_choice = Choice::new(20, 60, WIN_WIDTH - 40, 25, "");
    version_choice.set_color(Color::White);
    version_choice.set_text_font(text_font);
    version_choice.set_text_size(12);

    let type_labels = [
        ("Release", "release"),
        ("Snapshot", "snapshot"),
        ("Alpha", "old_alpha"),
        ("Beta", "old_beta"),
    ];

    let mut checkboxes = Vec::new();

    for (i, (label, _)) in type_labels.iter().enumerate() {
        let mut cb = CheckButton::new(20 + i as i32 * 85, 95, 80, 20, *label);
        cb.set_label_font(text_font);
        cb.set_label_size(12);
        cb.set_label_color(Color::White);
        cb.set_value(true);
        checkboxes.push(cb);
    }

    let mut java_choice = Choice::new(400, 95, 180, 20, "");
    java_choice.set_color(Color::White);
    java_choice.set_text_font(text_font);
    java_choice.set_text_size(12);

    let java_installations = find_all_java_installations();

    let mut short_display_names = Vec::new();

    if !java_installations.is_empty() {
        for (path, version) in &java_installations {
            let path_str = path.to_string_lossy().to_string();

            let formatted_path = if cfg!(windows) {
                path_str.replace('\\', "/")
            } else {
                path_str
            };

            let display_text = format!("Java {} - {}", version, formatted_path);
            java_choice.add_choice(&display_text);

            let path = std::path::Path::new(&formatted_path);
            let _filename = path.file_name().unwrap_or_default().to_string_lossy();

            let parent_dir = if let Some(parent) = path.parent() {
                let dir_name = parent.file_name().unwrap_or_default().to_string_lossy();
                if dir_name == "bin" {
                    if let Some(grandparent) = parent.parent() {
                        grandparent
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    } else {
                        dir_name.to_string()
                    }
                } else {
                    dir_name.to_string()
                }
            } else {
                "".to_string()
            };

            let short_name = format!("Java {} ({})", version, parent_dir);
            short_display_names.push(short_name);
        }
        java_choice.set_value(0);
    } else {
        java_choice.add_choice("No Java installations found");
        java_choice.set_value(0);
    }

    let mut profile_label = Frame::new(20, 125, 100, 20, "Profile:");
    profile_label.set_label_font(text_font);
    profile_label.set_label_size(12);
    profile_label.set_label_color(Color::White);
    profile_label.set_frame(FrameType::NoBox);
    profile_label.set_align(Align::Left | Align::Inside);

    let mut profile_choice = Choice::new(20, 145, WIN_WIDTH - 40, 25, "");
    profile_choice.set_color(Color::White);
    profile_choice.set_text_font(text_font);
    profile_choice.set_text_size(12);

    for name in profile_names {
        profile_choice.add_choice(name);
    }
    if !profile_names.is_empty() {
        profile_choice.set_value(0);
    }

    let buttons_y = 175;

    let mut new_profile = Button::new(20, buttons_y, 110, 25, "New Profile");
    new_profile.set_label_font(text_font);
    new_profile.set_label_size(12);

    let mut edit_profile = Button::new(140, buttons_y, 110, 25, "Edit Profile");
    edit_profile.set_label_font(text_font);
    edit_profile.set_label_size(12);
    edit_profile.deactivate();

    let folder_button_width = 30;
    let folder_button_height = 25;
    let play_x = 310;
    let play_y = buttons_y;

    let mut folder_button = Button::new(
        play_x - folder_button_width - 10,
        play_y,
        folder_button_width,
        folder_button_height,
        "",
    );

    folder_button.set_image(Some(
        load_image_from_data!("../themes/windows98/folder.png").unwrap(),
    ));
    folder_button.set_tooltip("Open launcher folder");
    folder_button.set_frame(FrameType::UpBox);
    folder_button.set_color(Color::from_rgb(192, 192, 192));
    folder_button.set_align(Align::Inside | Align::Top);
    folder_button.set_callback(move |_| {
        use crate::get_game_directory;
        let game_dir = get_game_directory();

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("explorer")
                .arg(game_dir.to_string_lossy().to_string())
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open")
                .arg(game_dir.to_string_lossy().to_string())
                .spawn();
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let _ = Command::new("xdg-open")
                .arg(game_dir.to_string_lossy().to_string())
                .spawn();
        }
    });

    let mut play = Button::new(play_x, play_y, button_width, button_height, "Play");
    play.set_label_font(text_font);
    play.set_label_size(12);

    let welcome_text = if !profile_names.is_empty() {
        format!("Welcome, {}", profile_names[0])
    } else {
        "Welcome, Guest".to_string()
    };

    let mut welcome_frame = Frame::new(
        play_x + button_width + 10,
        play_y + 5,
        220,
        20,
        welcome_text.as_str(),
    );
    welcome_frame.set_label_font(text_font);
    welcome_frame.set_label_size(12);
    welcome_frame.set_label_color(Color::White);
    welcome_frame.set_frame(FrameType::NoBox);
    welcome_frame.set_align(Align::Left | Align::Inside);

    let status_y = buttons_y + 35;

    let mut progress_label = Frame::new(20, status_y - 10, 100, 20, "Progress:");
    progress_label.set_label_font(text_font);
    progress_label.set_label_size(12);
    progress_label.set_label_color(Color::White);
    progress_label.set_frame(FrameType::NoBox);
    progress_label.set_align(Align::Left | Align::Inside);

    let mut status_label = Frame::new(20, status_y, WIN_WIDTH - 40, 20, "");
    status_label.set_label_font(text_font);
    status_label.set_label_size(12);
    status_label.set_label_color(Color::White);
    status_label.set_frame(FrameType::NoBox);
    status_label.set_align(Align::Left | Align::Inside);

    let progress_bar_y = status_y + 20;

    let mut progress_frame = Frame::new(20, progress_bar_y, WIN_WIDTH - 40, 20, "");
    progress_frame.set_frame(FrameType::DownBox);
    progress_frame.set_color(Color::White);

    let progress_bar = create_win98_progress_bar(20, progress_bar_y, WIN_WIDTH - 40, 20);

    let mut update_dropdown = {
        let versions_data = versions_data.clone();
        let mut choice = version_choice.clone();
        let checkboxes = checkboxes.clone();

        move || {
            let mut selected_types = Vec::new();
            for (i, (_, type_value)) in type_labels.iter().enumerate() {
                if checkboxes[i].value() {
                    selected_types.push(type_value.to_string());
                }
            }

            let filtered_versions = if selected_types.is_empty() {
                Vec::new()
            } else {
                versions_data
                    .iter()
                    .filter(|(_, version_type)| {
                        let mapped_type = match version_type.as_str() {
                            "alpha" => "old_alpha",
                            "beta" => "old_beta",
                            _ => version_type.as_str(),
                        };
                        selected_types.contains(&mapped_type.to_string())
                    })
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<String>>()
            };

            choice.clear();
            let filtered_versions_str = filtered_versions.join("|");
            if !filtered_versions_str.is_empty() {
                choice.add_choice(&filtered_versions_str);
                choice.set_value(0);
            }
        }
    };

    for (i, _) in type_labels.iter().enumerate() {
        let mut update = update_dropdown.clone();
        checkboxes[i].set_callback(move |_| {
            update();
        });
    }

    let mut welcome_frame_clone = welcome_frame.clone();
    let mut edit_profile_clone = edit_profile.clone();
    profile_choice.set_callback(move |c| {
        if let Some(selected) = c.choice() {
            welcome_frame_clone.set_label(&format!("Welcome, {}", selected));
            edit_profile_clone.activate();
        } else {
            welcome_frame_clone.set_label("Welcome, Guest");
            edit_profile_clone.deactivate();
        }
    });

    if !profile_names.is_empty() {
        edit_profile.activate();
    }

    update_dropdown();

    (
        version_choice,
        profile_choice,
        play,
        new_profile,
        edit_profile,
        welcome_frame,
        java_choice,
        status_label,
        progress_bar,
    )
}

pub fn setup_title_bar(title: &str, ico: &mut PngImage, font: Font, win: &Window) {
    let win_width = win.width();

    let mut title_frame = Frame::new(24, 4, 300, 20, title);
    title_frame.set_label_color(Color::White);
    title_frame.set_label_font(font);
    title_frame.set_label_size(12);
    title_frame.set_align(Align::Left | Align::Inside);
    title_frame.set_frame(FrameType::NoBox);

    let mut icon = Frame::new(8, 12, 16, 16, "");
    ico.scale(16, 16, false, false);
    icon.set_image(Some(ico.clone()));

    let is_main_window = win.label() == "Minecraft Launcher";

    let mut close_btn = Button::new(win_width - 22, 6, 16, 14, "");
    close_btn.set_image(Some(
        load_image_from_data!("../themes/windows98/close.png").unwrap(),
    ));
    close_btn.set_align(Align::Inside | Align::Top);
    close_btn.set_frame(FrameType::FlatBox);

    let mut win_clone = win.clone();
    close_btn.set_callback(move |_| {
        if is_main_window {
            app::quit();
        } else {
            win_clone.hide();
        }
    });

    let mut max_btn = Button::new(win_width - 39, 6, 16, 14, "");
    max_btn.set_image(Some(
        load_image_from_data!("../themes/windows98/maximize.png").unwrap(),
    ));
    max_btn.set_align(Align::Inside | Align::Top);
    max_btn.set_frame(FrameType::FlatBox);

    let mut win_clone_max = win.clone();
    let mut is_maximized = false;
    max_btn.set_callback(move |_| {
        if is_maximized {
            win_clone_max.fullscreen(false);
            is_maximized = false;
        } else {
            win_clone_max.fullscreen(true);
            is_maximized = true;
        }
    });

    let mut min_btn = Button::new(win_width - 56, 6, 16, 14, "");
    min_btn.set_image(Some(
        load_image_from_data!("../themes/windows98/hide.png").unwrap(),
    ));
    min_btn.set_align(Align::Inside | Align::Top);
    min_btn.set_frame(FrameType::FlatBox);

    let mut win_clone_min = win.clone();
    min_btn.set_callback(move |_| {
        win_clone_min.iconize();
    });
}

pub fn handle_drag(win: &mut fltk::window::Window) {
    let mut drag = Box::new((0, 0));
    win.handle(move |w, ev| match ev {
        Event::Push => {
            let sx = app::event_x_root();
            let sy = app::event_y_root();
            *drag = (sx - w.x(), sy - w.y());
            true
        }
        Event::Drag => {
            let (offx, offy) = *drag;
            let sx = app::event_x_root();
            let sy = app::event_y_root();
            w.set_pos(sx - offx, sy - offy);
            true
        }
        _ => false,
    });
}

pub fn create_win98_progress_bar(x: i32, y: i32, w: i32, h: i32) -> Frame {
    let mut progress_frame = Frame::new(x, y, w, h, "");
    progress_frame.set_frame(FrameType::DownBox);
    progress_frame.set_color(Color::White);

    let mut progress_bar = Frame::new(x + 2, y + 2, 0, h - 4, "");
    progress_bar.set_frame(FrameType::FlatBox);

    progress_bar.draw(move |f| {
        let fw = f.w();
        let fh = f.h();
        let fx = f.x();
        let fy = f.y();

        let win98_blue = Color::from_rgb(0, 0, 128);
        let block_width = 8;
        let block_spacing = 2;

        let max_blocks = fw / (block_width + block_spacing);

        for i in 0..max_blocks {
            let bx = fx + i * (block_width + block_spacing);
            if bx + block_width <= fx + fw {
                fltk::draw::draw_rect_fill(bx, fy, block_width, fh, win98_blue);
            }
        }
    });

    progress_bar
}
