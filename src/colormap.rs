//! Colormaps for scientific visualization, heatmaps, and data rendering.

use crate::Color;

/// Supported color palettes for scalar data mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colormap {
    Viridis,
    Inferno,
    Magma,
    Plasma,
    Jet,
    Turbo,
    Grayscale,
}

/// Control points for Viridis (17 points across [0, 1]).
const VIRIDIS: [(u8, u8, u8); 17] = [
    (68, 1, 84),
    (72, 35, 116),
    (64, 67, 135),
    (52, 94, 141),
    (41, 120, 142),
    (32, 144, 140),
    (34, 167, 132),
    (68, 190, 112),
    (121, 209, 81),
    (189, 222, 38),
    (253, 231, 37),
    (240, 228, 45),
    (210, 220, 60),
    (160, 210, 75),
    (100, 195, 95),
    (50, 170, 120),
    (30, 140, 135),
];

/// Control points for Inferno (17 points across [0, 1]).
const INFERNO: [(u8, u8, u8); 17] = [
    (0, 0, 4),
    (22, 11, 57),
    (66, 10, 104),
    (106, 23, 110),
    (147, 38, 103),
    (187, 55, 84),
    (221, 81, 58),
    (243, 118, 25),
    (252, 165, 10),
    (246, 215, 70),
    (252, 255, 164),
    (250, 240, 140),
    (240, 200, 60),
    (220, 150, 30),
    (190, 90, 60),
    (130, 40, 95),
    (40, 10, 70),
];

/// Control points for Magma (17 points across [0, 1]).
const MAGMA: [(u8, u8, u8); 17] = [
    (0, 0, 4),
    (20, 14, 54),
    (51, 16, 103),
    (82, 18, 114),
    (114, 25, 115),
    (146, 36, 110),
    (182, 54, 121),
    (215, 74, 115),
    (241, 104, 110),
    (251, 143, 115),
    (253, 186, 137),
    (252, 224, 169),
    (251, 252, 191),
    (245, 230, 160),
    (225, 170, 130),
    (170, 90, 120),
    (80, 20, 110),
];

/// Control points for Plasma (17 points across [0, 1]).
const PLASMA: [(u8, u8, u8); 17] = [
    (13, 8, 135),
    (60, 4, 156),
    (98, 2, 166),
    (132, 5, 161),
    (163, 23, 147),
    (189, 46, 128),
    (210, 71, 107),
    (227, 99, 87),
    (241, 130, 68),
    (249, 163, 50),
    (252, 199, 36),
    (240, 233, 53),
    (225, 215, 60),
    (200, 175, 80),
    (170, 120, 115),
    (125, 60, 150),
    (40, 10, 145),
];

/// Control points for Turbo (17 points across [0, 1]).
const TURBO: [(u8, u8, u8); 17] = [
    (48, 18, 59),
    (68, 81, 191),
    (66, 132, 241),
    (44, 183, 233),
    (38, 222, 188),
    (68, 246, 138),
    (119, 255, 87),
    (176, 246, 52),
    (225, 220, 47),
    (253, 178, 47),
    (254, 127, 44),
    (238, 77, 34),
    (205, 37, 23),
    (159, 13, 11),
    (122, 4, 2),
    (90, 2, 2),
    (50, 0, 1),
];

/// Interpolates linearly between points in a control point table.
fn interpolate_table(table: &[(u8, u8, u8)], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let n = (table.len() - 1) as f32;
    let scaled = t * n;
    let idx = (scaled.floor() as usize).min(table.len() - 2);
    let frac = scaled - idx as f32;
    let (r0, g0, b0) = table[idx];
    let (r1, g1, b1) = table[idx + 1];
    let r = (r0 as f32 + (r1 as f32 - r0 as f32) * frac).round().clamp(0.0, 255.0) as u8;
    let g = (g0 as f32 + (g1 as f32 - g0 as f32) * frac).round().clamp(0.0, 255.0) as u8;
    let b = (b0 as f32 + (b1 as f32 - b0 as f32) * frac).round().clamp(0.0, 255.0) as u8;
    Color::rgb(r, g, b)
}

/// Evaluates a colormap at scalar parameter `t` in `[0.0, 1.0]`. Out-of-bounds `t` is clamped.
pub fn sample(cm: Colormap, t: f32) -> Color {
    let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
    match cm {
        Colormap::Grayscale => {
            let v = (t * 255.0).round().clamp(0.0, 255.0) as u8;
            Color::rgb(v, v, v)
        }
        Colormap::Jet => {
            let r = (1.5 - (4.0 * t - 3.0).abs()).clamp(0.0, 1.0);
            let g = (1.5 - (4.0 * t - 2.0).abs()).clamp(0.0, 1.0);
            let b = (1.5 - (4.0 * t - 1.0).abs()).clamp(0.0, 1.0);
            Color::rgb(
                (r * 255.0).round() as u8,
                (g * 255.0).round() as u8,
                (b * 255.0).round() as u8,
            )
        }
        Colormap::Viridis => interpolate_table(&VIRIDIS, t),
        Colormap::Inferno => interpolate_table(&INFERNO, t),
        Colormap::Magma => interpolate_table(&MAGMA, t),
        Colormap::Plasma => interpolate_table(&PLASMA, t),
        Colormap::Turbo => interpolate_table(&TURBO, t),
    }
}

/// Maps a slice of scalar values to straight-alpha RGBA bytes (4 bytes per value).
///
/// NaN values produce fully transparent pixels `[0, 0, 0, 0]`. When `min == max`,
/// finite values map safely to `t = 0.5`.
pub fn heatmap_to_rgba(values: &[f32], min: f32, max: f32, cm: Colormap) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(values.len() * 4);
    let span = max - min;
    for &v in values {
        if v.is_nan() {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let t = if span <= 0.0 || !span.is_finite() {
                0.5
            } else {
                ((v - min) / span).clamp(0.0, 1.0)
            };
            let c = sample(cm, t);
            rgba.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }
    }
    rgba
}

/// Generates a straight-alpha RGBA image (width × height × 4 bytes) displaying a gradient legend.
///
/// When `horizontal` is true, colors ramp from left (`0.0`) to right (`1.0`).
/// When vertical, colors ramp from bottom (`0.0`) to top (`1.0`).
pub fn legend_rgba(cm: Colormap, width: u32, height: u32, horizontal: bool) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((width as usize) * (height as usize) * 4);
    for y in 0..height {
        for x in 0..width {
            let t = if horizontal {
                if width > 1 {
                    x as f32 / (width - 1) as f32
                } else {
                    0.0
                }
            } else if height > 1 {
                1.0 - (y as f32 / (height - 1) as f32)
            } else {
                0.0
            };
            let c = sample(cm, t);
            rgba.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }
    }
    rgba
}
