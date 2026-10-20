//! Regression tests for review findings on the editable Combobox: stale
//! `open_combobox` state after read-only selection, opening a second combobox
//! while one is open, `combobox_set_values`/`combobox_set_text` closing the
//! dropdown when nothing matches, and window focus loss (`clear_input`) resetting the highlight.

use rtk::{Combobox, Frame, Key, PackOpts, Tk, Var};

/// Builds an editable combobox over the fruit list bound to a fresh var (-1).
///
/// Returns `(tk, combo, var)`.
fn editable_fruit() -> (Tk, rtk::WidgetId, Var<i64>) {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(-1);
    let values = vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
        "Blackberry".to_string(),
        "Cherry".to_string(),
        "Grape".to_string(),
        "Mango".to_string(),
        "Papaya".to_string(),
    ];
    let combo = Combobox::editable(&mut tk, root, values, var).expect("editable combobox");
    tk.pack(combo, PackOpts::default());
    (tk, combo, var)
}

/// Builds a read-only combobox over the fruit list bound to a fresh var (0).
///
/// Returns `(tk, combo, var)`.
fn readonly_fruit() -> (Tk, rtk::WidgetId, Var<i64>) {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(0);
    let values = vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
        "Blackberry".to_string(),
        "Cherry".to_string(),
        "Grape".to_string(),
        "Mango".to_string(),
        "Papaya".to_string(),
    ];
    let combo = Combobox::new(&mut tk, root, values, var).expect("combobox");
    tk.pack(combo, PackOpts::default());
    (tk, combo, var)
}

/// Focuses an editable combobox by clicking its text area (left of the arrow).
fn focus_text_area(tk: &mut Tk, combo: rtk::WidgetId) {
    let rect = tk.widget_rect(combo).expect("combo rect");
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
}


/// Returns the logical height of one dropdown row for the default font size.
fn row_height() -> f32 {
    (14.0_f32 * 1.3 + 8.0).max(20.0)
}

/// Clicks the dropdown row at `pos` (0-based within the shown rows).
fn click_dropdown_row(tk: &mut Tk, combo: rtk::WidgetId, pos: usize) {
    let rect = tk.widget_rect(combo).expect("combo rect");
    let y = rect.y + rect.height + row_height() * (pos as f32 + 0.5);
    let x = rect.x + 10.0;
    tk.sim_click(x, y);
}

/// Finding 1: selecting a read-only row clears the global open-combobox state,
/// so a later click elsewhere does not hit the stale overlay.
#[test]
fn readonly_select_clears_open_state() {
    let (mut tk, combo, var) = readonly_fruit();
    let rect = tk.widget_rect(combo).expect("rect");
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
    assert!(tk.combobox_is_open(combo));
    click_dropdown_row(&mut tk, combo, 2);
    assert_eq!(var.get(&tk), Some(2));
    assert!(!tk.combobox_is_open(combo));
    // A click far below the (now closed) dropdown must not be swallowed by a
    // stale overlay: it should simply miss the combobox.
    let below_y = rect.y + rect.height + row_height() * 2.5;
    tk.sim_click(rect.x + 10.0, below_y);
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(var.get(&tk), Some(2));
}

/// Finding 2: opening a second combobox closes the first one.
#[test]
fn opening_second_combobox_closes_first() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var_a = tk.int_var(0);
    let var_b = tk.int_var(0);
    let values = vec!["A".to_string(), "B".to_string()];
    let a = Combobox::new(&mut tk, root, values.clone(), var_a).expect("a");
    let b = Combobox::new(&mut tk, root, values, var_b).expect("b");
    tk.pack(a, PackOpts::default());
    tk.pack(b, PackOpts::default());
    tk.combobox_open(a);
    assert!(tk.combobox_is_open(a));
    tk.combobox_open(b);
    assert!(tk.combobox_is_open(b));
    assert!(!tk.combobox_is_open(a));
}

/// Finding 2 (editable path): typing into a second editable combobox closes
/// the first one's dropdown.
#[test]
fn typing_in_second_editable_closes_first() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var_a = tk.int_var(-1);
    let var_b = tk.int_var(-1);
    let values = vec!["Apple".to_string(), "Apricot".to_string()];
    let a = Combobox::editable(&mut tk, root, values.clone(), var_a).expect("a");
    let b = Combobox::editable(&mut tk, root, values, var_b).expect("b");
    tk.pack(a, PackOpts::default());
    tk.pack(b, PackOpts::default());
    // Focus the second combobox before opening the first one's overlay:
    // otherwise the overlay intercepts the click intended to focus `b`.
    focus_text_area(&mut tk, b);
    tk.combobox_open(a);
    assert!(tk.combobox_is_open(a));
    tk.sim_text("ap");
    assert!(tk.combobox_is_open(b));
    assert!(!tk.combobox_is_open(a));
}

/// Finding 3: `combobox_set_values` closes the dropdown when nothing matches
/// and re-syncs the bound var to the current text.
#[test]
fn set_values_closes_when_empty_and_resyncs() {
    let (mut tk, combo, var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    assert!(tk.combobox_is_open(combo));
    assert!(tk.combobox_set_values(combo, vec!["Kiwi".to_string(), "Lime".to_string()]));
    assert!(tk.combobox_get_filtered(combo).is_empty());
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(var.get(&tk), Some(-1));
    // A value equal to the current text re-syncs the index.
    assert!(tk.combobox_set_values(combo, vec!["Kiwi".to_string(), "ap".to_string()]));
    assert_eq!(var.get(&tk), Some(1));
}

/// Finding 4: `combobox_set_text` clears the highlight and closes the dropdown
/// when the new text matches nothing.
#[test]
fn set_text_closes_when_empty() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    assert!(tk.combobox_is_open(combo));
    assert!(tk.combobox_get_highlighted(combo).is_some());
    assert!(tk.combobox_set_text(combo, "zzz"));
    assert_eq!(tk.combobox_get_highlighted(combo), None);
    assert!(!tk.combobox_is_open(combo));
    // A matching text keeps the highlight cleared but does not open.
    assert!(tk.combobox_set_text(combo, "ap"));
    assert_eq!(tk.combobox_get_highlighted(combo), None);
    assert!(!tk.combobox_is_open(combo));
}

/// Finding 5: `clear_input` (via a modal-free focus reset) clears the
/// highlight of the open combobox.
#[test]
fn clear_input_resets_highlight() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    assert!(tk.combobox_is_open(combo));
    assert_eq!(tk.combobox_get_highlighted(combo), Some(0));
    tk.sim_window_focus_lost();
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(tk.combobox_get_highlighted(combo), None);
}

/// Finding 6: a press on the arrow zone followed by a pointer move does not
/// extend the text selection.
#[test]
fn arrow_press_does_not_drag_selection() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.entry_select_range(combo, 0, 2);
    let rect = tk.widget_rect(combo).expect("rect");
    // Press on the arrow, then drag left across the text area.
    tk.sim_mouse_move(rect.x + rect.width - 10.0, rect.y + rect.height / 2.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(rect.x + 2.0, rect.y + rect.height / 2.0);
    tk.sim_mouse_up();
    // The selection must be unchanged by the drag.
    assert_eq!(tk.entry_selection(combo), Some((0, 2)));
}

/// A non-combobox id is rejected by `combobox_set_values`.
#[test]
fn set_values_rejects_non_combobox() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    assert!(!tk.combobox_set_values(frame, vec!["x".to_string()]));
}
