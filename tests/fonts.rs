//! A new Tk uses the bundled Noto Sans as its default font and draws text with it.

use rtk::{DefaultFont, Label, PackOpts, Tk};

/// The default font is the bundled one and a label renders visible pixels.
#[test]
fn tk_defaults_to_bundled_font() {
    let mut tk = Tk::new();
    assert_eq!(tk.default_font(), &DefaultFont::Bundled);
    assert_eq!(tk.default_font().family(), Some(rtk::fonts::BUNDLED_FAMILY));
    let root = tk.root();
    let blank = tk.render_to_pixmap(400, 60).unwrap();
    let l = Label::new(&mut tk, root, "Search  Add selected city  Remove").unwrap();
    tk.pack(l, PackOpts::default());
    let pm = tk.render_to_pixmap(400, 60).unwrap();
    assert_ne!(
        pm.data(),
        blank.data(),
        "label text changed the rendered pixels"
    );
}
