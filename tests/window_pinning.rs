//! Integration tests for single-axis window resizability pinning.
//!
//! These exercise the headless toolkit surface: the setters must mark the
//! toolkit dirty (so a redraw, and therefore `apply_window_props`, runs) and
//! the stored constraints must round-trip unchanged.

use rtk::Tk;

/// A single-axis `resizable` call must request a redraw and store the flags.
#[test]
fn resizable_marks_dirty_and_stores_flags() {
    let mut tk = Tk::new();
    tk.sim_resize(640, 480);
    // Clear any pending dirty state from construction/resize.
    let _ = tk.is_dirty();
    tk.resizable(false, true);
    assert_eq!(tk.is_resizable(), (false, true));
    assert!(tk.is_dirty(), "resizable() must request a redraw");
}

/// `minsize`/`maxsize` must mark the toolkit dirty and round-trip.
#[test]
fn size_constraints_mark_dirty_and_round_trip() {
    let mut tk = Tk::new();
    tk.sim_resize(640, 480);
    tk.minsize(200, 100);
    assert_eq!(tk.window_minsize(), Some((200, 100)));
    assert!(tk.is_dirty(), "minsize() must request a redraw");
    tk.maxsize(900, 700);
    assert_eq!(tk.window_maxsize(), Some((900, 700)));
    assert!(tk.is_dirty(), "maxsize() must request a redraw");
    tk.clear_minsize();
    assert_eq!(tk.window_minsize(), None);
    tk.clear_maxsize();
    assert_eq!(tk.window_maxsize(), None);
}

/// A headless render after changing constraints must succeed.
#[test]
fn render_after_constraint_change_succeeds() {
    let mut tk = Tk::new();
    tk.sim_resize(640, 480);
    tk.resizable(false, true);
    tk.minsize(200, 100);
    tk.maxsize(900, 700);
    let pixmap = tk.render_to_pixmap(640, 480).expect("render");
    assert_eq!(pixmap.width(), 640);
    assert_eq!(pixmap.height(), 480);
}
