use fltk::{
    enums::Font,
    prelude::{GroupExt, WidgetBase, WidgetExt, WindowExt},
    window::Window,
};

#[cfg(target_os = "windows")]
use crate::windows::adjust_window;

use crate::gui::{
    WIN_HEIGHT, WIN_WIDTH, handle_drag, setup_frame, setup_tiled_background, setup_title_bar,
};

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

    win.set_icon(Some(
        load_image_from_data!("../themes/minecraft_icon.png").unwrap(),
    ));

    win.show();

    #[cfg(target_os = "windows")]
    adjust_window(&win);
}