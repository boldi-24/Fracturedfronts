use super::{hud_slots, MatchWindow, START, board, button_hero, rect, viewport};
use crate::{
    gameplay::simulation::{Game, PLAYER},
    ui::{
        art,
        controls::fill,
        theme::{ACCENT, BACKGROUND, BORDER, HEADER, TEXT, rgb},
    },
};
use std::ptr::null_mut;
use windows_sys::Win32::{
    Foundation::{COLORREF, HWND, RECT},
    Graphics::Gdi::*,
    UI::{
        Controls::{DRAWITEMSTRUCT, ODS_FOCUS},
        WindowsAndMessaging::*,
    },
};

fn label(dc: HDC, font: HFONT, value: &str, mut area: RECT, color: COLORREF, flags: u32) {
    let words: Vec<u16> = value.encode_utf16().collect();
    text(dc, font, &words, &mut area, color, flags);
}

fn text(dc: HDC, font: HFONT, words: &[u16], area: &mut RECT, color: COLORREF, flags: u32) {
    if words.is_empty() {
        return;
    }
    unsafe {
        let previous_font = if font.is_null() {
            null_mut()
        } else {
            SelectObject(dc, font)
        };
        let previous_color = SetTextColor(dc, color);
        let previous_mode = SetBkMode(dc, TRANSPARENT as i32);
        DrawTextW(
            dc,
            words.as_ptr(),
            words.len() as i32,
            area,
            flags | DT_NOPREFIX,
        );
        SetBkMode(dc, previous_mode);
        SetTextColor(dc, previous_color);
        if !previous_font.is_null() {
            SelectObject(dc, previous_font);
        }
    }
}

pub(super) fn paint_button(state: &MatchWindow, item: &DRAWITEMSTRUCT) {
    let id = item.CtlID as u16;
    let hero = button_hero(id);
    let selected = hero == Some(state.hero.get());
    let highlighted = selected || id == START;
    let focused = item.itemState & ODS_FOCUS != 0;
    let edge = if focused || highlighted {
        ACCENT
    } else {
        BORDER
    };
    fill(item.hDC, &item.rcItem, edge);
    let mut inner = item.rcItem;
    let padding = if focused { 2 } else { 1 };
    unsafe { InflateRect(&mut inner, -padding, -padding) };
    fill(
        item.hDC,
        &inner,
        if highlighted {
            rgb(71, 37, 112)
        } else {
            HEADER
        },
    );

    if let Some(hero) = hero {
        unsafe { InflateRect(&mut inner, -3, -3) };
        art::hero_portrait(item.hDC, inner, hero, true);
        return;
    }
    let mut title = [0; 128];
    let length = unsafe { GetWindowTextW(item.hwndItem, title.as_mut_ptr(), 128) }.max(0) as usize;
    text(
        item.hDC,
        state.normal,
        &title[..length],
        &mut inner,
        if highlighted { ACCENT } else { TEXT },
        DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS,
    );
}

fn draw_selection(dc: HDC, state: &MatchWindow, area: RECT) {
    let width = area.right;
    fill(dc, &rect(0, 0, width, 88), HEADER);
    fill(dc, &rect(0, 87, width, 1), BORDER);
    label(
        dc,
        state.heading,
        "Wybierz postać",
        rect(22, 13, width - 310, 43),
        TEXT,
        DT_SINGLELINE,
    );
}

fn draw_arena(dc: HDC, state: &MatchWindow, area: RECT, game: &Game) {
    let arena = board(area);
    super::scene::draw(
        dc,
        super::scene::Camera::at(arena, state.camera.get(), state.top_down.get()),
        game, state.show_range.get(),
    );
    let width = area.right;
    if game.result == 0 {
        super::minimap::draw(dc, area, game);
        let hero = state.hero.get();
        for (_, slot, column, outer) in hud_slots(area, hero) {
            let seconds = game.units[PLAYER].cooldown(slot).ceil() as u32;
            fill(dc, &outer, if seconds == 0 { ACCENT } else { BORDER });
            let mut icon = outer;
            unsafe { InflateRect(&mut icon, -2, -2) };
            art::ability_icon(dc, icon, hero, column, seconds > 0);
            if seconds > 0 {
                label(dc, state.heading, &seconds.to_string(), icon, rgb(255,255,255),
                      DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            }
        }
    } else {
        let center = (arena.top + arena.bottom) / 2;
        let left = (width - 294) / 2;
        art::frame(
            dc,
            rect(
                left,
                center - 33,
                294,
                if game.result != 0 { 109 } else { 68 },
            ),
            ACCENT,
            BACKGROUND,
        );
        let status = match game.result {
            1 => "Zwycięstwo".to_owned(),
            2 => "Porażka".to_owned(),
            3 => "Remis".to_owned(),
            _ => String::new(),
        };
        label(
            dc,
            state.heading,
            &status,
            rect(left, center - 28, 294, 46),
            TEXT,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
    }
}

pub(super) fn draw(window: HWND, state: &MatchWindow) {
    let mut paint = PAINTSTRUCT::default();
    let screen = unsafe { BeginPaint(window, &mut paint) };
    let area = viewport(window);
    if !screen.is_null() && area.right > 0 && area.bottom > 0 {
        let buffer = unsafe { CreateCompatibleDC(screen) };
        let bitmap = if buffer.is_null() {
            null_mut()
        } else {
            unsafe { CreateCompatibleBitmap(screen, area.right, area.bottom) }
        };
        let memory = if bitmap.is_null() { screen } else { buffer };
        let previous = if bitmap.is_null() {
            null_mut()
        } else {
            unsafe { SelectObject(buffer, bitmap) }
        };
        fill(memory, &area, BACKGROUND);
        if let Some(round) = state.round.borrow().as_ref() {
            draw_arena(memory, state, area, &round.game);
        } else {
            draw_selection(memory, state, area);
        }
        if !bitmap.is_null() {
            unsafe {
                BitBlt(screen, 0, 0, area.right, area.bottom, buffer, 0, 0, SRCCOPY);
                SelectObject(buffer, previous);
                DeleteObject(bitmap);
            }
        }
        if !buffer.is_null() {
            unsafe { DeleteDC(buffer) };
        }
    }
    unsafe { EndPaint(window, &paint) };
}
