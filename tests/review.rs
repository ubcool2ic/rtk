//! Regression tests for findings from the gemini-3.8-flash review (see REVIEW.md).
use std::cell::Cell;
use std::rc::Rc;

use rtk::{Button, Entry, Key, Opt, PackOpts, Scale, Tk};

/// Ensures a scale with an extreme integer range stays within bounds during clicks and key presses.
#[test]
fn scale_with_extreme_range_stays_in_bounds() {
    let mut tk = Tk::new();
    let root = tk.root();
    let v = tk.int_var(0);
    let s = Scale::new(&mut tk, root, i64::MIN, i64::MAX, v).expect("scale");
    tk.pack(s, PackOpts { fill: rtk::Fill::X, ..PackOpts::default() });
    assert!(tk.render_to_pixmap(300, 80).is_ok());
    let r = tk.widget_rect(s).expect("rect");
    tk.sim_click(r.x + r.width - 1.0, r.y + r.height / 2.0);
    for _ in 0..3 { tk.sim_key(Key::Right); }
    tk.sim_click(r.x + 1.0, r.y + r.height / 2.0);
    for _ in 0..3 { tk.sim_key(Key::Left); }
    assert_eq!(v.get(&tk), Some(i64::MIN));
    assert!(tk.render_to_pixmap(300, 80).is_ok());
}

/// Verifies that destroying an entry from within its trace while typing does not cause errors.
#[test]
fn entry_destroyed_by_trace_while_typing() {
    let mut tk = Tk::new();
    let root = tk.root();
    let text = tk.string_var("");
    let e = Entry::new(&mut tk, root).expect("entry");
    tk.config(e, Opt::TextVariable(text));
    tk.pack(e, PackOpts::default());
    text.trace(&mut tk, move |tk| { tk.destroy(e); });
    tk.sim_key(Key::Tab);
    tk.sim_text("abc");
    tk.sim_key(Key::Backspace);
    assert!(!tk.contains(e));
    assert_eq!(text.get(&tk).as_deref(), Some("abc"));
    assert!(tk.render_to_pixmap(200, 80).is_ok());
}

/// Confirms tab navigation after destruction skips stale widgets and finds reused slots.
#[test]
fn tab_after_destroy_skips_stale_and_finds_reused_slot() {
    let mut tk = Tk::new();
    let root = tk.root();
    let a = Button::new(&mut tk, root, "a").expect("a");
    let b = Button::new(&mut tk, root, "b").expect("b");
    tk.pack(a, PackOpts::default());
    tk.pack(b, PackOpts::default());
    let made = Rc::new(Cell::new(None));
    let out = Rc::clone(&made);
    tk.set_command(a, move |tk| {
        tk.destroy(b);
        let c = Button::new(tk, root, "c").expect("c");
        tk.pack(c, PackOpts::default());
        out.set(Some(c));
    });
    tk.sim_key(Key::Tab);
    tk.sim_key(Key::Enter);
    tk.sim_key(Key::Tab);
    let c = made.get().expect("c");
    assert_eq!(c.index, b.index);
    assert_eq!(tk.focused_widget(), Some(c));
}

/// Ensures press-leave-release does not fire the command and does not leave the button stuck.
#[test]
fn press_leave_release_does_not_fire_and_does_not_stick() {
    let mut tk = Tk::new();
    let root = tk.root();
    let n = Rc::new(Cell::new(0));
    let b = Button::new(&mut tk, root, "x").expect("b");
    tk.pack(b, PackOpts::default());
    let k = Rc::clone(&n);
    tk.set_command(b, move |_| k.set(k.get() + 1));
    assert!(tk.render_to_pixmap(200, 80).is_ok());
    let r = tk.widget_rect(b).expect("rect");
    let (x, y) = (r.x + r.width / 2.0, r.y + r.height / 2.0);
    tk.sim_mouse_move(x, y);
    tk.sim_mouse_down();
    tk.sim_mouse_move(f32::NEG_INFINITY, f32::NEG_INFINITY);
    tk.sim_mouse_up();
    assert_eq!(n.get(), 0);
    tk.sim_click(x, y);
    assert_eq!(n.get(), 1);
}

/// Checks that reentrant sets inside a trace keep the last value, matching Tcl semantics.
#[test]
fn reentrant_set_in_trace_keeps_last_value_tcl_semantics() {
    let mut tk = Tk::new();
    let v = tk.int_var(0);
    let calls = Rc::new(Cell::new(0));
    let c = Rc::clone(&calls);
    v.trace(&mut tk, move |tk| { c.set(c.get() + 1); v.set(tk, 100); });
    v.set(&mut tk, 1);
    assert_eq!(v.get(&tk), Some(100));
    assert_eq!(calls.get(), 1);
}
