use fltk::{
    app,
    button::{Button, CheckButton},
    enums::{Align, Color, Event, Font, FrameType},
    frame::Frame,
    image::PngImage,
    input::Input,
    menu,
    prelude::*,
    window::Window
};

macro_rules! load_image_from_data {
    ($path:expr) => {
        PngImage::from_data(include_bytes!($path))
    };
}
use crate::models::Profile;
use crate::java_finder::find_all_java_installations;
use std::sync::Arc;
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use crate::windows::adjust_window;

pub const WIN_WIDTH: i32 = 600;
pub const WIN_HEIGHT: i32 = 225;

pub fn create_new_profile_dialog(text_font: Font) -> Option<Profile> {
    let mut win = Window::default().with_size(300, 160).with_label("New Profile");
    win.set_border(false);
    win.make_modal(true);
    
    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    win.set_pos((screen_width - 300) / 2, (screen_height - 160) / 2);
    
    let mut bg = Frame::new(0, 0, 300, 160, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(Color::from_rgb(192, 192, 192));
    
    let mut username_label = Frame::new(20, 40, 100, 25, "Username:");
    username_label.set_label_font(text_font);
    username_label.set_label_size(12);
    username_label.set_align(Align::Left | Align::Inside);
    
    let mut username_input = Input::new(20, 65, 260, 25, "");
    username_input.set_text_font(text_font);
    username_input.set_text_size(12);
    
    let mut create_button = Button::new(100, 110, 100, 25, "Create");
    create_button.set_label_font(text_font);
    create_button.set_label_size(12);
    create_button.set_frame(FrameType::UpBox);
    create_button.set_color(Color::from_rgb(192, 192, 192));

    setup_frame(win.width(), win.height());
    setup_title_bar("New Profile", &mut load_image_from_data!("minecraft_icon.png").unwrap(), text_font, &win);

    win.end();

    let result = Arc::new(Mutex::new(None::<Profile>));
    
    let result_clone = result.clone();
    let username_input_clone = username_input.clone();
    let mut win_clone = win.clone();
    create_button.set_callback(move |_| {
        let username = username_input_clone.value().trim().to_string();
        if !username.is_empty() {
            *result_clone.lock().unwrap() = Some(Profile { username });
        }
        win_clone.hide();
    });

    win.show();
    handle_drag(&mut win);

    #[cfg(target_os = "windows")]
    adjust_window(&win);
    
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    
    while win.shown() && start_time.elapsed() < timeout {
        let _ = app::wait_for(0.05);
        
        if win.visible() && username_input.changed() {
            let new_value = username_input.value();
            if !new_value.trim().is_empty() {
                *result.lock().unwrap() = Some(Profile { username: new_value.trim().to_string() });
            }
        }
    }
    
    if win.shown() {
        win.hide();
    }
    
    result.lock().unwrap().clone()
}

pub fn edit_profile_dialog(text_font: Font, profile: &Profile) -> Option<Profile> {
    let mut win = Window::default().with_size(300, 160).with_label("Edit Profile");
    win.set_border(false);
    win.make_modal(true);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    win.set_pos((screen_width - 300) / 2, (screen_height - 160) / 2);
    
    let mut bg = Frame::new(0, 0, 300, 160, "");
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
    
    let mut save_button = Button::new(100, 110, 100, 25, "Save");
    save_button.set_label_font(text_font);
    save_button.set_label_size(12);
    save_button.set_frame(FrameType::UpBox);
    save_button.set_color(Color::from_rgb(192, 192, 192));
    
    let mut title_frame = Frame::new(0, 0, 300, 24, "Edit Profile");
    title_frame.set_frame(FrameType::FlatBox);
    title_frame.set_color(Color::from_rgb(0, 0, 128));
    title_frame.set_label_color(Color::White);
    title_frame.set_label_font(text_font);
    
    win.end();
    
    let result = Arc::new(Mutex::new(None::<Profile>));
    
    let username_input_clone = username_input.clone();
    let result_clone = result.clone();
    let mut win_clone = win.clone();
    save_button.set_callback(move |_| {
        let username = username_input_clone.value().trim().to_string();
        if !username.is_empty() {
            *result_clone.lock().unwrap() = Some(Profile { username });
        }
        win_clone.hide();
    });
    
    win.show();
    
    #[cfg(target_os = "windows")]
    adjust_window(&win);
    
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    
    while win.shown() && start_time.elapsed() < timeout {
        let _ = app::wait_for(0.05);
        
        if win.visible() && username_input.changed() {
            let new_value = username_input.value();
            if !new_value.trim().is_empty() {
                *result.lock().unwrap() = Some(Profile { username: new_value.trim().to_string() });
            }
        }
    }
    
    if win.shown() {
        win.hide();
    }
    
    result.lock().unwrap().clone()
}

pub fn setup_font(app: app::App, font_path: &str) -> Font {
    match app::App::load_font(app, font_path) {
        Ok(f) => Font::by_name(&f),
        Err(_) => Font::Helvetica,
    }
}

pub fn show_error_dialog(message: &str, text_font: Font) {
    let mut dialog = Window::default().with_size(300, 125).with_label("Error");
    dialog.set_border(false);
    dialog.make_modal(true);

    let screen_width = app::screen_size().0 as i32;
    let screen_height = app::screen_size().1 as i32;
    dialog.set_pos((screen_width - 300) / 2, (screen_height - 125) / 2);

    let mut bg = Frame::new(0, 0, 300, 125, "");
    bg.set_frame(FrameType::FlatBox);
    bg.set_color(Color::from_rgb(192, 192, 192));

    let mut error_icon = Frame::new(30, 45, 48, 48, "");
    error_icon.set_label_font(text_font);
    error_icon.set_label_size(24);
    error_icon.set_label_color(Color::from_rgb(255, 0, 0));
    let mut icon_img = load_image_from_data!("windows98/error_icon.png").unwrap();
    icon_img.scale(48, 48, false, true);
    error_icon.set_image(Some(icon_img));
    
    let mut text = Frame::new(90, 45, 190, 30, message);
    text.set_align(Align::Left | Align::Inside);
    text.set_label_font(text_font);
    text.set_label_size(12);

    let mut ok_btn = Button::new(110, 90, 100, 25, "OK");
    ok_btn.set_label_font(text_font);
    ok_btn.set_label_size(12);
    ok_btn.set_frame(FrameType::UpBox);
    ok_btn.set_color(Color::from_rgb(192, 192, 192));
    
    setup_frame(dialog.width(), dialog.height());
    setup_title_bar("Error", &mut load_image_from_data!("windows98/error_icon.png").unwrap(), text_font, &dialog);
    
    dialog.end();
    
    let mut dialog_clone = dialog.clone();
    ok_btn.set_callback(move |_| {
        dialog_clone.hide();
    });
    
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
    let offset = 8;
    let bottom_adjust = 5;
    
    let top = load_image_from_data!("windows98/top_frame.png").unwrap();
    let bottom = load_image_from_data!("windows98/bottom_frame.png").unwrap();
    let left = load_image_from_data!("windows98/left_frame.png").unwrap();
    let right = load_image_from_data!("windows98/right_frame.png").unwrap();
    let lt_corner = load_image_from_data!("windows98/left_top_corner.png").unwrap();
    let rt_corner = load_image_from_data!("windows98/right_top_corner.png").unwrap();
    let lb_corner = load_image_from_data!("windows98/left_bottom_corner.png").unwrap();
    let rb_corner = load_image_from_data!("windows98/right_bottom_corner.png").unwrap();

    let tc_w = lt_corner.width();
    let tc_h = lt_corner.height();
    let bc_w = lb_corner.width();
    let bc_h = lb_corner.height();
    let v_w = left.width();
    let h_h = top.height();
    let b_h = bottom.height();

    let v_h = height - bc_h - tc_h - offset + bottom_adjust; 

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
    };

    add_frame(0, offset, tc_w, tc_h, &lt_corner, false);
    add_frame(width - rt_corner.width(), offset, rt_corner.width(), rt_corner.height(), &rt_corner, false);
    add_frame(tc_w, offset, width - tc_w - rt_corner.width(), h_h, &top, true);
 
    add_frame(0, height - bc_h + bottom_adjust, bc_w, bc_h, &lb_corner, false);
    add_frame(width - rb_corner.width(), height - bc_h + bottom_adjust, rb_corner.width(), bc_h, &rb_corner, false); 
    add_frame(lb_corner.width(), height - b_h + bottom_adjust, width - lb_corner.width() - rb_corner.width(), b_h, &bottom, true); 

    add_frame(0, tc_h + offset, v_w, v_h, &left, true);
    add_frame(width - v_w, tc_h + offset, v_w, v_h, &right, true);
}

pub fn setup_tiled_background() {
    let mut bg_tile = load_image_from_data!("background.png").unwrap();
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

pub fn setup_main_controls(text_font: Font, versions: String, profile_names: &Vec<String>) -> (menu::Choice, menu::Choice, Button, Button, Button, Frame, menu::Choice) {
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

    let mut version_choice = menu::Choice::new(20, 60, WIN_WIDTH - 40, 25, "");
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
    
    // All checkboxes on one row: Release, Snapshot, Alpha, Beta
    for (i, (label, _)) in type_labels.iter().enumerate() {
        let mut cb = CheckButton::new(20 + i as i32 * 85, 95, 80, 20, *label);
        cb.set_label_font(text_font);
        cb.set_label_size(12);
        cb.set_label_color(Color::White);
        cb.set_value(true);
        checkboxes.push(cb);
    }
    
    let mut java_choice = menu::Choice::new(400, 95, 180, 20, "");
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
                        grandparent.file_name().unwrap_or_default().to_string_lossy().to_string()
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

    let mut profile_choice = menu::Choice::new(20, 145, WIN_WIDTH - 40, 25, "");
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
    
    let mut new_profile = Button::new(
        20, 
        buttons_y, 
        110,
        25,
        "New Profile",
    );
    new_profile.set_label_font(text_font);
    new_profile.set_label_size(12);

    let mut edit_profile = Button::new(
        140,
        buttons_y,
        110,
        25,
        "Edit Profile",
    );
    edit_profile.set_label_font(text_font);
    edit_profile.deactivate();
    
    let play_x = 310;
    let play_y = buttons_y;
    
    let mut play = Button::new(
        play_x,
        play_y,
        button_width,
        button_height,
        "Play",
    );
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
        welcome_text.as_str()
    );
    welcome_frame.set_label_font(text_font);
    welcome_frame.set_label_size(12);
    welcome_frame.set_label_color(Color::White);
    welcome_frame.set_frame(FrameType::NoBox);
    welcome_frame.set_align(Align::Left | Align::Inside);

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
    profile_choice.set_callback(move |c| {
        if let Some(selected) = c.choice() {
            welcome_frame_clone.set_label(&format!("Welcome, {}", selected));
        }
    });

    update_dropdown();

    (version_choice, profile_choice, play, new_profile, edit_profile, welcome_frame, java_choice)
}

pub fn setup_title_bar(
    title: &str,
    ico: &mut PngImage,
    font: Font,
    win: &Window,
) {
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
    close_btn.set_image(Some(load_image_from_data!("windows98/close.png").unwrap()));
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
    max_btn.set_image(Some(load_image_from_data!("windows98/maximize.png").unwrap()));
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
    min_btn.set_image(Some(load_image_from_data!("windows98/hide.png").unwrap()));
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
