//! Golden-image tests for the `Image` widget across themes and scale factors.

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
#[cfg(feature = "svg")]
use rtk::{Button, Opt};
use rtk::{Frame, Image, ImageFit, Tk};

/// Builds the shared scene: checkerboard RGBA image, SVG icon image, and a
/// button carrying a small SVG icon.
fn build_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).unwrap();
    tk.pack(frame, rtk::PackOpts::default());

    // 8x8 checkerboard, Fill.
    let checker = Image::new(tk, frame, 96.0, 96.0).unwrap();
    let mut rgba = Vec::with_capacity(8 * 8 * 4);
    for y in 0..8 {
        for x in 0..8 {
            let on = (x + y) % 2 == 0;
            let v = if on { 255 } else { 0 };
            rgba.extend_from_slice(&[v, v, v, 255]);
        }
    }
    assert!(tk.image_set_rgba(checker, 8, 8, &rgba));
    assert!(tk.image_set_fit(checker, ImageFit::Fill));
    tk.pack(checker, rtk::PackOpts::default());

    // SVG icon image, Contain.
    #[cfg(feature = "svg")]
    {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><circle cx="16" cy="16" r="14" fill="#0072B2"/><path d="M16 4 L19 12 L28 12 L21 17 L23 26 L16 21 L9 26 L11 17 L4 12 L13 12 Z" fill="#E69F00"/></svg>"##;
        let icon = Image::new(tk, frame, 96.0, 96.0).unwrap();
        assert!(tk.image_set_svg(icon, svg));
        assert!(tk.image_set_fit(icon, ImageFit::Contain));
        tk.pack(icon, rtk::PackOpts::default());

        let btn_svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><circle cx="8" cy="8" r="7" fill="#0072B2"/></svg>"##;
        let icon_id = tk.load_svg(btn_svg, 16, 16).expect("svg icon");
        let btn = Button::new(tk, frame, "Go").unwrap();
        assert!(tk.config(btn, Opt::Image(icon_id)));
        tk.pack(btn, rtk::PackOpts::default());
    }
}

/// Renders the scene for a theme at a given scale and compares to a golden PNG.
fn check_theme(theme: rtk::Theme, scale: f64, name: &str) {
    let mut tk = Tk::new();
    tk.set_theme(theme);
    build_scene(&mut tk);
    assert!(tk.sim_scale_factor(scale));
    let (w, h) = if scale > 1.0 { (640, 480) } else { (320, 240) };
    assert_golden(&mut tk, golden_dir(), name, w, h, GoldenOpts::default());
}

/// Light theme at 1x.
#[test]
fn image_light_1x() {
    check_theme(rtk::Theme::Light, 1.0, "image_light_1x");
}

/// Light theme at 2x.
#[test]
fn image_light_2x() {
    check_theme(rtk::Theme::Light, 2.0, "image_light_2x");
}

/// Dark theme at 1x.
#[test]
fn image_dark_1x() {
    check_theme(rtk::Theme::Dark, 1.0, "image_dark_1x");
}

/// Dark theme at 2x.
#[test]
fn image_dark_2x() {
    check_theme(rtk::Theme::Dark, 2.0, "image_dark_2x");
}

/// High-contrast theme at 1x.
#[test]
fn image_highcontrast_1x() {
    check_theme(rtk::Theme::HighContrast, 1.0, "image_highcontrast_1x");
}

/// High-contrast theme at 2x.
#[test]
fn image_highcontrast_2x() {
    check_theme(rtk::Theme::HighContrast, 2.0, "image_highcontrast_2x");
}
