//! WP2 rendering probes: HiDPI damage (B1), canvas image paint (P1),
//! partial-fill identity (P3), Text visible-line paint (P6),
//! table/tree redraw (P10), and Spinbox `set_command` wiring.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::compare_pixmaps;
use rtk::{
    Button, Canvas, Entry, Frame, FrameDamage, Key, Label, Rect, Spinbox, Table, Text, Tk,
    TreeView, WidgetId,
};
use tiny_skia::Pixmap;

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

fn pixel(pm: &Pixmap, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * pm.width() + x) * 4) as usize;
    let d = pm.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

/// True when every device pixel *outside* `exclude_logical` inflated by
/// `inflate` logical px matches within `tol`.
fn region_unchanged(
    a: &Pixmap,
    b: &Pixmap,
    exclude_logical: Rect,
    scale: f32,
    inflate: f32,
    tol: u8,
) -> bool {
    if a.width() != b.width() || a.height() != b.height() {
        return false;
    }
    let ex_x0 = ((exclude_logical.x - inflate) * scale).floor();
    let ex_y0 = ((exclude_logical.y - inflate) * scale).floor();
    let ex_x1 = ((exclude_logical.x + exclude_logical.width + inflate) * scale).ceil();
    let ex_y1 = ((exclude_logical.y + exclude_logical.height + inflate) * scale).ceil();
    for y in 0..a.height() {
        let yf = y as f32;
        let in_y = yf >= ex_y0 && yf < ex_y1;
        for x in 0..a.width() {
            if in_y {
                let xf = x as f32;
                if xf >= ex_x0 && xf < ex_x1 {
                    continue;
                }
            }
            let pa = pixel(a, x, y);
            let pb = pixel(b, x, y);
            if (0..4).any(|c| pa[c].abs_diff(pb[c]) > tol) {
                return false;
            }
        }
    }
    true
}

fn first_diff_in_rect(
    a: &Pixmap,
    b: &Pixmap,
    logical: Rect,
    scale: f32,
    tol: u8,
) -> Option<(u32, u32)> {
    let x0 = ((logical.x * scale).floor().max(0.0) as u32).min(a.width());
    let y0 = ((logical.y * scale).floor().max(0.0) as u32).min(a.height());
    let x1 = (((logical.x + logical.width) * scale).ceil().max(0.0) as u32).min(a.width());
    let y1 = (((logical.y + logical.height) * scale).ceil().max(0.0) as u32).min(a.height());
    for y in y0..y1 {
        for x in x0..x1 {
            let pa = pixel(a, x, y);
            let pb = pixel(b, x, y);
            if (0..4).any(|c| pa[c].abs_diff(pb[c]) > tol) {
                return Some((x, y));
            }
        }
    }
    None
}

fn type_into_entry_and_check(tk: &mut Tk, entry: WidgetId, w: u32, h: u32, scale: f32, typ: &str) {
    let r = tk.widget_rect(entry).unwrap();
    tk.sim_click(r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.render_frame(w, h).unwrap();
    let before = tk.frame().unwrap().clone();
    tk.sim_text(typ);
    let d = tk.render_frame(w, h).unwrap();
    assert!(
        matches!(d, FrameDamage::Partial(_)),
        "expected a partial redraw after typing at scale {scale}, got {d:?}"
    );
    let cached = tk.frame().unwrap().clone();
    assert!(
        region_unchanged(&before, &cached, r, scale, 8.0, 2),
        "scale {scale}: pixels outside the inflated Entry rect changed during a partial repaint"
    );
    let fresh = tk.render_to_pixmap(w, h).unwrap();
    let (diff, _) = compare_pixmaps(&cached, &fresh, 8);
    assert_eq!(
        diff, 0,
        "scale {scale}: HiDPI/partial frame differs from a full render by {diff} px"
    );
}

/// Audit probe: at scale 2, a partial Entry redraw must match a full render.
#[test]
fn hidpi_partial_matches_full() {
    let (mut tk, _b1, entry) = scene();
    assert!(tk.sim_scale_factor(2.0));
    let (w, h) = (640, 400);
    assert_eq!(tk.render_frame(w, h).unwrap(), FrameDamage::Full);
    type_into_entry_and_check(&mut tk, entry, w, h, 2.0, "abc");
}

/// Scale 1 still matches, so the logical conversion is a no-op at 1×.
#[test]
fn partial_matches_full_at_1x() {
    let (mut tk, _b1, entry) = scene();
    let (w, h) = (320, 200);
    assert_eq!(tk.render_frame(w, h).unwrap(), FrameDamage::Full);
    type_into_entry_and_check(&mut tk, entry, w, h, 1.0, "xyz");
}

/// Fractional and high HiDPI scales: partial vs full identity, and pixels
/// outside the Entry stay untouched.
#[test]
fn hidpi_partial_matches_full_at_fractional_scales() {
    for &scale in &[1.25_f64, 1.5, 1.75, 3.0] {
        let (mut tk, _b1, entry) = scene();
        assert!(
            tk.sim_scale_factor(scale),
            "sim_scale_factor({scale}) failed"
        );
        let (w, h) = (640, 400);
        assert_eq!(
            tk.render_frame(w, h).unwrap(),
            FrameDamage::Full,
            "scale {scale}: initial frame must be Full"
        );
        type_into_entry_and_check(&mut tk, entry, w, h, scale as f32, "Frac");
    }
}

fn solid_rgba(w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        px.extend_from_slice(&[r, g, b, a]);
    }
    px
}

fn bordered_rgba(w: u32, h: u32, center: [u8; 4], border: [u8; 4]) -> Vec<u8> {
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let c = if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                border
            } else {
                center
            };
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&c);
        }
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

/// Reviewer1 repro: same-size update that keeps the border (first/last 8 bytes)
/// must still paint the new center (not a stale premultiplied cache entry).
#[test]
fn canvas_update_image_same_border_paints_new_center() {
    let mut tk = Tk::new();
    let root = tk.root();
    let cv = Canvas::new(&mut tk, root, 64.0, 64.0).unwrap();
    tk.pack(cv, Default::default());
    let border = [200u8, 20, 20, 255];
    let green = bordered_rgba(32, 32, [20, 200, 20, 255], border);
    let item = tk
        .canvas_create_image_rgba(cv, 0.0, 0.0, 32.0, 32.0, 32, 32, &green, false)
        .unwrap();

    let a = tk.render_to_pixmap(160, 120).unwrap();
    let cr = tk.widget_rect(cv).unwrap();
    let cx = ((cr.x + 16.0).floor() as i32).clamp(0, a.width() as i32 - 1) as u32;
    let cy = ((cr.y + 16.0).floor() as i32).clamp(0, a.height() as i32 - 1) as u32;
    let p1 = pixel(&a, cx, cy);
    assert!(
        p1[1] > p1[0] && p1[1] > p1[2],
        "expected green-dominant center before update, got {p1:?} at ({cx},{cy})"
    );

    let blue = bordered_rgba(32, 32, [20, 20, 200, 255], border);
    assert!(tk.canvas_update_image(cv, item, 32, 32, &blue));
    let b = tk.render_to_pixmap(160, 120).unwrap();
    let p2 = pixel(&b, cx, cy);
    assert!(
        p2[2] > p2[0] && p2[2] > p2[1],
        "expected blue-dominant center after same-border update, got {p2:?} (stale green would be {p1:?})"
    );

    let c = tk.render_to_pixmap(160, 120).unwrap();
    let p3 = pixel(&c, cx, cy);
    assert!(
        p3[2] > p3[0] && p3[2] > p3[1],
        "center must stay blue on the cached frame after the update, got {p3:?}"
    );
    let (diff, _) = compare_pixmaps(&b, &c, 0);
    assert_eq!(
        diff, 0,
        "frame after same-border image update is not stable"
    );
}

fn text_body(first: &str) -> String {
    let mut body = String::new();
    body.push_str(first);
    body.push('\n');
    for i in 1..80 {
        body.push_str(&format!("line {i:02} abcdef\n"));
    }
    body
}

/// A long Text document still paints the visible window and matches a second render.
/// Visible line 0 is also asserted by swapping a distinctive first line.
#[test]
fn text_visible_lines_render_stably() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    tk.text_set(t, text_body("AAAA_UNIQUE"));
    let a = tk.render_to_pixmap(400, 200).unwrap();
    let b = tk.render_to_pixmap(400, 200).unwrap();
    let (diff, _) = compare_pixmaps(&a, &b, 0);
    assert_eq!(diff, 0, "Text render of a long document is not stable");

    tk.text_set(t, text_body("BBBB_UNIQUE"));
    let c = tk.render_to_pixmap(400, 200).unwrap();
    let tr = tk.widget_rect(t).unwrap();
    assert!(
        first_diff_in_rect(&a, &c, tr, 1.0, 0).is_some(),
        "changing the visible first line of a Text widget must change pixels in its rect"
    );
}

/// Table and TreeView visible windows still render and stay stable.
/// Off-screen table edits must not change pixels; visible row/label edits must.
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
    assert!(tk.tree_set_open(tree, parent, true));
    let mut child_keys = Vec::new();
    for i in 0..12 {
        child_keys.push(
            tk.tree_insert(tree, Some(parent), &format!("c{i}"))
                .unwrap(),
        );
    }

    let a = tk.render_to_pixmap(400, 360).unwrap();
    let b = tk.render_to_pixmap(400, 360).unwrap();
    let (diff, _) = compare_pixmaps(&a, &b, 0);
    assert_eq!(diff, 0, "table/tree render is not stable");

    // Off-screen table row: 40 rows, 6 visible — row 39 must not appear.
    let mut rows_off: Vec<Vec<String>> = (0..40)
        .map(|i| vec![format!("n{i}"), format!("{i}")])
        .collect();
    rows_off[39][0] = "n39X".into();
    assert!(tk.table_set_rows(table, rows_off));
    let c = tk.render_to_pixmap(400, 360).unwrap();
    let (diff, _) = compare_pixmaps(&a, &c, 0);
    assert_eq!(
        diff, 0,
        "changing an off-screen table row must not change visible pixels"
    );

    let mut rows_vis: Vec<Vec<String>> = (0..40)
        .map(|i| vec![format!("n{i}"), format!("{i}")])
        .collect();
    rows_vis[0][0] = "VISIBLE_ROW0".into();
    assert!(tk.table_set_rows(table, rows_vis));
    let d = tk.render_to_pixmap(400, 360).unwrap();
    let table_rect = tk.widget_rect(table).unwrap();
    assert!(
        first_diff_in_rect(&a, &d, table_rect, 1.0, 0).is_some(),
        "changing a visible table row must change pixels inside the table rect"
    );

    assert!(tk.tree_set_text(tree, child_keys[0], "VISIBLE_C0"));
    let e = tk.render_to_pixmap(400, 360).unwrap();
    let tree_rect = tk.widget_rect(tree).unwrap();
    assert!(
        first_diff_in_rect(&d, &e, tree_rect, 1.0, 0).is_some(),
        "changing a visible tree child label must change pixels inside the tree rect"
    );
}

/// `Kind::has_command` used to omit Spinbox, so `set_command` returned false
/// and never stored the callback. After merging WP1, `run_command` is invoked
/// from events.rs on arrow keys/clicks; `spinbox_on_change` writes the same
/// `node.command` slot (and delegates to `set_command` now that has_command
/// includes Spinbox).
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

/// A command installed with `set_command` must fire on arrow keys and the
/// on-widget arrows. Installing `spinbox_on_change` as well must replace the
/// callback, not stack a second fire.
#[test]
fn spinbox_set_command_fires_on_key_without_double_fire() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(5);
    let spin = Spinbox::new(&mut tk, root, 0, 10, var).unwrap();
    tk.pack(spin, Default::default());
    let via_set = Rc::new(Cell::new(0));
    let via_change = Rc::new(Cell::new(0));
    let h = via_set.clone();
    assert!(
        tk.set_command(spin, move |_| h.set(h.get() + 1)),
        "set_command must accept a Spinbox"
    );
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(spin).unwrap();
    tk.sim_click(r.x + r.width * 0.3, r.y + r.height * 0.5);

    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(
        via_set.get(),
        1,
        "set_command callback must fire on Key::Up"
    );
    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(5));
    assert_eq!(
        via_set.get(),
        2,
        "set_command callback must fire on Key::Down"
    );
    tk.sim_click(r.x + r.width - 5.0, r.y + r.height * 0.25);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(
        via_set.get(),
        3,
        "set_command callback must fire on an up-arrow click"
    );

    // Last setter wins; the two APIs share one slot and must not both run.
    let h2 = via_change.clone();
    assert!(tk.spinbox_on_change(spin, move |_| h2.set(h2.get() + 1)));
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(7));
    assert_eq!(
        via_set.get(),
        3,
        "the replaced set_command callback must not fire again"
    );
    assert_eq!(
        via_change.get(),
        1,
        "spinbox_on_change must fire exactly once after replacing set_command"
    );

    let h3 = via_set.clone();
    assert!(tk.set_command(spin, move |_| h3.set(h3.get() + 1)));
    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(
        via_change.get(),
        1,
        "the replaced spinbox_on_change callback must not fire again"
    );
    assert_eq!(
        via_set.get(),
        4,
        "set_command after spinbox_on_change must fire exactly once (no double-fire)"
    );
}
