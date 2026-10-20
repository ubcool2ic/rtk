//! Headless integration tests for the editable (filter-as-you-type) Combobox
//! added in milestone 0.11, plus regression checks that read-only comboboxes
//! keep their classic behaviour.

use std::cell::RefCell;
use std::rc::Rc;

use rtk::shortcut::Modifiers;
use rtk::{Combobox, Frame, Key, Label, PackOpts, Tk, Var};

/// Builds an empty editable combobox over the fruit list bound to a fresh var (-1).
///
/// Returns `(tk, combo, var)`.
fn editable_fruit() -> (Tk, rtk::WidgetId, Var<i64>) {
    let mut tk = Tk::new();
    let root = tk.root();
    // -1 = no value selected, so the editable text starts empty.
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

/// Builds a read-only combobox over the fruit list bound to a fresh var.
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

/// Clicks the 22 px arrow area on the right of a combobox.
fn click_arrow(tk: &mut Tk, combo: rtk::WidgetId) {
    let rect = tk.widget_rect(combo).expect("combo rect");
    tk.sim_click(rect.x + rect.width - 10.0, rect.y + rect.height / 2.0);
}

/// Returns the logical height of one dropdown row for the default font size.
fn row_height() -> f32 {
    (14.0_f32 * 1.3 + 8.0).max(20.0)
}

/// Clicks the dropdown row at `pos` (0-based within the filtered rows).
fn click_dropdown_row(tk: &mut Tk, combo: rtk::WidgetId, pos: usize) {
    let rect = tk.widget_rect(combo).expect("combo rect");
    let y = rect.y + rect.height + row_height() * (pos as f32 + 0.5);
    let x = rect.x + 10.0;
    tk.sim_click(x, y);
}

/// Editable construction sets the initial text from the bound index.
#[test]
fn editable_initial_text() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(2);
    let values = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let combo = Combobox::editable(&mut tk, root, values, var).expect("combo");
    assert!(tk.combobox_is_editable(combo));
    assert_eq!(tk.combobox_text(combo), Some("C".to_string()));
    assert_eq!(tk.combobox_get(combo), Some((2, "C".to_string())));
}

/// Editable construction with an out-of-range index starts empty.
#[test]
fn editable_initial_empty_for_bad_index() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(99);
    let values = vec!["A".to_string(), "B".to_string()];
    let combo = Combobox::editable(&mut tk, root, values, var).expect("combo");
    assert_eq!(tk.combobox_text(combo), Some(String::new()));
}

/// Typing filters case-insensitively and opens the dropdown.
#[test]
fn typing_filters_and_opens() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    assert_eq!(tk.combobox_text(combo), Some("ap".to_string()));
    assert!(tk.combobox_is_open(combo));
    assert_eq!(
        tk.combobox_filtered(combo),
        vec![
            "Apple".to_string(),
            "Apricot".to_string(),
            "Grape".to_string(),
            "Papaya".to_string()
        ]
    );
}

/// Down/Up move the highlight and clamp at the ends.
#[test]
fn highlight_movement_and_clamping() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    assert_eq!(tk.combobox_highlighted(combo), Some(0));
    tk.sim_key(Key::Down);
    assert_eq!(tk.combobox_highlighted(combo), Some(1));
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Down);
    // `combobox_highlighted` reports the index into the full value list:
    // the filtered rows are Apple(0), Apricot(1), Grape(5), Papaya(7).
    assert_eq!(tk.combobox_highlighted(combo), Some(7));
    tk.sim_key(Key::Up);
    assert_eq!(tk.combobox_highlighted(combo), Some(5));
    tk.sim_key(Key::Up);
    tk.sim_key(Key::Up);
    tk.sim_key(Key::Up);
    assert_eq!(tk.combobox_highlighted(combo), Some(0));
}

/// Enter on a highlighted row selects it, updates the var and fires on_select.
#[test]
fn enter_selects_highlighted() {
    let (mut tk, combo, var) = editable_fruit();
    let got: Rc<RefCell<Option<(usize, String)>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&got);
    assert!(tk.combobox_on_select(combo, move |_tk, idx, text| {
        *sink.borrow_mut() = Some((idx, text.to_string()));
    }));
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Enter);
    assert_eq!(tk.combobox_text(combo), Some("Apricot".to_string()));
    assert_eq!(var.get(&tk), Some(1));
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(*got.borrow(), Some((1, "Apricot".to_string())));
}

/// Enter with no highlight keeps the free text and sets the var to -1.
#[test]
fn enter_without_highlight_keeps_text() {
    let (mut tk, combo, var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("zzz");
    assert!(!tk.combobox_is_open(combo));
    tk.sim_key(Key::Enter);
    assert_eq!(tk.combobox_text(combo), Some("zzz".to_string()));
    assert_eq!(var.get(&tk), Some(-1));
}

/// Escape closes the dropdown and clears the highlight.
#[test]
fn escape_closes() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    assert!(tk.combobox_is_open(combo));
    tk.sim_key(Key::Escape);
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(tk.combobox_highlighted(combo), None);
}

/// Backspace re-filters the dropdown.
#[test]
fn backspace_refilters() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("app");
    assert_eq!(tk.combobox_filtered(combo), vec!["Apple".to_string()]);
    tk.sim_key(Key::Backspace);
    assert_eq!(tk.combobox_text(combo), Some("ap".to_string()));
    assert_eq!(tk.combobox_filtered(combo).len(), 4);
}

/// on_text_change fires once per edit with the new text.
#[test]
fn text_change_fires_once_per_edit() {
    let (mut tk, combo, _var) = editable_fruit();
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    assert!(tk.combobox_on_text_change(combo, move |_tk, text| {
        sink.borrow_mut().push(text.to_string());
    }));
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    assert_eq!(*seen.borrow(), vec!["ap".to_string()]);
}

/// combobox_set_text does not fire callbacks and does not open the dropdown.
#[test]
fn set_text_is_silent() {
    let (mut tk, combo, var) = editable_fruit();
    let fired = Rc::new(RefCell::new(0usize));
    let sink = Rc::clone(&fired);
    assert!(tk.combobox_on_text_change(combo, move |_tk, _text| {
        *sink.borrow_mut() += 1;
    }));
    assert!(tk.combobox_set_text(combo, "Cherry"));
    assert_eq!(tk.combobox_text(combo), Some("Cherry".to_string()));
    assert_eq!(var.get(&tk), Some(4));
    assert!(!tk.combobox_is_open(combo));
    assert_eq!(*fired.borrow(), 0);
}

/// Clicking a filtered dropdown row picks the right value and fires on_select.
#[test]
fn click_filtered_row_selects() {
    let (mut tk, combo, var) = editable_fruit();
    let got: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&got);
    assert!(tk.combobox_on_select(combo, move |_tk, idx, _text| {
        *sink.borrow_mut() = Some(idx);
    }));
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    // Filtered rows: Apple(0), Apricot(1), Grape(5), Papaya(7).
    click_dropdown_row(&mut tk, combo, 1);
    assert_eq!(tk.combobox_text(combo), Some("Apricot".to_string()));
    assert_eq!(var.get(&tk), Some(1));
    assert_eq!(*got.borrow(), Some(1));
}

/// Clicking the text area of an editable combobox does not open the dropdown.
#[test]
fn click_text_area_does_not_open() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    assert!(!tk.combobox_is_open(combo));
}

/// Clicking the arrow area opens the dropdown.
#[test]
fn click_arrow_opens() {
    let (mut tk, combo, _var) = editable_fruit();
    click_arrow(&mut tk, combo);
    assert!(tk.combobox_is_open(combo));
}

/// Read-only comboboxes still toggle on click and Up/Down change the var.
#[test]
fn readonly_toggles_and_keys() {
    let (mut tk, combo, var) = readonly_fruit();
    assert!(!tk.combobox_is_editable(combo));
    let rect = tk.widget_rect(combo).expect("rect");
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
    assert!(tk.combobox_is_open(combo));
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
    assert!(!tk.combobox_is_open(combo));
    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(1));
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(0));
}

/// on_select fires for read-only dropdown clicks.
#[test]
fn readonly_click_fires_select() {
    let (mut tk, combo, var) = readonly_fruit();
    let got: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&got);
    assert!(tk.combobox_on_select(combo, move |_tk, idx, _text| {
        *sink.borrow_mut() = Some(idx);
    }));
    let rect = tk.widget_rect(combo).expect("rect");
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
    click_dropdown_row(&mut tk, combo, 2);
    assert_eq!(var.get(&tk), Some(2));
    assert_eq!(*got.borrow(), Some(2));
}

/// Pasting via the clipboard re-filters the dropdown.
#[test]
fn paste_refilters() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.clipboard_clear();
    tk.clipboard_append("ap");
    tk.sim_key_with(Key::Char('v'), Modifiers::CTRL);
    assert_eq!(tk.combobox_text(combo), Some("ap".to_string()));
    assert_eq!(tk.combobox_filtered(combo).len(), 4);
}

/// Undo after typing re-filters the dropdown.
#[test]
fn undo_refilters() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    assert_eq!(tk.combobox_filtered(combo).len(), 4);
    tk.sim_key_with(Key::Char('z'), Modifiers::CTRL);
    assert_eq!(tk.combobox_text(combo), Some(String::new()));
    assert_eq!(tk.combobox_filtered(combo).len(), 8);
}

/// IME commit inserts text into an editable combobox.
#[test]
fn ime_commit_inserts() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    assert!(tk.ime_wanted());
    tk.sim_ime_commit("ap");
    assert_eq!(tk.combobox_text(combo), Some("ap".to_string()));
    assert_eq!(tk.combobox_filtered(combo).len(), 4);
}

/// Non-combobox ids return false/None from the combobox API.
#[test]
fn non_combobox_ids_rejected() {
    let mut tk = Tk::new();
    let root = tk.root();
    let label = Label::new(&mut tk, root, "hi").expect("label");
    assert!(!tk.combobox_is_editable(label));
    assert_eq!(tk.combobox_text(label), None);
    assert!(!tk.combobox_set_text(label, "x"));
    assert!(!tk.combobox_on_select(label, |_tk, _i, _s| {}));
    assert!(!tk.combobox_on_text_change(label, |_tk, _s| {}));
    assert!(tk.combobox_filtered(label).is_empty());
    assert_eq!(tk.combobox_highlighted(label), None);
    assert!(!tk.combobox_is_open(label));
    assert_eq!(tk.combobox_get(label), None);
}

/// A frame is not a combobox either.
#[test]
fn frame_ids_rejected() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    assert!(!tk.combobox_is_editable(frame));
    assert_eq!(tk.combobox_text(frame), None);
    assert!(!tk.combobox_set_text(frame, "x"));
}

/// Select-all via the public entry helper then typing replaces the text.
#[test]
fn select_all_then_type_replaces() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.entry_select_range(combo, 0, 2);
    tk.sim_text("ch");
    assert_eq!(tk.combobox_text(combo), Some("ch".to_string()));
}

/// Cut removes the selection and re-filters.
#[test]
fn cut_refilters() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.entry_select_range(combo, 0, 2);
    tk.sim_key_with(Key::Char('x'), Modifiers::CTRL);
    assert_eq!(tk.combobox_text(combo), Some(String::new()));
    assert_eq!(tk.combobox_filtered(combo).len(), 8);
}

/// combobox_set_values recomputes the filter and clears the highlight.
#[test]
fn set_values_refilters() {
    let (mut tk, combo, _var) = editable_fruit();
    focus_text_area(&mut tk, combo);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    assert_eq!(tk.combobox_highlighted(combo), Some(0));
    assert!(tk.combobox_set_values(combo, vec!["Apple".to_string(), "Kiwi".to_string()]));
    assert_eq!(tk.combobox_filtered(combo), vec!["Apple".to_string()]);
    assert_eq!(tk.combobox_highlighted(combo), None);
}

/// combobox_open shows the filter for the current text.
#[test]
fn open_uses_filter() {
    let (mut tk, combo, _var) = editable_fruit();
    assert!(tk.combobox_set_text(combo, "ap"));
    tk.combobox_open(combo);
    assert!(tk.combobox_is_open(combo));
    assert_eq!(tk.combobox_filtered(combo).len(), 4);
    tk.combobox_close(combo);
    assert!(!tk.combobox_is_open(combo));
}
