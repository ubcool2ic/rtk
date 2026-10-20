//! Integration tests for clipboard, selection and undo/redo in Entry and Text.
//!
//! These tests exercise the public API only: `Tk::widget_rect`, `Tk::sim_*`
//! event simulation, the in-memory clipboard, and the Entry/Text editing
//! helpers. Golden images are produced through `rtk::testing`.

use rtk::shortcut::Modifiers;
use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{Entry, Key, Opt, Text, Tk, WidgetId};

/// Builds a toolkit with a single Entry and returns `(tk, entry_id)`.
///
/// The Entry is packed into the root frame so it is mapped by layout.
fn entry_tk() -> (Tk, WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let entry = Entry::new(&mut tk, root).unwrap();
    tk.pack(entry, rtk::PackOpts::default());
    tk.sim_resize(320, 80);
    (tk, entry)
}

/// Builds a toolkit with a single Text widget and returns `(tk, text_id)`.
///
/// The Text widget is packed into the root frame so it is mapped by layout.
fn text_tk() -> (Tk, WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let text = Text::new(&mut tk, root, 20, 5).unwrap();
    tk.pack(text, rtk::PackOpts::default());
    tk.sim_resize(320, 160);
    (tk, text)
}

/// Focuses `id` by clicking near its left edge (inside the text area).
fn focus(tk: &mut Tk, id: WidgetId) {
    let rect = tk.widget_rect(id).expect("widget should be laid out");
    tk.sim_click(rect.x + 4.0, rect.y + rect.height / 2.0);
}

/// Shift+Left extends the selection leftwards.
#[test]
fn shift_left_extends_selection() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "hello");
    focus(&mut tk, entry);
    tk.sim_key(Key::End);
    tk.sim_key_with(Key::Left, Modifiers::SHIFT);
    tk.sim_key_with(Key::Left, Modifiers::SHIFT);
    assert_eq!(tk.entry_selection(entry), Some((3, 5)));
    assert_eq!(tk.entry_selected_text(entry).as_deref(), Some("lo"));
}

/// Shift+Right extends the selection rightwards.
#[test]
fn shift_right_extends_selection() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "hello");
    focus(&mut tk, entry);
    tk.sim_key(Key::Home);
    tk.sim_key_with(Key::Right, Modifiers::SHIFT);
    tk.sim_key_with(Key::Right, Modifiers::SHIFT);
    assert_eq!(tk.entry_selection(entry), Some((0, 2)));
    assert_eq!(tk.entry_selected_text(entry).as_deref(), Some("he"));
}

/// Plain Left with a selection collapses to its start.
#[test]
fn plain_left_collapses_selection() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "hello");
    focus(&mut tk, entry);
    tk.entry_select_range(entry, 1, 4);
    assert_eq!(tk.entry_selection(entry), Some((1, 4)));
    tk.sim_key(Key::Left);
    assert_eq!(tk.entry_selection(entry), None);
}

/// Select-all then typing replaces the selection.
#[test]
fn select_all_then_type_replaces() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "hello");
    focus(&mut tk, entry);
    tk.sim_key_with(Key::Char('a'), Modifiers::CTRL);
    assert_eq!(tk.entry_selection(entry), Some((0, 5)));
    tk.sim_text("x");
    assert_eq!(tk.entry_get(entry).as_deref(), Some("x"));
    assert_eq!(tk.entry_selection(entry), None);
}

/// Copy then paste duplicates the selection.
#[test]
fn copy_paste_roundtrip() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "abc");
    focus(&mut tk, entry);
    tk.entry_select_range(entry, 0, 2);
    tk.sim_key_with(Key::Char('c'), Modifiers::CTRL);
    assert_eq!(tk.clipboard_get().as_deref(), Some("ab"));
    tk.sim_key(Key::End);
    tk.sim_key_with(Key::Char('v'), Modifiers::CTRL);
    assert_eq!(tk.entry_get(entry).as_deref(), Some("abcab"));
}

/// Cut removes the selection and stores it in the clipboard.
#[test]
fn cut_removes_selection() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "abcdef");
    focus(&mut tk, entry);
    tk.entry_select_range(entry, 1, 4);
    tk.sim_key_with(Key::Char('x'), Modifiers::CTRL);
    assert_eq!(tk.entry_get(entry).as_deref(), Some("aef"));
    assert_eq!(tk.clipboard_get().as_deref(), Some("bcd"));
}

/// Entry paste strips newlines.
#[test]
fn entry_paste_strips_newlines() {
    let (mut tk, entry) = entry_tk();
    focus(&mut tk, entry);
    tk.clipboard_clear();
    tk.clipboard_append("a\nb\r\nc");
    tk.sim_key_with(Key::Char('v'), Modifiers::CTRL);
    assert_eq!(tk.entry_get(entry).as_deref(), Some("abc"));
}

/// Undo and redo restore coalesced typing and the bound Var value.
#[test]
fn undo_restores_text_and_var() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.string_var("");
    let entry = Entry::new(&mut tk, root).unwrap();
    assert!(tk.config(entry, Opt::TextVariable(var)));
    tk.pack(entry, rtk::PackOpts::default());
    tk.sim_resize(320, 80);
    focus(&mut tk, entry);
    tk.sim_text("a");
    tk.sim_text("b");
    assert_eq!(tk.entry_get(entry).as_deref(), Some("ab"));
    assert_eq!(var.get(&tk).as_deref(), Some("ab"));
    assert!(tk.edit_undo(entry));
    assert_eq!(tk.entry_get(entry).as_deref(), Some(""));
    assert_eq!(var.get(&tk).as_deref(), Some(""));
    assert!(tk.edit_redo(entry));
    assert_eq!(tk.entry_get(entry).as_deref(), Some("ab"));
    assert_eq!(var.get(&tk).as_deref(), Some("ab"));
}

/// Redo re-applies an undone edit.
#[test]
fn redo_reapplies() {
    let (mut tk, entry) = entry_tk();
    focus(&mut tk, entry);
    tk.sim_text("a");
    tk.sim_text("b");
    assert!(tk.edit_undo(entry));
    assert_eq!(tk.entry_get(entry).as_deref(), Some(""));
    assert!(tk.edit_redo(entry));
    assert_eq!(tk.entry_get(entry).as_deref(), Some("ab"));
}

/// Consecutive typing coalesces into a single undo step.
#[test]
fn typing_coalesces() {
    let (mut tk, entry) = entry_tk();
    focus(&mut tk, entry);
    tk.sim_text("a");
    tk.sim_text("b");
    tk.sim_text("c");
    assert_eq!(tk.entry_get(entry).as_deref(), Some("abc"));
    tk.edit_undo(entry);
    assert_eq!(tk.entry_get(entry).as_deref(), Some(""));
}

/// Read-only Text cannot cut or paste but can copy.
#[test]
fn readonly_text_copy_only() {
    let (mut tk, text) = text_tk();
    tk.text_set(text, "hello");
    tk.text_set_readonly(text, true);
    focus(&mut tk, text);
    tk.text_select_range(text, 0, 5);
    tk.sim_key_with(Key::Char('c'), Modifiers::CTRL);
    assert_eq!(tk.clipboard_get().as_deref(), Some("hello"));
    tk.clipboard_clear();
    tk.clipboard_append("x");
    tk.sim_key_with(Key::Char('v'), Modifiers::CTRL);
    assert_eq!(tk.text_get(text).as_deref(), Some("hello"));
    tk.sim_key_with(Key::Char('x'), Modifiers::CTRL);
    assert_eq!(tk.text_get(text).as_deref(), Some("hello"));
}

/// Mouse drag extends the selection.
#[test]
fn mouse_drag_selects() {
    let (mut tk, entry) = entry_tk();
    tk.entry_set(entry, "hello world");
    let rect = tk.widget_rect(entry).unwrap();
    let y = rect.y + rect.height / 2.0;
    tk.sim_mouse_move(rect.x + 6.0, y);
    tk.sim_mouse_down();
    tk.sim_mouse_move(rect.x + 60.0, y);
    tk.sim_mouse_up();
    let sel = tk.entry_selection(entry);
    assert!(sel.is_some(), "expected a selection after drag");
    let (s, e) = sel.unwrap();
    assert!(s < e);
}

/// Unicode boundaries: selection respects char boundaries (é, emoji, combining mark).
#[test]
fn unicode_boundaries() {
    let (mut tk, entry) = entry_tk();
    // "café🎉" plus a combining acute accent on the trailing "e".
    let sample = "café🎉e\u{0301}";
    tk.entry_set(entry, sample);
    focus(&mut tk, entry);
    tk.sim_key(Key::End);
    tk.sim_key_with(Key::Left, Modifiers::SHIFT);
    let (s, e) = tk.entry_selection(entry).expect("selection after shift+left");
    assert_eq!(e, sample.len());
    assert!(s < e);
    assert!(sample.is_char_boundary(s));
    assert!(sample.is_char_boundary(e));
    // The selected suffix must be a valid slice of the original text.
    assert_eq!(tk.entry_selected_text(entry).as_deref(), sample.get(s..e));
}

/// Text selection API works across lines.
#[test]
fn text_selection_api() {
    let (mut tk, text) = text_tk();
    tk.text_set(text, "line1\nline2");
    tk.text_select_range(text, 0, 5);
    assert_eq!(tk.text_selection(text), Some((0, 5)));
    assert_eq!(tk.text_selected_text(text).as_deref(), Some("line1"));
    tk.selection_clear(text);
    assert_eq!(tk.text_selection(text), None);
}

/// edit_reset clears history so undo does nothing.
#[test]
fn edit_reset_clears_history() {
    let (mut tk, entry) = entry_tk();
    focus(&mut tk, entry);
    tk.sim_text("a");
    tk.edit_reset(entry);
    assert!(!tk.edit_undo(entry));
}

/// Clipboard append accumulates text.
#[test]
fn clipboard_append_accumulates() {
    let mut tk = Tk::new();
    tk.clipboard_clear();
    tk.clipboard_append("a");
    tk.clipboard_append("b");
    assert_eq!(tk.clipboard_get().as_deref(), Some("ab"));
    tk.clipboard_clear();
    assert_eq!(tk.clipboard_get(), None);
}

/// Golden render: Entry with a selection in three themes (320x80).
#[test]
fn edit_selection_golden() {
    for (theme, name) in [
        (rtk::Theme::Light, "light"),
        (rtk::Theme::Dark, "dark"),
        (rtk::Theme::HighContrast, "highcontrast"),
    ] {
        let mut tk = Tk::new();
        tk.set_theme(theme);
        let root = tk.root();
        let entry = Entry::new(&mut tk, root).unwrap();
        tk.pack(entry, rtk::PackOpts::default());
        tk.entry_set(entry, "selected text");
        tk.entry_select_range(entry, 0, 8);
        assert_golden(
            &mut tk,
            golden_dir(),
            &format!("edit_selection_{name}_1x"),
            320,
            80,
            GoldenOpts::default(),
        );
    }
}
