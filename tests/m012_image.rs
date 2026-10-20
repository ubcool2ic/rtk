//! Integration tests for the `Image` widget: RGBA, PNG, SVG, fit modes and
//! SVG icons on buttons.

use rtk::{Button, Frame, Image, ImageFit, Opt, Tk, WidgetId};

/// Creates a frame and an image widget of the given size under it, packing the
/// frame to fill the window and the image inside it.
fn make_image(tk: &mut Tk, w: f32, h: f32) -> (WidgetId, WidgetId) {
    let root = tk.root();
    let frame = Frame::new(tk, root).unwrap();
    let img = Image::new(tk, frame, w, h).unwrap();
    tk.pack(
        frame,
        rtk::PackOpts { fill: rtk::Fill::Both, expand: true, ..rtk::PackOpts::default() },
    );
    tk.pack(img, rtk::PackOpts::default());
    (frame, img)
}

/// `image_set_rgba` rejects zero sizes and wrong-length buffers, accepts valid ones.
#[test]
fn set_rgba_validation() {
    let mut tk = Tk::new();
    let (_, img) = make_image(&mut tk, 32.0, 32.0);
    assert!(!tk.image_set_rgba(img, 0, 4, &[]));
    assert!(!tk.image_set_rgba(img, 4, 0, &[]));
    assert!(!tk.image_set_rgba(img, 2, 2, &[0u8; 3]));
    let rgba = vec![255u8; 2 * 2 * 4];
    assert!(tk.image_set_rgba(img, 2, 2, &rgba));
    assert_eq!(tk.image_natural_size(img), Some((2, 2)));
}

/// PNG bytes encoded with tiny-skia load and report the natural size.
#[test]
fn load_png_bytes() {
    let mut tk = Tk::new();
    let (_, img) = make_image(&mut tk, 32.0, 32.0);
    let mut pixmap = tiny_skia::Pixmap::new(5, 3).unwrap();
    pixmap.fill(tiny_skia::Color::from_rgba8(10, 20, 30, 255));
    let png = pixmap.encode_png().unwrap();
    assert!(tk.image_load_png(img, &png));
    assert_eq!(tk.image_natural_size(img), Some((5, 3)));
}

/// `svg_to_rgba` rasterizes a red square to an opaque red centre pixel.
#[cfg(feature = "svg")]
#[test]
fn svg_to_rgba_red_square() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="#ff0000"/></svg>"##;
    let rgba = rtk::image::svg_to_rgba(svg, 10, 10).expect("valid svg");
    assert_eq!(rgba.len(), 10 * 10 * 4);
    let idx = (5 * 10 + 5) * 4;
    assert_eq!(&rgba[idx..idx + 4], &[255, 0, 0, 255]);
}

/// Invalid SVG input yields `None` from `svg_to_rgba` and `false` from `image_set_svg`.
#[cfg(feature = "svg")]
#[test]
fn invalid_svg_rejected() {
    let mut tk = Tk::new();
    let (_, img) = make_image(&mut tk, 32.0, 32.0);
    assert!(rtk::image::svg_to_rgba(b"not an svg", 10, 10).is_none());
    assert!(!tk.image_set_svg(img, b"not an svg"));
}

/// `load_svg` returns an image id usable as a Button icon via `Opt::Image`.
#[cfg(feature = "svg")]
#[test]
fn load_svg_button_icon() {
    let mut tk = Tk::new();
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><circle cx="8" cy="8" r="7" fill="#0072B2"/></svg>"##;
    let icon = tk.load_svg(svg, 16, 16).expect("svg icon");
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();
    let btn = Button::new(&mut tk, frame, "Go").unwrap();
    tk.pack(
        frame,
        rtk::PackOpts { fill: rtk::Fill::Both, expand: true, ..rtk::PackOpts::default() },
    );
    tk.pack(btn, rtk::PackOpts::default());
    let before = tk.render_to_pixmap(120, 60).unwrap().data().to_vec();
    assert!(tk.config(btn, Opt::Image(icon)));
    let pixmap = tk.render_to_pixmap(120, 60).unwrap();
    assert_eq!(pixmap.width(), 120);
    assert_eq!(pixmap.height(), 60);
    assert_ne!(pixmap.data(), &before[..], "icon should change the rendered button");
}

/// Contain fit keeps aspect ratio: a square image in a wide widget leaves the
/// left edge as background and the centre red.
#[test]
fn contain_keeps_aspect() {
    let mut tk = Tk::new();
    let (_, img) = make_image(&mut tk, 200.0, 100.0);
    let mut rgba = Vec::with_capacity(8 * 8 * 4);
    for _ in 0..8 * 8 {
        rgba.extend_from_slice(&[255, 0, 0, 255]);
    }
    assert!(tk.image_set_rgba(img, 8, 8, &rgba));
    assert!(tk.image_set_fit(img, ImageFit::Contain));
    let pixmap = tk.render_to_pixmap(200, 100).unwrap();
    let data = pixmap.data();
    let row = 50usize;
    let centre = &data[(row * 200 + 100) * 4..(row * 200 + 100) * 4 + 4];
    let left = &data[(row * 200 + 10) * 4..(row * 200 + 10) * 4 + 4];
    assert_eq!(&centre[..3], &[255, 0, 0], "centre should be red");
    assert_ne!(&left[..3], &[255, 0, 0], "left edge should be background");
}

/// Image APIs reject ids that are not live image widgets.
#[test]
fn wrong_kind_rejected() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();
    assert!(!tk.image_set_rgba(frame, 2, 2, &[0u8; 16]));
    assert!(!tk.image_set_fit(frame, ImageFit::Fill));
    assert!(!tk.image_clear(frame));
    assert_eq!(tk.image_natural_size(frame), None);
}
