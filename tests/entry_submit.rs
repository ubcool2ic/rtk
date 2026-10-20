//! Enter-to-submit for single-line Entry widgets (`Tk::entry_bind_submit`).

use rtk::{Button, Entry, Key, PackOpts, Tk};
use std::cell::Cell;
use std::rc::Rc;

/// Builds a window with one Entry, focused via Tab.
fn focused_entry() -> (Tk, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.pack(entry, PackOpts::default());
    tk.sim_resize(400, 100);
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(entry));
    (tk, entry)
}

/// Enter in a focused Entry runs the submit callback, which can read the text;
/// the text itself is unchanged (no newline inserted).
#[test]
fn enter_runs_submit_callback() {
    let (mut tk, entry) = focused_entry();
    let seen = Rc::new(Cell::new(0));
    let s = seen.clone();
    assert!(tk.entry_bind_submit(entry, move |tk| {
        assert_eq!(tk.entry_get(entry).as_deref(), Some("Indy"));
        s.set(s.get() + 1);
    }));
    tk.sim_text("Indy");
    tk.sim_key(Key::Enter);
    tk.sim_key(Key::Enter);
    assert_eq!(seen.get(), 2);
    assert_eq!(tk.entry_get(entry).as_deref(), Some("Indy"));
}

/// Without a callback Enter is a harmless no-op, and other widget kinds are rejected.
#[test]
fn no_callback_and_wrong_kind() {
    let (mut tk, entry) = focused_entry();
    tk.sim_text("abc");
    tk.sim_key(Key::Enter);
    assert_eq!(tk.entry_get(entry).as_deref(), Some("abc"));
    let root = tk.root();
    let b = Button::new(&mut tk, root, "x").expect("button");
    assert!(!tk.entry_bind_submit(b, |_| {}));
}

/// A disabled Entry does not submit.
#[test]
fn disabled_entry_does_not_submit() {
    let (mut tk, entry) = focused_entry();
    let seen = Rc::new(Cell::new(0));
    let s = seen.clone();
    assert!(tk.entry_bind_submit(entry, move |_| s.set(s.get() + 1)));
    // Positive control: while enabled, Enter submits.
    tk.sim_key(Key::Enter);
    assert_eq!(seen.get(), 1);
    tk.config(entry, rtk::Opt::Disabled(true));
    tk.sim_key(Key::Enter);
    assert_eq!(seen.get(), 1);
}
