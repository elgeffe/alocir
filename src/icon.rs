/// Generate the app icon as raw RGBA pixel data (256x256).
/// Matches the SVG logo: SNES grey background with treemap blocks in
/// purple, blue, red, green, yellow.
///
/// Includes ~10% transparent padding on all sides and uses a macOS-style
/// squircle (superellipse) for the background shape.
pub fn app_icon() -> eframe::egui::IconData {
    const SIZE: u32 = 256;
    const PAD: u32 = 26;
    const CONTENT: u32 = SIZE - PAD * 2; // 204
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    let bg = [200u8, 196, 203, 255]; // #C8C4CB
    let purple = [107u8, 77, 153, 255]; // #6B4D99
    let blue = [72u8, 72, 204, 255]; // #4848CC
    let red = [204u8, 60, 60, 255]; // #CC3C3C
    let green = [60u8, 158, 60, 255]; // #3C9E3C
    let yellow = [212u8, 184, 48, 255]; // #D4B830
    let transparent = [0u8, 0, 0, 0];

    let cx = PAD as f32 + CONTENT as f32 / 2.0;
    let cy = cx;
    let half = CONTENT as f32 / 2.0;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;

            let color = if in_squircle(x as f32, y as f32, cx, cy, half, half) {
                if in_rect(x, y, PAD + 28, PAD + 28, 88, 96) {
                    purple
                } else if in_rect(x, y, PAD + 119, PAD + 28, 57, 56) {
                    blue
                } else if in_rect(x, y, PAD + 119, PAD + 88, 57, 36) {
                    red
                } else if in_rect(x, y, PAD + 28, PAD + 128, 88, 48) {
                    green
                } else if in_rect(x, y, PAD + 119, PAD + 128, 57, 48) {
                    yellow
                } else {
                    bg
                }
            } else {
                transparent
            };

            rgba[idx] = color[0];
            rgba[idx + 1] = color[1];
            rgba[idx + 2] = color[2];
            rgba[idx + 3] = color[3];
        }
    }

    eframe::egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

/// macOS icon shape: superellipse (squircle) with exponent ~5.
/// Formula: |((x-cx)/rx)|^n + |((y-cy)/ry)|^n <= 1
fn in_squircle(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> bool {
    let nx = ((x - cx) / rx).abs();
    let ny = ((y - cy) / ry).abs();
    // n=5 closely matches the macOS Sonoma/Sequoia icon mask.
    nx.powi(5) + ny.powi(5) <= 1.0
}

fn in_rect(x: u32, y: u32, rx: u32, ry: u32, rw: u32, rh: u32) -> bool {
    x >= rx && x < rx + rw && y >= ry && y < ry + rh
}
