//! Offscreen integration test using the same DIB blit, geometry and spell animation as the match window.
use super::*;
use crate::characters::roster::Hero;
use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::*;
#[test]
fn duel_renders_offscreen() {
    let area = RECT {
        left: 0,
        top: 0,
        right: 1220,
        bottom: 650,
    };
    let camera = Camera::at(area, Point::at(410., 100.), false);
    let mut g = Game::with_hero(Hero::Deafult);
    let player = Point::at(435., 120.);
    let enemy = Point::at(528., 136.);
    g.units[0].position = player;
    g.units[1].position = enemy;
    g.destination = player;
    g.cast(0, 3, enemy);
    for _ in 0..25 {
        g.step(1. / 60.)
    }
    assert!(!g.effects.is_empty());
    let size = (area.right * area.bottom * 4) as usize;
    let mut info = BITMAPINFO::default();
    info.bmiHeader.biSize = 40;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biWidth = area.right;
    info.bmiHeader.biHeight = -area.bottom;
    unsafe {
        let dc = CreateCompatibleDC(null_mut());
        assert!(!dc.is_null());
        let mut bits = null_mut();
        let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        assert!(!bitmap.is_null() && !bits.is_null());
        let old = SelectObject(dc, bitmap);
        draw(dc, camera, &g, false);
        GdiFlush();
        let pixels = std::slice::from_raw_parts(bits.cast::<u8>(), size);
        assert!(
            pixels
                .chunks_exact(4)
                .any(|p| p[2] > 160 && p[1] > 110 && p[0] < 100)
        );
        if let Some(path) = std::env::var_os("FENRAE_PREVIEW_BMP") {
            let mut bitmap_file = b"BM".to_vec();
            bitmap_file.extend((54 + size as u32).to_le_bytes());
            bitmap_file.extend([0; 4]);
            bitmap_file.extend(54u32.to_le_bytes());
            bitmap_file.extend(std::slice::from_raw_parts(
                (&info.bmiHeader as *const BITMAPINFOHEADER).cast::<u8>(),
                40,
            ));
            bitmap_file.extend(pixels);
            std::fs::write(path, bitmap_file).unwrap();
        }
        if std::env::var_os("FENRAE_PROFILE_FOLLOW").is_some() {
            let mut samples = Vec::new();
            for frame in 0..240 {
                let center = player + Point::at(frame as f32 * 2., 0.);
                g.units[0].position = center;
                let start = std::time::Instant::now();
                draw(dc, Camera::at(area, center, false), &g, false);
                GdiFlush();
                samples.push(start.elapsed().as_micros());
            }
            samples.sort_unstable();
            eprintln!("follow 1220x650, frame time (us): median {}, p95 {}, max {}", samples[120], samples[228], samples[239]);
        }
        SelectObject(dc, old);
        DeleteObject(bitmap);
        DeleteDC(dc);
    }
}
