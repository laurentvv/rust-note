//! Procedural application icon generation for rust-note.

use eframe::egui::IconData;

/// Generates a 64x64 RGBA icon for the rust-note window and taskbar.
#[must_use]
pub fn generate_app_icon() -> IconData {
    const W: u32 = 64;
    const H: u32 = 64;
    IconData {
        rgba: render_icon_rgba(W),
        width: W,
        height: H,
    }
}

/// Renders the icon at an arbitrary pixel size by 2x2 supersampling the 64-unit
/// shader space (used for the window icon at 64 and the .ico assets up to 256).
#[must_use]
pub fn render_icon_rgba(size: u32) -> Vec<u8> {
    let scale = 64.0_f32 / size as f32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let mut acc = [0u32; 4];
            for (sx, sy) in [(0.25_f32, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                let (r, g, b, a) = pixel_shader((x as f32 + sx) * scale, (y as f32 + sy) * scale);
                acc[0] += u32::from(r);
                acc[1] += u32::from(g);
                acc[2] += u32::from(b);
                acc[3] += u32::from(a);
            }
            let idx = ((y * size + x) * 4) as usize;
            rgba[idx] = (acc[0] / 4) as u8;
            rgba[idx + 1] = (acc[1] / 4) as u8;
            rgba[idx + 2] = (acc[2] / 4) as u8;
            rgba[idx + 3] = (acc[3] / 4) as u8;
        }
    }
    rgba
}

/// Evaluates color and alpha at coordinate (x, y) for 64x64 icon canvas.
fn pixel_shader(x: f32, y: f32) -> (u8, u8, u8, u8) {
    // 1. Squircle background (SDF rounded box)
    let cx = 31.5_f32;
    let cy = 31.5_f32;
    let half_w = 27.0_f32;
    let radius = 10.0_f32;

    let dx = (x - cx).abs() - (half_w - radius);
    let dy = (y - cy).abs() - (half_w - radius);
    let dist = (dx.max(0.0_f32).hypot(dy.max(0.0_f32)) + dx.max(dy).min(0.0_f32)) - radius;

    if dist > 0.8_f32 {
        return (0, 0, 0, 0); // Fully transparent
    }

    let alpha_edge = (0.8_f32 - dist).clamp(0.0_f32, 1.0_f32);

    // Background gradient: Obsidian / Tokyo Night (#212234 to #12131d)
    let t = (y / 64.0_f32).clamp(0.0_f32, 1.0_f32);
    let mut r = (33.0_f32 * (1.0_f32 - t) + 18.0_f32 * t) as u8;
    let mut g = (35.0_f32 * (1.0_f32 - t) + 19.0_f32 * t) as u8;
    let mut b = (52.0_f32 * (1.0_f32 - t) + 29.0_f32 * t) as u8;
    let mut a = (255.0_f32 * alpha_edge) as u8;

    // Squircle border stroke
    if (-1.2_f32..=0.6_f32).contains(&dist) {
        r = 69;
        g = 71;
        b = 90;
    }

    // 2. Paper Sheet (#2e3047) with folded corner
    let paper_left = 15.0_f32;
    let paper_right = 49.0_f32;
    let paper_top = 11.0_f32;
    let paper_bottom = 53.0_f32;
    let fold_size = 8.0_f32;

    let in_paper_rect = (paper_left..=paper_right).contains(&x) && (paper_top..=paper_bottom).contains(&y);
    let in_fold_cut = x > (paper_right - fold_size)
        && y < (paper_top + fold_size)
        && ((x - (paper_right - fold_size)) + ((paper_top + fold_size) - y) > fold_size);

    if in_paper_rect && !in_fold_cut {
        // Paper background
        r = 45;
        g = 47;
        b = 68;

        // Paper border (2 source pixels wide: resolution-independent when supersampled)
        let on_border = (paper_left..=paper_left + 1.0).contains(&x)
            || (paper_right - 1.0..=paper_right).contains(&x)
            || (paper_top..=paper_top + 1.0).contains(&y)
            || (paper_bottom - 1.0..=paper_bottom).contains(&y);
        if on_border {
            r = 65;
            g = 68;
            b = 95;
        }

        // 3. Stylized Rust Letter "R" (Rust Orange #e05f38 / #fab387)
        // Vertical backbone
        let is_stem = (21.0_f32..=25.0_f32).contains(&x) && (20.0_f32..=44.0_f32).contains(&y);
        // Upper loop top
        let is_loop_top = (24.0_f32..=35.0_f32).contains(&x) && (20.0_f32..=24.0_f32).contains(&y);
        // Upper loop right
        let is_loop_right = (33.0_f32..=37.0_f32).contains(&x) && (22.0_f32..=32.0_f32).contains(&y);
        // Upper loop middle
        let is_loop_mid = (24.0_f32..=35.0_f32).contains(&x) && (29.0_f32..=33.0_f32).contains(&y);
        // Diagonal right leg
        let leg_dist = ((x - 27.0_f32) - (y - 32.0_f32)).abs();
        let is_leg = (32.0_f32..=44.0_f32).contains(&y) && leg_dist <= 2.2_f32 && (26.0_f32..=38.0_f32).contains(&x);

        if is_stem || is_loop_top || is_loop_right || is_loop_mid || is_leg {
            // Bright vibrant Rust Orange
            r = 238;
            g = 100;
            b = 58;
        } else {
            // Code lines on the right of R:
            // Line 1: Cyan (#89dceb)
            if (40.0_f32..=45.0_f32).contains(&x) && (24.0_f32..=26.0_f32).contains(&y) {
                r = 137;
                g = 220;
                b = 235;
            }
            // Line 2: Emerald (#a6e3a1)
            else if (40.0_f32..=45.0_f32).contains(&x) && (30.0_f32..=32.0_f32).contains(&y) {
                r = 166;
                g = 227;
                b = 161;
            }
            // Line 3: Lavender (#cba6f7)
            else if (40.0_f32..=45.0_f32).contains(&x) && (36.0_f32..=38.0_f32).contains(&y) {
                r = 203;
                g = 166;
                b = 247;
            }
            // Line 4: Peach/Amber underline (#fab387)
            else if (21.0_f32..=45.0_f32).contains(&x) && (47.0_f32..=49.0_f32).contains(&y) {
                r = 250;
                g = 179;
                b = 135;
            }
        }
    } else if in_paper_rect && in_fold_cut {
        // Fold corner flap
        r = 88;
        g = 91;
        b = 112;
    }

    // Full opacity inside icon
    if dist < 0.0_f32 {
        a = 255;
    }

    (r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_app_icon_validity() {
        let icon = generate_app_icon();
        assert_eq!(icon.width, 64);
        assert_eq!(icon.height, 64);
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);

        // Center pixel should be opaque and contain the orange R or paper
        let center_idx = ((32 * 64 + 32) * 4) as usize;
        let alpha = icon.rgba[center_idx + 3];
        assert_eq!(alpha, 255, "Center pixel must be completely opaque");

        // Corner pixel (0,0) must be transparent
        let corner_alpha = icon.rgba[3];
        assert_eq!(corner_alpha, 0, "Top-left corner must be transparent");
    }
}
