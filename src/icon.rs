/// Generate the app icon as raw RGBA pixel data (256x256).
/// Matches the SVG logo: SNES grey background with treemap blocks in
/// purple, blue, red, green, yellow.
pub fn app_icon() -> eframe::egui::IconData {
    const SIZE: u32 = 256;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    // SNES palette (RGBA)
    let bg = [200u8, 196, 203, 255]; // #C8C4CB
    let purple = [107u8, 77, 153, 255]; // #6B4D99
    let blue = [72u8, 72, 204, 255]; // #4848CC
    let red = [204u8, 60, 60, 255]; // #CC3C3C
    let green = [60u8, 158, 60, 255]; // #3C9E3C
    let yellow = [212u8, 184, 48, 255]; // #D4B830
    let transparent = [0u8, 0, 0, 0];

    // SVG coordinates (512x512) scaled to 256x256 (factor 0.5).
    // SVG blocks (x, y, w, h) -> 256 coords:
    //   purple:  70, 70, 220, 240  ->  35, 35, 110, 120
    //   blue:   300, 70, 142, 140  -> 150, 35,  71,  70
    //   red:    300,220, 142,  90  -> 150,110,  71,  45
    //   green:   70,320, 220, 122  ->  35,160, 110,  61
    //   yellow: 300,320, 142, 122  -> 150,160,  71,  61
    //   bg rounded rect radius: 80 -> 40

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;

            let color = if in_rounded_rect(x, y, 0, 0, SIZE, SIZE, 40) {
                if in_rounded_rect(x, y, 35, 35, 110, 120, 4) {
                    purple
                } else if in_rounded_rect(x, y, 150, 35, 71, 70, 4) {
                    blue
                } else if in_rounded_rect(x, y, 150, 110, 71, 45, 4) {
                    red
                } else if in_rounded_rect(x, y, 35, 160, 110, 61, 4) {
                    green
                } else if in_rounded_rect(x, y, 150, 160, 71, 61, 4) {
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

fn in_rounded_rect(x: u32, y: u32, rx: u32, ry: u32, rw: u32, rh: u32, radius: u32) -> bool {
    if x < rx || x >= rx + rw || y < ry || y >= ry + rh {
        return false;
    }
    let lx = x - rx;
    let ly = y - ry;
    let r = radius;
    if lx < r && ly < r {
        return distance(lx, ly, r, r) <= r as f32;
    }
    if lx >= rw - r && ly < r {
        return distance(lx, ly, rw - r - 1, r) <= r as f32;
    }
    if lx < r && ly >= rh - r {
        return distance(lx, ly, r, rh - r - 1) <= r as f32;
    }
    if lx >= rw - r && ly >= rh - r {
        return distance(lx, ly, rw - r - 1, rh - r - 1) <= r as f32;
    }
    true
}

fn distance(x1: u32, y1: u32, x2: u32, y2: u32) -> f32 {
    let dx = x1 as f32 - x2 as f32;
    let dy = y1 as f32 - y2 as f32;
    (dx * dx + dy * dy).sqrt()
}
