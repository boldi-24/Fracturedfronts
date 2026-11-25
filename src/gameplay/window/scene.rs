//! Native 3D battlefield. X runs along the lane, Y across it, and Z is elevation.
mod landscape;
mod models;
mod raster;
use super::rect;
use crate::{
    gameplay::{
        geometry::Point,
        simulation::{Game, Kind, PLAYER, Side, Tone},
    },
    ui::{controls::fill, theme::rgb},
};
use landscape::{nexus, ring};
use raster::{Mesh, Vec3, shade};
use windows_sys::Win32::{Foundation::RECT, Graphics::Gdi::HDC};

const COS: f32 = 0.9510565;
const SIN: f32 = 0.309017;
const ELEV_SIN: f32 = 0.6691306;
const ELEV_COS: f32 = 0.7431448;
#[derive(Clone, Copy)]
pub(super) struct Camera {
    area: RECT,
    center: Point,
    scale: f32,
    top: bool,
}
impl Camera {
    pub(super) fn at(area: RECT, center: Point, top: bool) -> Self {
        let w = (area.right - area.left).max(1) as f32;
        let h = (area.bottom - area.top).max(1) as f32;
        Self {
            area,
            center,
            scale: (w / 800.).min(h / 372.).max(0.12),
            top,
        }
    }
    fn project(self, v: Vec3) -> (f32, f32, f32) {
        let center_x = (self.area.left + self.area.right) as f32 * 0.5;
        let center_y = (self.area.top + self.area.bottom) as f32 * 0.5;
        if self.top {
            return (
                center_x + (v.x - self.center.x) * self.scale,
                center_y + (v.y - self.center.y) * self.scale,
                v.z,
            );
        }
        let dx = v.x - self.center.x;
        let dy = v.y - self.center.y;
        let forward = -SIN * dx + COS * dy;
        (
            center_x + (COS * dx + SIN * dy) * self.scale,
            center_y
                + (forward * ELEV_SIN - v.z * ELEV_COS) * self.scale,
            forward * ELEV_COS + v.z * ELEV_SIN,
        )
    }
    pub(super) fn offset(self, dx: f32, dy: f32) -> Point {
        if self.top {
            return Point::at(dx / self.scale, dy / self.scale);
        }
        let r = dx / self.scale;
        let f = dy / (self.scale * ELEV_SIN);
        Point::at(COS * r - SIN * f, SIN * r + COS * f)
    }
    pub(super) fn ground(self, sx: i32, sy: i32) -> Point {
        self.center
            + self.offset(
                sx as f32 - (self.area.left + self.area.right) as f32 * 0.5,
                sy as f32 - (self.area.top + self.area.bottom) as f32 * 0.5,
            )
    }
    pub(super) fn unit_at(
        self,
        game: &Game,
        sx: i32,
        sy: i32,
        include_allies: bool,
    ) -> Option<usize> {
        game.units
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                u.alive()
                    && (u.side == Side::Red
                        || (include_allies && !matches!(u.kind, Kind::Nexus)))
                    && !(u.side == Side::Red && u.status.concealed())
            })
            .filter_map(|(i, u)| {
                let (h, r) = model_bounds(u.kind);
                let (x, y, d) = self.project(Vec3::at(u.position, 0.));
                let top = y - h * ELEV_COS * self.scale;
                let radius = r * self.scale;
                let dy = sy as f32 - (sy as f32).clamp(top, y);
                (((sx as f32 - x) / radius).powi(2) + (dy / (radius * ELEV_SIN)).powi(2) <= 1.)
                    .then_some((i, d))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|v| v.0)
    }
    fn visible(self, p: Point, padding: f32) -> bool {
        let (x, y, _) = self.project(Vec3::at(p, 0.));
        let margin = padding * self.scale;
        x > self.area.left as f32 - margin
            && x < self.area.right as f32 + margin
            && y > self.area.top as f32 - margin
            && y < self.area.bottom as f32 + margin
    }
    fn x_range(self) -> (f32, f32) {
        let a = self.ground(self.area.left, self.area.bottom).x;
        let b = self.ground(self.area.right, self.area.top).x;
        (a - 240., b + 240.)
    }
}
fn model_bounds(kind: Kind) -> (f32, f32) {
    match kind {
        Kind::Nexus => (72., 39.),
        _ => (49., 16.),
    }
}
pub(super) fn team(side: Side) -> u32 {
    if side == Side::Blue {
        rgb(104, 216, 252)
    } else {
        rgb(237, 159, 72)
    }
}
pub(super) fn draw(dc: HDC, camera: Camera, game: &Game, show_range: bool) {
    raster::render(dc, camera, |m| {
        let player = game.units[PLAYER];
        if show_range && player.alive() && game.result == 0 {
            ring(m, player.position, player.attack_range(), 1.1, 1.6, team(player.side));
        }
        if player.alive()
            && game.target.is_none()
            && player.position.distance(game.destination) > 12.
        {
            ring(m, game.destination, 6., 1., 1.3, team(Side::Blue));
        }
        for (i, u) in game.units.iter().enumerate() {
            if !camera.visible(u.position, 130.) || (u.side == Side::Red && u.status.concealed()) {
                continue;
            }
            if !u.alive() {
                continue;
            }
            let accent = team(u.side);
            let (_, r) = model_bounds(u.kind);
            landscape::shadow(m, u.position, r * 1.1);
            if game.target == Some(i) || u.kind.hero().is_some() {
                ring(m, u.position, r + 3., 1., 1.6, accent)
            }
            if u.status.marked() {
                ring(m, u.position, r + 3., 6., 1., rgb(250, 168, 104))
            }
            if u.status.restricted() {
                ring(m, u.position, r, 9., 2., rgb(143, 225, 250))
            }
            match u.kind {
                Kind::Nexus => nexus(m, u.position, accent, game.elapsed),
                _ => models::hero(m, u, game.elapsed),
            }
        }
        for shot in &game.shots {
            let c = match shot.spell_tone(game) {
                Some(Tone::Eclipse) => rgb(148, 190, 209),
                Some(Tone::Solar) => rgb(254, 197, 88),
                None => team(shot.side),
            };
            let direction = shot.velocity.direction();
            let nose = Vec3::at(shot.position, 21.);
            m.segment(Vec3::at(shot.position - direction * 14., 21.), nose, 2., c);
            m.gem(nose, 4., 10., c);
        }
        for mark in &game.marks {
            m.segment(
                Vec3::at(mark.from, 15.),
                Vec3::at(mark.to, 16.),
                1.5,
                match mark.kind {
                    0 => team(Side::Blue),
                    1 => team(Side::Red),
                    _ => rgb(233, 221, 186),
                },
            );
        }
        m.translucent = true;
        for e in &game.effects {
            if camera.visible(e.to, e.radius + 80.) || camera.visible(e.from, e.radius + 80.) {
                models::effect(m, e, game.elapsed)
            }
        }
        m.translucent = false;
    });
    for u in game
        .units
        .iter()
        .filter(|u| u.alive() && !(u.side == Side::Red && u.status.concealed()))
    {
        let (h, r) = model_bounds(u.kind);
        let (x, y, _) = camera.project(Vec3::at(u.position, h + 7.));
        let width = ((r * 1.75) * camera.scale).clamp(13., 82.) as i32;
        let x = x as i32 - width / 2;
        let y = y as i32;
        if x < camera.area.left
            || x + width > camera.area.right
            || y < camera.area.top
            || y + 5 >= camera.area.bottom
        {
            continue;
        }
        fill(dc, &rect(x - 1, y - 1, width + 2, 5), rgb(23, 21, 38));
        fill(
            dc,
            &rect(
                x,
                y,
                (width as f32 * u.health / u.kind.health()).round() as i32,
                3,
            ),
            team(u.side),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_projection_and_inverse_agree_after_pan() {
        let cam = Camera::at(
            RECT {
                left: 11,
                top: 45,
                right: 1090,
                bottom: 679,
            },
            Point::at(1920., 360.),
            false,
        );
        for p in [Point::at(1890., 220.), Point::at(2130., 430.)] {
            let (x, y, _) = cam.project(Vec3::at(p, 0.));
            assert!(cam.ground(x.round() as i32, y.round() as i32).distance(p) < 1.);
        }
    }

    #[test]
    fn top_down_camera_projects_without_elevation() {
        let cam = Camera::at(
            RECT {
                left: 11,
                top: 45,
                right: 1090,
                bottom: 679,
            },
            Point::at(1920., 360.),
            true,
        );
        for p in [Point::at(1890., 220.), Point::at(2130., 430.)] {
            let (x, y, _) = cam.project(Vec3::at(p, 0.));
            assert!(cam.ground(x.round() as i32, y.round() as i32).distance(p) < 1.);
        }
    }
}

#[cfg(test)]
mod preview;
