use std::io;
use windows_sys::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, RECT},
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
    },
    w,
};

pub(crate) fn font(size: i32, weight: u32) -> HFONT {
    typeface(size, weight, w!("Segoe UI"))
}

pub(crate) fn typeface(size: i32, weight: u32, name: *const u16) -> HFONT {
    unsafe {
        CreateFontW(
            -size,
            0,
            0,
            0,
            weight as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            DEFAULT_PITCH as u32,
            name,
        )
    }
}

pub(crate) fn control(
    window: HWND,
    instance: HINSTANCE,
    id: u16,
    class: *const u16,
    label: *const u16,
    style: u32,
) -> io::Result<HWND> {
    let child = unsafe {
        CreateWindowExW(
            0,
            class,
            label,
            WS_CHILD | style,
            0,
            0,
            0,
            0,
            window,
            id as usize as HMENU,
            instance,
            std::ptr::null(),
        )
    };
    if child.is_null() {
        return Err(io::Error::last_os_error());
    }
    Ok(child)
}

pub(crate) fn child(window: HWND, id: u16) -> HWND {
    unsafe { GetDlgItem(window, id as i32) }
}

pub(crate) fn place(window: HWND, id: u16, x: i32, y: i32, width: i32, height: i32) {
    let control = child(window, id);
    if !control.is_null() {
        unsafe { MoveWindow(control, x, y, width, height, 1) };
    }
}

pub(crate) fn fill(dc: HDC, rect: &RECT, color: COLORREF) {
    let brush = unsafe { CreateSolidBrush(color) };
    if !brush.is_null() {
        unsafe {
            FillRect(dc, rect, brush);
            DeleteObject(brush);
        }
    }
}
