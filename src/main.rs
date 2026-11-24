#![windows_subsystem = "windows"]

mod characters;
mod gameplay;
mod inventory;
mod main_menu;
mod ui;

use std::ptr::null_mut;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};

fn main() {
    if let Err(error) = main_menu::run() {
        let message: Vec<u16> = format!("Błąd uruchamiania: {error}")
            .encode_utf16()
            .chain([0])
            .collect();
        unsafe {
            MessageBoxW(
                null_mut(),
                message.as_ptr(),
                main_menu::WINDOW_TITLE,
                MB_OK | MB_ICONERROR,
            )
        };
        std::process::exit(1);
    }
}
