mod minimap;
mod render;
mod scene;

use super::{
    geometry::Point,
    simulation::{Game, PLAYER},
};
use crate::{
    characters::roster::Hero,
    ui::{
        art,
        controls::{child, control, font, place, typeface},
    },
};
use render::{draw, paint_button};
use std::{
    cell::{Cell, RefCell},
    io,
    ptr::{null, null_mut},
    time::Instant,
};
use windows_sys::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::DRAWITEMSTRUCT,
            Input::KeyboardAndMouse::{GetFocus, SetFocus},
            WindowsAndMessaging::*,
        },
    },
    w,
};

pub(crate) const MATCH_CLOSED: u32 = WM_APP + 5;
const CLASS: *const u16 = w!("MobaMatchWindow");
const TITLE: *const u16 = w!("Fenrae Eclipse — rozgrywka");
const FIRST_HERO: u16 = 300;
const START: u16 = 202;
const CHANGE: u16 = 203;
const CLOSE: u16 = 204;
const Q: u16 = 205;
const E: u16 = 206;
const W: u16 = 207;
const R: u16 = 208;

const ABILITIES: [(u16, u32, &str); 4] = [(Q, 0, "Q"), (W, 2, "W"), (E, 1, "E"), (R, 3, "R")];
const TIMER: usize = 1;
const FIXED_STEP: f32 = 1.0 / 60.0;
const CAMERA_SPEED: f32 = 700.;

struct Round {
    game: Game,
    last_tick: Instant,
    remaining_time: f32,
}

fn button_hero(id: u16) -> Option<Hero> {
    Hero::ALL
        .get(usize::from(id.checked_sub(FIRST_HERO)?))
        .copied()
}

pub(crate) struct MatchWindow {
    pub(crate) handle: Cell<HWND>,
    hero: Cell<Hero>,
    camera: Cell<Point>,
    aim: Cell<Point>,
    round: RefCell<Option<Round>>,
    normal: HFONT,
    heading: HFONT,
    follow: Cell<bool>,
    top_down: Cell<bool>,
    show_range: Cell<bool>,
    cursor_bounds: Cell<Option<[i32; 4]>>,
}

impl MatchWindow {
    pub(crate) fn new() -> Self {
        Self {
            handle: Cell::new(null_mut()),
            hero: Cell::new(Hero::Taran),
            camera: Cell::new(Point::default()),
            aim: Cell::new(Point::at(550., 280.)),
            round: RefCell::new(None),
            normal: font(16, FW_NORMAL),
            heading: typeface(32, FW_SEMIBOLD, w!("Georgia")),
            follow: Cell::new(false),
            top_down: Cell::new(false),
            show_range: Cell::new(false),
            cursor_bounds: Cell::new(None),
        }
    }

    pub(crate) fn release_cursor(&self) {
        if self.cursor_bounds.take().is_some() {
            unsafe { ClipCursor(null()) };
        }
    }

    fn confine_cursor(&self, view: HWND) {
        let root = unsafe { GetAncestor(view, GA_ROOT) };
        let area = viewport(view);
        if area.right <= 0 || area.bottom <= 0 || unsafe {
            GetForegroundWindow() != root || IsIconic(root) != 0 || IsWindowVisible(view) == 0
        } {
            self.release_cursor();
            return;
        }
        let mut origin = POINT::default();
        if unsafe { ClientToScreen(view, &mut origin) } == 0 {
            self.release_cursor();
            return;
        }
        let bounds = [origin.x, origin.y, origin.x + area.right, origin.y + area.bottom];
        if self.cursor_bounds.get() != Some(bounds) {
            let clip = RECT { left: bounds[0], top: bounds[1], right: bounds[2], bottom: bounds[3] };
            if unsafe { ClipCursor(&clip) } != 0 { self.cursor_bounds.set(Some(bounds)); }
        }
    }

    pub(crate) fn open(&self, owner: HWND) -> io::Result<()> {
        let active = self.handle.get();
        if !active.is_null() {
            unsafe { SetFocus(active) };
            return Ok(());
        }
        self.round.replace(None);
        self.hero.set(Hero::Taran);
        let instance = unsafe { GetModuleHandleW(null()) };
        let area = viewport(owner);
        let window = unsafe {
            CreateWindowExW(
                0,
                CLASS,
                TITLE,
                WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                area.right,
                area.bottom,
                owner,
                null_mut(),
                instance,
                self as *const Self as *const _,
            )
        };
        if window.is_null() {
            return Err(io::Error::last_os_error());
        }

        for (id, label) in [
            (START, "Rozpocznij pojedynek"),
            (CHANGE, "Zmień postać"),
            (CLOSE, "Wróć do menu"),

        ]
        .into_iter()
        .chain(
            Hero::ALL
                .into_iter()
                .enumerate()
                .map(|(index, hero)| (FIRST_HERO + index as u16, hero.name())),
        ) {
            let label: Vec<u16> = label.encode_utf16().chain([0]).collect();
            if let Err(error) = control(
                window,
                instance,
                id,
                w!("BUTTON"),
                label.as_ptr(),
                WS_TABSTOP | BS_OWNERDRAW as u32,
            ) {
                unsafe { DestroyWindow(window) };
                return Err(error);
            }
        }
        sync_controls(self, window);
        unsafe {
            SetWindowPos(window, HWND_TOP, 0, 0, area.right, area.bottom, SWP_SHOWWINDOW);
            SetFocus(child(window, FIRST_HERO));
        }
        Ok(())
    }
}

impl Drop for MatchWindow {
    fn drop(&mut self) {
        for handle in [self.normal, self.heading] {
            if !handle.is_null() {
                unsafe { DeleteObject(handle) };
            }
        }
    }
}

pub(crate) fn register(instance: HINSTANCE) -> io::Result<()> {
    let class = WNDCLASSW {
        hInstance: instance,
        lpszClassName: CLASS,
        lpfnWndProc: Some(proc),
        hIcon: art::icon(instance),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        ..WNDCLASSW::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn board(area: RECT) -> RECT {
    area
}

fn rect(left: i32, top: i32, width: i32, height: i32) -> RECT {
    RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

fn viewport(window: HWND) -> RECT {
    let mut area = RECT::default();
    unsafe { GetClientRect(window, &mut area) };
    area
}

const ROLE_ROWS: &[(&str, &[Hero])] = &[
    ("Wojownicy", &[Hero::Aelor, Hero::Solviana, Hero::Taran]),
    ("Strzelcy", &[Hero::Arxen, Hero::Aurelia, Hero::Nera]),
    ("Obrońcy", &[Hero::Helvara, Hero::Veyrion]),
    ("Magowie", &[Hero::Elorim, Hero::Nyxalune]),
    ("Wsparcie", &[Hero::Luness, Hero::Selune]),
    ("Zabójcy", &[Hero::Kaelith, Hero::Zyraen]),
];

fn icon_rect(row: usize, column: usize) -> RECT {
    rect(178 + column as i32 * 70, 98 + row as i32 * 72, 60, 60)
}

fn control_layout(window: HWND, state: &MatchWindow) {
    let area = viewport(window);
    let width = area.right;
    let round = state.round.borrow();
    let active = round.is_some();
    let outcome = round.as_ref().map_or(0, |round| round.game.result);
    drop(round);

    for (row, (_, heroes)) in ROLE_ROWS.iter().enumerate() {
        for (column, &hero) in heroes.iter().enumerate() {
            let id = FIRST_HERO + hero as u16;
            let icon = icon_rect(row, column);
            place(window, id, icon.left, icon.top, icon.right - icon.left, icon.bottom - icon.top);
            let handle = child(window, id);
            if !handle.is_null() {
                unsafe { ShowWindow(handle, if active { SW_HIDE } else { SW_SHOWNA }) };
            }
        }
    }
    let arena = board(area);
    let start_top = if active {
        (arena.top + arena.bottom) / 2 + 27
    } else {
        area.bottom - 55
    };
    place(
        window,
        START,
        if active { (width - 220) / 2 } else { 22 },
        start_top,
        220,
        40,
    );
    let bottom = area.bottom - 49;
    place(window, CHANGE, width - 292, bottom, 130, 34);
    place(
        window,
        CLOSE,
        width - 152,
        if active { bottom } else { area.bottom - 55 },
        130,
        34,
    );

    for (id, show) in [
        (START, !active || outcome != 0),
        (CHANGE, outcome != 0),
        (CLOSE, !active || outcome != 0),
    ] {
        let control = child(window, id);
        if !control.is_null() {
            unsafe { ShowWindow(control, if show { SW_SHOWNA } else { SW_HIDE }) };
        }
    }
}

fn assign_text(window: HWND, id: u16, label: *const u16) {
    let handle = child(window, id);
    if !handle.is_null() {
        unsafe { SetWindowTextW(handle, label) };
    }
}

fn sync_controls(state: &MatchWindow, window: HWND) {
    let outcome = state
        .round
        .borrow()
        .as_ref()
        .map_or(0, |round| round.game.result);
    assign_text(
        window,
        START,
        if outcome != 0 {
            w!("Zagraj ponownie")
        } else {
            w!("Rozpocznij pojedynek")
        },
    );
    control_layout(window, state);
    unsafe { InvalidateRect(window, null(), 0) };
}

fn world_under_cursor(
    state: &MatchWindow,
    arena: RECT,
    mouse: POINT,
    game: &Game,
    include_allies: bool,
) -> Option<Point> {
    if mouse.x < arena.left
        || mouse.x >= arena.right
        || mouse.y < arena.top
        || mouse.y >= arena.bottom
    {
        return None;
    }
    let camera = scene::Camera::at(arena, state.camera.get(), state.top_down.get());
    Some(
        camera
            .unit_at(game, mouse.x, mouse.y, include_allies)
            .map_or_else(
                || camera.ground(mouse.x, mouse.y),
                |index| game.units[index].position,
            ),
    )
}

fn edge_speed(pixel: i32, start: i32, end: i32) -> f32 {
    const BAND: i32 = 24;
    if pixel < start + BAND { -1. } else if pixel >= end - BAND { 1. } else { 0. }

}

// One layout for both painting and pointer input. Legacy heroes only have Q and E.
fn hud_slots(area: RECT, hero: Hero) -> impl Iterator<Item = (u16, u32, usize, RECT)> {
    let full = hero.profile().is_some();
    let count = if full { 4 } else { 2 };
    let size = (area.bottom / 13).clamp(38, 64);
    let gap = 6;
    let left = (area.right - count * size - (count - 1) * gap) / 2;
    ABILITIES.into_iter().enumerate().filter(move |(_, (_, slot, _))| full || *slot < 2)
        .enumerate().map(move |(i, (column, (id, slot, _)))|
            (id, slot, column, rect(left + i as i32 * (size + gap), area.bottom - size - 12, size, size)))
}

fn hud_hit(area: RECT, hero: Hero, mouse: POINT) -> Option<u16> {
    hud_slots(area, hero).find(|(_, _, _, r)| mouse.x >= r.left && mouse.x < r.right &&
        mouse.y >= r.top && mouse.y < r.bottom).map(|s| s.0)
}

fn update_camera(window: HWND, state: &MatchWindow, game: &Game, dt: f32) {
    use super::geometry::{WORLD_HEIGHT, WORLD_WIDTH};
    let mut mouse = POINT::default();
    if unsafe { GetForegroundWindow() } != unsafe { GetAncestor(window, GA_ROOT) }
        || unsafe { GetCursorPos(&mut mouse) } == 0
        || unsafe { ScreenToClient(window, &mut mouse) } == 0
    {
        return;
    }
    let client = viewport(window);
    if mouse.x < 0 || mouse.y < 0 || mouse.x >= client.right || mouse.y >= client.bottom {
        return;
    }

    let arena = board(client);
    if !state.follow.get() {
        let delta = Point::at(edge_speed(mouse.x, arena.left, arena.right), edge_speed(mouse.y, arena.top, arena.bottom));
        let camera = scene::Camera::at(arena, state.camera.get(), state.top_down.get());
        let motion = delta * (CAMERA_SPEED * dt / delta.length().max(1.));
        let center = state.camera.get() + camera.offset(motion.x, motion.y);
        state.camera.set(Point::at(center.x.clamp(0., WORLD_WIDTH), center.y.clamp(0., WORLD_HEIGHT)));
    }
    if hud_hit(client, state.hero.get(), mouse).is_some() || minimap::world_at(client, mouse).is_some() { return; }
    if let Some(point) = world_under_cursor(state, arena, mouse, game, true) {
        state.aim.set(point);
    }
}

fn start(window: HWND, state: &MatchWindow) {
    let game = Game::with_hero(state.hero.get());
    let spawn = game.units[PLAYER].position;
    state.camera.set(spawn);
    state.aim.set(spawn);
    state.follow.set(false);
    state.show_range.set(false);
    state.round.replace(Some(Round {
        game,
        last_tick: Instant::now(),
        remaining_time: 0.,
    }));
    if unsafe { SetTimer(window, TIMER, 16, None) } == 0 {
        state.round.replace(None);
        unsafe {
            MessageBoxW(
                window,
                w!("Nie udało się uruchomić zegara meczu."),
                TITLE,
                MB_OK | MB_ICONERROR,
            )
        };
    }
    sync_controls(state, window);
    unsafe { SetFocus(window) };
}

fn timer(window: HWND, state: &MatchWindow) {
    let outcome = {
        let mut stored = state.round.borrow_mut();
        let Some(round) = stored.as_mut() else { return };
        let now = Instant::now();
        let elapsed = (now - round.last_tick).as_secs_f32().min(0.1);
        round.remaining_time += elapsed;
        round.last_tick = now;
        while round.remaining_time >= FIXED_STEP {
            round.game.step(FIXED_STEP);
            round.remaining_time -= FIXED_STEP;
        }
        if state.follow.get() {
            let player = round.game.units[PLAYER];
            if player.alive() {
                state.camera.set(player.position);
            }
        }
        update_camera(window, state, &round.game, elapsed);
        round.game.result
    };
    if outcome == 0 { state.confine_cursor(window); }
    if outcome != 0 {
        state.release_cursor();
        unsafe { KillTimer(window, TIMER) };
        sync_controls(state, window);
        unsafe { SetFocus(child(window, START)) };
    }
    unsafe { InvalidateRect(window, null(), 0) };
}

fn command(window: HWND, state: &MatchWindow, id: u16) {
    match id {
        id if id >= FIRST_HERO && state.round.borrow().is_none() => {
            let Some(hero) = button_hero(id) else { return };
            let previous = state.hero.replace(hero);
            unsafe {
                InvalidateRect(child(window, FIRST_HERO + previous as u16), null(), 0);
                InvalidateRect(child(window, id), null(), 0);
            }
        }
        START => start(window, state),
        CHANGE => {
            state.release_cursor();
            unsafe { KillTimer(window, TIMER) };
            state.round.replace(None);
            sync_controls(state, window);
            unsafe { SetFocus(child(window, FIRST_HERO + state.hero.get() as u16)) };
        }
        CLOSE => {
            unsafe { PostMessageW(window, WM_CLOSE, 0, 0) };
        }
        Q | W | E | R => {
            if let Some(round) = state.round.borrow_mut().as_mut() {
                if let Some((_, slot, _)) = ABILITIES.iter().find(|(button, _, _)| *button == id) {
                    round.game.cast(PLAYER, *slot, state.aim.get());
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn route(window: HWND, message: &MSG) -> bool {
    if window.is_null() || (message.hwnd != window && unsafe { IsChild(window, message.hwnd) } == 0)
    {
        return false;
    }
    if message.message == WM_KEYDOWN {
        match message.wParam as u32 {
            0x51 | 0x45 | 0x57 | 0x52 => {
                let id = match message.wParam {
                    0x51 => Q,
                    0x57 => W,
                    0x45 => E,
                    _ => R,
                };
                unsafe { SendMessageW(window, WM_COMMAND, id as WPARAM, 0) };
                return true;
            }
            0x20 => {
                let state =
                    unsafe { &*(GetWindowLongPtrW(window, GWLP_USERDATA) as *const MatchWindow) };
                if state.round.borrow().as_ref().is_some() {
                    state.follow.set(true);
                    return true;
                }
            }
            0x1b => {
                let state = unsafe { &*(GetWindowLongPtrW(window, GWLP_USERDATA) as *const MatchWindow) };
                if state.round.borrow().is_some() { command(window, state, CHANGE); }
                else { unsafe { PostMessageW(window, WM_CLOSE, 0, 0) }; }
                return true;
            }
            0x0d => {
                let focused = unsafe { GetFocus() };
                let id = unsafe { GetDlgCtrlID(focused) } as u16;
                if (START..=R).contains(&id) || button_hero(id).is_some() {
                    unsafe { SendMessageW(focused, BM_CLICK, 0, 0) };
                    return true;
                }
            }
            key @ (0x53 | 0x54 | 0x41 | 0x56) => {
                let state = unsafe { &*(GetWindowLongPtrW(window, GWLP_USERDATA) as *const MatchWindow) };
                let mut active = state.round.borrow_mut();
                let Some(round) = active.as_mut() else { return false };
                if round.game.result != 0 || message.lParam & (1 << 30) != 0 { return true; }
                match key {
                    0x53 => round.game.stop_player(),
                    0x54 => {
                        let mut mouse = POINT::default();
                        let area = viewport(window);
                        if unsafe { GetCursorPos(&mut mouse) } != 0
                            && unsafe { ScreenToClient(window, &mut mouse) } != 0
                            && mouse.x >= 0 && mouse.x < area.right && mouse.y >= 0 && mouse.y < area.bottom {
                            let camera = scene::Camera::at(board(area), state.camera.get(), state.top_down.get());
                            let point = minimap::world_at(area, mouse).unwrap_or_else(|| camera.ground(mouse.x, mouse.y));
                            round.game.attack_unit_near(point);
                        }
                    }
                    0x41 => state.show_range.set(true),
                    _ => state.top_down.set(!state.top_down.get()),
                }
                unsafe { InvalidateRect(window, null(), 0) };
                return true;
            }
            _ => {}
        }
    }
    if message.message == WM_KEYUP {
        if message.wParam as u32 == 0x20 {
            let state =
                unsafe { &*(GetWindowLongPtrW(window, GWLP_USERDATA) as *const MatchWindow) };
            if state.round.borrow().as_ref().is_some() {
                state.follow.set(false);
                return true;
            }
        }
        return false;
    }
    (unsafe { IsDialogMessageW(window, message) }) != 0
}

unsafe extern "system" fn proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
        let state = unsafe { &*(create.lpCreateParams as *const MatchWindow) };
        state.handle.set(window);
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize) };
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    }
    let pointer = unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) } as *const MatchWindow;
    let Some(state) = (unsafe { pointer.as_ref() }) else {
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    };
    match message {
        WM_GETMINMAXINFO => {
            let bounds = unsafe { &mut *(lparam as *mut MINMAXINFO) };
            bounds.ptMinTrackSize.x = 865;
            bounds.ptMinTrackSize.y = 690;
        }
        WM_SIZE => {
            control_layout(window, state);
            unsafe { InvalidateRect(window, null(), 0) };
        }
        WM_KILLFOCUS => { state.follow.set(false); state.release_cursor(); }
        WM_ERASEBKGND => return 1,
        WM_PAINT => draw(window, state),
        WM_DRAWITEM => {
            paint_button(state, unsafe { &*(lparam as *const DRAWITEMSTRUCT) });
            return 1;
        }
        WM_TIMER if wparam == TIMER => timer(window, state),
        WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
            if message != WM_MOUSEMOVE && state.show_range.replace(false) {
                unsafe { InvalidateRect(window, null(), 0) };
            }
            if message == WM_MBUTTONDOWN || message == WM_XBUTTONDOWN { return 1; }
            let mouse = POINT {
                x: lparam as i16 as i32,
                y: (lparam >> 16) as i16 as i32,
            };
            if state.round.borrow().as_ref().is_some_and(|r| r.game.result == 0) {
                if let Some(point) = minimap::world_at(viewport(window), mouse) {
                    if message == WM_LBUTTONDOWN { state.camera.set(point); }
                    return 0;
                }
                if let Some(id) = hud_hit(viewport(window), state.hero.get(), mouse) {
                    if message == WM_LBUTTONDOWN { command(window, state, id); }
                    return 0;
                }
            }
            if let Some(round) = state.round.borrow_mut().as_mut() {
                if let Some(point) = world_under_cursor(
                    state,
                    board(viewport(window)),
                    mouse,
                    &round.game,
                    message == WM_MOUSEMOVE,
                ) {
                    state.aim.set(point);
                    if message != WM_MOUSEMOVE {
                        round.game.command(point);
                        unsafe { SetFocus(window) };
                    }
                }
            }
        }
        WM_COMMAND if (wparam >> 16) as u32 == BN_CLICKED => command(window, state, wparam as u16),
        WM_NCDESTROY => {
            state.release_cursor();
            unsafe { KillTimer(window, TIMER) };
            state.round.replace(None);
            state.handle.set(null_mut());
            unsafe { PostMessageW(GetParent(window), MATCH_CLOSED, 0, 0) };
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, 0) };
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        _ => return unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_match_uses_only_clickable_ability_icons() {
        unsafe {
            let instance = GetModuleHandleW(null());
            register(instance).unwrap();
            let host = CreateWindowExW(0, w!("STATIC"), TITLE, WS_OVERLAPPEDWINDOW,
                0, 0, 1100, 720, null_mut(), null_mut(), instance, null());
            assert!(!host.is_null());
            let state = MatchWindow::new();
            state.open(host).unwrap();
            let view = state.handle.get();
            assert_eq!(GetAncestor(view, GA_ROOT), host);
            for hero in Hero::ALL {
                command(view, &state, FIRST_HERO + hero as u16);
                command(view, &state, START);
                let area = viewport(view);
                let slots: Vec<_> = hud_slots(area, hero).collect();
                assert_eq!(slots.len(), if hero.profile().is_some() { 4 } else { 2 });
                assert!((slots[0].3.left - (area.right - slots.last().unwrap().3.right)).abs() <= 1);
                for id in [START, CHANGE, CLOSE] {
                    assert_eq!(GetWindowLongW(child(view, id), GWL_STYLE) as u32 & WS_VISIBLE, 0);
                }
                state.aim.set(Point::at(200., 280.));
                let r = slots[0].3;
                let pos = ((r.top + 5) << 16) | (r.left + 5);
                let press_a = MSG { hwnd: view, message: WM_KEYDOWN, wParam: 0x41, ..MSG::default() };
                assert!(route(view, &press_a));
                assert!(state.show_range.get());
                SendMessageW(view, WM_MOUSEMOVE, 0, pos as isize);
                assert!(state.show_range.get());
                SendMessageW(view, WM_LBUTTONDOWN, 0, pos as isize);
                assert!(!state.show_range.get());
                {
                let round = state.round.borrow();
                let round = round.as_ref().unwrap();
                assert!(round.game.units[PLAYER].cooldown(0) > 0.);
                assert!(round.game.target.is_none());
                }
                // Exercise the normal paint path including the numeric overlay.
                InvalidateRect(view, null(), 0);
                SendMessageW(view, WM_PAINT, 0, 0);
                command(view, &state, CHANGE);
            }
            DestroyWindow(host);
            assert!(state.handle.get().is_null());
        }
    }

    #[test]
    fn every_icon_has_a_distinct_click_target_on_one_page() {
        for width in [840, 1184, 1920] {
            let area = rect(0, 0, width, 640);
            let mut placed: Vec<RECT> = Vec::new();
            let mut seen = Vec::new();
            for (row, (_, heroes)) in ROLE_ROWS.iter().enumerate() {
                for (column, &hero) in heroes.iter().enumerate() {
                    let icon = icon_rect(row, column);
                    assert_eq!(button_hero(FIRST_HERO + hero as u16), Some(hero));
                    assert!(icon.right <= area.right && icon.bottom <= area.bottom - 55);
                    assert!(!seen.contains(&hero));
                    assert!(placed.iter().all(|other| other.right <= icon.left || other.bottom <= icon.top));
                    placed.push(icon);
                    seen.push(hero);
                }
            }
            assert_eq!(seen.len(), Hero::ALL.len());
        }
    }

    #[test]
    fn edge_scrolling_is_constant_in_each_band() {
        assert_eq!(edge_speed(8, 8, 808), -1.);
        assert_eq!(edge_speed(20, 8, 808), -1.);
        assert_eq!(edge_speed(400, 8, 808), 0.);
        assert_eq!(edge_speed(795, 8, 808), 1.);
        assert_eq!(edge_speed(807, 8, 808), 1.);
    }
}
