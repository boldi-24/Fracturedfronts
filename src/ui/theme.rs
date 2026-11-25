use windows_sys::Win32::Foundation::COLORREF;

pub(crate) const BACKGROUND: COLORREF = rgb(26, 30, 29);
pub(crate) const HEADER: COLORREF = rgb(38, 44, 42);
pub(crate) const SURFACE: COLORREF = rgb(67, 72, 65);
pub(crate) const BORDER: COLORREF = rgb(111, 112, 96);
pub(crate) const ACCENT: COLORREF = rgb(235, 190, 98);
pub(crate) const TEXT: COLORREF = rgb(237, 233, 216);
pub(crate) const MUTED: COLORREF = rgb(186, 188, 173);
pub(crate) const SELECTED_FILL: COLORREF = rgb(82, 79, 56);

pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    red as u32 | (green as u32) << 8 | (blue as u32) << 16
}
