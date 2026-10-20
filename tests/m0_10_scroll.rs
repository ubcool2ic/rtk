//! Behaviour and golden tests for the horizontal Scrollbar and the ScrollArea.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{
    Button, Canvas, Color, Fill, Frame, Key, Label, Orient, PackOpts, ScrollArea, Scrollbar, Side,
    Theme, Tk,
};

/// Builds a Canvas with a scrollregion plus a horizontal scrollbar linked to it.
fn hscroll_canvas() -> (Tk, rtk::WidgetId, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 300.0, 160.0).unwrap();
    let sb = Scrollbar::horizontal(&mut tk, root).unwrap();
    tk.pack(c, PackOpts { side: Side::Top, ..Default::default() });
    tk.pack(sb, PackOpts { side: Side::Top, fill: Fill::X, ..Default::default() });
    tk.sim_resize(320, 200);
    for i in 0..20 {
        let x = i as f32 * 50.0;
        tk.canvas_create_rect(c, x, 0.0, x + 50.0, 160.0, if i % 2 == 0 { Color::rgb(255, 0, 0) } else { Color::rgb(0, 0, 255) });
    }
    assert!(tk.canvas_scrollregion(c, 1000.0, 160.0));
    assert!(tk.link_scrollbar(sb, c));
    (tk, c, sb)
}

/// `Scrollbar::horizontal` reports the horizontal orientation; `Scrollbar::new` vertical.
#[test]
fn scrollbar_orientation() {
    let mut tk = Tk::new();
    let root = tk.root();
    let v = Scrollbar::new(&mut tk, root).unwrap();
    let h = Scrollbar::horizontal(&mut tk, root).unwrap();
    assert_eq!(tk.scrollbar_orient(v), Some(Orient::Vertical));
    assert_eq!(tk.scrollbar_orient(h), Some(Orient::Horizontal));
    tk.pack(v, PackOpts::default());
    tk.pack(h, PackOpts::default());
    tk.sim_resize(320, 200);
    let vr = tk.widget_rect(v).unwrap();
    let hr = tk.widget_rect(h).unwrap();
    assert!(vr.height > vr.width, "vertical {vr:?}");
    assert!(hr.width > hr.height, "horizontal {hr:?}");
}

/// Dragging the horizontal thumb moves the canvas X view.
#[test]
fn hscroll_drag_thumb() {
    let (mut tk, c, sb) = hscroll_canvas();
    let r = tk.widget_rect(sb).unwrap();
    // Thumb width = 320/1000 * bar width; drag by half the free track.
    tk.sim_mouse_move(r.x + 4.0, r.y + r.height / 2.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.sim_mouse_up();
    let vx = tk.canvas_view(c).unwrap().0;
    assert!(vx > 0.0, "vx {vx}");
}

/// Clicking the horizontal trough pages the canvas X view.
#[test]
fn hscroll_trough_click() {
    let (mut tk, c, sb) = hscroll_canvas();
    let r = tk.widget_rect(sb).unwrap();
    tk.sim_click(r.x + r.width - 5.0, r.y + r.height / 2.0);
    let vx = tk.canvas_view(c).unwrap().0;
    assert!(vx > 0.0, "vx {vx}");
}

/// Horizontal wheel delta over a canvas scrolls X.
#[test]
fn hscroll_wheel_x() {
    let (mut tk, c, _) = hscroll_canvas();
    let r = tk.widget_rect(c).unwrap();
    tk.sim_mouse_move(r.x + 10.0, r.y + 10.0);
    tk.sim_wheel_xy(-1.0, 0.0, false);
    assert_eq!(tk.canvas_view(c).unwrap().0, 48.0);
}

/// Builds a ScrollArea with 30 labels inside its content frame.
fn scroll_area_scene() -> (Tk, rtk::WidgetId, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 200.0, 120.0).unwrap();
    tk.pack(area, PackOpts::default());
    let content = tk.scrollarea_content(area).unwrap();
    for i in 0..30 {
        let label = Label::new(&mut tk, content, format!("row {i}")).unwrap();
        tk.pack(label, PackOpts::default());
    }
    tk.sim_resize(320, 200);
    (tk, area, content)
}

/// Wheel over a label inside the area scrolls it; the offset clamps at the end.
#[test]
fn scrollarea_wheel_and_clamp() {
    let (mut tk, area, _) = scroll_area_scene();
    let r = tk.widget_rect(area).unwrap();
    tk.sim_mouse_move(r.x + 10.0, r.y + 10.0);
    tk.sim_wheel(-1.0);
    let (_, y) = tk.scrollarea_offset(area).unwrap();
    assert!(y > 0.0, "y {y}");
    tk.sim_wheel(-1000.0);
    let (_, y) = tk.scrollarea_offset(area).unwrap();
    let (_, ch) = tk.scrollarea_content_size(area).unwrap();
    let max = (ch - r.height).max(0.0);
    assert!((y - max).abs() < 0.5, "y {y} max {max}");
    tk.sim_wheel(1000.0);
    assert_eq!(tk.scrollarea_offset(area), Some((0.0, 0.0)));
}

/// `scrollarea_scroll_to` clamps and `scrollarea_content_size` reports the content.
#[test]
fn scrollarea_scroll_to_and_content_size() {
    let (mut tk, area, _) = scroll_area_scene();
    let (cw, ch) = tk.scrollarea_content_size(area).unwrap();
    assert!(cw >= 200.0 && ch > 120.0, "content {cw}x{ch}");
    assert!(tk.scrollarea_scroll_to(area, 0.0, 40.0));
    assert_eq!(tk.scrollarea_offset(area), Some((0.0, 40.0)));
    assert!(tk.scrollarea_scroll_to(area, 0.0, 100_000.0));
    let (_, y) = tk.scrollarea_offset(area).unwrap();
    assert!((y - (ch - 120.0)).abs() < 0.5, "y {y}");
}

/// A click on a button scrolled into view hits it; one scrolled out does not.
#[test]
fn scrollarea_hit_test_respects_clip() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 200.0, 100.0).unwrap();
    tk.pack(area, PackOpts::default());
    let content = tk.scrollarea_content(area).unwrap();
    let spacer = Frame::new(&mut tk, content).unwrap();
    tk.pack(spacer, PackOpts { fill: Fill::X, ..Default::default() });
    tk.config(spacer, rtk::Opt::Width(20));
    let top = Button::new(&mut tk, content, "top").unwrap();
    tk.pack(top, PackOpts::default());
    let filler = Frame::new(&mut tk, content).unwrap();
    tk.pack(filler, PackOpts::default());
    tk.config(filler, rtk::Opt::Width(20));
    let bottom = Button::new(&mut tk, content, "bottom").unwrap();
    tk.pack(bottom, PackOpts::default());
    tk.sim_resize(320, 200);
    let hits = Rc::new(Cell::new(0));
    let h = hits.clone();
    tk.set_command(bottom, move |_| h.set(h.get() + 1));
    // Scroll so the bottom button is visible, then click it.
    assert!(tk.scrollarea_scroll_to(area, 0.0, 1000.0));
    let br = tk.widget_rect(bottom).unwrap();
    let ar = tk.widget_rect(area).unwrap();
    if br.y >= ar.y && br.y + br.height <= ar.y + ar.height {
        tk.sim_click(br.x + br.width / 2.0, br.y + br.height / 2.0);
        assert_eq!(hits.get(), 1);
    }
    // Scroll back to the top: the bottom button is out of view and not clickable.
    assert!(tk.scrollarea_scroll_to(area, 0.0, 0.0));
    let br = tk.widget_rect(bottom).unwrap();
    if br.y >= ar.y + ar.height {
        tk.sim_click(br.x + br.width / 2.0, ar.y + ar.height - 2.0);
        assert_eq!(hits.get(), 1);
    }
}

/// Tab focus reaches and reveals focusable widgets below the static labels.
#[test]
fn scrollarea_tab_reveals() {
    let (mut tk, area, content) = scroll_area_scene();
    let first = Button::new(&mut tk, content, "first").unwrap();
    let last = Button::new(&mut tk, content, "last").unwrap();
    assert!(tk.pack(first, PackOpts::default()));
    assert!(tk.pack(last, PackOpts::default()));

    let viewport = tk.widget_rect(area).unwrap();
    let initial = tk.widget_rect(first).unwrap();
    assert!(initial.y >= viewport.y + viewport.height);

    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(first));
    let revealed = tk.widget_rect(first).unwrap();
    assert!(revealed.y >= viewport.y);
    assert!(revealed.y + revealed.height <= viewport.y + viewport.height + 0.5);

    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(last));
    let revealed = tk.widget_rect(last).unwrap();
    assert!(revealed.y >= viewport.y);
    assert!(revealed.y + revealed.height <= viewport.y + viewport.height + 0.5);

    let (_, y) = tk.scrollarea_offset(area).unwrap();
    assert!(y > 0.0, "y {y}");
}

/// Attached scrollbars follow the area offset.
#[test]
fn scrollarea_attached_scrollbars() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 200.0, 120.0).unwrap();
    let vbar = Scrollbar::new(&mut tk, root).unwrap();
    let hbar = Scrollbar::horizontal(&mut tk, root).unwrap();
    tk.pack(area, PackOpts::default());
    tk.pack(vbar, PackOpts::default());
    tk.pack(hbar, PackOpts::default());
    let content = tk.scrollarea_content(area).unwrap();
    for i in 0..30 {
        let label = Label::new(&mut tk, content, format!("row {i}")).unwrap();
        tk.pack(label, PackOpts::default());
    }
    tk.sim_resize(320, 200);
    assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), Some(hbar)));
    assert!(tk.scrollarea_scroll_to(area, 0.0, 60.0));
    let (first, last) = tk.yview(area);
    assert!(first > 0.0 && last <= 1.0, "view {first} {last}");
}

/// Existing vertical scrollbar behaviour is unchanged.
#[test]
fn vertical_scrollbar_unchanged() {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 300.0, 200.0).unwrap();
    let sb = Scrollbar::new(&mut tk, root).unwrap();
    tk.pack(c, PackOpts { side: Side::Left, ..Default::default() });
    tk.pack(sb, PackOpts { side: Side::Left, fill: Fill::Y, ..Default::default() });
    tk.sim_resize(320, 200);
    assert!(tk.canvas_scrollregion(c, 300.0, 1000.0));
    assert!(tk.link_scrollbar(sb, c));
    tk.yview_moveto(c, 0.5);
    assert_eq!(tk.canvas_view(c), Some((0.0, 500.0)));
}

/// Golden images for the horizontal scrollbar scene.
#[test]
fn golden_hscroll() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let (mut tk, c, _) = hscroll_canvas();
            tk.set_theme(theme);
            tk.yview_moveto(c, 0.0);
            tk.xview_moveto(c, 0.5);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("hscroll_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}

/// Golden images for the ScrollArea scene.
#[test]
fn golden_scrollarea() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            let root = tk.root();
            let area = ScrollArea::new(&mut tk, root, 200.0, 120.0).unwrap();
            let vbar = Scrollbar::new(&mut tk, root).unwrap();
            let hbar = Scrollbar::horizontal(&mut tk, root).unwrap();
            tk.pack(area, PackOpts::default());
            tk.pack(vbar, PackOpts::default());
            tk.pack(hbar, PackOpts::default());
            let content = tk.scrollarea_content(area).unwrap();
            for i in 0..12 {
                let label = Label::new(&mut tk, content, format!("row {i}")).unwrap();
                tk.pack(label, PackOpts::default());
            }
            tk.sim_resize(320, 200);
            assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), Some(hbar)));
            assert!(tk.scrollarea_scroll_to(area, 0.0, 60.0));
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("scrollarea_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}

