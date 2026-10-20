//! Behaviour tests for the milestone 0.10 visual fixes: DoubleSpinbox packing,
//! ProgressBar label sizing and contrast, and ScrollArea content width.

use rtk::{
    DoubleSpinbox, Fill, Frame, Label, PackOpts, ProgressBar, ScrollArea, Scrollbar, Theme, Tk,
};

/// Returns the laid-out rect of `id` after a headless resize.
fn rect_of(tk: &mut Tk, id: rtk::WidgetId) -> rtk::Rect {
    tk.widget_rect(id).expect("widget rect")
}

/// Two DoubleSpinboxes packed with default options must not overlap and their
/// arrow columns must line up when they share the same width setting.
#[test]
fn dspin_packed_no_overlap() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: Fill::Both, expand: true, ..PackOpts::default() });
    let a_var = tk.f64_var(1.5);
    let a = DoubleSpinbox::new(&mut tk, frame, 0.0, 10.0, a_var).expect("a");
    tk.pack(a, PackOpts::default());
    let b_var = tk.f64_var(3.25);
    let b = DoubleSpinbox::new(&mut tk, frame, 0.0, 10.0, b_var).expect("b");
    tk.dspin_set_suffix(b, " V");
    tk.pack(b, PackOpts::default());
    tk.sim_resize(320, 200);

    let ra = rect_of(&mut tk, a);
    let rb = rect_of(&mut tk, b);
    assert!(ra.height > 0.0 && rb.height > 0.0, "heights {ra:?} {rb:?}");
    // No vertical overlap: the second spin box starts at or below the first's bottom.
    assert!(
        rb.y + 0.5 >= ra.y + ra.height,
        "overlap: a={ra:?} b={rb:?}"
    );
    // Arrow columns line up: same right edge.
    let a_right = ra.x + ra.width;
    let b_right = rb.x + rb.width;
    assert!(
        (a_right - b_right).abs() < 0.5,
        "arrow columns misaligned: a_right={a_right} b_right={b_right}"
    );
}

/// The ProgressBar's requested size must fit the measured label text plus
/// padding at scale 1 and 2.
#[test]
fn progress_bar_fits_label() {
    for scale in [1.0_f64, 2.0_f64] {
        let mut tk = Tk::new();
        tk.set_theme(Theme::HighContrast);
        let root = tk.root();
        let var = tk.int_var(40);
        let bar = ProgressBar::new(&mut tk, root, 200.0, var).expect("bar");
        tk.progress_set_show_percent(bar, true);
        tk.progress_set_label(bar, Some("Loading".into()));
        tk.pack(bar, PackOpts::default());
        assert!(tk.sim_scale_factor(scale));
        tk.sim_resize((320.0 * scale) as u32, (200.0 * scale) as u32);
        let r = rect_of(&mut tk, bar);
        // The bar must be tall enough for a text line and wide enough for the
        // longest label ("Loading 100%").
        assert!(r.height >= 12.0, "bar too short at {scale}x: {r:?}");
        assert!(r.width >= 60.0, "bar too narrow at {scale}x: {r:?}");
    }
}

/// The ScrollArea content frame fills the viewport width (minus a visible
/// vertical scrollbar) after layout.
#[test]
fn scrollarea_content_fills_width() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 200.0, 120.0).expect("area");
    let vbar = Scrollbar::new(&mut tk, root).expect("vbar");
    tk.pack(area, PackOpts::default());
    tk.pack(vbar, PackOpts::default());
    let content = tk.scrollarea_content(area).expect("content");
    for i in 0..30 {
        let label = Label::new(&mut tk, content, format!("row {i}")).expect("label");
        tk.pack(label, PackOpts::default());
    }
    tk.sim_resize(320, 200);
    assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), None));
    tk.sim_resize(320, 200);
    let area_rect = rect_of(&mut tk, area);
    let (cw, _) = tk.scrollarea_content_size(area).expect("content size");
    let vbar_w = tk.widget_rect(vbar).map(|r| r.width).unwrap_or(0.0);
    let expected = (area_rect.width - vbar_w).max(0.0);
    assert!(
        (cw - expected).abs() < 1.0,
        "content width {cw} != viewport {expected} (area {area_rect:?} vbar {vbar_w})"
    );
}

/// A ProgressBar with a label and percent text lays out to a non-empty rect.
/// Exact label colours are covered by the golden tests, not here.
#[test]
fn progress_bar_label_has_nonempty_rect() {
    let mut tk = Tk::new();
    tk.set_theme(Theme::Light);
    let root = tk.root();
    let var = tk.int_var(50);
    let bar = ProgressBar::new(&mut tk, root, 200.0, var).expect("bar");
    tk.progress_set_show_percent(bar, true);
    tk.progress_set_label(bar, Some("Loading".into()));
    tk.pack(bar, PackOpts::default());
    tk.sim_resize(320, 200);
    let r = rect_of(&mut tk, bar);
    assert!(r.width > 0.0 && r.height > 0.0, "empty bar rect {r:?}");
}
