use super::{Camera, Mesh, Point, Vec3, rgb, shade};
use crate::gameplay::{
    geometry::{LANE_MAX_Y, LANE_MIN_Y, WORLD_WIDTH},
    simulation::{Side, Unit},
};
use std::f32::consts::TAU;

const STONE: u32 = rgb(101, 99, 123);
const SNOW: u32 = rgb(179, 194, 210);
const IRON: u32 = rgb(65, 65, 89);
const COPPER: u32 = rgb(173, 135, 94);
fn vertex(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}
fn tile(m: &mut Mesh, x: f32, y: f32, z: f32, w: f32, d: f32, c: u32) {
    m.face(
        &[
            vertex(x, y, z),
            vertex(x + w, y, z),
            vertex(x + w, y + d, z),
            vertex(x, y + d, z),
        ],
        c,
    );
}
fn frustum(
    m: &mut Mesh,
    p: Point,
    bottom: f32,
    top: f32,
    r1: f32,
    r2: f32,
    n: usize,
    start: f32,
    color: u32,
) {
    let at = |i: usize, r: f32, z: f32| {
        let t = i as f32 * TAU / n as f32 + start;
        vertex(p.x + r * t.cos(), p.y + r * t.sin(), z)
    };
    for i in 0..n {
        let a = at(i, r1, bottom);
        let b = at(i + 1, r1, bottom);
        let c = at(i + 1, r2, top);
        let d = at(i, r2, top);
        let t = (i as f32 + 0.5) * TAU / n as f32 + start;
        m.face(
            &[a, b, c, d],
            shade(color, 0.78 - 0.18 * t.cos() - 0.13 * t.sin()),
        );
        if r2 > 0. {
            m.face(&[d, c, Vec3::at(p, top)], shade(color, 1.06));
        }
    }
}
fn block(m: &mut Mesh, p: Point, z: f32, w: f32, d: f32, h: f32, c: u32) {
    let (x, y) = (p.x - w / 2., p.y - d / 2.);
    let a = vertex(x, y, z);
    let b = vertex(x + w, y, z);
    let e = vertex(x, y + d, z);
    let f = vertex(x + w, y + d, z);
    let lift = vertex(0., 0., h);
    let (aa, bb, ee, ff) = (a.add(lift), b.add(lift), e.add(lift), f.add(lift));
    m.face(&[a, b, bb, aa], shade(c, 0.91));
    m.face(&[b, f, ff, bb], shade(c, 0.61));
    m.face(&[e, f, ff, ee], shade(c, 0.78));
    m.face(&[e, a, aa, ee], shade(c, 0.95));
    m.face(&[aa, bb, ff, ee], c);
}
pub(super) fn ring(m: &mut Mesh, p: Point, r: f32, z: f32, width: f32, color: u32) {
    let steps = 28;
    let at = |i: usize, r: f32| {
        let a = i as f32 * TAU / steps as f32;
        vertex(p.x + a.cos() * r, p.y + a.sin() * r, z)
    };
    for i in 0..steps {
        m.face(
            &[
                at(i, r),
                at(i + 1, r),
                at(i + 1, r - width),
                at(i, r - width),
            ],
            color,
        );
    }
}
pub(super) fn shadow(m: &mut Mesh, p: Point, r: f32) {
    let p = p + Point::at(r * 0.18, r * 0.25);
    for i in 0..12 {
        let at = |j| {
            let a = j as f32 * TAU / 12.;
            vertex(p.x + a.cos() * r, p.y + a.sin() * r * 0.7, 0.42)
        };
        m.face(&[Vec3::at(p, 0.42), at(i), at(i + 1)], rgb(60, 59, 79));
    }
}
fn noise(a: i32, b: i32) -> f32 {
    let mut n = (a as u32).wrapping_mul(0x27d4eb2d) ^ (b as u32).wrapping_mul(0x165667b1);
    n = (n ^ (n >> 15)).wrapping_mul(0x85ebca6b);
    ((n ^ (n >> 13)) & 0xffff) as f32 / 65535.
}
fn spruce(m: &mut Mesh, p: Point, z: f32, h: f32) {
    frustum(
        m,
        p,
        z,
        z + h * 0.32,
        h * 0.045,
        h * 0.036,
        5,
        0.,
        rgb(72, 65, 82),
    );
    for (base, r, tip) in [(0.2, 0.28, 0.7), (0.4, 0.22, 0.86), (0.58, 0.16, 1.)] {
        frustum(
            m,
            p,
            z + h * base,
            z + h * tip,
            h * r,
            0.,
            6,
            0.3,
            rgb(61, 83, 101),
        );
        frustum(
            m,
            p,
            z + h * (base + 0.09),
            z + h * tip,
            h * r * 0.82,
            0.,
            6,
            0.3,
            rgb(149, 165, 183),
        );
    }
}
// The road is built from individually bevelled stone slabs. Small variations repeat deterministically in world space.
fn paving(m: &mut Mesh, i: i32, j: i32, x: f32, y: f32, w: f32, d: f32) {
    let v = noise(i, j);
    let c = shade(
        if (j == 0 || j == 6) && v < 0.57 {
            rgb(156, 166, 184)
        } else {
            rgb(119, 119, 142)
        },
        0.90 + v * 0.23,
    );
    let (x, y) = (x + 1., y + 1.);
    let w = w - 2.;
    let d = d - 2.;
    let bevel = 2.5 + 3. * v;
    let a = vertex(x + bevel, y, 0.);
    let b = vertex(x + w, y, 0.);
    let e = vertex(x + w - bevel, y + d, 0.);
    let f = vertex(x, y + d, 0.);
    m.face(
        &[
            a,
            b,
            vertex(x + w, y + d - bevel, 0.),
            e,
            f,
            vertex(x, y + bevel, 0.),
        ],
        c,
    );
    m.face(
        &[
            a,
            b,
            vertex(x + w - 2., y + 1.2, 0.10),
            vertex(x + bevel, y + 1.2, 0.1),
        ],
        shade(c, 1.18),
    );
    // Mineral seams and wind-worn flecks stay anchored to individual stones during camera movement.
    for vein in 0..7 {
        let u = noise(i * 13 + vein, j + 31);
        let k = noise(i + 71, j * 11 + vein);
        let start = vertex(x + 4. + (w - 12.) * u, y + 4. + (d - 10.) * k, 0.13);
        m.face(&[start, vertex(start.x + 1.3 + 4. * k, start.y + u, 0.13),
                 vertex(start.x + 2., start.y + 1.4, 0.13)], shade(c, 0.86 + 0.30*k));
    }
    if v > 0.78 {
        let a = vertex(x + w * 0.63, y, 0.12);
        let b = vertex(x + w * 0.5, y + d * 0.4, 0.12);
        m.face(&[a, b, vertex(b.x + 1., b.y + 0.4, 0.12)], shade(c, 0.78));
    }
}
fn pier(m: &mut Mesh, p: Point) {
    frustum(m, p, -125., -10., 15., 24., 4, 0.785, IRON);
    block(m, p, -13., 34., 32., 18., STONE);
    block(m, p, 5., 24., 23., 19., rgb(113, 112, 136));
    block(m, p, 24., 28., 27., 5., SNOW);
}
pub(super) fn terrain(m: &mut Mesh, camera: Camera) {
    let (lo, hi) = camera.x_range();
    let start = (lo / 120.).floor() as i32;
    let end = (hi / 120.).ceil() as i32;
    // Steep, tiered gorge walls run parallel to the elevated path.
    for i in start - 1..=end + 1 {
        let x = i as f32 * 120.;
        for (side, edge) in [(-1., LANE_MIN_Y), (1., LANE_MAX_Y)] {
            for row in 0..2 {
                let n = noise(i, row + if side > 0. { 8 } else { 21 });
                let y = edge + side * (190. + row as f32 * 156. + n * 58.);
                let p = Point::at(x + n * 52., y);
                let top = -126. + n * 53. + row as f32 * 35.;
                frustum(
                    m,
                    p,
                    -330.,
                    top,
                    123.,
                    64. + n * 37.,
                    6,
                    n * 3.,
                    rgb(70, 74, 103),
                );
                frustum(
                    m,
                    p,
                    top - 4.,
                    top + 8.,
                    64. + n * 37.,
                    61. + n * 34.,
                    6,
                    n * 3.,
                    rgb(139, 154, 180),
                );
                spruce(m, p, top + 8., 49. + 65. * n);
                if n > 0.4 {
                    spruce(m, p + Point::at(-29., 16.), top + 8., 40. + 20. * n)
                }
            }
        }
    }
    let column = 64.;
    let road_left = -42.;
    let road_right = WORLD_WIDTH + 42.;
    let min = (lo.max(road_left) / column).floor() as i32;
    let max = (hi.min(road_right) / column).ceil() as i32;
    let depth = (LANE_MAX_Y - LANE_MIN_Y) / 7.;
    for j in 0..7 {
        let shift = if j % 2 == 0 { 0. } else { 32. };
        for i in min - 1..=max {
            let x = (i as f32 * column + shift).max(road_left);
            let right = ((i + 1) as f32 * column + shift).min(road_right);
            if right <= x {
                continue;
            }
            let y = LANE_MIN_Y + depth * j as f32;
            tile(m, x, y, -0.4, right - x, depth, rgb(77, 78, 101));
            paving(m, i, j, x, y, right - x, depth);
        }
    }
    for (edge, sign) in [(LANE_MIN_Y, -1.), (LANE_MAX_Y, 1.)] {
        for i in min - 1..=max {
            let x = i as f32 * column;
            if x < road_left - 32. || x > road_right {
                continue;
            }
            let p = Point::at(x + 32., edge + sign * 8.);
            block(
                m,
                p,
                -53.,
                63.8,
                17.,
                56.,
                shade(STONE, 0.87 + noise(i, 8) * 0.18),
            );
            block(m, p, 3., 63.5, 17., 4., SNOW);
            // Alternating low balustrade spans leave the characters readable at the camera-side edge.
            if i % 2 == 0 {
                pier(m, Point::at(x, edge + sign * 10.));
                if sign < 0. {
                    block(m, Point::at(x + 64., edge - 11.), 10., 106., 5., 5., IRON)
                }
            }
            if i % 6 == 0 {
                let p = Point::at(x, edge + sign * 11.);
                frustum(m, p, 29., 36., 9., 7., 8, 0., COPPER);
                let glow = if x < WORLD_WIDTH * 0.5 { rgb(173, 200, 254) } else { rgb(246, 183, 98) };
                m.gem(Vec3::at(p, 42.), 5., 18., glow);
            }
        }
        let x = lo.max(road_left);
        let w = hi.min(road_right) - x;
        if w > 0. {
            tile(m, x, edge - sign * 13., 0.25, w, 1.8, rgb(186, 164, 137));
        }
    }
    // A radial inlay marks the meeting point of the two armies.
    let center = Point::at(WORLD_WIDTH * 0.5, (LANE_MAX_Y + LANE_MIN_Y) * 0.5);
    if camera.visible(center, 90.) {
        ring(m, center, 71., 0.25, 2.7, rgb(164, 146, 146));
        ring(m, center, 65., 0.25, 1.2, rgb(185, 170, 161));
        for i in 0..8 {
            let a = i as f32 * TAU / 8.;
            let arm = Point::at(a.cos(), a.sin());
            let side = Point::at(-arm.y, arm.x);
            m.face(
                &[
                    Vec3::at(center + arm * 14., 0.25),
                    Vec3::at(center + arm * 40. + side * 6., 0.25),
                    Vec3::at(center + arm * 61., 0.25),
                    Vec3::at(center + arm * 40. - side * 6., 0.25),
                ],
                rgb(158, 146, 151),
            );
        }
    }
}
pub(super) fn ruins(m: &mut Mesh, p: Point) {
    frustum(m, p, 0.5, 5., 29., 25., 8, 0., rgb(100, 91, 106));
    for i in 0..7 {
        let a = i as f32 * TAU / 7.;
        block(
            m,
            p + Point::at(17. * a.cos(), 16. * a.sin()),
            5.,
            8.,
            9.,
            3. + noise(i, 9) * 8.,
            STONE,
        );
    }
}
pub(super) fn tower(m: &mut Mesh, p: Point, accent: u32, direction: Point) {
    frustum(m, p, 0.6, 5., 35., 35., 8, 0.39, IRON);
    frustum(m, p, 5., 12., 33., 26., 8, 0.39, STONE);
    frustum(m, p, 12., 64., 23., 18.5, 8, 0.39, STONE);
    for z in [22., 37., 52.] {
        let r = 23. - (z - 12.) * 0.088;
        frustum(
            m,
            p,
            z,
            z + 1.,
            r + 0.65,
            r + 0.65,
            8,
            0.39,
            rgb(79, 79, 102),
        );
    }
    // Narrow lancet windows and banded upper parapet identify the structure as a stone keep.
    for j in 0..4 {
        let a = j as f32 * TAU / 4.;
        let n = Point::at(a.cos(), a.sin());
        let t = Point::at(-n.y, n.x);
        let q = p + n * 19.9;
        m.face(
            &[
                Vec3::at(q - t * 2.1, 40.),
                Vec3::at(q + t * 2.1, 40.),
                Vec3::at(q + t * 1.9, 52.),
                Vec3::at(q, 55.),
                Vec3::at(q - t * 1.9, 52.),
            ],
            rgb(37, 38, 56),
        );
    }
    frustum(m, p, 61., 68., 20., 25., 8, 0.39, IRON);
    frustum(m, p, 68., 74., 25., 25., 8, 0.39, STONE);
    for i in 0..8 {
        let a = i as f32 * TAU / 8.;
        let q = p + Point::at(20. * a.cos(), 20. * a.sin());
        block(m, q, 74., 9., 9., 10., SNOW);
    }
    frustum(m, p, 74., 77., 12., 9., 8, 0., COPPER);
    frustum(m, p, 76., 84., 11., 9., 10, 0., IRON);
    let axis = |offset: f32| Vec3::at(p + direction * offset, 87.);
    m.segment(axis(-9.), axis(32.), 5.6, IRON);
    for offset in [1., 17., 29.] {
        m.segment(axis(offset), axis(offset + 3.), 6.8, COPPER);
    }
    m.segment(axis(32.), axis(33.), 3.9, accent);
    for k in 0..4 {
        let angle = TAU * k as f32 / 4. + 0.78;
        let q = p + Point::at(angle.cos(), angle.sin()) * 22.;
        frustum(m, q, 5., 42., 8., 5., 6, angle, STONE);
    }
    let q = p + Point::at(0., 21.7);
    m.face(
        &[
            vertex(q.x - 5., q.y, 59.),
            vertex(q.x + 5., q.y, 59.),
            vertex(q.x + 5., q.y + 1., 46.),
            vertex(q.x, q.y + 1.4, 42.),
            vertex(q.x - 5., q.y + 1., 46.),
        ],
        shade(accent, 0.87),
    );
}
pub(super) fn nexus(m: &mut Mesh, p: Point, c: u32, time: f32) {
    frustum(m, p, 0.6, 6., 46., 43., 8, 0.39, IRON);
    frustum(m, p, 6., 13., 38., 34., 8, 0.39, STONE);
    ring(m, p, 32., 13.2, 2., c);
    frustum(m, p, 13., 20., 25., 19., 8, 0.39, COPPER);
    // Solar rays rise radially, lunar horns curve inward on either side of the crystal.
    let solar = p.x > WORLD_WIDTH * 0.5;
    let points = if solar { 8 } else { 2 };
    for i in 0..points {
        let a = i as f32 * TAU / points as f32 + 0.48;
        let d = Point::at(a.cos(), a.sin());
        let n = Point::at(-d.y, d.x);
        let foot = p + d * 27.;
        let ridge = p + d * 38.;
        let tip = p + d * if solar { 45. } else { 20. };
        let peak = if solar { 35. } else { 58. };
        for side in [-1., 1.] {
            m.face(&[Vec3::at(foot + n * (side * 5.), 12.), Vec3::at(ridge, 31.),
                     Vec3::at(tip, peak)], shade(if solar { COPPER } else { SNOW }, 0.91 + side * 0.12));
        }
    }
    let hover = 2.5 * (time * 1.7).sin();
    m.gem(Vec3::at(p, 57. + hover), 21., 70., c);
    for i in 0..4 {
        let t = i as f32 * TAU / 4. + 0.3;
        let v = Point::at(t.cos(), t.sin());
        let q = p + v * 32.;
        frustum(m, q, 8., 19., 7., 6., 6, 0., STONE);
        m.segment(Vec3::at(q, 18.), Vec3::at(p + v * 29., 37.), 3.5, COPPER);
        m.gem(Vec3::at(p + v * 29., 39.), 3.5, 10., c);
    }
}
pub(super) fn minion(m: &mut Mesh, u: &Unit, time: f32) {
    let p = u.position;
    let forward = if u.facing.length() > 0.1 {
        u.facing.direction()
    } else if u.side == Side::Blue {
        Point::at(1., 0.)
    } else {
        Point::at(-1., 0.)
    };
    let right = Point::at(-forward.y, forward.x);
    let local = |f: f32, l: f32, z: f32| Vec3::at(p + forward * f + right * l, z);
    let stride = if u.moving {
        (time * 9. + p.x * 0.012).sin() * 2.4
    } else {
        0.
    };
    let c = super::team(u.side);
    let cloth = shade(c, 0.58);
    for s in [-1., 1.] {
        m.segment(
            local(0., s * 3., 9.),
            local(s * stride, s * 3.5, 2.),
            2.4,
            IRON,
        );
        m.segment(
            local(s * stride, s * 3.5, 1.8),
            local(s * stride + 2.8, s * 3.5, 1.8),
            2.1,
            STONE,
        );
    }
    frustum(m, p, 7., 16., 6.7, 5.3, 6, 0., cloth);
    frustum(m, p, 13., 17., 6.4, 5.5, 6, 0., IRON);
    m.gem(local(0., 0., 21.), 5.6, 11., rgb(174, 175, 189));
    m.segment(
        local(4.5, -2.8, 21.),
        local(4.5, 2.8, 21.),
        1.,
        rgb(62, 54, 78),
    );
    m.segment(local(0., -5., 16.), local(4., -7., 12.), 2., cloth);
    if u.minion_ranged {
        // A drawn bow and a tall feather distinguish the rear rank at match scale.
        let hand = local(5., -7., 14.);
        for sign in [-1., 1.] {
            let nock = local(2., -8., 14. + sign * 11.);
            m.segment(hand, local(8., -8., 14. + sign * 6.), 1.4, COPPER);
            m.segment(local(8., -8., 14. + sign * 6.), nock, 1.1, COPPER);
            m.segment(nock, local(0., -7., 14.), 0.5, SNOW);
        }
        m.face(&[local(1., 0., 24.), local(-7., 1., 32.), local(-5., -2., 24.)], c);
        return;
    }
    m.segment(local(4., -7., 7.), local(9., -7., 23.), 1., COPPER);
    m.gem(local(9., -7., 23.), 2.1, 6., c);
    m.face(
        &[
            local(3., 5., 18.),
            local(5., 9., 16.),
            local(5., 8., 10.),
            local(3., 6., 7.),
            local(2., 4., 11.),
        ],
        c,
    );
}
