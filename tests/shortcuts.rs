//! Integration tests for the keyboard shortcut API: parsing, labels, binding,
//! dispatch through `sim_key_with`, unbinding, and reentrancy safety.

use rtk::{Key, Modifiers, Shortcut, ShortcutError, Tk};

/// Parses a shortcut and checks its label round-trips through `parse`.
#[test]
fn parse_and_label_round_trip() {
    let cases = [
        ("Ctrl+S", Modifiers::CTRL, Key::Char('s')),
        ("Primary+Shift+Z", Modifiers::primary().with_shift(), Key::Char('z')),
        ("Alt+F4", Modifiers::ALT, Key::F(4)),
        ("F5", Modifiers::NONE, Key::F(5)),
        ("Ctrl+Plus", Modifiers::CTRL, Key::Char('+')),
        ("Escape", Modifiers::NONE, Key::Escape),
    ];
    for (text, mods, key) in cases {
        let sc = Shortcut::parse(text).unwrap_or_else(|e| panic!("parse {text}: {e}"));
        assert_eq!(sc.mods, mods, "mods for {text}");
        assert_eq!(sc.key, key, "key for {text}");
        // The label must itself parse back to the same shortcut.
        let label = sc.label();
        let again = Shortcut::parse(&label).unwrap_or_else(|e| panic!("reparse {label}: {e}"));
        assert_eq!(again, sc, "round trip for {text}");
    }
}

/// Case-insensitive modifier names and aliases are accepted.
#[test]
fn modifier_aliases_are_case_insensitive() {
    let a = Shortcut::parse("control+shift+a").unwrap();
    let b = Shortcut::parse("CTRL+SHIFT+A").unwrap();
    assert_eq!(a, b);
    assert_eq!(a.mods, Modifiers::CTRL.with_shift());
    assert_eq!(a.key, Key::Char('a'));
    assert_eq!(Shortcut::parse("option+x").unwrap().mods, Modifiers::ALT);
    assert_eq!(Shortcut::parse("cmd+x").unwrap().mods, Modifiers::META);
    assert_eq!(Shortcut::parse("super+x").unwrap().mods, Modifiers::META);
    assert_eq!(Shortcut::parse("win+x").unwrap().mods, Modifiers::META);
}

/// Empty and unknown inputs produce the documented errors.
#[test]
fn parse_errors() {
    assert_eq!(Shortcut::parse(""), Err(ShortcutError::Empty));
    assert!(matches!(Shortcut::parse("Ctrl+"), Err(ShortcutError::UnknownKey(_))));
    assert!(matches!(Shortcut::parse("Bogus+S"), Err(ShortcutError::UnknownModifier(_))));
}

/// A bound shortcut fires when the matching chord is delivered.
#[test]
fn shortcut_fires_on_sim_key_with() {
    let mut tk = Tk::new();
    let fired = std::rc::Rc::new(std::cell::Cell::new(0));
    let f = fired.clone();
    let _id = tk
        .bind_shortcut("Ctrl+S", move |_tk| f.set(f.get() + 1))
        .unwrap();
    tk.sim_key_with(Key::Char('s'), Modifiers::CTRL);
    assert_eq!(fired.get(), 1);
    // A different chord does not fire it.
    tk.sim_key_with(Key::Char('s'), Modifiers::NONE);
    assert_eq!(fired.get(), 1);
}

/// Unbinding a shortcut stops it from firing.
#[test]
fn unbind_stops_shortcut() {
    let mut tk = Tk::new();
    let fired = std::rc::Rc::new(std::cell::Cell::new(0));
    let f = fired.clone();
    let id = tk
        .bind_shortcut("Ctrl+S", move |_tk| f.set(f.get() + 1))
        .unwrap();
    assert!(tk.unbind_shortcut(id));
    assert!(!tk.unbind_shortcut(id));
    tk.sim_key_with(Key::Char('s'), Modifiers::CTRL);
    assert_eq!(fired.get(), 0);
    assert!(tk.shortcuts().is_empty());
}

/// The last bound shortcut wins when two share a chord.
#[test]
fn last_bound_wins() {
    let mut tk = Tk::new();
    let order = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let a = order.clone();
    tk.bind_shortcut("Ctrl+S", move |_tk| a.borrow_mut().push(1)).unwrap();
    let b = order.clone();
    tk.bind_shortcut("Ctrl+S", move |_tk| b.borrow_mut().push(2)).unwrap();
    tk.sim_key_with(Key::Char('s'), Modifiers::CTRL);
    assert_eq!(*order.borrow(), vec![2]);
}

/// Unbinding a shortcut from inside its own callback is safe.
#[test]
fn reentrant_unbind_is_safe() {
    let mut tk = Tk::new();
    let fired = std::rc::Rc::new(std::cell::Cell::new(0));
    let f = fired.clone();
    // The id is not known until after binding, so stash it in a cell the
    // callback reads when it runs.
    let slot = std::rc::Rc::new(std::cell::Cell::new(None));
    let s2 = slot.clone();
    let id = tk
        .bind_shortcut("Ctrl+S", move |tk| {
            f.set(f.get() + 1);
            if let Some(id) = s2.get() {
                let _ = tk.unbind_shortcut(id);
            }
        })
        .unwrap();
    slot.set(Some(id));
    tk.sim_key_with(Key::Char('s'), Modifiers::CTRL);
    assert_eq!(fired.get(), 1);
    // A second press does nothing because the binding is gone.
    tk.sim_key_with(Key::Char('s'), Modifiers::CTRL);
    assert_eq!(fired.get(), 1);
}

/// `modifiers()` reflects the most recent key press.
#[test]
fn modifiers_track_last_press() {
    let mut tk = Tk::new();
    assert_eq!(tk.modifiers(), Modifiers::NONE);
    tk.sim_key_with(Key::F(1), Modifiers::CTRL.with_shift());
    assert_eq!(tk.modifiers(), Modifiers::CTRL.with_shift());
}

/// Shift+Tab still traverses focus backwards (no widgets -> focus stays None).
#[test]
fn shift_tab_traverses_backwards() {
    let mut tk = Tk::new();
    tk.sim_key_with(Key::ShiftTab, Modifiers::SHIFT);
    assert_eq!(tk.focused_widget(), None);
}
