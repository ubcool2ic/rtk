//! Colormaps for scientific visualization, heatmaps, and data rendering.

use crate::Color;

/// Supported color palettes for scalar data mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colormap {
    /// Perceptually uniform blue-green-yellow (matplotlib default).
    Viridis,
    /// Perceptually uniform black-red-yellow.
    Inferno,
    /// Perceptually uniform black-purple-pink.
    Magma,
    /// Perceptually uniform blue-purple-orange-yellow.
    Plasma,
    /// Classic rainbow (blue-cyan-yellow-red).
    Jet,
    /// High-contrast rainbow with a dark blue start.
    Turbo,
    /// Linear black-to-white ramp.
    Grayscale,
}

/// Control points for Viridis (17 points across [0, 1], sampled from matplotlib's reference data).
const VIRIDIS: [(u8, u8, u8); 17] = [
    (68, 1, 84),
    (72, 24, 106),
    (71, 45, 123),
    (66, 64, 134),
    (59, 82, 139),
    (51, 99, 141),
    (44, 114, 142),
    (38, 130, 142),
    (33, 145, 140),
    (31, 160, 136),
    (40, 174, 128),
    (63, 188, 115),
    (94, 201, 98),
    (132, 212, 75),
    (173, 220, 48),
    (216, 226, 25),
    (253, 231, 37),
];

/// Control points for Inferno (17 points across [0, 1], sampled from matplotlib's reference data).
const INFERNO: [(u8, u8, u8); 17] = [
    (0, 0, 4),
    (11, 7, 36),
    (33, 12, 74),
    (61, 9, 101),
    (87, 16, 110),
    (113, 25, 110),
    (138, 34, 106),
    (163, 44, 97),
    (188, 55, 84),
    (210, 70, 68),
    (228, 90, 49),
    (241, 115, 29),
    (249, 142, 9),
    (252, 172, 17),
    (249, 203, 53),
    (242, 234, 105),
    (252, 255, 164),
];

/// Control points for Magma (17 points across [0, 1], sampled from matplotlib's reference data).
const MAGMA: [(u8, u8, u8); 17] = [
    (0, 0, 4),
    (10, 8, 34),
    (29, 17, 71),
    (54, 16, 107),
    (81, 18, 124),
    (106, 28, 129),
    (131, 38, 129),
    (156, 46, 127),
    (183, 55, 121),
    (208, 65, 111),
    (231, 82, 99),
    (245, 107, 92),
    (252, 137, 97),
    (254, 167, 114),
    (254, 196, 136),
    (253, 226, 163),
    (252, 253, 191),
];

/// Control points for Plasma (17 points across [0, 1], sampled from matplotlib's reference data).
const PLASMA: [(u8, u8, u8); 17] = [
    (13, 8, 135),
    (49, 5, 151),
    (76, 2, 161),
    (102, 0, 167),
    (126, 3, 168),
    (149, 17, 161),
    (170, 35, 149),
    (188, 53, 135),
    (204, 71, 120),
    (218, 90, 106),
    (230, 108, 92),
    (240, 128, 78),
    (248, 149, 64),
    (253, 172, 51),
    (253, 197, 39),
    (248, 223, 37),
    (240, 249, 33),
];

/// Control points for Turbo (17 points across [0, 1], sampled from matplotlib's reference data).
const TURBO: [(u8, u8, u8); 17] = [
    (48, 18, 59),
    (64, 64, 162),
    (70, 107, 227),
    (66, 148, 255),
    (40, 188, 235),
    (24, 221, 194),
    (50, 242, 152),
    (109, 254, 98),
    (164, 252, 60),
    (205, 236, 52),
    (238, 207, 58),
    (253, 172, 52),
    (251, 126, 33),
    (235, 80, 14),
    (208, 47, 5),
    (169, 22, 1),
    (122, 4, 3),
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
