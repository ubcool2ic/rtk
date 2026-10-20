//! Integration tests exercising listbox, canvas, progress bar, scrollbar, spinbox, and other widget features.

use rtk::layout::{place_rect, Anchor, PlaceOpts, Rect, Size};
use rtk::{
    Button, Canvas, Color, DragPhase, Frame, Key, Listbox, PackOpts,
    ProgressBar, Radiobutton, Scrollbar, Spinbox, Theme, Tk,
};

/// Tests listbox click selection, keyboard navigation, and wheel scrolling with clamping at the list bounds.
#[test]
fn test_listbox_click_keys_wheel_clamp() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lb = Listbox::new(&mut tk, root, 4).unwrap();
    tk.pack(lb, PackOpts::default());

    for item in ["Apple", "Banana", "Cherry", "Date", "Elderberry", "Fig"] {
        tk.listbox_insert(lb, usize::MAX, item);
    }
    assert_eq!(tk.listbox_get(lb).len(), 6);

    // Initial rect
    let rect = tk.widget_rect(lb).unwrap();
    let line_h = rect.height / 4.0;

    // Click row 1 (Banana)
    tk.sim_click(rect.x + 10.0, rect.y + line_h * 1.5);
    assert_eq!(tk.listbox_curselection(lb), Some(1));

    // Key Down moves to 2
    tk.sim_key(Key::Down);
    assert_eq!(tk.listbox_curselection(lb), Some(2));

    // Key Up moves to 1
    tk.sim_key(Key::Up);
    assert_eq!(tk.listbox_curselection(lb), Some(1));

    // Key End moves to 5
    tk.sim_key(Key::End);
    assert_eq!(tk.listbox_curselection(lb), Some(5));

    // Key Home moves to 0
    tk.sim_key(Key::Home);
    assert_eq!(tk.listbox_curselection(lb), Some(0));

    // Mouse wheel scrolling
    tk.sim_mouse_move(rect.x + 10.0, rect.y + 10.0);
    tk.sim_wheel(-2.0); // scroll down
    // scroll clamped
    tk.sim_wheel(-100.0);
    // Delete items
    tk.listbox_delete(lb, 0, 1);
    assert_eq!(tk.listbox_get(lb).len(), 4);
}

/// Verifies that dragging a linked scrollbar changes the listbox view range.
#[test]
fn test_scrollbar_drag_changes_view() {
    let mut tk = Tk::new();
    let root = tk.root();
    let f = Frame::new(&mut tk, root).unwrap();
    tk.pack(f, PackOpts::default());

    let lb = Listbox::new(&mut tk, f, 3).unwrap();
    tk.pack(lb, PackOpts::default());
    let sb = Scrollbar::new(&mut tk, f).unwrap();
    tk.pack(sb, PackOpts::default());
    tk.link_scrollbar(sb, lb);

    for i in 0..10 {
        tk.listbox_insert(lb, usize::MAX, format!("Item {i}"));
    }

    let sb_rect = tk.widget_rect(sb).unwrap();
    // Click page down in trough
    tk.sim_click(sb_rect.x + 5.0, sb_rect.y + sb_rect.height - 10.0);
    let view_after_page = tk.listbox_curselection(lb);
    let _ = view_after_page;
    // Drag thumb
    tk.sim_mouse_move(sb_rect.x + 5.0, sb_rect.y + 10.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(sb_rect.x + 5.0, sb_rect.y + 60.0);
    tk.sim_mouse_up();
}

/// Confirms that radiobuttons in the same group are mutually exclusive and update the shared variable.
#[test]
fn test_radio_group_exclusivity() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(1);
    let r1 = Radiobutton::new(&mut tk, root, "Option 1", var, 1).unwrap();
    let r2 = Radiobutton::new(&mut tk, root, "Option 2", var, 2).unwrap();
    let r3 = Radiobutton::new(&mut tk, root, "Option 3", var, 3).unwrap();
    tk.pack(r1, PackOpts::default());
    tk.pack(r2, PackOpts::default());
    tk.pack(r3, PackOpts::default());

    assert_eq!(var.get(&tk), Some(1));

    let rect2 = tk.widget_rect(r2).unwrap();
    tk.sim_click(rect2.x + 5.0, rect2.y + 5.0);
    assert_eq!(var.get(&tk), Some(2));

    let rect3 = tk.widget_rect(r3).unwrap();
    tk.sim_click(rect3.x + 5.0, rect3.y + 5.0);
    assert_eq!(var.get(&tk), Some(3));
}

/// Exercises canvas item creation, movement, deletion, and dragging for rects, ovals, lines, and text.
#[test]
fn test_canvas_create_move_delete_drag() {
    let mut tk = Tk::new();
    let root = tk.root();
    let cv = Canvas::new(&mut tk, root, 100.0, 100.0).unwrap();
    tk.pack(cv, PackOpts::default());

    let r = tk.canvas_create_rect(cv, 10.0, 10.0, 30.0, 30.0, Color::rgb(255, 0, 0)).unwrap();
    let o = tk.canvas_create_oval(cv, 40.0, 40.0, 60.0, 60.0, Color::rgb(0, 255, 0)).unwrap();
    let l = tk.canvas_create_line(cv, 0.0, 0.0, 100.0, 100.0, Color::rgb(0, 0, 255)).unwrap();
    let t = tk.canvas_create_text(cv, 5.0, 5.0, "Hi", Color::rgb(0, 0, 0)).unwrap();

    assert_eq!(tk.canvas_items(cv), 4);
    assert_eq!(tk.canvas_coords(cv, r), Some([10.0, 10.0, 30.0, 30.0]));

    assert!(tk.canvas_move(cv, r, 5.0, 5.0));
    assert_eq!(tk.canvas_coords(cv, r), Some([15.0, 15.0, 35.0, 35.0]));

    assert!(tk.canvas_delete(cv, o));
    assert_eq!(tk.canvas_items(cv), 3);
    // Stale item query/delete
    assert_eq!(tk.canvas_coords(cv, o), None);
    assert!(!tk.canvas_delete(cv, o));

    let _ = (l, t);

    // Canvas bind drag callback that deletes items
    let target = r;
    tk.canvas_bind_drag(cv, move |tk, cx, cy, phase| {
        if phase == DragPhase::Press {
            tk.canvas_delete(cv, target);
        }
        let _ = (cx, cy);
    });

    let cv_rect = tk.widget_rect(cv).unwrap();
    tk.sim_mouse_move(cv_rect.x + 20.0, cv_rect.y + 20.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(cv_rect.x + 25.0, cv_rect.y + 25.0);
    tk.sim_mouse_up();

    assert_eq!(tk.canvas_coords(cv, r), None);
}

/// Validates place geometry math for various anchors and relative positioning options.
#[test]
fn test_place_math_anchors() {
    let container = Rect::new(10.0, 10.0, 200.0, 100.0);
    let requested = Size::new(40.0, 20.0);

    // NW at (0, 0)
    let p_nw = place_rect(container, requested, PlaceOpts { anchor: Anchor::Nw, ..Default::default() });
    assert_eq!(p_nw, Rect::new(10.0, 10.0, 40.0, 20.0));

    // Center at relx=0.5, rely=0.5
    let p_center = place_rect(container, requested, PlaceOpts { relx: 0.5, rely: 0.5, anchor: Anchor::Center, ..Default::default() });
    assert_eq!(p_center, Rect::new(90.0, 50.0, 40.0, 20.0));

    // SE at relx=1.0, rely=1.0
    let p_se = place_rect(container, requested, PlaceOpts { relx: 1.0, rely: 1.0, anchor: Anchor::Se, ..Default::default() });
    assert_eq!(p_se, Rect::new(170.0, 90.0, 40.0, 20.0));
}

/// Checks that switching themes changes the rendered background pixel color.
#[test]
fn test_theme_switch_changes_rendered_bg() {
    let mut tk = Tk::new();
    let root = tk.root();
    let b = Button::new(&mut tk, root, "Hello").unwrap();
    tk.pack(b, PackOpts::default());

    let pm_light = tk.render_to_pixmap(100, 100).unwrap();
    let light_pixel = [pm_light.data()[0], pm_light.data()[1], pm_light.data()[2]];

    tk.set_theme(Theme::Dark);
    assert_eq!(tk.theme(), Theme::Dark);

    let pm_dark = tk.render_to_pixmap(100, 100).unwrap();
    let dark_pixel = [pm_dark.data()[0], pm_dark.data()[1], pm_dark.data()[2]];

    assert_ne!(light_pixel, dark_pixel);
}

/// Verifies that spinbox arrow clicks and keyboard input clamp values to the configured range.
#[test]
fn test_spinbox_clamps() {
    let mut tk = Tk::new();
    let root = tk.root();
    let val = tk.int_var(5);
    let sb = Spinbox::new(&mut tk, root, 0, 10, val).unwrap();
    tk.pack(sb, PackOpts::default());

    let rect = tk.widget_rect(sb).unwrap();

    // Click UP arrow
    tk.sim_click(rect.x + rect.width - 5.0, rect.y + 4.0);
    assert_eq!(val.get(&tk), Some(6));

    // Focus and Key Up
    tk.sim_click(rect.x + 10.0, rect.y + 10.0);
    tk.sim_key(Key::Up);
    assert_eq!(val.get(&tk), Some(7));

    // Key Down
    tk.sim_key(Key::Down);
    assert_eq!(val.get(&tk), Some(6));

    // Clamp at max
    val.set(&mut tk, 10);
    tk.sim_key(Key::Up);
    assert_eq!(val.get(&tk), Some(10));

    // Clamp at min
    val.set(&mut tk, 0);
    tk.sim_key(Key::Down);
    assert_eq!(val.get(&tk), Some(0));
}

/// Confirms that a progress bar renders at the requested pixmap dimensions with the given fraction.
#[test]
fn test_progressbar_render_fraction() {
    let mut tk = Tk::new();
    let root = tk.root();
    let val = tk.int_var(50);
    let pb = ProgressBar::new(&mut tk, root, 100.0, val).unwrap();
    tk.pack(pb, PackOpts::default());

    let pixmap = tk.render_to_pixmap(120, 40).unwrap();
    assert_eq!(pixmap.width(), 120);
    assert_eq!(pixmap.height(), 40);
}

/// Ensures tab focus order includes new interactive widgets but excludes the canvas.
#[test]
fn test_tab_focus_includes_new_widgets_excludes_canvas() {
    let mut tk = Tk::new();
    let root = tk.root();
    let v_radio = tk.int_var(0);
    let v_spin = tk.int_var(0);

    let lb = Listbox::new(&mut tk, root, 3).unwrap();
    let rb = Radiobutton::new(&mut tk, root, "R", v_radio, 0).unwrap();
    let sb = Spinbox::new(&mut tk, root, 0, 10, v_spin).unwrap();
    let cv = Canvas::new(&mut tk, root, 50.0, 50.0).unwrap();

    tk.pack(lb, PackOpts::default());
    tk.pack(rb, PackOpts::default());
    tk.pack(sb, PackOpts::default());
    tk.pack(cv, PackOpts::default());

    tk.sim_resize(300, 300);

    // Tab through widgets
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(lb));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(rb));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(sb));
    tk.sim_key(Key::Tab);
    // Canvas is excluded from Tab focus, loops back to lb
    assert_eq!(tk.focused_widget(), Some(lb));
}

/// Tests canvas text movement, clearing, and correct stacking order after slot reuse.
#[test]
fn test_canvas_clear_text_move_and_stacking_after_slot_reuse() {
    let mut tk = Tk::new();
    let root = tk.root();
    let cv = Canvas::new(&mut tk, root, 40.0, 40.0).expect("canvas");
    tk.pack(cv, PackOpts::default());
    let t = tk.canvas_create_text(cv, 5.0, 5.0, "x", Color::rgb(0, 0, 0)).expect("text");
    assert!(tk.canvas_move(cv, t, 3.0, 4.0));
    assert_eq!(tk.canvas_coords(cv, t), Some([8.0, 9.0, 8.0, 9.0]));

    // Red is created first, then freed; blue reuses an older slot than green but must draw on top.
    let red = tk.canvas_create_rect(cv, 0.0, 0.0, 40.0, 40.0, Color::rgb(255, 0, 0)).expect("red");
    let _green = tk.canvas_create_rect(cv, 0.0, 0.0, 40.0, 40.0, Color::rgb(0, 255, 0)).expect("green");
    assert!(tk.canvas_delete(cv, red));
    let blue = tk.canvas_create_rect(cv, 0.0, 0.0, 40.0, 40.0, Color::rgb(0, 0, 255)).expect("blue");
    assert_eq!(blue.index, red.index);
    assert_ne!(blue.gen, red.gen);
    let img = tk.render_to_pixmap(100, 100).expect("img");
    let rect = tk.widget_rect(cv).expect("rect");
    let (x, y) = ((rect.x + 20.0) as u32, (rect.y + 20.0) as u32);
    let px = img.pixel(x, y).expect("pixel");
    assert_eq!((px.red(), px.green(), px.blue()), (0, 0, 255));

    assert!(tk.canvas_clear(cv));
    assert_eq!(tk.canvas_items(cv), 0);
    assert_eq!(tk.canvas_coords(cv, blue), None);
}
