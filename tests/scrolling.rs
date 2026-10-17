use rtk::{Canvas, Color, DragPhase, Key, Listbox, PackOpts, ScrollUnit, Scrollbar, Side, Tk};
use std::{cell::Cell, rc::Rc};

fn scroll_canvas() -> (Tk, rtk::WidgetId, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let c = Canvas::new(&mut tk, root, 300.0, 200.0).unwrap();
    let sb = Scrollbar::new(&mut tk, root).unwrap();
    tk.pack(c, PackOpts { side: Side::Left, ..Default::default() });
    tk.pack(sb, PackOpts { side: Side::Left, fill: rtk::Fill::Y, ..Default::default() });
    tk.sim_resize(320, 200);
    for i in 0..20 {
        let y = i as f32 * 50.0;
        tk.canvas_create_rect(c, 0.0, y, 300.0, y + 50.0, if i % 2 == 0 { Color::rgb(255, 0, 0) } else { Color::rgb(0, 0, 255) });
    }
    assert!(tk.canvas_scrollregion(c, 300.0, 1000.0));
    assert!(tk.link_scrollbar(sb, c));
    (tk, c, sb)
}

#[test]
fn yview_moveto_scroll_and_clamp() {
    let (mut tk, c, _) = scroll_canvas();
    assert_eq!(tk.yview(c), (0.0, 0.2));
    tk.yview_moveto(c, 0.5);
    assert_eq!(tk.canvas_view(c), Some((0.0, 500.0)));
    tk.yview_scroll(c, 1, ScrollUnit::Units);
    assert_eq!(tk.canvas_view(c).unwrap().1, 548.0);
    tk.yview_scroll(c, 10, ScrollUnit::Pages);
    assert_eq!(tk.canvas_view(c).unwrap().1, 800.0); // clamped to region - visible
    assert_eq!(tk.yview(c), (0.8, 1.0));
    tk.yview_moveto(c, -3.0);
    assert_eq!(tk.canvas_view(c).unwrap().1, 0.0);
    // shrinking the region clamps the view
    tk.yview_moveto(c, 1.0);
    tk.canvas_scrollregion(c, 300.0, 300.0);
    assert_eq!(tk.canvas_view(c).unwrap().1, 100.0);
}

#[test]
fn wheel_touchpad_and_render_offset() {
    let (mut tk, c, _) = scroll_canvas();
    let r = tk.widget_rect(c).unwrap();
    tk.sim_mouse_move(r.x + 10.0, r.y + 10.0);
    tk.sim_wheel(-1.0); // one notch down
    assert_eq!(tk.canvas_view(c).unwrap().1, 48.0);
    tk.sim_wheel_xy(0.0, -2.0, true); // touchpad: 2 px
    assert_eq!(tk.canvas_view(c).unwrap().1, 50.0);
    // At y=50 the second (blue) band is at the top of the view.
    let p = tk.render_to_pixmap(320, 200).unwrap();
    let i = (((r.y as u32 + 5) * 320 + r.x as u32 + 5) * 4) as usize;
    assert_eq!(&p.data()[i..i + 3], &[0, 0, 255]);
}

#[test]
fn canvas_events_use_content_coordinates() {
    let (mut tk, c, _) = scroll_canvas();
    let got = Rc::new(Cell::new((0.0f32, 0.0f32)));
    let g = got.clone();
    tk.canvas_bind_drag(c, move |_, x, y, ph| if ph == DragPhase::Press { g.set((x, y)) });
    tk.yview_moveto(c, 0.3);
    let r = tk.widget_rect(c).unwrap();
    tk.sim_click(r.x + 10.0, r.y + 20.0);
    assert_eq!(got.get(), (10.0, 320.0));
}

#[test]
fn keyboard_scrolls_focused_canvas() {
    let (mut tk, c, _) = scroll_canvas();
    let r = tk.widget_rect(c).unwrap();
    tk.sim_click(r.x + 5.0, r.y + 5.0); // focus
    tk.sim_key(Key::PageDown);
    assert_eq!(tk.canvas_view(c).unwrap().1, 152.0);
    tk.sim_key(Key::End);
    assert_eq!(tk.canvas_view(c).unwrap().1, 800.0);
    tk.sim_key(Key::PageUp);
    assert_eq!(tk.canvas_view(c).unwrap().1, 648.0);
    tk.sim_key(Key::Home);
    assert_eq!(tk.canvas_view(c).unwrap().1, 0.0);
}

#[test]
fn scrollbar_click_track_and_drag_thumb() {
    let (mut tk, c, sb) = scroll_canvas();
    let r = tk.widget_rect(sb).unwrap();
    // click the track below the thumb = page down
    tk.sim_click(r.x + 4.0, r.y + r.height - 5.0);
    assert_eq!(tk.canvas_view(c).unwrap().1, 200.0);
    tk.yview_moveto(c, 0.0);
    // drag the thumb (height 40 of 200) down by half the free track (80 px) => half of max (400)
    tk.sim_mouse_move(r.x + 4.0, r.y + 10.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(r.x + 4.0, r.y + 90.0);
    tk.sim_mouse_up();
    assert!((tk.canvas_view(c).unwrap().1 - 400.0).abs() < 0.5, "{:?}", tk.canvas_view(c));
    // wheel over the scrollbar scrolls its target
    tk.sim_mouse_move(r.x + 4.0, r.y + 100.0);
    tk.sim_wheel(1.0);
    assert!((tk.canvas_view(c).unwrap().1 - 352.0).abs() < 0.5);
}

#[test]
fn listbox_scrolling_still_works() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lb = Listbox::new(&mut tk, root, 5).unwrap();
    tk.pack(lb, PackOpts::default());
    for i in 0..20 { tk.listbox_insert(lb, i, format!("row {i}")); }
    tk.yview_scroll(lb, 3, ScrollUnit::Units);
    assert_eq!(tk.yview(lb), (0.15, 0.4));
    tk.yview_moveto(lb, 1.0);
    assert_eq!(tk.yview(lb), (0.75, 1.0));
}
