//! Integration tests for milestone 0.13 dirty-rectangle (partial) redraw.

use rtk::testing::compare_pixmaps;
use rtk::{
    Button, Entry, Frame, FrameDamage, Label, Opt, ProgressBar, Tk, WidgetId,
};

/// Builds a small scene and returns the toolkit plus the ids of the two buttons
/// and the entry.
fn scene() -> (Tk, WidgetId, WidgetId, WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, Default::default());
    let label = Label::new(&mut tk, frame, "Hello").expect("label");
    tk.pack(label, Default::default());
    let b1 = Button::new(&mut tk, frame, "One").expect("button one");
    tk.pack(b1, Default::default());
    let b2 = Button::new(&mut tk, frame, "Two").expect("button two");
    tk.pack(b2, Default::default());
    let entry = Entry::new(&mut tk, frame).expect("entry");
    tk.pack(entry, Default::default());
    let progress = tk.int_var(0);
    let bar = ProgressBar::new(&mut tk, frame, 100.0, progress).expect("progress");
    tk.pack(bar, Default::default());
    (tk, b1, b2, entry)
}

/// Returns the centre of a widget's rect.
fn centre(tk: &mut Tk, id: WidgetId) -> (f32, f32) {
    let rect = tk.widget_rect(id).expect("rect");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

/// True when `rect` lies inside the widget's inflated area (focus ring plus
/// the default button drop shadow: offset + blur, so up to 24 logical px).
fn inside_inflated(tk: &mut Tk, id: WidgetId, rect: rtk::Rect) -> bool {
    let r = tk.widget_rect(id).expect("rect");
    let pad = 24.0;
    rect.x >= r.x - pad
        && rect.y >= r.y - pad
        && rect.x + rect.width <= r.x + r.width + pad
        && rect.y + rect.height <= r.y + r.height + pad
}

/// Asserts the cached frame matches a fresh full render.
fn assert_frame_matches_full(tk: &mut Tk, width: u32, height: u32) {
    let cached = tk.frame().expect("cached frame").clone();
    let fresh = tk.render_to_pixmap(width, height).expect("full render");
    let (diff, _) = compare_pixmaps(&cached, &fresh, 8);
    assert_eq!(diff, 0, "partial frame differs from full render");
}

/// The first frame is full, an unchanged frame is `None`.
#[test]
fn first_frame_full_then_none() {
    let (mut tk, _, _, _) = scene();
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::None);
}

/// Hovering a button then advancing time yields a partial redraw confined to it.
#[test]
fn hover_button_partial() {
    let (mut tk, b1, _, _) = scene();
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    let (x, y) = centre(&mut tk, b1);
    tk.sim_mouse_move(x, y);
    tk.sim_advance_time(1000);
    match tk.render_frame(320, 200).expect("frame") {
        FrameDamage::Partial(rects) => {
            assert!(!rects.is_empty());
            for r in rects {
                assert!(inside_inflated(&mut tk, b1, r), "rect {r:?} outside button");
                assert!(r.width < 320.0 && r.height < 200.0);
            }
        }
        other => panic!("expected partial, got {other:?}"),
    }
    assert_frame_matches_full(&mut tk, 320, 200);
}

/// Typing into the entry yields a partial redraw matching a full render.
#[test]
fn entry_typing_partial() {
    let (mut tk, _, _, entry) = scene();
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    let (x, y) = centre(&mut tk, entry);
    tk.sim_click(x, y);
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    tk.sim_text("abc");
    match tk.render_frame(320, 200).expect("frame") {
        FrameDamage::Partial(rects) => assert!(!rects.is_empty()),
        other => panic!("expected partial, got {other:?}"),
    }
    assert_frame_matches_full(&mut tk, 320, 200);
}

/// Changing the theme forces a full redraw.
#[test]
fn theme_change_full() {
    let (mut tk, _, _, _) = scene();
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    tk.set_theme(rtk::Theme::Dark);
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
}

/// Resizing forces a full redraw.
#[test]
fn resize_full() {
    let (mut tk, _, _, _) = scene();
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    assert_eq!(tk.render_frame(400, 240).expect("frame"), FrameDamage::Full);
}

/// More than 16 invalidated rects coalesce into a single bounding box.
#[test]
fn invalidate_coalesces() {
    let (mut tk, _, _, _) = scene();
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    for i in 0..20 {
        tk.invalidate_rect(rtk::Rect::new(i as f32 * 4.0, 0.0, 2.0, 2.0));
    }
    match tk.render_frame(320, 200).expect("frame") {
        FrameDamage::Partial(rects) => assert_eq!(rects.len(), 1),
        other => panic!("expected partial, got {other:?}"),
    }
}

/// At scale 1.5 a partial frame still matches a full render.
#[test]
fn partial_matches_full_at_scale() {
    let (mut tk, b1, _, _) = scene();
    assert!(tk.sim_scale_factor(1.5));
    assert_eq!(tk.render_frame(480, 300).expect("frame"), FrameDamage::Full);
    let (x, y) = centre(&mut tk, b1);
    tk.sim_mouse_move(x, y);
    tk.sim_advance_time(1000);
    match tk.render_frame(480, 300).expect("frame") {
        FrameDamage::Partial(rects) => assert!(!rects.is_empty()),
        other => panic!("expected partial, got {other:?}"),
    }
    assert_frame_matches_full(&mut tk, 480, 300);
}

/// `Opt::Text` on a label requests a full redraw.
///
/// `config` can change a widget's requested size (e.g. a label's text), which
/// may reflow the layout, so it conservatively sets the full-redraw flag rather
/// than a partial damage rectangle. The resulting frame must still match a
/// fresh full render.
#[test]
fn config_text_full() {
    let mut tk = Tk::new();
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Hello").expect("label");
    tk.pack(label, Default::default());
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    tk.config(label, Opt::Text("x".into()));
    assert!(tk.needs_redraw());
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::Full);
    assert_frame_matches_full(&mut tk, 320, 200);
}
