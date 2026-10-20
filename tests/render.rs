//! Tests that widgets render correctly to an offscreen pixmap without a real window.

use rtk::{Button, Fill, Frame, Label, PackOpts, Tk};

/// Renders a simple hello-world layout to a pixmap without requiring a window.
#[test]
fn hello_renders_without_a_window() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, PackOpts {
        fill: Fill::Both,
        expand: true,
        padx: 12.0,
        pady: 12.0,
        ..PackOpts::default()
    });
    let label = Label::new(&mut tk, frame, "Hello, rtk!").expect("label");
    let button = Button::new(&mut tk, frame, "Quit").expect("button");
    let options = PackOpts { pady: 8.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(button, options);

    let pixmap = tk.render_to_pixmap(320, 160).expect("image");
    assert_eq!(pixmap.width(), 320);
    assert_eq!(pixmap.height(), 160);
    assert_eq!(pixmap.data().len(), 320 * 160 * 4);
    let first = &pixmap.data()[..4];
    assert!(pixmap.data().chunks_exact(4).any(|pixel| pixel != first));
    assert!(!tk.is_dirty());

    if let Some(path) = std::env::var_os("RTK_PNG") {
        tk.save_png(std::path::PathBuf::from(path), 320, 160).expect("PNG");
    }
}

/// Checks that invalid pixmap sizes return errors while valid sizes succeed.
#[test]
fn invalid_image_sizes_are_errors() {
    let mut tk = Tk::new();
    assert!(tk.render_to_pixmap(0, 100).is_err());
    assert!(tk.render_to_pixmap(100, 0).is_err());
    assert!(tk.render_to_pixmap(u32::MAX, u32::MAX).is_err());
    assert!(tk.render_to_pixmap(40, 40).is_ok());
}

/// Confirms HiDPI scaling preserves logical geometry while scaling rendered pixel dimensions.
#[test]
fn hidpi_keeps_logical_geometry_and_scales_pixels() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: Fill::Both, expand: true, ..PackOpts::default() });
    tk.render_to_pixmap(200, 100).expect("image");
    let before = tk.widget_rect(frame).expect("rect");
    assert!(tk.sim_scale_factor(2.0));
    let image = tk.render_to_pixmap(400, 200).expect("scaled image");
    assert_eq!(tk.widget_rect(frame), Some(before));
    assert_eq!(image.width(), 400);
    assert!(!tk.sim_scale_factor(f64::NAN));
}
