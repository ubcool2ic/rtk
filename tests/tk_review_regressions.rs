//! Regression tests for the Fable 5.1 review, pass 1 (part 1: F1-F4).
//!
//! - `f1_notebook_page_children_get_size`: F1, widgets inside Notebook pages get a non-zero size.
//! - `f2_opt_text_with_multibyte_does_not_panic`: F2, a stale mid-char cursor after `Opt::Text`
//!   must not panic. This uses `Entry`, because the `Text` constructor is not in the API index.
//! - `f3_canvas_add_many_items_is_fast`: F3, building a large canvas is not quadratic.
//! - `f3_canvas_delete_then_add_reuses_slot`: F3, freed slots are reused and counts stay exact.
//! - `f4_destroy_page_clears_notebook_state`: F4, destroying a page leaves no dangling selection.
//! - `f4_destroy_notebook_clears_state`: F4, destroying the notebook drops its state.
//!
//! The F4 modal case is not covered here, because the dialog constructors are not in the API index.

use rtk::{Canvas, Color, Entry, Frame, Notebook, Opt, PackOpts, Tk};
use std::time::{Duration, Instant};

/// F1: a widget placed inside a notebook page must receive a non-zero size after layout.
#[test]
fn f1_notebook_page_children_get_size() {
    let mut tk = Tk::new();
    let root = tk.root();
    let nb = Notebook::new(&mut tk, root).unwrap();
    tk.pack(nb, PackOpts::default());
    let page = Frame::new(&mut tk, nb).unwrap();
    let canvas = Canvas::new(&mut tk, page, 80.0, 40.0).unwrap();
    tk.pack(canvas, PackOpts::default());
    assert!(tk.notebook_add(nb, page, "One"));
    tk.sim_resize(400, 300);
    let r = tk.widget_rect(canvas).expect("canvas has a rect");
    assert!(r.width > 0.0 && r.height > 0.0, "zero-size child in tab: {r:?}");
}

/// F2: replacing text with `Opt::Text` and then typing multi-byte text must not panic on a
/// non-boundary cursor, and the entry must still hold valid text afterwards.
#[test]
fn f2_opt_text_with_multibyte_does_not_panic() {
    let mut tk = Tk::new();
    let root = tk.root();
    let e = Entry::new(&mut tk, root).unwrap();
    tk.pack(e, PackOpts::default());
    tk.sim_resize(300, 200);
    let r = tk.widget_rect(e).expect("entry has a rect");
    tk.sim_click(r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.sim_text("日本語テキスト");
    tk.config(e, Opt::Text("é".to_string()));
    tk.sim_text("ñ日");
    tk.config(e, Opt::Text("a".to_string()));
    tk.sim_text("é");
    let s = tk.entry_get(e).expect("entry text");
    assert_eq!(s.chars().count(), 2, "unexpected entry text: {s:?}");
    assert!(s.contains('a') && s.contains('é'), "unexpected entry text: {s:?}");
}

/// F3: adding many canvas items must scale roughly linearly, and the count must be exact.
#[test]
fn f3_canvas_add_many_items_is_fast() {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 100.0, 100.0).unwrap();
    let n = 20_000usize;
    let start = Instant::now();
    for i in 0..n {
        let x = (i % 100) as f32;
        assert!(tk.canvas_create_rect(c, x, 0.0, x + 1.0, 1.0, Color::BLACK).is_some());
    }
    assert_eq!(tk.canvas_items(c), n);
    assert!(start.elapsed() < Duration::from_secs(10), "canvas item insertion looks quadratic");
}

/// F3: deleting an item and adding another keeps the item count consistent (slot reuse).
#[test]
fn f3_canvas_delete_then_add_reuses_slot() {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 100.0, 100.0).unwrap();
    let a = tk.canvas_create_rect(c, 0.0, 0.0, 5.0, 5.0, Color::BLACK).unwrap();
    let _b = tk.canvas_create_oval(c, 1.0, 1.0, 6.0, 6.0, Color::BLACK).unwrap();
    assert!(tk.canvas_delete(c, a));
    assert_eq!(tk.canvas_items(c), 1);
    let d = tk.canvas_create_line(c, 0.0, 0.0, 9.0, 9.0, Color::BLACK).unwrap();
    assert_eq!(tk.canvas_items(c), 2);
    assert!(tk.canvas_coords(c, d).is_some());
}

/// F4: destroying a notebook page must not leave a dangling page or selection behind.
#[test]
fn f4_destroy_page_clears_notebook_state() {
    let mut tk = Tk::new();
    let root = tk.root();
    let nb = Notebook::new(&mut tk, root).unwrap();
    let p0 = Frame::new(&mut tk, nb).unwrap();
    let p1 = Frame::new(&mut tk, nb).unwrap();
    assert!(tk.notebook_add(nb, p0, "A"));
    assert!(tk.notebook_add(nb, p1, "B"));
    assert!(tk.notebook_select(nb, 1));
    assert!(tk.destroy(p1));
    assert!(!tk.contains(p1));
    assert!(!tk.notebook_select(nb, 1), "destroyed page must not be selectable");
    match tk.notebook_selected(nb) {
        None | Some(0) => {}
        other => panic!("dangling selection: {other:?}"),
    }
    tk.sim_resize(300, 200);
}

/// F4: destroying the notebook itself must drop its state and not break later layout.
#[test]
fn f4_destroy_notebook_clears_state() {
    let mut tk = Tk::new();
    let root = tk.root();
    let nb = Notebook::new(&mut tk, root).unwrap();
    let p = Frame::new(&mut tk, nb).unwrap();
    assert!(tk.notebook_add(nb, p, "A"));
    assert!(tk.destroy(nb));
    assert!(!tk.contains(nb));
    assert_eq!(tk.notebook_selected(nb), None);
    assert!(!tk.notebook_select(nb, 0));
    tk.sim_resize(320, 240);
    tk.render_frame(320, 240).unwrap();
}
