//! WP2 rendering probes: HiDPI damage (B1), canvas image paint (P1),
//! partial-fill identity (P3), Text visible-line paint (P6),
//! table/tree redraw (P10), and Spinbox `set_command` wiring.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::compare_pixmaps;
use rtk::{
    Button, Canvas, Entry, Frame, FrameDamage, Label, Spinbox, Table, Text, Tk, TreeView, WidgetId,
};

fn scene() -> (Tk, WidgetId, WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();
    tk.pack(frame, Default::default());
    let label = Label::new(&mut tk, frame, "Hello").unwrap();
    tk.pack(label, Default::default());
    let b1 = Button::new(&mut tk, frame, "One").unwrap();
    tk.pack(b1, Default::default());
    let entry = Entry::new(&mut tk, frame).unwrap();
    tk.pack(entry, Default::default());
    (tk, b1, entry)
}

/// Audit probe: at scale 2, a partial Entry redraw must match a full render.
#[test]
fn hidpi_partial_matches_full() {
    let (mut tk, _b1, entry) = scene();
    assert!(tk.sim_scale_factor(2.0));
    let (w, h) = (640, 400);
    assert_eq!(tk.render_frame(w, h).unwrap(), FrameDamage::Full);
    let r = tk.widget_rect(entry).unwrap();
    tk.sim_click(r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.render_frame(w, h).unwrap();
    tk.sim_text("abc");
    let d = tk.render_frame(w, h).unwrap();
    assert!(
        matches!(d, FrameDamage::Partial(_)),
        "expected a partial redraw after typing, got {d:?}"
    );
    let cached = tk.frame().unwrap().clone();
    let fresh = tk.render_to_pixmap(w, h).unwrap();
    let (diff, _) = compare_pixmaps(&cached, &fresh, 8);
    assert_eq!(
        diff, 0,
        "HiDPI partial frame differs from full render by {diff} px"
    );
}

/// Scale 1 still matches, so the logical conversion is a no-op at 1×.
#[test]
fn partial_matches_full_at_1x() {
    let (mut tk, _b1, entry) = scene();
    let (w, h) = (320, 200);
    assert_eq!(tk.render_frame(w, h).unwrap(), FrameDamage::Full);
    let r = tk.widget_rect(entry).unwrap();
    tk.sim_click(r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.render_frame(w, h).unwrap();
    tk.sim_text("xyz");
    tk.render_frame(w, h).unwrap();
    let cached = tk.frame().unwrap().clone();
    let fresh = tk.render_to_pixmap(w, h).unwrap();
    let (diff, _) = compare_pixmaps(&cached, &fresh, 8);
    assert_eq!(
        diff, 0,
        "1× partial frame differs from full render by {diff} px"
    );
}

fn solid_rgba(w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        px.extend_from_slice(&[r, g, b, a]);
    }
    px
}

/// A canvas image survives multiple frames and an in-place update without
/// changing coordinates. Pixel output must match a fresh full render.
#[test]
fn canvas_image_paints_and_updates() {
    let mut tk = Tk::new();
    let root = tk.root();
    let cv = Canvas::new(&mut tk, root, 80.0, 80.0).unwrap();
    tk.pack(cv, Default::default());
    let red = solid_rgba(16, 16, 200, 20, 20, 255);
    let item = tk
        .canvas_create_image_rgba(cv, 8.0, 8.0, 32.0, 32.0, 16, 16, &red, false)
        .unwrap();
    assert_eq!(tk.canvas_coords(cv, item).unwrap(), [8.0, 8.0, 40.0, 40.0]);

    let a = tk.render_to_pixmap(160, 120).unwrap();
    let b = tk.render_to_pixmap(160, 120).unwrap();
    let (diff, _) = compare_pixmaps(&a, &b, 0);
    assert_eq!(diff, 0, "two full canvas renders of the same image differ");

    let blue = solid_rgba(16, 16, 20, 20, 200, 255);
    assert!(tk.canvas_update_image(cv, item, 16, 16, &blue));
    let c = tk.render_to_pixmap(160, 120).unwrap();
    let (diff, _) = compare_pixmaps(&a, &c, 8);
    assert!(
        diff > 0,
        "updating canvas image pixels did not change the frame"
    );

    // Cached premul must follow the update on the next frame too.
    let d = tk.render_to_pixmap(160, 120).unwrap();
    let (diff, _) = compare_pixmaps(&c, &d, 0);
    assert_eq!(diff, 0, "frame after image update is not stable");
}

/// A long Text document still paints the visible window and matches a second render.
#[test]
fn text_visible_lines_render_stably() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    let mut body = String::new();
    for i in 0..80 {
        body.push_str(&format!("line {i:02} abcdef\n"));
    }
    tk.text_set(t, &body);
    let a = tk.render_to_pixmap(400, 200).unwrap();
    let b = tk.render_to_pixmap(400, 200).unwrap();
    let (diff, _) = compare_pixmaps(&a, &b, 0);
    assert_eq!(diff, 0, "Text render of a long document is not stable");
}

/// Table and TreeView visible windows still render and stay stable.
#[test]
fn table_and_tree_visible_rows_render() {
    let mut tk = Tk::new();
    let root = tk.root();
    let table = Table::new(&mut tk, root, &["Name", "Score"], 6).unwrap();
    tk.pack(table, Default::default());
    let rows: Vec<Vec<String>> = (0..40)
        .map(|i| vec![format!("n{i}"), format!("{i}")])
        .collect();
    assert!(tk.table_set_rows(table, rows));

    let tree = TreeView::new(&mut tk, root, 8).unwrap();
    tk.pack(tree, Default::default());
    let parent = tk.tree_insert(tree, None, "root").unwrap();
    for i in 0..12 {
        tk.tree_insert(tree, Some(parent), &format!("c{i}"))
            .unwrap();
    }

    let a = tk.render_to_pixmap(400, 360).unwrap();
    let b = tk.render_to_pixmap(400, 360).unwrap();
    let (diff, _) = compare_pixmaps(&a, &b, 0);
    assert_eq!(diff, 0, "table/tree render is not stable");
}

/// `Kind::has_command` used to omit Spinbox, so `set_command` returned false
/// and never stored the callback. Wiring is the WP2 half of this; WP1 PR #12
/// calls `run_command` from events.rs on arrow keys/clicks (and added
/// `spinbox_on_change` as a workaround that writes `node.command` directly).
/// This test checks the widgets.rs side only and does not click the widget,
/// because this branch does not include the events.rs `run_command` calls.
#[test]
fn spinbox_set_command_wires() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(5);
    let spin = Spinbox::new(&mut tk, root, 0, 10, var).unwrap();
    tk.pack(spin, Default::default());
    let hits = Rc::new(Cell::new(0));
    let h = hits.clone();
    assert!(
        tk.set_command(spin, move |_| h.set(h.get() + 1)),
        "set_command should accept a Spinbox once Kind::has_command includes it"
    );
    // The callback is stored; invoking it via the public command path
    // (config again replaces it, which also has to succeed).
    let h2 = hits.clone();
    assert!(tk.set_command(spin, move |_| h2.set(h2.get() + 10)));
    // A kind that still has no command must keep returning false.
    let label = Label::new(&mut tk, root, "nope").unwrap();
    assert!(!tk.set_command(label, move |_| {}));
    assert_eq!(
        hits.get(),
        0,
        "set_command must not run the callback when installing it"
    );
}
