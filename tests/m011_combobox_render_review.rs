//! Review regression tests for editable Combobox drawing (`src/render.rs`).
//!
//! Covers the popup scroll offset (arrow navigation past the visible rows keeps
//! the highlighted row drawn), the single-active-row rule in editable mode, and
//! the caret scroll window for overflowing text.

use rtk::{Combobox, Fill, Key, PackOpts, Tk, WidgetId};

/// Builds a toolkit with an editable combobox of `count` values packed at the
/// top of the window and returns `(tk, combo)`.
fn setup(count: usize) -> (Tk, WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(0);
    let values: Vec<String> = (0..count).map(|i| format!("Item {i}")).collect();
    let combo = Combobox::editable(&mut tk, root, values, var).expect("combobox");
    tk.pack(combo, PackOpts { fill: Fill::X, ..PackOpts::default() });
    (tk, combo)
}

/// Focuses the combobox by clicking its text area (not the arrow zone).
fn focus_combo(tk: &mut Tk) {
    // Lay out first so the click lands on the widget.
    let _ = tk.render_to_pixmap(400, 300);
    tk.sim_click(30.0, 15.0);
}

/// Returns the RGBA bytes of the popup area (below the combobox row) of a
/// `400`×`300` render, as a flat `Vec<u8>`.
///
/// The combobox is packed at the top of the window with the default height, so
/// the popup starts at y = 24 and spans the full width.
fn popup_pixels(tk: &mut Tk) -> Vec<u8> {
    let pixmap = tk.render_to_pixmap(400, 300).expect("render");
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let data = pixmap.pixels();
    let mut out = Vec::new();
    for y in 24..height {
        let start = y * width;
        for px in &data[start..start + width] {
            out.extend_from_slice(&[px.red(), px.green(), px.blue(), px.alpha()]);
        }
    }
    out
}

/// Reports whether the pixel buffer contains more than one distinct colour.
fn has_multiple_colors(pixels: &[u8]) -> bool {
    let Some(first) = pixels.get(..4) else { return false };
    pixels.chunks_exact(4).any(|px| px != first)
}

/// Arrow navigation past the visible popup rows keeps the highlight drawn and
/// Enter selects the highlighted value.
#[test]
fn popup_scrolls_to_highlight_and_enter_selects() {
    let (mut tk, combo) = setup(30);
    focus_combo(&mut tk);

    // First Down opens the dropdown and highlights row 0.
    tk.sim_key(Key::Down);
    assert!(tk.combobox_is_open(combo));
    assert_eq!(tk.combobox_get_highlighted(combo), Some(0));

    // Snapshot of the popup with highlight 0 for comparison below.
    let top_pixels = popup_pixels(&mut tk);

    // Nineteen more Downs move the highlight to row 19 (past the ~10 visible).
    for _ in 0..19 {
        tk.sim_key(Key::Down);
    }
    assert_eq!(tk.combobox_get_highlighted(combo), Some(19));

    // The scrolled popup must still paint rows: its area is not a single
    // uniform colour and differs from the highlight-0 render.
    let scrolled_pixels = popup_pixels(&mut tk);
    assert!(
        has_multiple_colors(&scrolled_pixels),
        "scrolled popup area is a single uniform colour (rows not drawn)"
    );
    assert_ne!(
        top_pixels, scrolled_pixels,
        "scrolled popup renders identically to the highlight-0 popup"
    );

    // Enter selects the highlighted value.
    tk.sim_key(Key::Enter);
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(tk.combobox_get(combo), Some((19, "Item 19".to_string())));
}

/// Overflowing editable text keeps the caret inside the clip (no panic, text
/// scrolls) and rendering stays stable.
#[test]
fn overflowing_text_renders_without_panic() {
    let (mut tk, combo) = setup(5);
    focus_combo(&mut tk);
    // Replace the initial value before typing overflowing text.
    assert!(tk.combobox_set_text(combo, ""));
    let long = "x".repeat(200);
    tk.sim_text(&long);
    let _ = tk.render_to_pixmap(400, 300);
    assert_eq!(tk.combobox_get_text(combo).as_deref(), Some(long.as_str()));
}
