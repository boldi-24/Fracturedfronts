use windows_sys::Win32::Foundation::COLORREF;

pub(crate) const BACKGROUND: COLORREF = rgb(13, 10, 25);
pub(crate) const HEADER: COLORREF = rgb(24, 17, 45);
pub(crate) const SURFACE: COLORREF = rgb(55, 33, 84);
pub(crate) const BORDER: COLORREF = rgb(100, 75, 131);
pub(crate) const ACCENT: COLORREF = rgb(216, 169, 255);
pub(crate) const TEXT: COLORREF = rgb(242, 234, 255);
pub(crate) const MUTED: COLORREF = rgb(188, 173, 214);
pub(crate) const SELECTED_FILL: COLORREF = rgb(54, 27, 90);

pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    red as u32 | (green as u32) << 8 | (blue as u32) << 16
}
