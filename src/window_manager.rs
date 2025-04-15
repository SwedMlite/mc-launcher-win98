use fltk::{enums::Font, window::Window};
use fltk::prelude::{WidgetBase, WidgetExt, GroupExt,WindowExt};

#[cfg(target_os = "windows")]
use crate::windows::{adjust_window, set_window_icon};

use crate::gui::{handle_drag, setup_frame, setup_tiled_background, setup_title_bar, WIN_HEIGHT, WIN_WIDTH};

macro_rules! load_image_from_data {
    ($path:expr) => {
        fltk::image::PngImage::from_data(include_bytes!($path))
    };
}

pub fn setup_window(font: Font) -> Window {
    let mut win = Window::new(100, 100, WIN_WIDTH, WIN_HEIGHT, "Minecraft Launcher");
    win.set_border(false);

    setup_tiled_background();
    setup_frame(WIN_WIDTH, WIN_HEIGHT);
    setup_title_bar(
        "Minecraft Launcher",
        &mut load_image_from_data!("../themes/minecraft_icon.png").unwrap(),
        font,
        &win,
    );
    
    win
}

pub fn finalize_window(mut win: Window) {
    handle_drag(&mut win);
    win.end();
    
    #[cfg(target_os = "windows")]
    set_window_icon(
        &mut win,
        &load_image_from_data!("../themes/minecraft_icon.png").unwrap(),
    );
    
    win.show();
    
    #[cfg(target_os = "windows")]
    adjust_window(&win);
}