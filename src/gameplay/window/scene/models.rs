//! Small articulated character meshes, built in their own forward/side/up coordinates.
use super::{Mesh, Vec3};
use crate::characters::roster::Hero;
use crate::gameplay::simulation::Unit;
use crate::ui::theme::rgb;
use std::f32::consts::{PI, TAU};

#[path = "spell_models.rs"]
mod spell_models;
pub(super) use spell_models::effect;

type P = [f32; 3];

const GOLD: u32 = rgb(233, 168, 63);
const SILVER: u32 = rgb(178, 169, 208);
const IRON: u32 = rgb(57, 49, 73);
const FUR: u32 = rgb(84, 74, 94);
const IVORY: u32 = rgb(236, 225, 211);
const NIGHT: u32 = rgb(56, 30, 90);
const PURPLE: u32 = rgb(163, 77, 244);
const SUN: u32 = rgb(255, 220, 103);
const BLOND: u32 = rgb(222, 184, 136);
const PALE: u32 = rgb(231, 186, 165);
const LILAC: u32 = rgb(186, 171, 217);
const ROSE: u32 = rgb(158, 43, 58);

struct Costume {
    metal: u32,
    cloth: u32,
    lining: u32,
    hair: u32,
    light: u32,
    width: f32,
    cape: f32,
}

impl Costume {
    fn for_hero(hero: Hero) -> Self {
        // Plate, mantle, contrasting trim, hair, magic, shoulder width and cape length.
        let (metal, cloth, lining, hair, light, width, cape) = match hero {
            Hero::Taran => (IRON, FUR, SILVER, rgb(63, 43, 48), PURPLE, 1.22, 0.48),
            Hero::Nera => (SILVER, NIGHT, LILAC, LILAC, rgb(186, 140, 255), 0.83, 0.72),
            Hero::Aelor => (IRON, NIGHT, SILVER, rgb(71, 56, 88), PURPLE, 1.10, 1.00),
            Hero::Arxen => (
                rgb(86, 54, 114),
                rgb(41, 26, 63),
                SILVER,
                LILAC,
                PURPLE,
                0.90,
                0.74,
            ),
            Hero::Aurelia => (GOLD, IVORY, ROSE, BLOND, SUN, 0.82, 0.79),
            Hero::Helvara => (IVORY, rgb(228, 214, 181), GOLD, BLOND, SUN, 1.10, 0.95),
            Hero::Kaelith => (
                rgb(98, 71, 127),
                NIGHT,
                PURPLE,
                LILAC,
                rgb(212, 95, 253),
                0.82,
                0.61,
            ),
            Hero::Luness => (GOLD, IVORY, rgb(255, 236, 170), BLOND, SUN, 0.95, 1.02),
            Hero::Nyxalune => (
                rgb(89, 66, 127),
                NIGHT,
                SILVER,
                LILAC,
                rgb(145, 103, 255),
                0.81,
                1.03,
            ),
            Hero::Selune => (
                SILVER,
                rgb(156, 120, 199),
                IVORY,
                rgb(216, 192, 231),
                rgb(229, 163, 254),
                0.86,
                1.05,
            ),
            Hero::Solviana => (GOLD, ROSE, IVORY, BLOND, SUN, 0.98, 1.00),
            Hero::Veyrion => (
                rgb(70, 53, 99),
                rgb(63, 40, 93),
                SILVER,
                rgb(59, 51, 68),
                PURPLE,
                1.27,
                1.05,
            ),
            Hero::Zyraen => (IVORY, ROSE, GOLD, BLOND, SUN, 0.91, 0.72),
            Hero::Elorim => (GOLD, IVORY, rgb(61, 48, 64), BLOND, SUN, 0.95, 1.04),
        };
        Self {
            metal,
            cloth,
            lining,
            hair,
            light,
            width,
            cape,
        }
    }
}

// The rig places the same authored meshes at any heading in world space.
struct Rig {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    scale: f32,
    bob: f32,
}
impl Rig {
    fn point(&self, [f, s, z]: P) -> Vec3 {
        Vec3::new(
            self.x + self.scale * (f * self.dx - s * self.dy),
            self.y + self.scale * (f * self.dy + s * self.dx),
            (z + self.bob) * self.scale,
        )
    }
    fn face<const N: usize>(&self, mesh: &mut Mesh, verts: [P; N], color: u32) {
        let [a, b, c, ..] = &verts[..] else {
            return;
        };
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let (nx, ny, nz) = (
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        );
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
        // Rotate the local surface normal into the world before applying sky light.
        let wx = nx * self.dx - ny * self.dy;
        let wy = nx * self.dy + ny * self.dx;
        let light = (0.31 * wx - 0.54 * wy + 0.78 * nz) / len;
        let lit = super::shade(color, 0.85 + 0.27 * light);
        mesh.face(&verts.map(|p| self.point(p)), lit);
    }
    fn limb(&self, mesh: &mut Mesh, a: P, b: P, r: f32, color: u32) {
        mesh.segment(self.point(a), self.point(b), r * self.scale, color);
    }
    fn jewel(&self, mesh: &mut Mesh, p: P, r: f32, height: f32, color: u32) {
        mesh.gem(self.point(p), r * self.scale, height * self.scale, color);
    }
    // Elliptical rings contour the boots, torso, cheekbones, pauldrons and hair.
    fn shell(&self, mesh: &mut Mesh, origin: P, rings: &[(f32, f32, f32)], color: u32) {
        const N: usize = 12;
        let vertex = |ring: (f32, f32, f32), i: usize| {
            let (sin, cos) = (TAU * i as f32 / N as f32).sin_cos();
            [
                origin[0] + ring.1 * cos,
                origin[1] + ring.2 * sin,
                origin[2] + ring.0,
            ]
        };
        for pair in rings.windows(2) {
            for i in 0..N {
                let j = (i + 1) % N;
                self.face(
                    mesh,
                    [
                        vertex(pair[0], i),
                        vertex(pair[0], j),
                        vertex(pair[1], j),
                        vertex(pair[1], i),
                    ],
                    color,
                );
            }
        }
    }
}

pub(super) fn hero(mesh: &mut Mesh, unit: &Unit, elapsed: f32) {
    let Some(id) = unit.kind.hero() else { return };
    let c = Costume::for_hero(id);
    let gait = if unit.moving {
        (elapsed * 12.).sin()
    } else {
        0.
    };
    let cast = unit.cast_animation.clamp(0., 1.);
    let attack = unit.attack_animation.clamp(0., 1.);
    let reach = ((1. - attack) * PI).sin().max(0.) * if attack > 0. { 1. } else { 0. };
    let lift = ((1. - cast) * PI).sin().max(0.) * if cast > 0. { 1. } else { 0. };
    // Ultimate poses follow the weapon: bow draw, shield brace, sword sweep or lifted focus.
    let ultimate = if unit.cast_slot == 3 { lift } else { 0. };
    let twist = ultimate * match id {
        Hero::Zyraen | Hero::Kaelith => 1.35,
        Hero::Aelor | Hero::Taran | Hero::Solviana => 0.95,
        Hero::Arxen | Hero::Nera | Hero::Aurelia => -0.48,
        _ => 0.20,
    };
    let (sin, cos) = twist.sin_cos();
    let rig = Rig {
        x: unit.position.x,
        y: unit.position.y,
        dx: unit.facing.x * cos - unit.facing.y * sin,
        dy: unit.facing.y * cos + unit.facing.x * sin,
        scale: 1.,
        bob: 0.7 * gait.abs() + 0.32 * (elapsed * 2.3 + id as usize as f32).sin(),
    };
    let w = c.width;
    let armored = matches!(
        id,
        Hero::Taran | Hero::Aelor | Hero::Helvara | Hero::Veyrion | Hero::Solviana
    );
    let robed = matches!(
        id,
        Hero::Luness | Hero::Nyxalune | Hero::Selune | Hero::Elorim
    );
    for sign in [-1., 1.] {
        let step = gait * sign;
        let hip = [0., 3.5 * w * sign, 21.];
        let knee = [2. + 2.8 * step, 4. * w * sign, 12. + step.max(0.) * 2.];
        let ankle = [-0.8 + 5.2 * step, 4.8 * w * sign, 3. + step.max(0.) * 3.];
        if armored {
            rig.shell(mesh, knee, &[(-3., 2.5, 2.6), (0., 4.2, 3.5), (2.6, 2.2, 2.7)], c.metal);
        }
        rig.limb(mesh, hip, knee, 3.4, c.cloth);
        rig.limb(mesh, knee, ankle, 2.7, c.metal);
        rig.shell(
            mesh,
            [ankle[0] + 1.3, ankle[1], ankle[2] - 2.4],
            &[(0., 4.2, 2.8), (2., 4.4, 2.8), (4.5, 2.3, 2.1)],
            IRON,
        );
    }
    let skirt = if robed { 4. } else { 17. };
    rig.shell(
        mesh,
        [0., 0., 0.],
        &[
            (skirt, 6.1, w * 8.),
            (21., 4., w * 5.3),
            (24., 4.1, w * 5.5),
        ],
        c.cloth,
    );
    rig.shell(
        mesh,
        [0., 0., 0.],
        &[
            (22., 4.5, w * 6.1),
            (24., 4.5, w * 6.1),
            (24.8, 4.1, w * 6.),
        ],
        c.lining,
    );
    rig.shell(
        mesh,
        [0., 0., 0.],
        &[(24.5, 4., w * 5.7), (33., 5.1, w * 8.), (35.5, 3.4, w * 5.)],
        c.metal,
    );
    // Split cape panels sway independently and reveal a contrasting inner hem.
    for side in [-1., 1.] {
        let flutter = (elapsed * 3.8 + side).sin() * (if unit.moving { 2.4 } else { 1. });
        let z = 35. - 30. * c.cape;
        let shoulder = [-2.7, side * 7.4 * w, 35.];
        let hem = [
            -8. - 4. * c.cape + flutter,
            side * (10. * w + flutter),
            z + 1.8,
        ];
        let notch = [-11. + flutter, side * 2.2, z - 1.];
        rig.face(mesh, [shoulder, hem, notch, [-4., 0., 34.]], c.cloth);
        rig.face(
            mesh,
            [shoulder, hem, [hem[0], hem[1] - side * 1.7, hem[2] + 1.]],
            c.lining,
        );
    }
    rig.jewel(mesh, [4.7, 0., 31.], 2.7, 5., c.light);
    for s in [-1., 1.] {
        let shoulder = [0., s * 7.5 * w, 33.];
        let elbow = [1. + s * gait, s * (9.3 * w + lift * 2.), 26. + lift * 5.];
        let mut hand = [
            5.7 + reach * 8. + lift * 4.,
            s * (10. * w + lift * 3.),
            25. + lift * 12. + reach * 2.,
        ];
        if robed {
            hand[2] += 10. * ultimate;
            hand[1] += s * 6. * ultimate;
        } else {
            hand[0] += s * 8. * ultimate;
        }
        rig.limb(mesh, shoulder, elbow, 2.5, c.cloth);
        rig.limb(mesh, elbow, hand, 2.25, c.metal);
        rig.jewel(mesh, hand, 2.6, 4.3, PALE);
        let size = if armored { 5.9 } else { 4.0 };
        rig.shell(
            mesh,
            shoulder,
            &[
                (-2., size, size * 0.88),
                (0.7, size * 1.1, size),
                (4.5, 1.2, 2.),
            ],
            c.metal,
        );
        rig.face(
            mesh,
            [
                [1.8, s * 6.7 * w, 35.],
                [0., s * (7.6 * w + size), 33.],
                [0., s * (9. * w + size), 37. + size * 0.35],
            ],
            c.lining,
        );
        weapon(mesh, &rig, id, &c, s, hand, elapsed);
    }
    rig.shell(
        mesh,
        [0.4, 0., 35.5],
        &[(0., 2., 2.1), (2., 2.3, 2.3), (4., 3.0, 2.7)],
        PALE,
    );
    rig.shell(
        mesh,
        [1., 0., 0.],
        &[
            (38., 2.8, 2.9),
            (40.5, 4., 4.),
            (44., 4.4, 4.3),
            (46., 3.4, 3.8),
        ],
        PALE,
    );
    rig.shell(
        mesh,
        [0., 0., 0.],
        &[
            (42.8, 4.1, 4.65),
            (46., 4.5, 4.8),
            (48., 2.9, 3.4),
            (48.6, 0.1, 0.1),
        ],
        c.hair,
    );
    for s in [-1., 1.] {
        // Brow, inset eye and a raised nose make the face legible at closer scales.
        rig.face(
            mesh,
            [
                [4.3, s * 3.0, 43.8],
                [5.2, s * 0.6, 43.7],
                [5.3, s * 1.9, 42.8],
            ],
            c.hair,
        );
        rig.face(
            mesh,
            [
                [5.3, s * 0.9, 43.1],
                [5.2, s * 2.3, 43.3],
                [5.1, s * 1.7, 42.6],
            ],
            IRON,
        );
        rig.face(
            mesh,
            [[5.1, s * 0.65, 43.2], [6.0, 0., 41.5], [5.1, s * 0.8, 41.3]],
            PALE,
        );
        rig.face(
            mesh,
            [
                [2.8, s * 6.8 * w, 34.],
                [5.1, s * 2., 31.],
                [4.9, s * 2.5, 29.9],
                [2.2, s * 7.2 * w, 34.4],
            ],
            c.lining,
        );
    }
    let long = matches!(
        id,
        Hero::Kaelith
            | Hero::Nera
            | Hero::Aurelia
            | Hero::Helvara
            | Hero::Nyxalune
            | Hero::Selune
            | Hero::Solviana
    );
    for s in [-1., 1.] {
        rig.face(
            mesh,
            [
                [-2., s * 3.5, 46.5],
                [1.4, s * 4.35, 45.],
                [-4.5, s * 4.6, if long { 32. } else { 40. }],
            ],
            c.hair,
        );
        if long {
            rig.face(
                mesh,
                [
                    [-2.8, s * 3.5, 45.],
                    [-3.9, 0., 45.],
                    [-7., s * 3., 33.],
                    [-6., s * 5., 29. + s * gait],
                ],
                c.hair,
            );
        }
    }
    // Layered tabards and overlapping plates catch the light as the body turns.
    for side in [-1., 1.] {
        let panel = |z, f, y| [f, side * y * w, z];
        if armored {
            for i in 0..3 {
                let z = 22. - 2.9 * i as f32;
                let spread = 5.6 + i as f32 * 0.8;
                rig.face(mesh, [panel(z, 3.5, 2.2), panel(z, 1.3, spread),
                    panel(z - 3., 1., spread + 1.2), panel(z - 4., 4.6, 2.9)], c.metal);
                rig.limb(mesh, panel(z - 4., 4.6, 2.9), panel(z - 3., 1., spread + 1.2), 0.55, c.lining);
            }
        } else {
            let hem = if robed { 3. } else { 14. };
            let sway = gait * 1.6 + (elapsed * 2.6 + side).sin() * 0.7;
            let tip = panel(hem, 6.7 + sway, 5.);
            rig.face(mesh, [panel(23., 4.6, 1.), panel(23., 2.9, 5.), tip, panel(hem + 2., 7., 1.)], c.cloth);
            rig.limb(mesh, panel(23., 4.6, 1.), tip, 0.65, c.lining);
        }
        if id.solar() {
            // Laurel leaves on the temples are visible from the inclined camera.
            for leaf in 0..3 {
                let d = leaf as f32;
                rig.face(mesh, [[3.8-d*1.8, side*4., 44.8+d*0.6],
                    [2.-d*1.8, side*5.4, 46.+d*0.6],
                    [1.2-d*1.8, side*4.8, 49.+d*0.6]], GOLD);
            }
        }
    }
    signature(mesh, &rig, id, &c, elapsed);
    if cast > 0. {
        mesh.translucent = true;
        spell_models::cast_release(mesh, unit, id, c.light, cast);
        mesh.translucent = false;
    }
}

fn weapon(mesh: &mut Mesh, r: &Rig, id: Hero, c: &Costume, s: f32, h: P, t: f32) {
    let rel = |f: f32, y: f32, z: f32| [h[0] + f, h[1] + s * y, h[2] + z];
    let bow = matches!(id, Hero::Nera | Hero::Arxen | Hero::Aurelia);
    let dual = matches!(id, Hero::Kaelith | Hero::Zyraen);
    if bow && s > 0. {
        for v in [-1., 1.] {
            let nock = rel(-3., 4., v * 16.);
            r.limb(mesh, h, rel(4., 2., v * 8.), 1.55, c.lining);
            r.limb(mesh, rel(4., 2., v * 8.), nock, 1.15, c.metal);
            r.limb(mesh, nock, rel(-4., 2., 0.), 0.45, c.light);
            r.face(
                mesh,
                [
                    rel(3., 1., v * 7.),
                    rel(5., 2., v * 6.),
                    rel(5., 4., v * 13.),
                ],
                c.lining,
            );
        }
    } else if s > 0. && matches!(id, Hero::Helvara | Hero::Veyrion) {
        let crown = rel(1., 4., 11.);
        let point = rel(0., 4., -13.);
        let boss = rel(5., 7., 0.);
        for f in [-1., 1.] {
            let wing = rel(f * 8., 3., 6.);
            r.face(mesh, [crown, wing, boss], c.lining);
            r.face(mesh, [wing, point, boss], c.metal);
            r.face(mesh, [wing, rel(f * 5., 3., 7.), point], c.lining);
        }
        r.jewel(mesh, boss, 2.3, 6., c.light);
    } else if s < 0. && id == Hero::Luness {
        r.limb(mesh, rel(-1., 0., -21.), rel(1., 0., 23.), 1.4, GOLD);
        for v in [-1., 1.] {
            r.limb(mesh, rel(1., 0., 17.), rel(2., 5. * v, 23.), 1.3, GOLD);
            r.limb(mesh, rel(2., 5. * v, 23.), rel(1., 1.8 * v, 27.), 1.2, GOLD);
        }
        r.jewel(mesh, rel(1., 0., 23.), 3.3, 7., SUN);
    } else if matches!(id, Hero::Nyxalune | Hero::Selune | Hero::Elorim) && s < 0. {
        let orb = rel(3., 1., 7. + (t * 3.1).sin() * 1.5);
        r.jewel(
            mesh,
            orb,
            if id == Hero::Elorim { 5.2 } else { 3.8 },
            7.5,
            c.light,
        );
        if id == Hero::Elorim {
            for i in 0..12 {
                let p = |j: usize| {
                    let (si, co) = (j as f32 * TAU / 12. + t * 1.9).sin_cos();
                    [orb[0] + co * 6.5, orb[1] + si * 4.3, orb[2] + si * 4.6]
                };
                r.limb(mesh, p(i), p(i + 1), 0.55, GOLD);
            }
        }
    } else if dual
        || (s < 0.
            && matches!(
                id,
                Hero::Taran | Hero::Aelor | Hero::Veyrion | Hero::Helvara | Hero::Solviana
            ))
    {
        let length = if dual {
            15.
        } else if id == Hero::Solviana {
            32.
        } else {
            26.
        };
        let wide = if id == Hero::Taran {
            4.7
        } else if dual {
            2.7
        } else {
            3.4
        };
        let tip = rel(length, if dual { 4. } else { 0. }, 10.);
        let spine = rel(length * 0.4, 0., 6.);
        let base = rel(3., 0., 1.8);
        let edge1 = rel(length * 0.30, wide, 3.);
        let edge2 = rel(length * 0.30, -wide, 3.);
        r.face(mesh, [base, edge1, tip, spine], c.lining);
        r.face(mesh, [base, spine, tip, edge2], c.metal);
        r.limb(mesh, rel(2., -4., 1.), rel(2., 4., 1.), 1.1, c.lining);
    }
}

fn signature(mesh: &mut Mesh, r: &Rig, id: Hero, c: &Costume, t: f32) {
    match id {
        Hero::Taran | Hero::Veyrion | Hero::Aelor => {
            for s in [-1., 1.] {
                for i in 0..3 {
                    let n = i as f32;
                    r.jewel(
                        mesh,
                        [-1.5 - n, s * (4. + n * 2.), 34.8 + n * 0.45],
                        3.4,
                        5.,
                        FUR,
                    );
                }
            }
            if id != Hero::Aelor {
                r.face(
                    mesh,
                    [[5., -3.3, 41.], [5., 3.3, 41.], [3.5, 0., 36.8]],
                    c.hair,
                );
            }
        }
        Hero::Nera => {
            r.shell(
                mesh,
                [-0.7, 0., 0.],
                &[
                    (40., 4.3, 5.),
                    (45., 5.3, 5.7),
                    (48., 4.3, 4.2),
                    (50., 0.1, 0.1),
                ],
                NIGHT,
            );
            r.face(
                mesh,
                [[4.1, 0., 48.], [5., -3.6, 44.9], [5., 3.6, 44.9]],
                SILVER,
            );
        }
        Hero::Arxen | Hero::Aurelia => {
            for i in 0..3 {
                let d = i as f32 * 1.8;
                r.limb(
                    mesh,
                    [-4.5, -2. + d, 22.],
                    [-5.5, 6. + d, 39. + d],
                    0.7,
                    c.lining,
                );
                r.jewel(mesh, [-5.5, 6. + d, 39. + d], 1.2, 4., c.light);
            }
        }
        Hero::Kaelith => {
            for s in [-1., 1.] {
                r.face(
                    mesh,
                    [[2., s * 4., 45.], [-3., s * 4.3, 44.], [-7.5, s * 7.3, 49.]],
                    SILVER,
                );
            }
        }
        Hero::Selune | Hero::Nyxalune => {
            for i in 0..5 {
                let a = i as f32 * TAU / 5.;
                r.jewel(
                    mesh,
                    [1.2 + a.cos() * 1.85, -4.1 + a.sin() * 1.85, 45.6],
                    1.5,
                    3.,
                    if id == Hero::Selune {
                        rgb(242, 154, 220)
                    } else {
                        GOLD
                    },
                );
            }
        }
        Hero::Elorim | Hero::Helvara | Hero::Solviana | Hero::Zyraen | Hero::Luness => {
            let halo = matches!(id, Hero::Helvara | Hero::Luness);
            if halo {
                for i in 0..14 {
                    let p = |j: usize| {
                        let (s, c) = (TAU * j as f32 / 14.).sin_cos();
                        [-3.9, c * 8.4, 43. + s * 8.5]
                    };
                    r.limb(mesh, p(i), p(i + 1), 0.65, GOLD);
                    if i % 2 == 0 {
                        let [x, y, z] = p(i);
                        r.limb(
                            mesh,
                            [x, y, z],
                            [x, y * 1.28, 43. + (z - 43.) * 1.28],
                            0.7,
                            SUN,
                        );
                    }
                }
            } else if id == Hero::Zyraen {
                for s in [-1., 1.] {
                    r.face(
                        mesh,
                        [
                            [-3., s * 4., 34.],
                            [-12., s * 7., 28.],
                            [-20., s * (8. + (t * 4.).sin() * 1.7), 23.],
                        ],
                        ROSE,
                    );
                }
            } else {
                r.jewel(mesh, [4.1, 0., 44.7], 1.7, 3.3, GOLD);
            }
        }
    }
}
