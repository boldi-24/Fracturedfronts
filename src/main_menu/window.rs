use super::navigation::{BOT_DUEL, HOME, INVENTORY, Menu, PLAY, PROFILE, SHOP};
use crate::gameplay::window as match_window;
use crate::ui::{
    art,
    controls::{child, control, fill, font, place, typeface},
    theme::{ACCENT, BACKGROUND, BORDER, HEADER, MUTED, SELECTED_FILL, SURFACE, TEXT},
};
use std::{io, ptr::null_mut};
use windows_sys::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{DRAWITEMSTRUCT, ODS_FOCUS, ODS_SELECTED},
            Input::KeyboardAndMouse::{GetFocus, SetFocus, VK_ESCAPE, VK_RETURN},
            WindowsAndMessaging::*,
        },
    },
    w,
};

pub(crate) const WINDOW_TITLE: *const u16 = w!("Fenrae Eclipse");
const WINDOW_CLASS: *const u16 = w!("MobaPrototypeWindow");

const TITLE: u16 = 106;
const SUBTITLE: u16 = 107;
const DETAILS: u16 = 108;
const LAUNCH: u16 = 109;
const INVENTORY_LIST: u16 = 110;

const PADDING: i32 = 22;
const GAP: i32 = 8;
const HEADER_HEIGHT: i32 = 72;
const BUTTON_HEIGHT: i32 = 38;
const NAVIGATION: [(u16, *const u16, i32); 5] = [
    (HOME, w!("Menu główne"), 146),
    (PLAY, w!("Graj"), 100),
    (INVENTORY, w!("Ekwipunek"), 124),
    (SHOP, w!("Sklep"), 88),
    (PROFILE, w!("Profil"), 96),
];

struct State {
    menu: Menu,
    match_window: match_window::MatchWindow,
    background: HBRUSH,
    body_font: HFONT,
    heading_font: HFONT,
    button_font: HFONT,
}

impl State {
    fn new() -> Self {
        Self {
            menu: Menu::new(),
            match_window: match_window::MatchWindow::new(),
            // These handles are owned by the window state and released after the message loop.
            background: unsafe { CreateSolidBrush(BACKGROUND) },
            body_font: font(17, FW_NORMAL),
            heading_font: typeface(38, FW_SEMIBOLD, w!("Georgia")),
            button_font: font(17, FW_SEMIBOLD),
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        for object in [
            self.background,
            self.body_font,
            self.heading_font,
            self.button_font,
        ] {
            if !object.is_null() {
                unsafe { DeleteObject(object) };
            }
        }
    }
}

fn add_controls(window: HWND, instance: HINSTANCE, state: &State) -> io::Result<()> {
    for (id, label, _) in NAVIGATION {
        control(
            window,
            instance,
            id,
            w!("BUTTON"),
            label,
            WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
        )?;
    }
    control(
        window,
        instance,
        BOT_DUEL,
        w!("BUTTON"),
        w!("Pojedynek 1 na 1 z botem"),
        WS_TABSTOP | BS_OWNERDRAW as u32,
    )?;
    control(
        window,
        instance,
        LAUNCH,
        w!("BUTTON"),
        w!("Rozpocznij mecz"),
        WS_TABSTOP | BS_OWNERDRAW as u32,
    )?;
    let list = control(
        window,
        instance,
        INVENTORY_LIST,
        w!("LISTBOX"),
        w!(""),
        WS_TABSTOP | WS_VSCROLL | LBS_NOINTEGRALHEIGHT as u32,
    )?;
    unsafe { SendMessageW(list, WM_SETFONT, state.body_font as WPARAM, 0) };
    for hero in state.menu.inventory.champions() {
        let name: Vec<u16> = hero.name().encode_utf16().chain([0]).collect();
        unsafe {
            SendMessageW(list, LB_ADDSTRING, 0, name.as_ptr() as LPARAM);
        }
    }
    for (id, font) in [
        (TITLE, state.heading_font),
        (SUBTITLE, state.body_font),
        (DETAILS, state.body_font),
    ] {
        let child = control(window, instance, id, w!("STATIC"), w!(""), 0)?;
        if !font.is_null() {
            unsafe { SendMessageW(child, WM_SETFONT, font as WPARAM, 0) };
        }
    }
    Ok(())
}

fn layout(window: HWND) {
    let mut area = RECT::default();
    unsafe { GetClientRect(window, &mut area) };
    let right_width = NAVIGATION[2..].iter().map(|entry| entry.2).sum::<i32>() + 2 * GAP;
    let mut x = PADDING;

    for (index, (id, _, width)) in NAVIGATION.into_iter().enumerate() {
        if index == 2 {
            x = x.max(area.right - PADDING - right_width);
        }
        place(
            window,
            id,
            x,
            (HEADER_HEIGHT - BUTTON_HEIGHT) / 2,
            width,
            BUTTON_HEIGHT,
        );
        x += width + GAP;
    }

    let content_width = (area.right / 2 - 64).max(280);
    place(window, TITLE, 32, 120, content_width, 48);
    place(window, BOT_DUEL, 32, 190, 290, 46);
    place(window, LAUNCH, 32, 253, 240, 44);
    place(window, SUBTITLE, 32, 177, content_width, 28);
    let details_y = 217;
    place(window, DETAILS, 32, details_y, content_width, 80);
    place(
        window,
        INVENTORY_LIST,
        32,
        details_y,
        content_width,
        (area.bottom - details_y - 22).max(35),
    );
}

fn set_label(window: HWND, id: u16, text: &str) {
    let text: Vec<u16> = text.encode_utf16().chain([0]).collect();
    let label = child(window, id);
    unsafe {
        SetWindowTextW(label, text.as_ptr());
        ShowWindow(label, if text.len() > 1 { SW_SHOWNA } else { SW_HIDE });
    }
}

fn refresh_view(window: HWND, state: &State) {
    let screen = state.menu.screen();
    unsafe {
        ShowWindow(
            child(window, BOT_DUEL),
            if screen == PLAY { SW_SHOWNA } else { SW_HIDE },
        )
    };
    unsafe {
        ShowWindow(
            child(window, INVENTORY_LIST),
            if screen == INVENTORY {
                SW_SHOWNA
            } else {
                SW_HIDE
            },
        );
    }
    let can_launch = screen == PLAY && state.menu.selected_duel();
    unsafe {
        ShowWindow(
            child(window, LAUNCH),
            if can_launch { SW_SHOWNA } else { SW_HIDE },
        )
    };
    let (title, subtitle, details) = match screen {
        PLAY => ("Wybierz tryb gry", "", String::new()),
        INVENTORY => ("Ekwipunek", "Odblokowane postacie", String::new()),
        SHOP => ("Sklep", "", String::new()),
        PROFILE => ("Profil", "", String::new()),
        _ => ("", "", String::new()),
    };
    set_label(window, TITLE, title);
    set_label(window, SUBTITLE, subtitle);
    set_label(window, DETAILS, &details);
    layout(window);
    for id in [HOME, PLAY, INVENTORY, SHOP, PROFILE, BOT_DUEL, LAUNCH] {
        unsafe { InvalidateRect(child(window, id), std::ptr::null(), 0) };
    }
    unsafe { InvalidateRect(window, std::ptr::null(), 0) };
}

fn paint_window(window: HWND, state: &State) {
    let mut paint = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(window, &mut paint) };
    if !dc.is_null() {
        let mut area = RECT::default();
        unsafe { GetClientRect(window, &mut area) };
        unsafe { FillRect(dc, &area, state.background) };
        let header = RECT {
            bottom: HEADER_HEIGHT,
            ..area
        };
        fill(dc, &header, HEADER);
        fill(
            dc,
            &RECT {
                top: HEADER_HEIGHT - 1,
                ..header
            },
            BORDER,
        );
        let available = area.bottom - HEADER_HEIGHT;

        if available > 50 {
            if state.menu.screen() == HOME {
                let slot = art::box_at(30, HEADER_HEIGHT + 18, area.right - 60, available - 36);
                let image = art::contain(art::KEY_ART, slot);
                art::frame(
                    dc,
                    art::box_at(
                        image.left - 3,
                        image.top - 3,
                        image.right - image.left + 6,
                        image.bottom - image.top + 6,
                    ),
                    BORDER,
                    HEADER,
                );
                art::draw(dc, art::KEY_ART, image);
            } else {
                let left = area.right / 2;
                let slot = art::box_at(
                    left,
                    HEADER_HEIGHT + 24,
                    area.right - left - 28,
                    available - 48,
                );
                let image = art::contain(art::CITADEL, slot);
                art::frame(
                    dc,
                    art::box_at(
                        image.left - 3,
                        image.top - 3,
                        image.right - image.left + 6,
                        image.bottom - image.top + 6,
                    ),
                    BORDER,
                    HEADER,
                );
                art::draw(dc, art::CITADEL, image);
                fill(dc, &art::box_at(32, 105, 72, 2), ACCENT);
            }
        }
    }
    unsafe { EndPaint(window, &paint) };
}

fn draw_button(item: &DRAWITEMSTRUCT, state: &State) {
    let id = item.CtlID as u16;
    let selected = if id == BOT_DUEL {
        state.menu.selected_duel()
    } else {
        state.menu.screen() == id
    };
    let pressed = item.itemState & ODS_SELECTED != 0;
    let focus = item.itemState & ODS_FOCUS != 0;
    let highlighted = selected || id == PLAY || id == LAUNCH;
    let border = if highlighted || focus { ACCENT } else { BORDER };
    let color = if selected || pressed {
        SELECTED_FILL
    } else if id == PLAY || id == LAUNCH {
        SURFACE
    } else {
        HEADER
    };
    fill(item.hDC, &item.rcItem, border);

    let inset = if focus { 2 } else { 1 };
    let mut inner = item.rcItem;
    unsafe { InflateRect(&mut inner, -inset, -inset) };
    fill(item.hDC, &inner, color);

    let mut text = [0u16; 128];
    unsafe {
        let length = GetWindowTextW(item.hwndItem, text.as_mut_ptr(), text.len() as i32);
        let previous_font = if state.button_font.is_null() {
            null_mut()
        } else {
            SelectObject(item.hDC, state.button_font)
        };
        let previous_mode = SetBkMode(item.hDC, TRANSPARENT as i32);
        let previous_color = SetTextColor(item.hDC, if highlighted { ACCENT } else { TEXT });
        DrawTextW(
            item.hDC,
            text.as_ptr(),
            length,
            &mut inner,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
        SetTextColor(item.hDC, previous_color);
        SetBkMode(item.hDC, previous_mode);
        if !previous_font.is_null() {
            SelectObject(item.hDC, previous_font);
        }
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // The pointed-to State lives until the window is destroyed and the loop exits.
        let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize) };
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    }
    let pointer = unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) } as *const State;
    let Some(state) = (unsafe { pointer.as_ref() }) else {
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    };

    match message {
        WM_GETMINMAXINFO => {
            let info = unsafe { &mut *(lparam as *mut MINMAXINFO) };
            info.ptMinTrackSize.x = 665;
            info.ptMinTrackSize.y = 660;
        }
        WM_ACTIVATEAPP if wparam == 0 => {
            state.match_window.release_cursor();
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        match_window::MATCH_CLOSED => {
            if state.match_window.handle.get().is_null() {
                for (id, _, _) in NAVIGATION { unsafe { ShowWindow(child(window, id), SW_SHOWNA) }; }
                refresh_view(window, state);
                unsafe { SetFocus(child(window, LAUNCH)) };
            }
        }
        WM_SIZE => {
            let view = state.match_window.handle.get();
            if !view.is_null() {
                let mut area = RECT::default();
                unsafe { GetClientRect(window, &mut area); MoveWindow(view, 0, 0, area.right, area.bottom, 1); }
                return 0;
            }
            layout(window);
            unsafe { InvalidateRect(window, std::ptr::null(), 0) };
        }
        WM_ERASEBKGND => return 1,
        WM_PAINT => paint_window(window, state),
        WM_CTLCOLORLISTBOX => unsafe {
            let dc = wparam as HDC;
            SetTextColor(dc, TEXT);
            SetBkColor(dc, BACKGROUND);
            return state.background as LRESULT;
        },
        WM_CTLCOLORSTATIC => unsafe {
            let dc = wparam as HDC;
            let color = match GetDlgCtrlID(lparam as HWND) as u16 {
                TITLE => TEXT,
                SUBTITLE => ACCENT,
                _ => MUTED,
            };
            SetTextColor(dc, color);
            SetBkColor(dc, BACKGROUND);
            return state.background as LRESULT;
        },
        WM_DRAWITEM => {
            draw_button(unsafe { &*(lparam as *const DRAWITEMSTRUCT) }, state);
            return 1;
        }
        WM_COMMAND if lparam != 0 && (wparam >> 16) as u32 == BN_CLICKED => {
            if wparam as u16 == LAUNCH && state.menu.screen() == PLAY && state.menu.selected_duel()
            {
                if let Err(error) = state.match_window.open(window) {
                    let message: Vec<u16> = format!("Nie udało się otworzyć meczu: {error}")
                        .encode_utf16()
                        .chain([0])
                        .collect();
                    unsafe {
                        MessageBoxW(window, message.as_ptr(), WINDOW_TITLE, MB_OK | MB_ICONERROR)
                    };
                } else {
                    for id in [HOME, PLAY, INVENTORY, SHOP, PROFILE, BOT_DUEL, LAUNCH, INVENTORY_LIST, TITLE, SUBTITLE, DETAILS] {
                        unsafe { ShowWindow(child(window, id), SW_HIDE) };
                    }
                }
                return 0;
            }
            if !state.menu.choose(wparam as u16) {
                return 0;
            }
            refresh_view(window, state);
        }
        WM_DESTROY => unsafe { PostQuitMessage(0) },
        WM_NCDESTROY => {
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, 0) };
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        _ => return unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
    0
}

pub(crate) fn run() -> io::Result<()> {
    let state = State::new();
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    match_window::register(instance)?;
    let class = WNDCLASSW {
        hInstance: instance,
        lpszClassName: WINDOW_CLASS,
        lpfnWndProc: Some(window_proc),
        hIcon: art::icon(instance),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        ..WNDCLASSW::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let window = unsafe {
        CreateWindowExW(
            0,
            WINDOW_CLASS,
            WINDOW_TITLE,
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1160,
            730,
            null_mut(),
            null_mut(),
            instance,
            &state as *const State as *const _,
        )
    };
    if window.is_null() {
        return Err(io::Error::last_os_error());
    }
    art::dress(window);
    if let Err(error) = add_controls(window, instance, &state) {
        unsafe { DestroyWindow(window) };
        return Err(error);
    }

    unsafe { ShowWindow(window, SW_SHOW) };
    refresh_view(window, &state);
    let mut message = MSG::default();
    loop {
        match unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } {
            -1 => {
                let error = io::Error::last_os_error();
                unsafe { DestroyWindow(window) };
                return Err(error);
            }
            0 => return Ok(()),
            _ => {}
        }
        if match_window::route(state.match_window.handle.get(), &message) {
            continue;
        }
        if message.message == WM_KEYDOWN && message.wParam == VK_ESCAPE as usize {
            state.menu.choose(HOME);
            unsafe { SetFocus(child(window, HOME)) };
            refresh_view(window, &state);
        } else if message.message == WM_KEYDOWN && message.wParam == VK_RETURN as usize {
            let focused = unsafe { GetFocus() };
            if matches!(
                unsafe { GetDlgCtrlID(focused) } as u16,
                HOME..=BOT_DUEL | LAUNCH
            ) {
                unsafe { SendMessageW(focused, BM_CLICK, 0, 0) };
            }
        } else if unsafe { IsDialogMessageW(window, &message) } == 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}
