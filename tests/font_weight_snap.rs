//! Font weights and styles the default font lacks still draw real letters.
//!
//! Regression for garbled Windows button labels: cosmic-text 0.12 uses the requested family
//! only at an exact weight, otherwise the first font anywhere with that weight (often a
//! symbol font). rtk snaps every request to a face the family has, so weight 500 must draw
//! exactly like 400 with the bundled Noto Sans, whatever other fonts are installed.

use rtk::{Button, Canvas, Color, PackOpts, Tk};

/// Renders one button whose `TButton` style uses `css_weight`.
fn button_pixels(css_weight: &str) -> Vec<u8> {
    let mut tk = Tk::new();
    tk.load_stylesheet(&format!("TButton {{ font-weight: {css_weight}; }}")).unwrap();
    let root = tk.root();
    let b = Button::new(&mut tk, root, "Start download  Browse...  Cancel").unwrap();
    tk.pack(b, PackOpts::default());
    tk.render_to_pixmap(420, 60).unwrap().data().to_vec()
}

/// Renders canvas `text_ex` at `weight`.
fn text_ex_pixels(weight: u16) -> Vec<u8> {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 420.0, 60.0).unwrap();
    tk.pack(c, PackOpts::default());
    tk.canvas_create_text_ex(c, 10.0, 10.0, "Save to folder", Color::rgb(20, 20, 20), 20.0, weight).unwrap();
    tk.render_to_pixmap(420, 60).unwrap().data().to_vec()
}

#[test]
fn medium_button_weight_draws_like_regular() {
    assert!(button_pixels("500") == button_pixels("400"), "weight 500 must snap to Noto Sans 400");
    assert!(button_pixels("600") == button_pixels("700"), "weight 600 must snap to Noto Sans 700");
}

#[test]
fn canvas_text_ex_weight_snaps() {
    assert!(text_ex_pixels(500) == text_ex_pixels(400));
    assert!(text_ex_pixels(300) == text_ex_pixels(400));
}
