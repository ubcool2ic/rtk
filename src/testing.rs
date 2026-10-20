//! Golden-image (screenshot) testing helpers.
//!
//! This module renders a [`Tk`] scene to a [`Pixmap`] and compares it against a
//! checked-in PNG "golden" image. It is intended for integration tests under
//! `tests/` and works fully headlessly (no window required).
//!
//! Typical usage:
//!
//! ```no_run
//! use rtk::{testing::{assert_golden, golden_dir, GoldenOpts}, Tk};
//!
//! let mut tk = Tk::new();
//! // ... build widgets ...
//! assert_golden(&mut tk, golden_dir(), "my_scene", 320, 200, GoldenOpts::default());
//! ```
//!
//! Set the environment variable `RTK_BLESS=1` to (re)write golden images
//! instead of comparing against them.

use std::fs;
use std::path::{Path, PathBuf};

use tiny_skia::Pixmap;

use crate::Tk;

/// Options controlling golden-image comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoldenOpts {
    /// Maximum allowed per-channel difference (0..=255) for a pixel to be
    /// considered equal. Defaults to `8`.
    pub tolerance: u8,
    /// Maximum number of differing pixels allowed before the assertion fails.
    /// Defaults to `0` (pixel-exact within `tolerance`).
    pub max_diff_pixels: usize,
}

impl Default for GoldenOpts {
    /// Returns the default options: tolerance `8`, `max_diff_pixels` `0`.
    fn default() -> Self {
        Self { tolerance: 8, max_diff_pixels: 0 }
    }
}

/// Returns the directory where golden images live.
///
/// Uses the `CARGO_MANIFEST_DIR` environment variable at runtime (set by Cargo
/// when running tests) and appends `tests/golden`. Falls back to the relative
/// path `tests/golden` when the variable is not set.
pub fn golden_dir() -> PathBuf {
    match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) => PathBuf::from(dir).join("tests").join("golden"),
        None => PathBuf::from("tests").join("golden"),
    }
}

/// Compares two pixmaps channel-by-channel.
///
/// Returns `(differing_pixel_count, max_channel_delta)`. A pixel is counted as
/// differing when any of its RGBA channels differs by more than `tolerance`.
/// `max_channel_delta` is the largest absolute per-channel difference observed
/// across all pixels (0 when the images are identical). If the sizes differ,
/// the comparison is limited to the overlapping region and the size mismatch is
/// reported by the caller; here the count reflects only overlapping pixels.
pub fn compare_pixmaps(a: &Pixmap, b: &Pixmap, tolerance: u8) -> (usize, u8) {
    let width = a.width().min(b.width());
    let height = a.height().min(b.height());
    let a_data = a.data();
    let b_data = b.data();
    let a_stride = a.width() as usize * 4;
    let b_stride = b.width() as usize * 4;

    let mut diff_count = 0usize;
    let mut max_delta = 0u8;

    for y in 0..height as usize {
        let a_row = &a_data[y * a_stride..y * a_stride + width as usize * 4];
        let b_row = &b_data[y * b_stride..y * b_stride + width as usize * 4];
        for (pa, pb) in a_row.chunks_exact(4).zip(b_row.chunks_exact(4)) {
            let mut pixel_differs = false;
            for channel in 0..4 {
                let delta = pa[channel].abs_diff(pb[channel]);
                if delta > max_delta {
                    max_delta = delta;
                }
                if delta > tolerance {
                    pixel_differs = true;
                }
            }
            if pixel_differs {
                diff_count += 1;
            }
        }
    }

    (diff_count, max_delta)
}

/// Renders `tk` at `width`×`height` and compares it against `<dir>/<name>.png`.
///
/// * If `RTK_BLESS=1` is set, or the golden file does not exist, the rendered
///   image is written to `<dir>/<name>.png` (creating directories as needed)
///   and the function returns.
/// * On a size mismatch, the function panics.
/// * On a pixel mismatch exceeding `opts.max_diff_pixels`, it writes
///   `<dir>/<name>.actual.png` and `<dir>/<name>.diff.png` (differing pixels in
///   red, others dimmed) and panics with a descriptive message.
///
/// # Panics
///
/// Panics if rendering fails, the golden PNG cannot be decoded, the sizes
/// differ, or the number of differing pixels exceeds `opts.max_diff_pixels`.
pub fn assert_golden(
    tk: &mut Tk,
    dir: impl AsRef<Path>,
    name: &str,
    width: u32,
    height: u32,
    opts: GoldenOpts,
) {
    let dir = dir.as_ref();
    let actual = tk
        .render_to_pixmap(width, height)
        .unwrap_or_else(|err| panic!("failed to render golden scene '{name}': {err}"));

    let golden_path = dir.join(format!("{name}.png"));
    let bless = std::env::var("RTK_BLESS").map(|v| v == "1").unwrap_or(false);

    if bless || !golden_path.exists() {
        write_png(&golden_path, &actual, name);
        return;
    }

    let golden = Pixmap::load_png(&golden_path).unwrap_or_else(|err| {
        panic!(
            "failed to decode golden image '{}': {err}",
            golden_path.display()
        )
    });

    if golden.width() != actual.width() || golden.height() != actual.height() {
        panic!(
            "golden size mismatch for '{name}': expected {}x{} ({}), got {}x{} (rendered). \
             Re-run with RTK_BLESS=1 to update.",
            golden.width(),
            golden.height(),
            golden_path.display(),
            actual.width(),
            actual.height(),
        );
    }

    let (diff_count, max_delta) = compare_pixmaps(&golden, &actual, opts.tolerance);
    if diff_count <= opts.max_diff_pixels {
        return;
    }

    let actual_path = dir.join(format!("{name}.actual.png"));
    let diff_path = dir.join(format!("{name}.diff.png"));
    write_png(&actual_path, &actual, name);
    let diff_image = build_diff_image(&golden, &actual, opts.tolerance);
    write_png(&diff_path, &diff_image, name);

    panic!(
        "golden mismatch for '{name}': {diff_count} differing pixel(s) \
         (max channel delta {max_delta}, tolerance {}, allowed {}). \
         Golden: {} | Actual: {} | Diff: {}. \
         Re-run with RTK_BLESS=1 to update.",
        opts.tolerance,
        opts.max_diff_pixels,
        golden_path.display(),
        actual_path.display(),
        diff_path.display(),
    );
}

/// Writes `pixmap` to `path`, creating parent directories as needed.
///
/// Panics if the directory cannot be created or the PNG cannot be saved.
fn write_png(path: &Path, pixmap: &Pixmap, name: &str) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!(
                    "failed to create golden directory '{}': {err}",
                    parent.display()
                )
            });
        }
    }
    pixmap
        .save_png(path)
        .unwrap_or_else(|err| panic!("failed to write golden image '{}': {err}", path.display()));
    let _ = name;
}

/// Builds a visual diff image: differing pixels are painted red, all others
/// are dimmed copies of the actual image.
///
/// The returned pixmap has the same dimensions as `actual`.
fn build_diff_image(golden: &Pixmap, actual: &Pixmap, tolerance: u8) -> Pixmap {
    let width = actual.width();
    let height = actual.height();
    let mut diff = Pixmap::new(width, height).expect("diff pixmap allocation");
    let actual_data = actual.data();
    let golden_data = golden.data();
    let stride = width as usize * 4;
    let out = diff.data_mut();

    for y in 0..height as usize {
        let row_start = y * stride;
        for x in 0..width as usize {
            let idx = row_start + x * 4;
            let pa = &actual_data[idx..idx + 4];
            let pb = &golden_data[idx..idx + 4];
            let differs = (0..4).any(|c| pa[c].abs_diff(pb[c]) > tolerance);
            let out_px = &mut out[idx..idx + 4];
            if differs {
                out_px.copy_from_slice(&[255, 0, 0, 255]);
            } else {
                // Dim the matching pixels so red stands out.
                out_px[0] = pa[0] / 3;
                out_px[1] = pa[1] / 3;
                out_px[2] = pa[2] / 3;
                out_px[3] = 255;
            }
        }
    }

    diff
}
