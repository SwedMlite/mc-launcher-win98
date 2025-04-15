use winapi::shared::windef::HWND;
use fltk::prelude::WindowExt;
use winapi::um::winuser::{
    GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, SetActiveWindow, SetWindowLongPtrW, WS_CAPTION,
    WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
    WS_VISIBLE,
};

pub fn adjust_window(win: &fltk::window::Window) {
    let hwnd = win.raw_handle() as HWND;
    unsafe {
        let old_style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let new_style = (old_style
            & !(WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX))
            | WS_POPUP
            | WS_VISIBLE;
        SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize);

        let old_ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let toolwindow_mask = 0x80; 
        let new_ex = (old_ex & !toolwindow_mask) | WS_EX_APPWINDOW;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex as isize);

        SetActiveWindow(hwnd);
    }
}