use std::ptr::null;
use windows_sys::Win32::{
    Foundation::{COLORREF, HINSTANCE, HWND, RECT},
    Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DwmSetWindowAttribute,
    },
    Graphics::Gdi::{
        BITMAPINFO, BITMAPINFOHEADER, COLORONCOLOR, CreateEllipticRgn, DIB_RGB_COLORS,
        DeleteObject, ExtSelectClipRgn, HALFTONE, HDC, RGN_AND, RestoreDC, SRCCOPY, SaveDC,
        SetStretchBltMode, StretchDIBits,
    },
    UI::WindowsAndMessaging::{HICON, LoadIconW},
};

pub(crate) const KEY_ART: &[u8] = include_bytes!("../../assets/key-art.bmp");
pub(crate) const CITADEL: &[u8] = include_bytes!("../../assets/citadel.bmp");
pub(crate) const BRIDGE: &[u8] = include_bytes!("../../assets/eclipse-bridge.bmp");
const ROSTER: &[u8] = include_bytes!("../../assets/roster.bmp");
pub(crate) const CHAMPIONS: &[u8] = include_bytes!("../../assets/champions.bmp");

pub(crate) fn icon(instance: HINSTANCE) -> HICON {
    // This ID matches the RT_GROUP_ICON in assets/fenrae.rc.
    unsafe { LoadIconW(instance, 1_usize as *const u16) }
}

pub(crate) fn box_at(x: i32, y: i32, w: i32, h: i32) -> RECT {
    RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}

fn number(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(
        bytes[at..at + 4]
            .try_into()
            .expect("embedded bitmap header"),
    )
}

fn dimensions(bitmap: &[u8]) -> (i32, i32) {
    (number(bitmap, 18) as i32, number(bitmap, 22) as i32)
}

pub(crate) fn frame(dc: HDC, area: RECT, line: COLORREF, interior: COLORREF) {
    super::controls::fill(dc, &area, line);
    let inner = box_at(
        area.left + 1,
        area.top + 1,
        area.right - area.left - 2,
        area.bottom - area.top - 2,
    );
    if inner.right > inner.left && inner.bottom > inner.top {
        super::controls::fill(dc, &inner, interior);
    }
}

pub(crate) fn draw(dc: HDC, bitmap: &[u8], target: RECT) {
    let (width, height) = dimensions(bitmap);
    paint(dc, bitmap, target, box_at(0, 0, width, height), true);
}

pub(crate) fn draw_fast(dc: HDC, bitmap: &[u8], target: RECT) {
    let (width, height) = dimensions(bitmap);
    paint(dc, bitmap, target, box_at(0, 0, width, height), false);
}

pub(crate) fn contain(bitmap: &[u8], target: RECT) -> RECT {
    let (width, height) = dimensions(bitmap);
    let w = (target.right - target.left).max(0);
    let h = (target.bottom - target.top).max(0);
    let scale = (w as f32 / width as f32).min(h as f32 / height as f32);
    let scaled_w = (width as f32 * scale).round() as i32;
    let scaled_h = (height as f32 * scale).round() as i32;
    box_at(
        target.left + (w - scaled_w) / 2,
        target.top + (h - scaled_h) / 2,
        scaled_w,
        scaled_h,
    )
}

pub(crate) fn hero_portrait(
    dc: HDC,
    target: RECT,
    hero: crate::characters::roster::Hero,
    face: bool,
) {
    use crate::characters::roster::Hero;
    let Some(index) = hero.portrait_index() else {
        old_portrait(dc, target, hero == Hero::Nera, face);
        return;
    };
    roster_portrait(dc, target, index as i32, face);
}

fn roster_portrait(dc: HDC, target: RECT, index: i32, face: bool) {
    let width = (target.right - target.left).max(1);
    let height = (target.bottom - target.top).max(1);
    let source = if face {
        box_at(176 * index + 32, 22, 112, 112)
    } else {
        let h = (176 * height / width).clamp(1, 220);
        let w = (220 * width / height).clamp(1, 176);
        box_at(176 * index + (176 - w) / 2, (220 - h) / 4, w, h)
    };
    paint_portrait(dc, ROSTER, target, source, face);
}

fn old_portrait(dc: HDC, target: RECT, ranged: bool, face: bool) {
    let (width, height) = dimensions(CHAMPIONS);
    let half_width = width / 2;
    let left = if ranged { half_width } else { 0 };
    let source = if face {
        box_at(
            left + half_width / 4,
            height / 7,
            half_width / 2,
            height / 2,
        )
    } else {
        let target_w = (target.right - target.left).max(1);
        let target_h = (target.bottom - target.top).max(1);
        let h = (half_width * target_h / target_w).clamp(1, height);
        let w = (height * target_w / target_h).clamp(1, half_width);
        box_at(left + (half_width - w) / 2, (height - h) / 3, w, h)
    };
    paint_portrait(dc, CHAMPIONS, target, source, face);
}

fn paint_portrait(dc: HDC, bitmap: &[u8], target: RECT, source: RECT, face: bool) {
    if face {
        let saved = unsafe { SaveDC(dc) };
        if saved != 0 {
            let mask =
                unsafe { CreateEllipticRgn(target.left, target.top, target.right, target.bottom) };
            if !mask.is_null() {
                unsafe {
                    ExtSelectClipRgn(dc, mask, RGN_AND);
                    DeleteObject(mask);
                }
            }
            paint(dc, bitmap, target, source, true);
            unsafe { RestoreDC(dc, saved) };
            return;
        }
    }
    paint(dc, bitmap, target, source, true);
}

fn paint(dc: HDC, bitmap: &[u8], target: RECT, source: RECT, smooth: bool) {
    if target.right <= target.left || target.bottom <= target.top {
        return;
    }
    let (width, height) = dimensions(bitmap);
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: height,
            biPlanes: 1,
            biBitCount: 24,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    };
    let offset = number(bitmap, 10) as usize;
    let pixels = bitmap.get(offset..).map_or(null(), |bytes| bytes.as_ptr());
    if pixels.is_null() {
        return;
    }
    unsafe {
        let previous = SetStretchBltMode(dc, if smooth { HALFTONE } else { COLORONCOLOR });
        StretchDIBits(
            dc,
            target.left,
            target.top,
            target.right - target.left,
            target.bottom - target.top,
            source.left,
            height - source.bottom,
            source.right - source.left,
            source.bottom - source.top,
            pixels.cast(),
            &info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        if previous != 0 {
            SetStretchBltMode(dc, previous);
        }
    }
}

pub(crate) fn dress(window: HWND) {
    let enabled: i32 = 1;
    let caption = super::theme::HEADER;
    let border = super::theme::BORDER;
    let text = super::theme::TEXT;
    unsafe {
        DwmSetWindowAttribute(
            window,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            (&enabled as *const i32).cast(),
            4,
        );
        DwmSetWindowAttribute(
            window,
            DWMWA_CAPTION_COLOR as u32,
            (&caption as *const COLORREF).cast(),
            4,
        );
        DwmSetWindowAttribute(
            window,
            DWMWA_BORDER_COLOR as u32,
            (&border as *const COLORREF).cast(),
            4,
        );
        DwmSetWindowAttribute(
            window,
            DWMWA_TEXT_COLOR as u32,
            (&text as *const COLORREF).cast(),
            4,
        );
    }
}

pub(crate) fn ability_icon(dc: HDC, area: RECT, hero: crate::characters::roster::Hero, column: usize, cooling: bool) {
    const ICONS: &[u8] = include_bytes!("../../assets/ability-icons.bmp");
    let row = hero as i32 + if cooling { crate::characters::roster::Hero::ALL.len() as i32 } else { 0 };
    paint(dc, ICONS, area, box_at(column as i32 * 64, row * 64, 64, 64), true);
}
