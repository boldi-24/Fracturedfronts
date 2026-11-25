use super::rect;
use crate::gameplay::{
    geometry::{LANE_MAX_Y, LANE_MIN_Y, Point, WORLD_HEIGHT, WORLD_WIDTH},
    simulation::{Game, Kind, PLAYER, Side},
};
use crate::ui::{art, controls::fill, theme::rgb};
use windows_sys::Win32::{Foundation::{POINT, RECT}, Graphics::Gdi::HDC};

fn bounds(area: RECT) -> RECT {
    let width = (area.right / 5).clamp(130, 224);
    let height = width;
    rect(area.right - width - 12, area.bottom - height - 12, width, height)
}

const INSET: i32 = 8;

fn project(map: RECT, p: Point) -> POINT {
    let axis = |v: f32, max: f32, lo: i32, hi: i32| {
        lo + INSET + ((v / max).clamp(0., 1.) * (hi - lo - 2 * INSET) as f32).round() as i32
    };
    POINT { x: axis(p.x, WORLD_WIDTH, map.left, map.right),
            y: axis(p.y, WORLD_HEIGHT, map.top, map.bottom) }
}

pub(super) fn world_at(area: RECT, mouse: POINT) -> Option<Point> {
    let m = bounds(area);
    if mouse.x < m.left || mouse.x >= m.right || mouse.y < m.top || mouse.y >= m.bottom {
        return None;
    }
    let axis = |v: i32, lo: i32, hi: i32| {
        ((v - lo - INSET) as f32 / (hi - lo - 2 * INSET) as f32).clamp(0., 1.)
    };
    Some(Point::at(WORLD_WIDTH * axis(mouse.x, m.left, m.right),
                   WORLD_HEIGHT * axis(mouse.y, m.top, m.bottom)))
}

pub(super) fn draw(dc: HDC, area: RECT, game: &Game) {
    let m = bounds(area);
    art::frame(dc, m, rgb(150,144,119), rgb(34,38,34));
    let a = project(m, Point::at(0., LANE_MIN_Y));
    let b = project(m, Point::at(WORLD_WIDTH, LANE_MAX_Y));
    art::frame(dc, rect(a.x, a.y, b.x-a.x+1, b.y-a.y+1), rgb(109,108,88), rgb(66,69,55));

    // Draw actors above the objectives.
    for layer in 0..2 {
        for (i, unit) in game.units.iter().enumerate() {
            if !unit.alive() || (unit.side == Side::Red && unit.status.concealed()) { continue; }
            let order = usize::from(!unit.kind.structure());
            if order != layer { continue; }
            let POINT { x, y } = project(m, unit.position);
            let color = super::scene::team(unit.side);
            match unit.kind {
                Kind::Nexus => {
                    fill(dc, &rect(x-4, y-2, 9, 5), color);
                    fill(dc, &rect(x-2, y-4, 5, 9), color);
                }
                _ => {
                    for dy in -5_i32..=5 {
                        let half = 5 - dy.abs();
                        fill(dc, &rect(x-half, y+dy, half*2+1, 1), color);
                    }
                    if i == PLAYER { fill(dc, &rect(x-1, y-1, 3, 3), rgb(255,255,255)); }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimap_click_maps_back_to_lane_and_stays_clear_of_skills() {
        for width in [645, 1160, 1920] {
            let area = rect(0, 0, width, 780);
            let m = bounds(area);
            for (_, _, _, icon) in super::super::hud_slots(area, crate::characters::roster::Hero::Deafult) {
                assert!(icon.right < m.left);
            }
            for p in [Point::at(0., 0.), Point::at(WORLD_WIDTH * 0.6, WORLD_HEIGHT * 0.3), Point::at(WORLD_WIDTH, WORLD_HEIGHT)] {
                let actual = world_at(area, project(m, p)).unwrap();
                assert!((actual.x-p.x).abs() <= WORLD_WIDTH / (m.right-m.left-2*INSET) as f32);
                assert!((actual.y-p.y).abs() <= WORLD_HEIGHT / (m.bottom-m.top-2*INSET) as f32);
            }
            assert!(world_at(area, POINT { x: m.left-1, y: m.top }).is_none());
        }
    }
}
