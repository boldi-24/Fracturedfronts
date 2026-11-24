use super::{Mesh, Vec3};
use crate::characters::roster::Hero;
use crate::gameplay::geometry::Point;
use crate::gameplay::simulation::{Effect, Shape, Tone, Unit};
use crate::ui::theme::rgb;
use std::f32::consts::{PI, TAU};

const WHITE: u32 = rgb(255, 248, 225);
fn pos(center: Point, angle: f32, r: f32, z: f32) -> Vec3 {
    Vec3::new(center.x + angle.cos() * r, center.y + angle.sin() * r, z)
}

// A swept, taperable sheet is visible from the overhead camera, including at glancing angles.
fn band(
    mesh: &mut Mesh,
    center: Point,
    r: f32,
    width: f32,
    z: f32,
    rise: f32,
    start: f32,
    sweep: f32,
    color: u32,
) {
    let steps = ((sweep.abs() * 3.4).ceil() as usize).max(2);
    let vertex = |i: usize, inner: bool| {
        let t = i as f32 / steps as f32;
        pos(
            center,
            start + sweep * t,
            if inner { (r - width).max(0.) } else { r },
            z + rise * t + if inner { 0.9 } else { 0. },
        )
    };
    for i in 0..steps {
        mesh.face(
            &[
                vertex(i, false),
                vertex(i + 1, false),
                vertex(i + 1, true),
                vertex(i, true),
            ],
            color,
        );
    }
}

// A feather or shard has a central ridge and two differently lit faces.
fn feather(mesh: &mut Mesh, a: Point, b: Point, za: f32, zb: f32, width: f32, color: u32) {
    let delta = b - a;
    let n = Point::at(-delta.y, delta.x).direction() * width;
    let p = a + delta * 0.38;
    let z = za + (zb - za) * 0.38;
    let v = |q: Point, h: f32| Vec3::new(q.x, q.y, h);
    mesh.face(
        &[v(a, za), v(p + n, z), v(b, zb), v(p, z + width * 0.65)],
        color,
    );
    mesh.face(
        &[v(a, za), v(p, z + width * 0.65), v(b, zb), v(p - n, z)],
        WHITE,
    );
}

enum Glyph {
    Cleave,
    Twins,
    Volley,
    Spark,
    Guard,
    Sight,
    Veil,
    Tether,
    Spring,
    Bloom,
    Halo,
    Rain,
    Wings,
    Fissure,
    Pillar,
    VolleyField,
    Crescent,
    Ward,
    Flurry,
}
use Glyph::*;

fn glyph(hero: Hero, slot: u32) -> Glyph {
    match (hero, slot) {
        (Hero::Taran | Hero::Aelor | Hero::Helvara | Hero::Veyrion, 0) => Cleave,
        (Hero::Solviana | Hero::Zyraen, 0) => Crescent,
        (Hero::Kaelith, 0) => Twins,
        (Hero::Nera | Hero::Elorim, 0) => Spark,
        (Hero::Arxen | Hero::Aurelia, 0) => Volley,
        (Hero::Nyxalune, 0) => Crescent,
        (Hero::Luness | Hero::Selune, 0) => Bloom,
        (Hero::Arxen, 2) => Sight,
        (Hero::Kaelith | Hero::Zyraen, 2) => Veil,
        (Hero::Aurelia, 2) => Wings,
        (Hero::Elorim, 2) => Halo,
        (_, 2) => Guard,
        (Hero::Aelor | Hero::Nyxalune, 1) => Wings,
        (Hero::Veyrion | Hero::Helvara, 1) => Tether,
        (Hero::Aurelia, 1) => Ward,
        (Hero::Luness | Hero::Selune, 1) => Bloom,
        (_, 1) => Spring,
        (Hero::Elorim, 3) => Pillar,
        (Hero::Arxen, 3) => VolleyField,
        (Hero::Aurelia, 3) => Volley,
        (Hero::Nyxalune, 3) => Rain,
        (Hero::Veyrion, 3) => Guard,
        (Hero::Kaelith, 3) => Flurry,
        (Hero::Zyraen, 3) => Crescent,
        (Hero::Aelor, 3) => Wings,
        (Hero::Selune, 3) => Bloom,
        (Hero::Luness, 3) => Spring,
        (Hero::Helvara, 3) => Halo,
        (Hero::Solviana, 3) => Fissure,
        _ => Wings,
    }
}

pub(super) fn cast_release(mesh: &mut Mesh, unit: &Unit, hero: Hero, color: u32, remaining: f32) {
    let t = 1. - remaining;
    let base = unit.position;
    let aim = unit.cast_point - base;
    let heading = if aim.length() > 1. {
        aim.y.atan2(aim.x)
    } else {
        unit.facing.y.atan2(unit.facing.x)
    };
    let forward = Point::at(heading.cos(), heading.sin());
    let side = Point::at(-forward.y, forward.x);
    let at = |x: f32, y: f32| base + forward * x + side * y;
    let slot = unit.cast_slot;
    let m = (t * PI).sin().max(0.08);
    let size = if slot == 3 { 1.5 } else { 1. };
    // This flash starts on release. Projectile, hit and zone geometry follows simulation events.
    match glyph(hero, slot as u32) {
        Cleave | Crescent | Twins | Flurry => {
            let repeats = match glyph(hero, slot as u32) {
                Twins => 2,
                Flurry => 4,
                _ => 1,
            };
            for i in 0..repeats {
                let k = i as f32;
                let start = heading - 1.45 + k * 0.62 + t * 1.9;
                band(
                    mesh,
                    base,
                    (25. + t * 18. + k * 3.) * size,
                    6. * m + 1.,
                    9. + k * 4.,
                    16.,
                    start,
                    1.5,
                    color,
                );
            }
        }
        Volley | VolleyField | Spark => {
            let count = if matches!(hero, Hero::Arxen) {
                3
            } else if hero == Hero::Aurelia {
                2
            } else {
                1
            };
            for i in 0..count {
                let spread = (i as f32 - (count - 1) as f32 * 0.5) * 6.8;
                let p = at(12. + t * 31., spread * (0.3 + t));
                feather(
                    mesh,
                    p - forward * 10.,
                    p + forward * 9.,
                    25.,
                    28.,
                    (2. + m * 2.) * size,
                    color,
                );
            }
            if slot == 3 {
                band(mesh, base, 32. + t * 8., 2., 3., 0., heading, TAU, color);
            }
        }
        Guard | Halo | Ward => {
            let n = if slot == 3 { 8 } else { 5 };
            for i in 0..n {
                let a = heading + i as f32 * TAU / n as f32 + t * 0.4;
                let r = 20. + 5. * m;
                let z = 3. + 7. * t;
                mesh.face(
                    &[
                        pos(base, a - 0.16, r, z + 9.),
                        pos(base, a, r + 1., z),
                        pos(base, a + 0.16, r, z + 9.),
                        pos(base, a, r, z + 21. * m),
                    ],
                    color,
                );
            }
            band(
                mesh,
                base,
                (28. + t * 12.) * size,
                2. + m,
                2.,
                0.,
                heading,
                TAU,
                color,
            );
        }
        Sight | Tether => {
            for i in 0..6 {
                let f = i as f32;
                let p = at(13. + f * 5. + t * 14., (f * PI * 0.65).sin() * m * 2.);
                mesh.gem(
                    Vec3::new(p.x, p.y, 23. + f * 0.45),
                    1.5 + m,
                    3.2 + m * 2.,
                    color,
                );
            }
            if slot == 2 {
                band(
                    mesh,
                    at(44. + t * 12., 0.),
                    8. + m * 3.,
                    1.8,
                    23.,
                    0.,
                    heading,
                    TAU,
                    color,
                );
            }
        }
        Veil | Spring => {
            for i in 0..5 {
                let k = i as f32 - 2.;
                let tail = at(-8. - t * 29. - k.abs() * 3., k * 3.);
                feather(
                    mesh,
                    at(8., k * 3.),
                    tail,
                    16. + k * 2.,
                    7. + k * 2. + m * 6.,
                    2.5 * m + 1.,
                    color,
                );
            }
        }
        Bloom | Rain => {
            let petals = if slot == 3 { 8 } else { 4 };
            for i in 0..petals {
                let a = TAU * i as f32 / petals as f32 + t * 1.2;
                let dir = Point::at(a.cos(), a.sin());
                feather(
                    mesh,
                    base + dir * 5.,
                    base + dir * (16. + 10. * t) * size,
                    17.,
                    23. + 15. * m,
                    3.3 * m + 1.,
                    color,
                );
            }
        }
        Wings => {
            phoenix(mesh, base, heading, 24. * size, t, color);
        }
        Pillar | Fissure => {
            let r = (15. + t * 17.) * size;
            band(mesh, base, r, 3. * m + 1., 2., 0., heading, TAU, color);
            for i in 0..7 {
                let a = TAU * i as f32 / 7.;
                mesh.gem(
                    pos(base, a, r, 9. + m * 14.),
                    3. * m + 1.,
                    6. + 20. * m,
                    color,
                );
            }
        }
    }
}

fn phoenix(mesh: &mut Mesh, center: Point, heading: f32, r: f32, t: f32, color: u32) {
    let forward = Point::at(heading.cos(), heading.sin());
    let side = Point::at(-forward.y, forward.x);
    let flap = (t * TAU).sin() * r * 0.31;
    let p = |f: f32, s: f32| center + forward * (f * r) + side * (s * r);
    feather(mesh, p(-0.4, 0.), p(0.65, 0.), 21., 34., r * 0.13, color);
    for sign in [-1., 1.] {
        for i in 0..4 {
            let k = i as f32 / 3.;
            feather(
                mesh,
                p(0.10, 0.14 * sign),
                p(-0.62 + 0.55 * k, (0.56 + 0.48 * k) * sign),
                26.,
                29. + flap + k * 10.,
                r * 0.11,
                color,
            );
        }
    }
}

pub(in crate::gameplay::window::scene) fn effect(mesh: &mut Mesh, fx: &Effect, elapsed: f32) {
    let remaining = (fx.t / fx.duration.max(0.001)).clamp(0., 1.);
    let t = 1. - remaining;
    let m = (t * PI).sin().max(0.07);
    let color = match fx.tone {
        Tone::Solar => rgb(255, 174, 51),
        Tone::Eclipse => rgb(174, 98, 254),
        Tone::Heal => rgb(115, 246, 200),
    };
    let base = fx.to;
    let path = fx.to - fx.from;
    let heading = path.y.atan2(path.x);
    let forward = path.direction();
    let side = Point::at(-forward.y, forward.x);
    let r = fx.radius.max(5.);
    match fx.shape {
        Shape::Arc => {
            band(
                mesh,
                fx.from,
                r * (0.72 + 0.28 * t),
                r * 0.12 * m + 1.,
                8.,
                18.,
                heading - 1.32 + t * 0.42,
                2.25,
                color,
            );
            band(
                mesh,
                fx.from,
                r * (0.72 + 0.28 * t) + 1.,
                1.6,
                8.,
                18.,
                heading - 1.32 + t * 0.42,
                2.25,
                WHITE,
            );
        }
        Shape::Beam | Shape::Dash => {
            let head = fx.from + path * (0.22 + t * 0.78);
            let tail = head - path * (0.18 + remaining * 0.25);
            feather(
                mesh,
                tail,
                head,
                if fx.shape == Shape::Dash { 4. } else { 19. },
                22.,
                (r * 0.24).clamp(2., 6.) * m + 1.,
                color,
            );
            if fx.shape == Shape::Dash {
                for sign in [-1., 1.] {
                    let offset = side * (7. * sign);
                    feather(
                        mesh,
                        tail + offset,
                        head + offset * 0.4,
                        3.,
                        10.,
                        2.5 * m + 0.6,
                        color,
                    );
                }
            }
        }
        Shape::Shield => {
            // An open dome leaves the shielded model visible.
            for i in 0..4 {
                let a = TAU * i as f32 / 4. + t * 1.3;
                let rim = pos(base, a, r * 0.72, 4.);
                let arch = pos(base, a, r * 0.65, 23.);
                let peak = pos(base, a, r * 0.20, 39. + m * 4.);
                mesh.face(
                    &[rim, arch, peak, pos(base, a + 0.09, r * 0.64, 22.)],
                    color,
                );
            }
            band(mesh, base, r * 0.74, 2., 3., 0., elapsed, TAU, color);
        }
        Shape::Mark => {
            let z = 39. + t * 6.;
            let span = (r * 0.26).clamp(4., 11.);
            for i in 0..4 {
                let a = TAU * i as f32 / 4. + t;
                mesh.face(
                    &[
                        pos(base, a - 0.36, span, z),
                        pos(base, a, span * 1.3, z + 6.),
                        pos(base, a + 0.36, span, z),
                        pos(base, a, span * 0.74, z - 3.),
                    ],
                    color,
                );
            }
        }
        Shape::Rain => {
            band(mesh, base, r, 2.3, 2., 0., 0., TAU, color);
            for i in 0..10 {
                let seed = i as f32;
                let angle = seed * 2.39996;
                let radial = r * ((seed + 0.5) / 10.).sqrt() * 0.87;
                let p = base + Point::at(angle.cos(), angle.sin()) * radial;
                let q = (t * 3. + seed * 0.381966).fract();
                let z = 4. + 64. * (1. - q);
                feather(
                    mesh,
                    p + Point::at(-8., 4.) * (1. - q),
                    p + Point::at(2., -1.),
                    z + 13.,
                    z,
                    1.9,
                    color,
                );
            }
        }
        Shape::Trap => {
            band(mesh, base, r, 2.1, 2., 0., 0., TAU, color);
            for i in 0..4 {
                let a = TAU * i as f32 / 4. + elapsed * 0.55;
                mesh.face(
                    &[
                        pos(base, a - 0.20, r * 0.72, 2.),
                        pos(base, a, r * 0.92, 4. + 4. * m),
                        pos(base, a + 0.20, r * 0.72, 2.),
                    ],
                    color,
                );
            }
        }
        Shape::Burst => {
            let radius = r * (0.23 + 0.77 * t);
            band(
                mesh,
                base,
                radius,
                (r * 0.09 * remaining).max(1.),
                2.,
                0.,
                heading,
                TAU,
                color,
            );
            for i in 0..7 {
                let a = i as f32 * TAU / 7. + heading;
                let dir = Point::at(a.cos(), a.sin());
                feather(
                    mesh,
                    base + dir * radius * 0.84,
                    base + dir * radius,
                    3.,
                    4. + 29. * m,
                    (3. + r * 0.028) * remaining + 0.7,
                    color,
                );
            }
        }
        Shape::Phoenix => {
            let flight = fx.from + path * (0.15 + 0.85 * t);
            phoenix(
                mesh,
                flight,
                heading,
                r.clamp(21., 74.) * (0.67 + m * 0.33),
                t,
                color,
            );
        }
    }
}
