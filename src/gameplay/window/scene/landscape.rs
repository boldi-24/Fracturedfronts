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

// Perimeter buildings sit outside the movement boundary. The square itself is a walkable depot yard.
fn depot(m: &mut Mesh, at: Point, along: Point, inward: Point, seed: i32) {
    let pos = |x: f32, y: f32| at + along*x + inward*y;
    let piece = |m: &mut Mesh, x, y, z, w, d, h, color| {
        let (w,d) = if along.x == 0. { (d,w) } else { (w,d) };
        block(m, pos(x,y), z, w, d, h, color);
    };
    let height = 64. + noise(seed, 3)*57.;
    let c = shade(PLASTER, 0.84 + 0.19*noise(seed, 5));
    piece(m, 0., -62., 0., 184., 108., height, c);
    piece(m, 0., -62., height, 193., 118., 5., rgb(161,143,117));
    piece(m, 48., -61., height+5., 38., 25., 13., STEEL);
    for j in 0..3 {
        let x = -58. + j as f32*54.;
        piece(m,x,-6.,height-35.,31.,4.,21.,rgb(76,86,82));
        piece(m,x,-3.,height-36.,37.,7.,3.,rgb(161,141,111));
        piece(m,x,-3.,height-34.,2.,3.,20.,c);
    }
    piece(m, -35., -5., 0., 54., 5., 38., STEEL);
    for j in 1..7 { piece(m,-35.,-1.5,j as f32*5.,53.,2.,1.,rgb(96,104,96)); }
    piece(m,-35.,0.,39.,66.,20.,3.,rgb(117,128,115));
    piece(m,68.,-3.,1.,3.,4.,height-4.,rgb(120,111,91));
}

fn crate_stack(m: &mut Mesh, p: Point, height: f32) {
    block(m,p,0.,50.,41.,height,rgb(147,112,75));
    for x in [-19.,19.] {
        block(m,p+Point::at(x,0.),0.,4.,43.,height+1.,rgb(93,83,65));
    }
    for z in [3.,height-6.] {
        block(m,p,z,52.,43.,4.,rgb(177,142,96));
    }
}

pub(super) fn terrain(m: &mut Mesh, camera: Camera) {
    let size = WORLD_WIDTH;
    tile(m,-4000.,-4000.,-2.,size+8000.,size+8000.,rgb(158,146,124));
    let cell = size/16.;
    for j in 0..16 {
        for i in 0..16 {
            let x = i as f32*cell;
            let y = j as f32*cell;
            if !camera.visible(Point::at(x+cell/2.,y+cell/2.),160.) { continue; }
            let lane = (6..10).contains(&i) || (6..10).contains(&j);
            let slab = shade(if lane {rgb(161,157,142)} else {rgb(196,180,151)},0.94+0.08*noise(i,j));
            tile(m,x,y,0.,cell,cell,slab);
            tile(m,x+0.4,y,0.06,0.8,cell,shade(slab,0.73));
            tile(m,x,y+0.4,0.06,cell,0.8,shade(slab,0.78));
            for k in 0..11 {
                let u=noise(i*13+k,j+8); let v=noise(i+17,j*11+k);
                tile(m,x+3.+u*(cell-12.),y+3.+v*(cell-10.),0.1,1.+5.*v,0.7+v,shade(slab,0.85+0.22*u));
            }
            if j==6 || j==9 {tile(m,x+4.,y+if j==6 {5.} else {cell-6.},0.15,cell-11.,2.8,MARK);}
            if i==7 && j%2==0 {tile(m,x+cell-2.,y+15.,0.15,3.,42.,rgb(205,199,174));}
            if (j+i*3)%11==0 {
                let p=Point::at(x+47.,y+46.);
                tile(m,p.x,p.y,0.2,25.,13.,STEEL);
                for n in 1..7 {tile(m,p.x+n as f32*3.3,p.y+1.,0.24,1.5,11.,rgb(119,121,107));}
            }
        }
    }
    for (base,along,inward) in [(Point::at(0.,0.),Point::at(1.,0.),Point::at(0.,1.)),
        (Point::at(size,size),Point::at(-1.,0.),Point::at(0.,-1.)),
        (Point::at(0.,size),Point::at(0.,-1.),Point::at(1.,0.)),
        (Point::at(size,0.),Point::at(0.,1.),Point::at(-1.,0.))] {
        for i in 0..8 {
            let p=base+along*(96.+192.*i as f32);
            if !camera.visible(p,370.) {continue;}
            depot(m,p+inward*(-95.),along,inward,i);
            let (w,d)=if along.x!=0. {(192.,9.)} else {(9.,192.)};
            block(m,p,0.,w,d,12.,shade(PLASTER,0.79));
            if i%3!=1 {crate_stack(m,p+inward*(-33.),28.+9.*(i%2) as f32);}
        }
    }
    // Recessed service bays are flush with the floor, so units can cross the grilles.
    for u in [0.25,0.75] {
        for v in [0.25,0.75] {
            let x=size*u-78.; let y=size*v-38.;
            if !camera.visible(Point::at(x+78.,y+38.),150.) { continue; }
            tile(m,x-4.,y-4.,0.18,156.,82.,rgb(106,103,90));
            tile(m,x,y,0.22,148.,74.,rgb(59,66,62));
            for k in 0..24 {
                tile(m,x+k as f32*6.1,y,0.25,2.3,74.,rgb(125,126,111));
            }
            for k in 0..4 {tile(m,x,y+k as f32*24.,0.27,148.,2.,rgb(92,96,86));}
            for k in 0..8 {
                let start=x+k as f32*18.;
                m.face(&[vertex(start,y-9.,0.22),vertex(start+9.,y-9.,0.22),
                         vertex(start+14.,y-4.,0.22),vertex(start+5.,y-4.,0.22)],MARK);
            }
        }
    }
    // Painted loading aprons around the two objectives.
    for x in [195.,size-195.] {
        let y=size/2.;
        for s in [-1.,1.] {
            tile(m,x-62.,y+s*51.,0.3,124.,3.,MARK);
            tile(m,x+s*62.,y-51.,0.3,3.,105.,MARK);
            for i in 0..5 {tile(m,x-58.+i as f32*25.,y+s*59.,0.3,12.,4.,MARK);}
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
