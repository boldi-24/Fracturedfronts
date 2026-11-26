use super::{Camera, Mesh, Point, Vec3, rgb, shade};
use crate::gameplay::geometry::WORLD_WIDTH;
use std::f32::consts::TAU;

const PLASTER: u32 = rgb(207, 176, 131);
const STEEL: u32 = rgb(66, 78, 74);
const MARK: u32 = rgb(208, 158, 68);
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
        m.face(&[Vec3::at(p, 0.42), at(i), at(i + 1)], rgb(109, 102, 87));
    }
}
fn noise(a: i32, b: i32) -> f32 {
    let mut n = (a as u32).wrapping_mul(0x27d4eb2d) ^ (b as u32).wrapping_mul(0x165667b1);
    n = (n ^ (n >> 15)).wrapping_mul(0x85ebca6b);
    ((n ^ (n >> 13)) & 0xffff) as f32 / 65535.
}

pub(super) fn terrain(m: &mut Mesh, camera: Camera) {
    use crate::gameplay::map::{CELL, COLS, ROWS, floor};
    tile(m, -4096., -4096., -2., WORLD_WIDTH+8192., WORLD_WIDTH+8192., rgb(52,48,43));
    for row in 0..ROWS {
        for col in 0..COLS {
            if !floor(col,row) { continue; }
            let x=col as f32*CELL;
            let y=row as f32*CELL;
            let p=Point::at(x+CELL/2.,y+CELL/2.);
            if !camera.visible(p,48.) {continue;}
            let slab=shade(PLASTER,0.86+0.08*noise((col/3) as i32,(row/3) as i32));
            tile(m,x,y,0.,CELL,CELL,slab);
            if col%3==0 { tile(m,x,y,0.1,0.9,CELL,shade(slab,0.77)); }
            if row%3==0 { tile(m,x,y,0.1,CELL,0.9,shade(slab,0.77)); }
            let stone=shade(rgb(120,109,91),0.96+0.09*noise(col as i32,row as i32));
            for (dx,dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
                if !floor(col.wrapping_add_signed(dx),row.wrapping_add_signed(dy)) {
                    let (w,d)=if dx==0 {(CELL+2.,4.)} else {(4.,CELL+2.)};
                    let wall=p+Point::at(dx as f32*(CELL/2.+2.),dy as f32*(CELL/2.+2.));
                    block(m,wall,0.,w,d,14.,stone);
                }
            }
        }
    }
    // The round chamber in the northeast and the columns in the southern hall.
    let shrine=Point::at(561.*crate::gameplay::map::PLAN_SCALE,102.*crate::gameplay::map::PLAN_SCALE);
    ring(m,shrine,43.,0.2,4.,MARK);
    ring(m,shrine,17.,0.2,3.,STEEL);
    for x in [302.,320.,338.,355.,373.] {
        for y in [313.,333.] {
            let base=Point::at(x*crate::gameplay::map::PLAN_SCALE,y*crate::gameplay::map::PLAN_SCALE);
            if camera.visible(base,45.) { frustum(m,base,0.,20.,6.,5.,8,0.,PLASTER); }
        }
    }
}

pub(super) fn nexus(m: &mut Mesh, p: Point, accent: u32, time: f32) {
    block(m,p,0.5,79.,63.,5.,rgb(118,116,103));
    block(m,p,5.5,67.,49.,33.,STEEL);
    block(m,p,38.5,70.,52.,3.,rgb(111,116,99));
    for j in 0..9 {
        block(m,p+Point::at(-23.+j as f32*5.,0.),41.5,2.5,27.,2.,rgb(52,57,53));
    }
    block(m,p+Point::at(0.,25.),14.,40.,2.,13.,rgb(33,42,42));
    for j in 0..4 {block(m,p+Point::at(-14.+j as f32*9.,26.),18.,5.,2.,4.,accent);}
    let mast=p+Point::at(25.,-15.);
    frustum(m,mast,41.,67.,2.,1.4,6,0.,STEEL);
    frustum(m,mast,66.,71.,3.,3.,8,0.,shade(accent,0.82+0.18*(time*3.).sin()));
}
