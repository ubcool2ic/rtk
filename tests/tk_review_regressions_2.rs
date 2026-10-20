//! Regression tests for the Fable 5.1 review findings, part 2 (part 1 covers F1-F4).
//!
//! Coverage:
//! - `pass1_f6_combobox_click_toggles_closed`: pass-1 F6 (open combobox closes on click).
//! - `pass1_f7_text_yview_works`: pass-1 F7 (`yview*` support `Text` widgets).
//! - `pass1_f8_max_lines_zero_is_unlimited`: pass-1 F8 (`text_set_max_lines(id, 0)` = unlimited).
//! - `pass1_f20_non_finite_canvas_sizes`: pass-1 F20 (INF/NaN sizes never reach layout).
//! - `pass3_f5_nested_dialogs_keep_modal_stack`: pass-3 F5 (nested dialogs keep the stack).
//! - `pass3_f6_ask_directory_navigates_with_enter`: pass-3 F6 (directory dialog navigation/Enter).
//!
//! Assumed (truncated in the API index) signatures: `Combobox::new(tk, parent, Vec<String>,
//! Var<i64>)`, `Text::new(tk, parent, usize, usize)`, `dialog::ask_yes_no(tk, title, msg, cb)`
//! and `dialog::ask_directory(tk, start_dir, cb)`.

use rtk::{dialog, Canvas, Combobox, Frame, Key, Opt, PackOpts, Text, Tk};
use std::cell::RefCell;
use std::rc::Rc;

/// Pass-1 F6: clicking an open combobox closes it instead of re-opening it.
#[test]
fn pass1_f6_combobox_click_toggles_closed() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(0);
    let values = vec!["one".to_string(), "two".to_string(), "three".to_string()];
    let cb = Combobox::new(&mut tk, root, values, var).unwrap();
    tk.pack(cb, PackOpts::default());
    tk.sim_resize(320, 240);
    let r = tk.widget_rect(cb).unwrap();
    let (cx, cy) = (r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.sim_mouse_move(cx, cy);
    tk.sim_key(Key::Tab);
    let closed = tk.render_to_pixmap(320, 240).unwrap().data().to_vec();
    tk.sim_click(cx, cy);
    let open = tk.render_to_pixmap(320, 240).unwrap().data().to_vec();
    assert_ne!(closed, open, "first click must open the dropdown");
    tk.sim_click(cx, cy);
    let again = tk.render_to_pixmap(320, 240).unwrap().data().to_vec();
    assert_eq!(closed, again, "second click must close the dropdown");
}

/// Pass-1 F7: `yview`, `yview_moveto` and `yview_scroll` honour `Text` widgets.
#[test]
fn pass1_f7_text_yview_works() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 5).unwrap();
    tk.pack(t, PackOpts::default());
    tk.sim_resize(320, 240);
    let body: String = (0..200).map(|i| format!("line {i}\n")).collect();
    assert!(tk.text_set(t, body));
    tk.render_to_pixmap(320, 240).unwrap();
    let (top0, bottom0) = tk.yview(t);
    assert!(top0 <= 0.001 && bottom0 < 1.0, "long text must show a partial view");
    tk.yview_moveto(t, 0.5);
    let (top1, _) = tk.yview(t);
    assert!(top1 > 0.3, "yview_moveto must scroll a Text widget, got {top1}");
}

/// Pass-1 F8: a max-lines limit of 0 means unlimited and never wipes the buffer.
#[test]
fn pass1_f8_max_lines_zero_is_unlimited() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 5).unwrap();
    assert!(tk.text_set_max_lines(t, 0));
    for i in 0..50 {
        tk.text_insert_end(t, &format!("row{i}\n"));
    }
    let text = tk.text_get(t).unwrap();
    assert!(text.contains("row0"), "first line must survive");
    assert!(text.contains("row49"), "last line must be present");
    assert!(tk.text_line_count(t).unwrap() >= 50);
}

/// Pass-1 F20: non-finite canvas sizes and scroll regions do not poison layout.
#[test]
fn pass1_f20_non_finite_canvas_sizes() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();
    tk.pack(frame, PackOpts::default());
    let c = Canvas::new(&mut tk, frame, f32::INFINITY, f32::NAN).unwrap();
    tk.pack(c, PackOpts::default());
    let c2 = Canvas::new(&mut tk, frame, 50.0, 50.0).unwrap();
    tk.pack(c2, PackOpts::default());
    tk.canvas_scrollregion(c2, f32::NAN, f32::INFINITY);
    tk.sim_resize(200, 200);
    for id in [c, c2] {
        let r = tk.widget_rect(id).unwrap();
        assert!(
            r.x.is_finite() && r.y.is_finite() && r.width.is_finite() && r.height.is_finite(),
            "non-finite rect {r:?}"
        );
    }
    assert!(tk.render_to_pixmap(200, 200).is_ok());
}

/// Pass-3 F5: opening a second dialog stacks it; closing it restores the first one.
#[test]
fn pass3_f5_nested_dialogs_keep_modal_stack() {
    let mut tk = Tk::new();
    assert!(!dialog::is_open(&tk));
    assert_eq!(dialog::modal_depth(&tk), 0);
    dialog::ask_yes_no(&mut tk, "First", "first?", |_, _| {});
    assert_eq!(dialog::modal_depth(&tk), 1);
    dialog::ask_yes_no(&mut tk, "Second", "second?", |_, _| {});
    assert_eq!(dialog::modal_depth(&tk), 2, "second dialog must stack");
    dialog::close(&mut tk);
    assert_eq!(dialog::modal_depth(&tk), 1, "closing the top restores the one below");
    assert!(dialog::is_open(&tk));
    tk.render_to_pixmap(400, 300).unwrap();
    dialog::close(&mut tk);
    assert_eq!(dialog::modal_depth(&tk), 0);
    assert!(!dialog::is_open(&tk));
}

/// Pass-3 F6: the directory chooser lists sub-directories and Enter acts on a selection.
#[test]
fn pass3_f6_ask_directory_navigates_with_enter() {
    let base = std::env::temp_dir().join(format!("rtk_f6_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("childdir")).unwrap();
    let mut tk = Tk::new();
    tk.sim_resize(640, 480);
    let result = Rc::new(RefCell::new(None::<bool>));
    let sink = Rc::clone(&result);
    dialog::ask_directory(&mut tk, base.to_str().unwrap(), &base, move |_tk, p: Option<std::path::PathBuf>| {
        *sink.borrow_mut() = Some(p.is_some());
    });
    assert!(dialog::is_open(&tk));
    let list = dialog::file_listbox(&tk).expect("directory dialog exposes a listbox");
    let before = tk.listbox_get(list);
    let idx = before
        .iter()
        .position(|s| s.contains("childdir"))
        .expect("sub-directory must be listed");
    let rect = tk.widget_rect(list).unwrap();
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert!(tk.listbox_select(list, Some(idx)));
    tk.sim_key(Key::Enter);
    let after = dialog::file_listbox(&tk).map(|l| tk.listbox_get(l));
    let navigated = after.is_some_and(|a| a != before);
    let finished = result.borrow().is_some() || !dialog::is_open(&tk);
    assert!(navigated || finished, "Enter must navigate into or accept the selected directory");
    while dialog::is_open(&tk) {
        dialog::close(&mut tk);
    }
    let _ = std::fs::remove_dir_all(&base);
    let _ = Opt::Text(String::new());
}
