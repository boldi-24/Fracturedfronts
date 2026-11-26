//! Orthographic triangle rasterizer. The depth buffer is shared by the terrain, units and spells.
use super::{Camera, Point};
use std::cell::RefCell;
use windows_sys::Win32::Graphics::Gdi::*;

#[derive(Clone, Copy, Default)]
pub(super) struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub(super) fn at(p: Point, z: f32) -> Self {
        Self::new(p.x, p.y, z)
    }
    pub fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
    fn minus(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
    fn times(self, k: f32) -> Self {
        Self::new(self.x * k, self.y * k, self.z * k)
    }
    fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    fn normalized(self) -> Self {
        self.times(
            (self.x * self.x + self.y * self.y + self.z * self.z)
                .sqrt()
                .max(0.001)
                .recip(),
        )
    }
}
pub(super) fn shade(color: u32, k: f32) -> u32 {
    let channel =
        |shift: u32| ((((color >> shift) & 255) as f32 * k).clamp(0., 255.) as u32) << shift;
    channel(0) | channel(8) | channel(16)
}
#[derive(Default)]
pub(super) struct Mesh {
    fog: Vec<u32>,
    pixels: Vec<u32>,
    depth: Vec<f32>,
    width: usize,
    height: usize,
    camera: Option<Camera>,
    ratio: f32,
    pub(super) translucent: bool,
    terrain: Option<TerrainPatch>,
    pending: Option<std::thread::JoinHandle<TerrainPatch>>,
    // Dynamic geometry uses the terrain reference for both pixel alignment and depth.
    crop: (usize, usize),
}

struct TerrainPatch {
    camera: Camera,
    view_size: (usize, usize),
    ratio: f32,
    stride: usize,
    pixels: Vec<u32>,
    depth: Vec<f32>,
}

impl TerrainPatch {
    const PAD: i32 = 192;

    fn build(mut camera: Camera, ratio: f32, w: usize, h: usize) -> Self {
        let pad = Self::PAD;
        camera.area.left -= pad;
        camera.area.right += pad;
        camera.area.top -= pad;
        camera.area.bottom += pad;
        let stride = ((camera.area.right - camera.area.left) as f32 * ratio).ceil() as usize;
        let rows = ((camera.area.bottom - camera.area.top) as f32 * ratio).ceil() as usize;
        let mut terrain = Mesh {
            camera: Some(camera), ratio, width: stride, height: rows,
            pixels: vec![0x222239; stride * rows],
            depth: vec![f32::NEG_INFINITY; stride * rows], ..Mesh::default()
        };
        super::landscape::terrain(&mut terrain, camera);
        // Anchor the fine surface grain to the projected world instead of the screen.
        let (ox, oy, _) = camera.project(Vec3::new(0.,0.,0.));
        let dx = ((camera.area.left as f32-ox)*ratio).round() as i32;
        let dy = ((camera.area.top as f32-oy)*ratio).round() as i32;
        for (y,row) in terrain.pixels.chunks_exact_mut(stride).enumerate() {
            let line = ((y as i32 + dy) as u32).wrapping_mul(0x85ebca6b);
            for (x,color) in row.iter_mut().enumerate() {
                let key = ((x as i32 + dx) as u32).wrapping_mul(0x9e3779b1) ^ line;
                let grit = (key ^ (key >> 17)).wrapping_mul(0x27d4eb2d) >> 28;
                let delta = grit as i32 - 8;
                let c = |shift:u32| ((((*color >> shift) & 255) as i32 + delta).clamp(0,255) as u32) << shift;
                *color = c(0)|c(8)|c(16);
            }
        }
        Self { camera, view_size: (w, h), ratio, stride, pixels: terrain.pixels, depth: terrain.depth }
    }

    fn crop_for(&self, camera: Camera, ratio: f32, w: usize, h: usize) -> Option<(usize, usize)> {
        if self.view_size != (w,h) || self.ratio != ratio || self.camera.top != camera.top ||
            self.camera.scale != camera.scale { return None; }
        let (x, y, _) = self.camera.project(Vec3::at(camera.center, 0.));
        let left = ((x - self.camera.area.left as f32 - (camera.area.right-camera.area.left) as f32 * 0.5) * ratio).round() as i32;
        let top = ((y - self.camera.area.top as f32 - (camera.area.bottom-camera.area.top) as f32 * 0.5) * ratio).round() as i32;
        (left >= 0 && top >= 0 && left as usize + w <= self.stride &&
            top as usize + h <= self.pixels.len()/self.stride).then_some((left as usize, top as usize))
    }
}

thread_local! { static FRAME:RefCell<Mesh> = RefCell::new(Mesh::default()); }
pub(super) fn render(dc: HDC, camera: Camera, game: &super::Game, submit: impl FnOnce(&mut Mesh)) {
    FRAME.with_borrow_mut(|m| {
        let area = camera.area;
        let w = (area.right - area.left).max(1);
        let h = (area.bottom - area.top).max(1);
        let ratio = (1420. / w as f32).min(900. / h as f32).min(1.);
        m.ratio = ratio;
        m.width = (w as f32 * ratio).round().max(1.) as usize;
        m.height = (h as f32 * ratio).round().max(1.) as usize;
        let (width, height) = (m.width, m.height);
        let crop = |patch: &TerrainPatch| patch.crop_for(camera, ratio, width, height);
        if m.pending.as_ref().is_some_and(|job| job.is_finished()) {
            if let Ok(next) = m.pending.take().unwrap().join() {
                if crop(&next).is_some() { m.terrain = Some(next); }
            }
        }
        if m.terrain.as_ref().and_then(crop).is_none() {
            m.terrain = Some(TerrainPatch::build(camera, ratio, width, height));
        }
        let patch = m.terrain.as_ref().unwrap();
        let (left, top) = crop(patch).unwrap();
        m.camera = Some(patch.camera);
        m.crop = (left, top);
        let pad = TerrainPatch::PAD as f32 * ratio;
        if m.pending.is_none() && ((left as f32 - pad).abs().max((top as f32 - pad).abs()) > pad * 0.5) {
            m.pending = Some(std::thread::spawn(move || TerrainPatch::build(camera, ratio, width, height)));
        }
        m.pixels.resize(width * height, 0);
        m.depth.resize(width * height, 0.);
        for row in 0..height {
            let src = (top + row) * patch.stride + left;
            let dest = row * width..(row+1) * width;
            m.pixels[dest.clone()].copy_from_slice(&patch.pixels[src..src+width]);
            m.depth[dest].copy_from_slice(&patch.depth[src..src+width]);
        }
        // Interpolate visibility between neighboring samples at shadow edges.
        const STEP: usize = 4;
        let span = STEP as u32;
        let cols = width.div_ceil(STEP) + 1;
        let rows = height.div_ceil(STEP) + 1;
        m.fog.resize(cols * rows, 0);
        for (i, light) in m.fog.iter_mut().enumerate() {
            let ground = camera.ground(
                area.left + ((i % cols * STEP) as f32 / ratio).round() as i32,
                area.top + ((i / cols * STEP) as f32 / ratio).round() as i32,
            );
            *light = if game.sees(ground) { 256 } else { 64 };
        }
        for y in 0..height {
            let fy = (y % STEP) as u32;
            for (col, pixels) in m.pixels[y*width..(y+1)*width].chunks_mut(STEP).enumerate() {
                let i = y / STEP * cols + col;
                let left = m.fog[i] * (span - fy) + m.fog[i+cols] * fy;
                let right = m.fog[i+1] * (span - fy) + m.fog[i+cols+1] * fy;
                if left == 256 * span && right == 256 * span { continue; }
                for (x, pixel) in pixels.iter_mut().enumerate() {
                    let x = x as u32;
                    let light = (left * (span - x) + right * x) / (span * span);
                    let rb = ((*pixel & 0xff00ff) * light >> 8) & 0xff00ff;
                    let g = ((*pixel & 0x00ff00) * light >> 8) & 0x00ff00;
                    *pixel = rb | g;
                }
            }
        }
        submit(m);
        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = m.width as i32;
        info.bmiHeader.biHeight = -(m.height as i32);
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;
        unsafe {
            StretchDIBits(
                dc,
                area.left,
                area.top,
                w,
                h,
                0,
                0,
                m.width as i32,
                m.height as i32,
                m.pixels.as_ptr().cast(),
                &info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    })
}
impl Mesh {
    fn screen(&self, v: Vec3) -> Vec3 {
        let c = self.camera.unwrap();
        let (x, y, depth) = c.project(v);
        Vec3::new(
            (x - c.area.left as f32) * self.ratio - self.crop.0 as f32,
            (y - c.area.top as f32) * self.ratio - self.crop.1 as f32,
            depth,
        )
    }
    pub fn face(&mut self, points: &[Vec3], color: u32) {
        if points.len() < 3 {
            return;
        }
        let a = self.screen(points[0]);
        let mut b = self.screen(points[1]);
        let dib = ((color & 255) << 16) | (color & 0xff00) | (color >> 16);
        for p in &points[2..] {
            let c = self.screen(*p);
            self.triangle(a, b, c, dib);
            b = c;
        }
    }
    fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, color: u32) {
        let x0 = a.x.min(b.x).min(c.x).floor().max(0.) as usize;
        let y0 = a.y.min(b.y).min(c.y).floor().max(0.) as usize;
        let x1 = a.x.max(b.x).max(c.x).ceil().clamp(0., self.width as f32) as usize;
        let y1 = a.y.max(b.y).max(c.y).ceil().clamp(0., self.height as f32) as usize;
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let (dx1, dy1, dx2, dy2) = (b.x - a.x, b.y - a.y, c.x - a.x, c.y - a.y);
        let area = dx1 * dy2 - dx2 * dy1;
        if area.abs() < 0.012 {
            return;
        }
        let inv = 1. / area;
        let du = dy2 * inv;
        let dv = -dy1 * inv;
        let x = x0 as f32 + 0.5 - a.x;
        for y in y0..y1 {
            let sy = y as f32 + 0.5 - a.y;
            let mut u = (x * dy2 - sy * dx2) * inv;
            let mut v = (sy * dx1 - x * dy1) * inv;
            let start = y * self.width;
            for i in x0..x1 {
                if u >= -0.00005 && v >= -0.00005 && u + v <= 1.00005 {
                    let z = a.z + u * (b.z - a.z) + v * (c.z - a.z);
                    let idx = start + i;
                    if z >= self.depth[idx] {
                        self.depth[idx] = z;
                        self.pixels[idx] = if self.translucent {
                            // Average RGB channels separately so lights retain the underlying silhouette.
                            let background = self.pixels[idx];
                            (background & color) + (((background ^ color) & 0xfefefe) >> 1)
                        } else { color }
                    }
                }
                u += du;
                v += dv;
            }
        }
    }
    pub fn segment(&mut self, a: Vec3, b: Vec3, r: f32, color: u32) {
        let dir = b.minus(a).normalized();
        let across = dir
            .cross(if dir.z.abs() < 0.9 {
                Vec3::new(0., 0., 1.)
            } else {
                Vec3::new(0., 1., 0.)
            })
            .normalized()
            .times(r);
        let up = dir.cross(across);
        let offsets = [across, up, across.times(-1.), up.times(-1.)];
        for i in 0..4 {
            let v = offsets[i];
            let w = offsets[(i + 1) % 4];
            self.face(
                &[a.add(v), b.add(v), b.add(w), a.add(w)],
                shade(color, [1.05, 0.86, 0.65, 0.90][i]),
            );
        }
        self.face(&offsets.map(|p| b.add(p)), color);
        self.face(&offsets.map(|p| a.add(p)), color);
    }
    pub fn gem(&mut self, c: Vec3, r: f32, height: f32, color: u32) {
        let top = c.add(Vec3::new(0., 0., height / 2.));
        let low = c.add(Vec3::new(0., 0., -height / 2.));
        let rim = |i: usize| {
            let t = i as f32 * std::f32::consts::TAU / 6.;
            c.add(Vec3::new(r * t.cos(), r * t.sin(), 0.))
        };
        for i in 0..6 {
            let (a, b) = (rim(i), rim(i + 1));
            let k = 0.85 + 0.22 * (i as f32).cos();
            self.face(&[top, a, b], shade(color, k + 0.14));
            self.face(&[low, a, b], shade(color, k - 0.15));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreground_triangle_occludes_background_in_both_draw_orders() {
        let mut m = Mesh {
            width: 16,
            height: 16,
            pixels: vec![0; 256],
            depth: vec![f32::NEG_INFINITY; 256],
            ..Mesh::default()
        };
        for (z, color) in [(8., 11), (2., 22), (12., 33)] {
            m.triangle(
                Vec3::new(2., 2., z),
                Vec3::new(13., 2., z),
                Vec3::new(6., 13., z),
                color,
            );
            assert_eq!(m.pixels[6 * 16 + 6], if z <= 8. { 11 } else { 33 });
        }
    }
}
