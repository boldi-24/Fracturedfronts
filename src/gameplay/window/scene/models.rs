//! Small articulated character meshes, built in their own forward/side/up coordinates.
use super::{Mesh, Vec3};
use crate::characters::roster::Hero;
use crate::gameplay::simulation::Unit;
use crate::ui::theme::rgb;


#[path = "spell_models.rs"]
mod spell_models;
pub(super) use spell_models::effect;

type P = [f32; 3];

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
}

// Each face stores three u16 vertex indices, its original material RGB and limb group.
struct Model {
    vertices: Vec<P>,
    faces: Vec<([usize; 3], u32, u8)>,
}

fn model(hero: Hero) -> &'static Model {
    use std::sync::OnceLock;
    static MODELS: [OnceLock<Model>; 2] = [OnceLock::new(), OnceLock::new()];
    MODELS[hero as usize].get_or_init(|| {
        let bytes: &[u8] = match hero {
            Hero::One => include_bytes!("../../../../assets/blue.mesh"),
            Hero::Two => include_bytes!("../../../../assets/red.mesh"),
        };
        let count = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
        let (points, triangles) = bytes[4..].split_at(count * 12);
        let vertices = points.chunks_exact(12).map(|p| std::array::from_fn(|i|
            f32::from_le_bytes(p[4*i..4*i+4].try_into().unwrap()))).collect();
        let faces = triangles.chunks_exact(10).map(|f| {
            let indices = std::array::from_fn(|i| u16::from_le_bytes([f[2*i], f[2*i+1]]) as usize);
            (indices, rgb(f[6], f[7], f[8]), f[9])
        }).collect();
        Model { vertices, faces }
    })
}

pub(super) fn hero(mesh: &mut Mesh, unit: &Unit, elapsed: f32) {
    let Some(id) = unit.kind.hero() else { return };
    let gait = if unit.moving { (elapsed * 12.).sin() } else { 0. };
    let rig = Rig { x: unit.position.x, y: unit.position.y, dx: unit.facing.x,
        dy: unit.facing.y, scale: 1., bob: gait.abs() * 0.65 };
    let model = model(id);
    for &(indices, color, limb) in &model.faces {
        let points = indices.map(|index| {
            let mut p = model.vertices[index];
            let sign = if limb % 2 == 0 { 1. } else { -1. };
            if limb == 1 || limb == 2 {
                p[0] += gait * sign * (23. - p[2]).max(0.) * 0.20;
            } else if limb >= 3 {
                let swing = -gait * sign * 0.13 + unit.attack_animation * 0.35 + unit.cast_animation * 0.45;
                p[0] += swing * (39. - p[2]).max(0.);
            }
            p
        });
        rig.face(mesh, points, color);
    }
    if unit.cast_animation > 0. {
        mesh.translucent = true;
        spell_models::cast_release(mesh, unit, rgb(118,191,219), unit.cast_animation);
        mesh.translucent = false;
    }
}
