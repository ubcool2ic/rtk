//! Tests that command callbacks can safely destroy their own widget, create replacements, mutate state, and quit.

use std::cell::Cell;
use std::rc::Rc;

use rtk::{Button, Key, Label, Opt, PackOpts, Tk};

/// Tests that a command can destroy its own widget, reuse the slot, mutate state, and quit safely.
#[test]
fn command_can_destroy_itself_reuse_slot_mutate_and_quit() {
    let mut tk = Tk::new();
    let root = tk.root();
    let value = tk.int_var(0);
    let button = Button::new(&mut tk, root, "run").expect("button");
    tk.pack(button, PackOpts::default());
    let created = Rc::new(Cell::new(None));
    let output = Rc::clone(&created);
    tk.set_command(button, move |tk| {
        assert!(tk.destroy(button));
        let label = Label::new(tk, root, "created").expect("label");
        assert_eq!(label.index, button.index);
        tk.pack(label, PackOpts::default());
        tk.config(label, Opt::Text("updated".into()));
        output.set(Some(label));
        value.set(tk, 42);
        tk.quit();
    });
    tk.sim_key(Key::Tab);
    tk.sim_key(Key::Enter);
    assert!(tk.is_quit_requested());
    assert!(!tk.contains(button));
    assert_eq!(value.get(&tk), Some(42));
    assert_eq!(tk.widget_text(created.get().expect("created")).as_deref(), Some("updated"));
    assert!(tk.render_to_pixmap(240, 100).is_ok());
}

/// Confirms that a replacement command takes effect and the running command is not re-entered.
#[test]
fn replacement_command_wins_and_running_command_is_not_reentered() {
    let mut tk = Tk::new();
    let root = tk.root();
    let count = tk.int_var(0);
    let button = Button::new(&mut tk, root, "replace").expect("button");
    tk.pack(button, PackOpts::default());
    tk.set_command(button, move |tk| {
        count.set(tk, 1);
        tk.set_command(button, move |tk| {
            count.set(tk, 10);
        });
        tk.sim_key(Key::Enter);
        assert_eq!(count.get(tk), Some(1));
    });
    tk.sim_key(Key::Tab);
    tk.sim_key(Key::Enter);
    assert_eq!(count.get(&tk), Some(1));
    tk.sim_key(Key::Enter);
    assert_eq!(count.get(&tk), Some(10));
}

/// Ensures cyclic variable traces are bounded and that equal trace sets are cleaned up correctly.
#[test]
fn cyclic_var_traces_are_bounded_and_equal_sets_are_clean() {
    let mut tk = Tk::new();
    let a = tk.int_var(0);
    let b = tk.int_var(0);
    let calls = Rc::new(Cell::new(0));
    let a_calls = Rc::clone(&calls);
    a.trace(&mut tk, move |tk| {
        a_calls.set(a_calls.get() + 1);
        b.set(tk, a.get(tk).unwrap_or(0) + 1);
    });
    let b_calls = Rc::clone(&calls);
    b.trace(&mut tk, move |tk| {
        b_calls.set(b_calls.get() + 1);
        a.set(tk, b.get(tk).unwrap_or(0) + 1);
    });
    assert!(a.set(&mut tk, 1));
    assert_eq!(calls.get(), 2);
    assert_eq!(a.get(&tk), Some(3));
    assert_eq!(b.get(&tk), Some(2));
    tk.render_to_pixmap(20, 20).expect("image");
    assert!(!tk.is_dirty());
    assert!(a.set(&mut tk, 3));
    assert!(!tk.is_dirty());
    assert_eq!(calls.get(), 2);
}

/// Verifies that a trace callback can destroy its bound entry and itself without causing errors.
#[test]
fn trace_may_destroy_its_bound_entry_and_itself() {
    let mut tk = Tk::new();
    let root = tk.root();
    let text = tk.string_var("");
    let entry = rtk::Entry::new(&mut tk, root).expect("entry");
    tk.config(entry, Opt::TextVariable(text));
    tk.pack(entry, PackOpts::default());
    text.trace(&mut tk, move |tk| {
        tk.destroy(entry);
        text.destroy(tk);
    });
    tk.sim_key(Key::Tab);
    tk.sim_text("gone");
    assert!(!tk.contains(entry));
    assert_eq!(text.get(&tk), None);
}
