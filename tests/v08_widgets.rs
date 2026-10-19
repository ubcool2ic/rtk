//! Integration tests for rtk v0.8.0 features.

use rtk::colormap::{self, Colormap};
use rtk::{
    Button, Canvas, Color, Combobox, DragPhase, Entry, Frame, Key, Label, Listbox,
    Notebook, Opt, PackOpts, ProgressBar, ScrollUnit, Text, Tk,
};

#[test]
fn test_text_widget_basic_and_editing() {
    let mut tk = Tk::new();
    let root = tk.root();
    let text = Text::new(&mut tk, root, 30, 5).expect("create Text");
    tk.pack(text, PackOpts::default());

    assert_eq!(tk.text_get(text), Some(String::new()));
    assert_eq!(tk.text_line_count(text), Some(1));

    // text_set
    assert!(tk.text_set(text, "Hello\nWorld"));
    assert_eq!(tk.text_get(text).as_deref(), Some("Hello\nWorld"));
    assert_eq!(tk.text_line_count(text), Some(2));

    // text_insert_end
    assert!(tk.text_insert_end(text, "\nThird line"));
    assert_eq!(tk.text_line_count(text), Some(3));

    // text_set_max_lines
    assert!(tk.text_set_max_lines(text, 2));
    assert_eq!(tk.text_line_count(text), Some(2));
    assert_eq!(tk.text_get(text).as_deref(), Some("World\nThird line"));

    // text_clear
    assert!(tk.text_clear(text));
    assert_eq!(tk.text_get(text).as_deref(), Some(""));

    // Keyboard editing when focused
    tk.text_set(text, "abc");
    let rect = tk.widget_rect(text).expect("widget_rect");
    tk.sim_click(rect.x + 10.0, rect.y + 10.0);
    assert_eq!(tk.focused_widget(), Some(text));
    // Clicking places the caret under the pointer; move it to the end first.
    tk.sim_key(Key::End);

    tk.sim_text("d");
    assert_eq!(tk.text_get(text).as_deref(), Some("abcd"));

    tk.sim_key(Key::Enter);
    assert_eq!(tk.text_get(text).as_deref(), Some("abcd\n"));

    tk.sim_key(Key::Backspace);
    assert_eq!(tk.text_get(text).as_deref(), Some("abcd"));

    // read_only toggle
    tk.text_set_readonly(text, true);
    tk.sim_text("z");
    assert_eq!(tk.text_get(text).as_deref(), Some("abcd")); // unchanged
    tk.text_set_readonly(text, false);
    tk.sim_text("z");
    assert_eq!(tk.text_get(text).as_deref(), Some("abcdz"));
}

#[test]
fn test_listbox_select_callback() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lb = Listbox::new(&mut tk, root, 4).expect("create listbox");
    tk.pack(lb, PackOpts::default());

    tk.listbox_insert(lb, 0, "Item 0");
    tk.listbox_insert(lb, 1, "Item 1");
    tk.listbox_insert(lb, 2, "Item 2");

    let sel_var = tk.int_var(-1);
    tk.listbox_bind_select(lb, move |tk, sel| {
        sel_var.set(tk, sel.map(|s| s as i64).unwrap_or(-1));
    });

    // select via method
    tk.listbox_select(lb, Some(1));
    assert_eq!(sel_var.get(&tk), Some(1));

    // select via keyboard Down
    let rect = tk.widget_rect(lb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    // Clicking the first row selects 0 (callback fires), Down then moves to 1.
    assert_eq!(sel_var.get(&tk), Some(0));
    tk.sim_key(Key::Down);
    assert_eq!(sel_var.get(&tk), Some(1));
}

#[test]
fn test_combobox() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(0);
    let values = vec!["Apple".into(), "Banana".into(), "Cherry".into()];
    let cb = Combobox::new(&mut tk, root, values, var).expect("create combobox");
    tk.pack(cb, PackOpts::default());

    assert_eq!(tk.combobox_get(cb), Some((0, "Apple".into())));

    // Focus and step with keys
    let rect = tk.widget_rect(cb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0); // opens dropdown
    tk.sim_key(Key::Escape); // closes dropdown

    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(1));
    assert_eq!(tk.combobox_get(cb), Some((1, "Banana".into())));

    // Wheel step
    tk.sim_mouse_move(rect.x + 5.0, rect.y + 5.0);
    tk.sim_wheel(-1.0); // next
    assert_eq!(var.get(&tk), Some(2));
    assert_eq!(tk.combobox_get(cb), Some((2, "Cherry".into())));
}

#[test]
fn test_notebook_and_geometry_forget() {
    let mut tk = Tk::new();
    let root = tk.root();

    let nb = Notebook::new(&mut tk, root).expect("create notebook");
    tk.pack(nb, PackOpts::default());

    let p1 = Frame::new(&mut tk, nb).expect("p1");
    let l1 = Label::new(&mut tk, p1, "Page 1 Content").expect("l1");
    tk.pack(l1, PackOpts::default());
    tk.notebook_add(nb, p1, "Tab 1");

    let p2 = Frame::new(&mut tk, nb).expect("p2");
    let l2 = Label::new(&mut tk, p2, "Page 2 Content").expect("l2");
    tk.pack(l2, PackOpts::default());
    tk.notebook_add(nb, p2, "Tab 2");

    assert_eq!(tk.notebook_selected(nb), Some(0));
    assert!(tk.is_mapped(p1));
    assert!(!tk.is_mapped(p2));

    // Switch page
    let changed_var = tk.int_var(-1);
    tk.notebook_bind_changed(nb, move |tk, idx| {
        changed_var.set(tk, idx as i64);
    });

    tk.notebook_select(nb, 1);
    assert_eq!(tk.notebook_selected(nb), Some(1));
    assert_eq!(changed_var.get(&tk), Some(1));
    assert!(!tk.is_mapped(p1));
    assert!(tk.is_mapped(p2));

    // pack_forget / is_mapped
    let btn = Button::new(&mut tk, root, "Test").expect("btn");
    tk.pack(btn, PackOpts::default());
    assert!(tk.is_mapped(btn));
    assert!(tk.pack_forget(btn));
    assert!(!tk.is_mapped(btn));
}

#[test]
fn test_progressbar_f64_and_indeterminate() {
    let mut tk = Tk::new();
    let root = tk.root();
    let ivar = tk.int_var(0);
    let pb = ProgressBar::new(&mut tk, root, 100.0, ivar).expect("pb");
    tk.pack(pb, PackOpts::default());

    let fvar = tk.f64_var(0.75);
    assert!(tk.progress_bind(pb, fvar));

    assert!(!tk.is_animating());
    assert!(tk.progress_set_indeterminate(pb, true));
    tk.sim_resize(200, 100);
    assert!(tk.is_animating());

    tk.sim_advance_time(100);
    assert!(tk.is_animating());

    tk.progress_set_indeterminate(pb, false);
    assert!(!tk.is_animating());
}

#[test]
fn test_monospace_font_option() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lbl = Label::new(&mut tk, root, "Code text").expect("lbl");
    assert!(tk.config(lbl, Opt::Monospace(true)));
    tk.pack(lbl, PackOpts::default());

    let pm = tk.render_to_pixmap(100, 50);
    assert!(pm.is_ok());
}

#[test]
fn test_tooltips() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Hover me").expect("btn");
    tk.pack(btn, PackOpts::default());

    assert!(tk.tooltip(btn, "Click to run"));
    assert_eq!(tk.active_tooltip(), None);

    let rect = tk.widget_rect(btn).expect("rect");
    tk.sim_mouse_move(rect.x + 5.0, rect.y + 5.0);

    tk.sim_advance_time(200);
    assert_eq!(tk.active_tooltip(), None);

    tk.sim_advance_time(350);
    assert_eq!(tk.active_tooltip().as_deref(), Some("Click to run"));

    // Move out hides tooltip
    tk.sim_mouse_move(500.0, 500.0);
    assert_eq!(tk.active_tooltip(), None);

    // Remove tooltip
    assert!(tk.tooltip_remove(btn));
    tk.sim_mouse_move(rect.x + 5.0, rect.y + 5.0);
    tk.sim_advance_time(600);
    assert_eq!(tk.active_tooltip(), None);
}

#[test]
fn test_canvas_image_rgba() {
    let mut tk = Tk::new();
    let root = tk.root();
    let cv = Canvas::new(&mut tk, root, 100.0, 100.0).expect("cv");
    tk.pack(cv, PackOpts::default());

    let rgba = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255]; // 2x2
    let item = tk.canvas_create_image_rgba(cv, 10.0, 10.0, 20.0, 20.0, 2, 2, &rgba, false)
        .expect("canvas image");

    let coords = tk.canvas_coords(cv, item).expect("coords");
    assert_eq!(coords, [10.0, 10.0, 30.0, 30.0]);

    tk.canvas_move(cv, item, 5.0, 5.0);
    let coords2 = tk.canvas_coords(cv, item).expect("coords2");
    assert_eq!(coords2, [15.0, 15.0, 35.0, 35.0]);

    assert!(tk.canvas_update_image(cv, item, 2, 2, &rgba));
    assert!(tk.canvas_delete(cv, item));
}

#[test]
fn test_colormaps() {
    let c_vir = colormap::sample(Colormap::Viridis, 0.5);
    assert_eq!(c_vir.a, 255);

    let vals = [0.0, 50.0, 100.0, f32::NAN];
    let rgba = colormap::heatmap_to_rgba(&vals, 0.0, 100.0, Colormap::Inferno);
    assert_eq!(rgba.len(), 16);
    // NaN is transparent
    assert_eq!(&rgba[12..16], &[0, 0, 0, 0]);

    // min == max safe
    let rgba_flat = colormap::heatmap_to_rgba(&[10.0], 10.0, 10.0, Colormap::Jet);
    assert_eq!(rgba_flat.len(), 4);
    assert_eq!(rgba_flat[3], 255);

    let leg_h = colormap::legend_rgba(Colormap::Magma, 10, 2, true);
    assert_eq!(leg_h.len(), 10 * 2 * 4);

    let leg_v = colormap::legend_rgba(Colormap::Turbo, 2, 10, false);
    assert_eq!(leg_v.len(), 2 * 10 * 4);
}
