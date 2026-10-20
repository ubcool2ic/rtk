//! Tests for widget ID validity, stale ID rejection, and slot reuse after destruction.

use rtk::{Button, Entry, Frame, GridOpts, Label, Opt, PackOpts, Tk, WidgetId};

/// Verifies that operations on stale or destroyed widget IDs fail gracefully without panicking.
#[test]
fn stale_ids_and_slot_reuse() {
    let mut tk = Tk::new();
    let root = tk.root();
    let button = Button::new(&mut tk, root, "old").expect("button");
    assert!(tk.destroy(button));
    assert!(!tk.destroy(button));
    assert!(!tk.config(button, Opt::Text("stale".into())));
    assert!(!tk.pack(button, PackOpts::default()));
    assert!(!tk.grid(button, GridOpts::default()));
    assert!(!tk.set_command(button, |_| {}));
    assert!(!tk.grid_columnconfigure(button, 0, 1));
    assert!(!tk.grid_rowconfigure(button, 0, 1));
    assert!(!tk.entry_set(button, "x"));
    assert_eq!(tk.entry_get(button), None);
    assert_eq!(tk.widget_rect(button), None);
    let replacement = Button::new(&mut tk, root, "new").expect("replacement");
    assert_eq!(replacement.index, button.index);
    assert_ne!(replacement.gen, button.gen);
    assert!(!tk.config(button, Opt::Bg(rtk::Color::WHITE)));
    assert_eq!(tk.widget_text(replacement).as_deref(), Some("new"));
    let invalid = WidgetId { index: u32::MAX, gen: u32::MAX };
    assert!(!tk.contains(invalid));
    assert!(!tk.destroy(invalid));
    assert!(Label::new(&mut tk, invalid, "bad parent").is_err());
    assert!(Entry::new(&mut tk, replacement).is_err());
}

/// Confirms that destroying a parent widget removes its descendants and clears focus from them.
#[test]
fn destroying_parent_removes_descendants_and_focus() {
    let mut tk = Tk::new();
    let root = tk.root();
    let parent = Frame::new(&mut tk, root).expect("parent");
    let nested = Frame::new(&mut tk, parent).expect("nested");
    let entry = Entry::new(&mut tk, nested).expect("entry");
    for id in [parent, nested, entry] {
        tk.pack(id, PackOpts::default());
    }
    tk.sim_key(rtk::Key::Tab);
    assert_eq!(tk.focused_widget(), Some(entry));
    assert!(tk.destroy(parent));
    for id in [parent, nested, entry] {
        assert!(!tk.contains(id));
        assert_eq!(tk.widget_rect(id), None);
    }
    assert_eq!(tk.focused_widget(), None);
}

/// Ensures variables reject handles from other Tk instances and destroyed handles.
#[test]
fn variables_reject_foreign_and_destroyed_handles() {
    let mut first = Tk::new();
    let mut second = Tk::new();
    let value = first.int_var(3);
    let other = second.int_var(4);
    assert_eq!(value.get(&second), None);
    assert!(!value.set(&mut second, 9));
    assert_eq!(other.get(&second), Some(4));
    assert!(value.destroy(&mut first));
    assert_eq!(value.get(&first), None);
    assert!(!value.set(&mut first, 1));
    assert!(!value.trace(&mut first, |_| {}));
    assert!(!value.destroy(&mut first));
}

/// Checks that incompatible geometry managers and invalid configuration values are rejected.
#[test]
fn incompatible_configuration_is_rejected() {
    let mut tk = Tk::new();
    let root = tk.root();
    let a = Entry::new(&mut tk, root).expect("entry");
    let b = Label::new(&mut tk, root, "label").expect("label");
    assert!(tk.pack(a, PackOpts::default()));
    assert!(!tk.grid(b, GridOpts::default()));
    assert!(!tk.config(a, Opt::FontSize(f32::NAN)));
    assert!(!tk.config(a, Opt::FontSize(0.0)));
    assert!(!tk.set_command(b, |_| {}));
    let int = tk.int_var(0);
    assert!(!tk.config(a, Opt::TextVariableInt(int)));
    assert!(!tk.pack(root, PackOpts::default()));
}
