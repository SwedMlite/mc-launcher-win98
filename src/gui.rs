use fltk::{
    app,
    button::{Button, CheckButton},
    draw,
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
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[cfg(target_os = "windows")]
use crate::windows::adjust_window;

pub const WIN_WIDTH: i32 = 600;
pub const WIN_HEIGHT: i32 = 300;

const CENTER_DIVISOR: i32 = 2;
const FONT_SIZE: i32 = 12;
const SMALL_FONT_SIZE: i32 = 10;
const PADDING: i32 = 15;
const LEFT_MARGIN: i32 = 20;
const BUTTON_X: i32 = 100;

const DIALOG_WIDTH: i32 = 300;
const DIALOG_HEIGHT: i32 = 200;

const LABEL_HEIGHT: i32 = 20;
const CONTROL_HEIGHT: i32 = 25;
const BUTTON_HEIGHT: i32 = 30;
const BUTTON_WIDTH: i32 = 110;
const BUTTON_SPACING: i32 = 10;
const INPUT_WIDTH: i32 = 260;
const LABEL_WIDTH: i32 = 100;
const FOLDER_BUTTON_WIDTH: i32 = 30;
const JAVA_FIELD_WIDTH: i32 = 60;
const JVM_ARGS_WIDTH: i32 = 160;
const JVM_HINT_WIDTH: i32 = 135;
const VERSION_LABEL_WIDTH: i32 = 100;
const PROGRESS_LABEL_WIDTH: i32 = 100;
const JAVA_LABEL_WIDTH: i32 = 50;
const CHECKBOX_WIDTH: i32 = 75;
const CHECKBOX_HEIGHT: i32 = 20;

const FRAME_OFFSET: i32 = 8;
const PADDING_MULTIPLIER: i32 = 3;

const TOP_MARGIN: i32 = 40;
const USERNAME_Y: i32 = TOP_MARGIN;
const INPUT_Y: i32 = 65;
const VERSION_Y: i32 = 60;
const JVM_ARGS_Y: i32 = 95;
const CHECKBOX_Y: i32 = 95;
const JVM_HINT_X: i32 = 165;
const JVM_INPUT_Y: i32 = 120;
const JAVA_LABEL_Y: i32 = 130;
const BUTTON_Y: i32 = 160;
const BOTTOM_SECTION_Y: i32 = 165;
const BUTTONS_Y_OFFSET: i32 = 25;
const BUTTONS_MARGIN: i32 = 35;
const WELCOME_FRAME_HEIGHT: i32 = 20;

const GRAY_COLOR: Color = Color::from_rgb(192, 192, 192);
const HINT_TEXT_COLOR: Color = Color::from_rgb(80, 80, 80);
const ERROR_ICON_COLOR: Color = Color::from_rgb(255, 0, 0);

const ERROR_TEXT_MAX_WIDTH: i32 = 190;
const CHARS_PER_LINE_DIVISOR: i32 = 7;
const DEFAULT_TEXT_HEIGHT: i32 = 30;
const LINE_HEIGHT: i32 = 15;
const ERROR_BASE_HEIGHT: i32 = 125;
const ERROR_ICON_X: i32 = 30;
const ERROR_ICON_Y: i32 = 45;
const ERROR_ICON_SIZE: i32 = 48;
const ERROR_ICON_FONT_SIZE: i32 = FONT_SIZE * 2;
const ERROR_TEXT_X: i32 = 90;
const OK_BUTTON_X_WITH_DETAILS: i32 = 60;
const OK_BUTTON_X_SOLO: i32 = 110;
const DETAILS_BUTTON_X: i32 = 170;

const PROGRESS_MARGIN: i32 = 15;
const PROGRESS_BORDER: i32 = 2;
const PROGRESS_HEIGHT: i32 = 16;

const BACKGROUND_TILE_SIZE: i32 = 64;

static DIALOG_RUNNING: AtomicBool = AtomicBool::new(false);
static mut MAX_PROGRESS_WIDTH: i32 = 100;

pub fn create_new_profile_dialog(text_font: Font) -> Option<Profile> {
    if DIALOG_RUNNING.swap(true, Ordering::SeqCst) {
        return None;
    }

    let mut win = Window::default()
        .with_size(DIALOG_WIDTH, DIALOG_HEIGHT)
        .with_label("New Profile");
    win.set_border(false);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    win.set_pos(
        (screen_width - DIALOG_WIDTH) / CENTER_DIVISOR,
        (screen_height - DIALOG_HEIGHT) / CENTER_DIVISOR,
    );

    let mut bg = Frame::new(0, 0, DIALOG_WIDTH, DIALOG_HEIGHT, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(GRAY_COLOR);

    let mut username_label = Frame::new(
        LEFT_MARGIN,
        USERNAME_Y,
        LABEL_WIDTH,
        CONTROL_HEIGHT,
        "Username:",
    );
    username_label.set_label_font(text_font);
    username_label.set_label_size(FONT_SIZE);
    username_label.set_align(Align::Left | Align::Inside);

    let mut username_input = Input::new(LEFT_MARGIN, INPUT_Y, INPUT_WIDTH, CONTROL_HEIGHT, "");
    username_input.set_text_font(text_font);
    username_input.set_text_size(FONT_SIZE);

    let mut jvm_args_label = Frame::new(
        LEFT_MARGIN,
        JVM_ARGS_Y,
        JVM_ARGS_WIDTH,
        CONTROL_HEIGHT,
        "JVM Arguments (optional):",
    );
    jvm_args_label.set_label_font(text_font);
    jvm_args_label.set_label_size(FONT_SIZE);
    jvm_args_label.set_align(Align::Left | Align::Inside);

    let mut jvm_hint_label = Frame::new(
        JVM_HINT_X,
        JVM_ARGS_Y,
        JVM_HINT_WIDTH,
        CONTROL_HEIGHT,
        "Example: -Xmx2G -Xms512M",
    );
    jvm_hint_label.set_label_font(text_font);
    jvm_hint_label.set_label_size(SMALL_FONT_SIZE);
    jvm_hint_label.set_label_color(HINT_TEXT_COLOR);
    jvm_hint_label.set_align(Align::Left | Align::Inside);

    let mut jvm_args_input = Input::new(LEFT_MARGIN, JVM_INPUT_Y, INPUT_WIDTH, CONTROL_HEIGHT, "");
    jvm_args_input.set_text_font(text_font);
    jvm_args_input.set_text_size(FONT_SIZE);

    let mut create_button = Button::new(BUTTON_X, BUTTON_Y, BUTTON_X, CONTROL_HEIGHT, "Create");
    create_button.set_label_font(text_font);
    create_button.set_label_size(FONT_SIZE);
    create_button.set_frame(FrameType::UpBox);
    create_button.set_color(GRAY_COLOR);

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
        .with_size(DIALOG_WIDTH, DIALOG_HEIGHT)
        .with_label("Edit Profile");
    win.set_border(false);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    win.set_pos(
        (screen_width - DIALOG_WIDTH) / CENTER_DIVISOR,
        (screen_height - DIALOG_HEIGHT) / CENTER_DIVISOR,
    );

    let mut bg = Frame::new(0, 0, DIALOG_WIDTH, DIALOG_HEIGHT, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(GRAY_COLOR);

    let mut username_label = Frame::new(
        LEFT_MARGIN,
        USERNAME_Y,
        LABEL_WIDTH,
        CONTROL_HEIGHT,
        "Username:",
    );
    username_label.set_label_font(text_font);
    username_label.set_label_size(FONT_SIZE);
    username_label.set_align(Align::Left | Align::Inside);

    let mut username_input = Input::new(LEFT_MARGIN, INPUT_Y, INPUT_WIDTH, CONTROL_HEIGHT, "");
    username_input.set_text_font(text_font);
    username_input.set_text_size(FONT_SIZE);
    username_input.set_value(&profile.username);

    let mut jvm_args_label = Frame::new(
        LEFT_MARGIN,
        JVM_ARGS_Y,
        JVM_ARGS_WIDTH,
        CONTROL_HEIGHT,
        "JVM Arguments (optional):",
    );
    jvm_args_label.set_label_font(text_font);
    jvm_args_label.set_label_size(FONT_SIZE);
    jvm_args_label.set_align(Align::Left | Align::Inside);

    let mut jvm_hint_label = Frame::new(
        JVM_HINT_X,
        JVM_ARGS_Y,
        JVM_HINT_WIDTH,
        CONTROL_HEIGHT,
        "Example: -Xmx2G -Xms512M",
    );
    jvm_hint_label.set_label_font(text_font);
    jvm_hint_label.set_label_size(SMALL_FONT_SIZE);
    jvm_hint_label.set_label_color(HINT_TEXT_COLOR);
    jvm_hint_label.set_align(Align::Left | Align::Inside);

    let mut jvm_args_input = Input::new(LEFT_MARGIN, JVM_INPUT_Y, INPUT_WIDTH, CONTROL_HEIGHT, "");
    jvm_args_input.set_text_font(text_font);
    jvm_args_input.set_text_size(FONT_SIZE);
    if let Some(args) = &profile.jvm_args {
        jvm_args_input.set_value(args);
    }

    let mut save_button = Button::new(BUTTON_X, BUTTON_Y, BUTTON_X, CONTROL_HEIGHT, "Save");
    save_button.set_label_font(text_font);
    save_button.set_label_size(FONT_SIZE);
    save_button.set_frame(FrameType::UpBox);
    save_button.set_color(GRAY_COLOR);

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

pub fn setup_font(app: app::App) -> Font {
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

    let error_width = DIALOG_WIDTH;
    let chars_per_line = ERROR_TEXT_MAX_WIDTH / CHARS_PER_LINE_DIVISOR;

    let mut text_height = DEFAULT_TEXT_HEIGHT;
    let line_count = (display_message.len() as f32 / chars_per_line as f32).ceil() as i32;
    if line_count > 1 {
        text_height = line_count * LINE_HEIGHT;
    }

    let window_height = ERROR_BASE_HEIGHT + (text_height - DEFAULT_TEXT_HEIGHT).max(0);

    let mut dialog = Window::default()
        .with_size(error_width, window_height)
        .with_label("Error");
    dialog.set_border(false);
    dialog.make_modal(true);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    dialog.set_pos(
        (screen_width - error_width) / CENTER_DIVISOR,
        (screen_height - window_height) / CENTER_DIVISOR,
    );

    let mut bg = Frame::new(0, 0, error_width, window_height, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(GRAY_COLOR);

    let mut error_icon = Frame::new(
        ERROR_ICON_X,
        ERROR_ICON_Y,
        ERROR_ICON_SIZE,
        ERROR_ICON_SIZE,
        "",
    );
    error_icon.set_label_font(text_font);
    error_icon.set_label_size(ERROR_ICON_FONT_SIZE);
    error_icon.set_label_color(ERROR_ICON_COLOR);
    let mut icon_img = load_image_from_data!("../themes/windows98/error_icon.png").unwrap();
    icon_img.scale(ERROR_ICON_SIZE, ERROR_ICON_SIZE, false, true);
    error_icon.set_image(Some(icon_img));

    let mut text = Frame::new(
        ERROR_TEXT_X,
        ERROR_ICON_Y,
        ERROR_TEXT_MAX_WIDTH,
        text_height,
        display_message,
    );
    text.set_align(Align::Left | Align::Inside | Align::Wrap);
    text.set_label_font(text_font);
    text.set_label_size(FONT_SIZE);

    let show_details = display_message != message;
    let buttons_y = window_height - BUTTONS_MARGIN;
    let button_width = BUTTON_X;

    let mut ok_btn = Button::new(
        if show_details {
            OK_BUTTON_X_WITH_DETAILS
        } else {
            OK_BUTTON_X_SOLO
        },
        buttons_y,
        button_width,
        CONTROL_HEIGHT,
        "OK",
    );
    ok_btn.set_label_font(text_font);
    ok_btn.set_label_size(FONT_SIZE);
    ok_btn.set_frame(FrameType::UpBox);
    ok_btn.set_color(GRAY_COLOR);

    let mut details_btn = Button::new(
        DETAILS_BUTTON_X,
        buttons_y,
        button_width,
        CONTROL_HEIGHT,
        "Details",
    );
    if show_details {
        details_btn.set_label_font(text_font);
        details_btn.set_label_size(FONT_SIZE);
        details_btn.set_frame(FrameType::UpBox);
        details_btn.set_color(GRAY_COLOR);
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
    let top_img = load_image_from_data!("../themes/windows98/top_frame.png").unwrap();
    let bottom_img = load_image_from_data!("../themes/windows98/bottom_frame.png").unwrap();
    let left_img = load_image_from_data!("../themes/windows98/left_frame.png").unwrap();
    let right_img = load_image_from_data!("../themes/windows98/right_frame.png").unwrap();
    let lt_corner_img = load_image_from_data!("../themes/windows98/left_top_corner.png").unwrap();
    let rt_corner_img = load_image_from_data!("../themes/windows98/right_top_corner.png").unwrap();
    let lb_corner_img =
        load_image_from_data!("../themes/windows98/left_bottom_corner.png").unwrap();
    let rb_corner_img =
        load_image_from_data!("../themes/windows98/right_bottom_corner.png").unwrap();

    let mut frame = Frame::new(0, 0, width, height, "");
    frame.set_frame(FrameType::NoBox);

    let lt_w = lt_corner_img.width();
    let lt_h = lt_corner_img.height();
    let rt_w = rt_corner_img.width();
    let rt_h = rt_corner_img.height();
    let lb_w = lb_corner_img.width();
    let lb_h = lb_corner_img.height();
    let rb_w = rb_corner_img.width();
    let rb_h = rb_corner_img.height();
    let top_h = top_img.height();
    let bottom_h = bottom_img.height();
    let left_w = left_img.width();
    let right_w = right_img.width();

    let mut top_img = top_img;
    let mut bottom_img = bottom_img;
    let mut left_img = left_img;
    let mut right_img = right_img;
    let mut lt_corner_img = lt_corner_img;
    let mut rt_corner_img = rt_corner_img;
    let mut lb_corner_img = lb_corner_img;
    let mut rb_corner_img = rb_corner_img;

    frame.draw(move |f| {
        let fw = f.w();
        let fh = f.h();
        let fx = f.x();
        let fy = f.y();

        lt_corner_img.draw(fx, fy, lt_w, lt_h);
        rt_corner_img.draw(fx + fw - rt_w, fy, rt_w, rt_h);

        lb_corner_img.draw(fx, fy + fh - lb_h, lb_w, lb_h);
        rb_corner_img.draw(fx + fw - rb_w, fy + fh - rb_h, rb_w, rb_h);

        let top_width = fw - lt_w - rt_w;
        let top_tile_w = top_img.width();
        let mut x_pos = fx + lt_w;
        while x_pos < fx + lt_w + top_width {
            let draw_width = (fx + lt_w + top_width - x_pos).min(top_tile_w);
            top_img.draw(x_pos, fy, draw_width, top_h);
            x_pos += draw_width;
        }

        let bottom_width = fw - lb_w - rb_w;
        let bottom_tile_w = bottom_img.width();
        let mut x_pos = fx + lb_w;
        while x_pos < fx + lb_w + bottom_width {
            let draw_width = (fx + lb_w + bottom_width - x_pos).min(bottom_tile_w);
            bottom_img.draw(x_pos, fy + fh - bottom_h, draw_width, bottom_h);
            x_pos += draw_width;
        }

        let left_height = fh - lt_h - lb_h;
        let left_tile_h = left_img.height();
        let mut y_pos = fy + lt_h;
        while y_pos < fy + lt_h + left_height {
            let draw_height = (fy + lt_h + left_height - y_pos).min(left_tile_h);
            left_img.draw(fx, y_pos, left_w, draw_height);
            y_pos += draw_height;
        }

        let right_height = fh - rt_h - rb_h;
        let right_tile_h = right_img.height();
        let mut y_pos = fy + rt_h;
        while y_pos < fy + rt_h + right_height {
            let draw_height = (fy + rt_h + right_height - y_pos).min(right_tile_h);
            right_img.draw(fx + fw - right_w, y_pos, right_w, draw_height);
            y_pos += draw_height;
        }
    });

    frame.redraw();
}

pub fn setup_tiled_background() {
    let mut bg_tile = load_image_from_data!("../themes/background.png").unwrap();
    bg_tile.scale(BACKGROUND_TILE_SIZE, BACKGROUND_TILE_SIZE, false, true);
    let tile_w = bg_tile.width();
    let tile_h = bg_tile.height();

    let mut tiled_frame = Frame::new(0, FRAME_OFFSET, WIN_WIDTH, WIN_HEIGHT, "");
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
    let column_width = (WIN_WIDTH - (PADDING * PADDING_MULTIPLIER)) / 2;
    let left_x = PADDING;
    let right_x = left_x + column_width + PADDING;

    let mut ver_label = Frame::new(
        left_x,
        TOP_MARGIN,
        VERSION_LABEL_WIDTH,
        LABEL_HEIGHT,
        "Version:",
    );
    ver_label.set_label_font(text_font);
    ver_label.set_label_size(FONT_SIZE);
    ver_label.set_label_color(Color::White);
    ver_label.set_frame(FrameType::NoBox);
    ver_label.set_align(Align::Left | Align::Inside);

    let mut version_choice = Choice::new(left_x, VERSION_Y, column_width, CONTROL_HEIGHT, "");
    version_choice.set_color(Color::White);
    version_choice.set_text_font(text_font);
    version_choice.set_text_size(FONT_SIZE);

    let mut profile_label = Frame::new(
        right_x,
        TOP_MARGIN,
        VERSION_LABEL_WIDTH,
        LABEL_HEIGHT,
        "Profile:",
    );
    profile_label.set_label_font(text_font);
    profile_label.set_label_size(FONT_SIZE);
    profile_label.set_label_color(Color::White);
    profile_label.set_frame(FrameType::NoBox);
    profile_label.set_align(Align::Left | Align::Inside);

    let mut profile_choice = Choice::new(right_x, VERSION_Y, column_width, CONTROL_HEIGHT, "");
    profile_choice.set_color(Color::White);
    profile_choice.set_text_font(text_font);
    profile_choice.set_text_size(FONT_SIZE);

    for name in profile_names {
        profile_choice.add_choice(name);
    }
    if !profile_names.is_empty() {
        profile_choice.set_value(0);
    }

    let type_labels = [
        ("Release", "release"),
        ("Snapshot", "snapshot"),
        ("Alpha", "old_alpha"),
        ("Beta", "old_beta"),
    ];

    let mut checkboxes = Vec::new();

    for (i, (label, _)) in type_labels.iter().enumerate() {
        let mut cb = CheckButton::new(
            left_x + i as i32 * CHECKBOX_WIDTH,
            CHECKBOX_Y,
            CHECKBOX_WIDTH,
            CHECKBOX_HEIGHT,
            *label,
        );
        cb.set_label_font(text_font);
        cb.set_label_size(FONT_SIZE);
        cb.set_label_color(Color::White);
        cb.set_value(true);
        checkboxes.push(cb);
    }

    let profile_button_width = (column_width - PADDING) / 2;

    let mut new_profile = Button::new(
        right_x,
        CHECKBOX_Y,
        profile_button_width,
        CONTROL_HEIGHT,
        "New Profile",
    );
    new_profile.set_label_font(text_font);
    new_profile.set_label_size(FONT_SIZE);

    let mut edit_profile = Button::new(
        right_x + profile_button_width + PADDING / 2,
        CHECKBOX_Y,
        profile_button_width,
        CONTROL_HEIGHT,
        "Edit Profile",
    );
    edit_profile.set_label_font(text_font);
    edit_profile.set_label_size(FONT_SIZE);
    edit_profile.deactivate();

    let mut java_label = Frame::new(
        left_x,
        JAVA_LABEL_Y,
        JAVA_LABEL_WIDTH,
        LABEL_HEIGHT,
        "Java:",
    );
    java_label.set_label_font(text_font);
    java_label.set_label_size(FONT_SIZE);
    java_label.set_label_color(Color::White);
    java_label.set_frame(FrameType::NoBox);
    java_label.set_align(Align::Left | Align::Inside);

    let mut java_choice = Choice::new(
        left_x + JAVA_FIELD_WIDTH,
        JAVA_LABEL_Y,
        WIN_WIDTH - PADDING * 2 - JAVA_FIELD_WIDTH,
        CONTROL_HEIGHT,
        "",
    );
    java_choice.set_color(Color::White);
    java_choice.set_text_font(text_font);
    java_choice.set_text_size(FONT_SIZE);

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

    let welcome_text = if !profile_names.is_empty() {
        format!("Welcome, {}", profile_names[0])
    } else {
        "Welcome, Guest".to_string()
    };

    let mut welcome_frame = Frame::new(
        left_x,
        BOTTOM_SECTION_Y,
        WIN_WIDTH - PADDING * 2,
        WELCOME_FRAME_HEIGHT,
        welcome_text.as_str(),
    );
    welcome_frame.set_label_font(text_font);
    welcome_frame.set_label_size(FONT_SIZE);
    welcome_frame.set_label_color(Color::White);
    welcome_frame.set_frame(FrameType::NoBox);
    welcome_frame.set_align(Align::Center | Align::Inside);

    let buttons_y = BOTTOM_SECTION_Y + BUTTONS_Y_OFFSET;
    let play_button_x =
        (WIN_WIDTH - BUTTON_WIDTH - FOLDER_BUTTON_WIDTH - BUTTON_SPACING) / CENTER_DIVISOR;

    let mut folder_button = Button::new(
        play_button_x,
        buttons_y,
        FOLDER_BUTTON_WIDTH,
        BUTTON_HEIGHT,
        "",
    );

    let folder_img = load_image_from_data!("../themes/windows98/folder.png").unwrap();
    folder_button.set_frame(FrameType::UpBox);
    folder_button.set_color(GRAY_COLOR);
    folder_button.set_tooltip("Open launcher folder");

    let mut icon = folder_img.clone();
    let icon_w = icon.width();
    let icon_h = icon.height();

    folder_button.draw(move |b| {
        draw::draw_box(b.frame(), b.x(), b.y(), b.w(), b.h(), b.color());

        let center_x = b.x() + (b.w() - icon_w) / 2;
        let center_y = b.y() + (b.h() - icon_h) / 2;

        icon.draw(center_x, center_y, icon_w, icon_h);
    });

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

    let mut play = Button::new(
        play_button_x + FOLDER_BUTTON_WIDTH + BUTTON_SPACING,
        buttons_y,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
        "Play",
    );
    play.set_label_font(text_font);
    play.set_label_size(FONT_SIZE);

    let progress_section_y = buttons_y + BUTTON_HEIGHT + PROGRESS_MARGIN;

    let mut progress_label = Frame::new(
        left_x,
        progress_section_y,
        PROGRESS_LABEL_WIDTH,
        LABEL_HEIGHT,
        "Progress:",
    );
    progress_label.set_label_font(text_font);
    progress_label.set_label_size(FONT_SIZE);
    progress_label.set_label_color(Color::White);
    progress_label.set_frame(FrameType::NoBox);
    progress_label.set_align(Align::Left | Align::Inside);

    let progress_bar_x = left_x + PROGRESS_LABEL_WIDTH;
    let progress_bar_width = WIN_WIDTH - PADDING * 2 - PROGRESS_LABEL_WIDTH;

    let mut status_label = Frame::new(
        left_x,
        progress_section_y + LABEL_HEIGHT,
        WIN_WIDTH - PADDING * 2,
        LABEL_HEIGHT,
        "",
    );
    status_label.set_label_font(text_font);
    status_label.set_label_size(FONT_SIZE);
    status_label.set_label_color(Color::White);
    status_label.set_frame(FrameType::NoBox);
    status_label.set_align(Align::Left | Align::Inside);

    let mut progress_frame = Frame::new(
        progress_bar_x,
        progress_section_y,
        progress_bar_width,
        LABEL_HEIGHT,
        "",
    );
    progress_frame.set_frame(FrameType::DownBox);
    progress_frame.set_color(Color::White);

    let progress_bar = create_win98_progress_bar(
        progress_bar_x + PROGRESS_BORDER,
        progress_section_y + PROGRESS_BORDER,
        progress_bar_width - PROGRESS_BORDER * 2,
        PROGRESS_HEIGHT,
    );

    let mut update_dropdown = {
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
    const TITLE_FRAME_WIDTH: i32 = 300;
    const TITLE_HEIGHT: i32 = 20;
    const ICON_SIZE: i32 = 16;
    const BTN_WIDTH: i32 = 16;
    const BTN_HEIGHT: i32 = 14;
    const TITLE_X: i32 = 24;
    const TITLE_Y: i32 = 4;
    const ICON_X: i32 = 8;
    const ICON_Y: i32 = 12;
    const CLOSE_OFFSET: i32 = 22;
    const MAX_OFFSET: i32 = 39;
    const MIN_OFFSET: i32 = 56;
    const BTN_Y: i32 = 6;

    let mut title_frame = Frame::new(TITLE_X, TITLE_Y, TITLE_FRAME_WIDTH, TITLE_HEIGHT, title);
    title_frame.set_label_color(Color::White);
    title_frame.set_label_font(font);
    title_frame.set_label_size(FONT_SIZE);
    title_frame.set_align(Align::Left | Align::Inside);
    title_frame.set_frame(FrameType::NoBox);

    let mut icon = Frame::new(ICON_X, ICON_Y, ICON_SIZE, ICON_SIZE, "");
    ico.scale(ICON_SIZE, ICON_SIZE, false, false);
    icon.set_image(Some(ico.clone()));

    let is_main_window = win.label() == "Minecraft Launcher";

    let mut close_btn = Button::new(win_width - CLOSE_OFFSET, BTN_Y, BTN_WIDTH, BTN_HEIGHT, "");
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

    let mut max_btn = Button::new(win_width - MAX_OFFSET, BTN_Y, BTN_WIDTH, BTN_HEIGHT, "");
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

    let mut min_btn = Button::new(win_width - MIN_OFFSET, BTN_Y, BTN_WIDTH, BTN_HEIGHT, "");
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
    unsafe {
        MAX_PROGRESS_WIDTH = w;
    }

    let mut progress_bar = Frame::new(x, y, 0, h, "");
    progress_bar.set_frame(FrameType::FlatBox);

    progress_bar.draw(move |f| {
        let fw = f.w();
        let fh = f.h();
        let fx = f.x();
        let fy = f.y();

        if fw <= 0 {
            return;
        }

        let win98_blue = Color::from_rgb(0, 0, 128);
        const BLOCK_WIDTH: i32 = 8;
        const BLOCK_SPACING: i32 = 2;

        let max_blocks = fw / (BLOCK_WIDTH + BLOCK_SPACING);

        for i in 0..max_blocks {
            let bx = fx + i * (BLOCK_WIDTH + BLOCK_SPACING);
            fltk::draw::draw_rect_fill(bx, fy, BLOCK_WIDTH, fh, win98_blue);
        }

        let remaining_width = fw % (BLOCK_WIDTH + BLOCK_SPACING);
        if remaining_width > 0 && remaining_width <= BLOCK_WIDTH {
            let bx = fx + max_blocks * (BLOCK_WIDTH + BLOCK_SPACING);
            fltk::draw::draw_rect_fill(bx, fy, remaining_width, fh, win98_blue);
        }
    });

    progress_bar
}

pub fn update_win98_progress_bar(progress_bar: &mut Frame, percentage: f64) {
    let percentage = percentage.max(0.0).min(100.0);
    let max_width = unsafe { MAX_PROGRESS_WIDTH };

    let new_width = ((percentage / 100.0) * (max_width as f64)) as i32;
    let final_width = new_width.max(0).min(max_width);

    progress_bar.set_size(final_width, progress_bar.h());
    progress_bar.redraw();
}
