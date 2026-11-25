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

const IRON: u32 = rgb(49, 56, 56);
const PALE: u32 = rgb(187, 147, 116);

struct Costume {
    metal: u32, cloth: u32, lining: u32, light: u32, width: f32,
}
impl Costume {
    fn for_hero(id: Hero) -> Self {
        let khaki = id.solar();
        Self {
            metal: if khaki { rgb(115,108,80) } else { rgb(66,77,80) },
            cloth: if khaki { rgb(150,135,103) } else { rgb(78,88,93) },
            lining: rgb(61,62,50),
            light: if khaki { rgb(242,186,77) } else { rgb(118,191,219) },
            width: 0.98,
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
    let ultimate = if unit.cast_slot == 3 { lift } else { 0. };
    let twist = ultimate * -0.48;
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
    for sign in [-1., 1.] {
        let step = gait * sign;
        let hip = [0., 3.5 * w * sign, 21.];
        let knee = [2. + 2.8 * step, 4. * w * sign, 12. + step.max(0.) * 2.];
        let ankle = [-0.8 + 5.2 * step, 4.8 * w * sign, 3. + step.max(0.) * 3.];
        rig.limb(mesh, hip, knee, 3.4, c.cloth);
        rig.limb(mesh, knee, ankle, 2.7, c.metal);
        rig.shell(
            mesh,
            [ankle[0] + 1.3, ankle[1], ankle[2] - 2.4],
            &[(0., 4.2, 2.8), (2., 4.4, 2.8), (4.5, 2.3, 2.1)],
            IRON,
        );
    }
    let skirt = 18.;
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
    for side in [-1.,1.] {
        rig.shell(mesh,[4.3,side*3.1*w,26.], &[(0.,2.,2.5),(4.5,2.,2.5),(5.,1.8,2.3)],c.lining);
    }
    for s in [-1., 1.] {
        let shoulder = [0., s * 7.5 * w, 33.];
        let elbow = [1. + s * gait, s * (9.3 * w + lift * 2.), 26. + lift * 5.];
        let mut hand = [
            5.7 + reach * 8. + lift * 4.,
            s * (10. * w + lift * 3.),
            25. + lift * 12. + reach * 2.,
        ];

            hand[0] += s * 8. * ultimate;
        rig.limb(mesh, shoulder, elbow, 2.5, c.cloth);
        rig.limb(mesh, elbow, hand, 2.25, c.metal);
        rig.jewel(mesh, hand, 2.6, 4.3, PALE);
        let size = 4.0;
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
        if s < 0. {
            let p = |x, y, z| [hand[0]+x,hand[1]+y,hand[2]+z];
            let length = 23.;
            rig.limb(mesh,p(-7.,0.,2.),p(8.,0.,2.),2.1,IRON);
            rig.limb(mesh,p(8.,0.,2.),p(length,0.,2.),1.,IRON);
            rig.limb(mesh,p(4.,0.,2.),p(5.,0.,-3.),1.4,c.lining);
        }
    }
    rig.shell(mesh,[0.,0.,0.],&[(36.,2.8,2.9),(39.,4.,3.9),(44.,4.1,4.2),(46.,3.,3.2)],PALE);
    rig.shell(mesh,[0.,0.,0.],&[(39.,4.7,4.9),(44.,5.3,5.2),(48.,4.3,4.4),(50.,1.,1.4)],c.metal);
    rig.limb(mesh,[4.3,-3.8,42.],[4.3,3.8,42.],1.3,IRON);
    rig.shell(mesh,[-4.2,0.,26.],&[(-3.,3.2,5.),(6.,3.6,5.6),(9.,2.4,4.8)],c.lining);
    if cast > 0. {
        mesh.translucent = true;
        spell_models::cast_release(mesh, unit, c.light, cast);
        mesh.translucent = false;
    }
}
