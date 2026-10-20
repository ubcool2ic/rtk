//! Headless integration tests for IME (input-method composition) support on
//! Entry and Text widgets (milestone 0.13, step 3).

use rtk::testing::{assert_golden, compare_pixmaps, golden_dir, GoldenOpts};
use rtk::{Button, Entry, Label, Opt, PackOpts, Side, Text, Tk};

/// Builds a toolkit with a single Entry packed at the top; returns `(tk, entry)`.
fn entry_tk() -> (Tk, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.pack(entry, PackOpts { side: Side::Top, fill: rtk::Fill::X, ..PackOpts::default() });
    tk.sim_resize(240, 60);
    (tk, entry)
}

/// A preedit must not change the widget value nor any bound variable.
#[test]
fn preedit_does_not_change_value_or_var() {
    let (mut tk, entry) = entry_tk();
    let var = tk.string_var("abc");
    tk.config(entry, Opt::TextVariable(var));
    tk.sim_click(20.0, 10.0);
    assert_eq!(tk.entry_get(entry).as_deref(), Some("abc"));
    tk.sim_ime_preedit("にほん", Some((0, 9)));
    assert_eq!(tk.entry_get(entry).as_deref(), Some("abc"));
    assert_eq!(var.get(&tk).as_deref(), Some("abc"));
    assert!(tk.ime_preedit().is_some());
}

/// Committing text inserts it at the caret and clears the preedit.
#[test]
fn commit_inserts_and_clears_preedit() {
    let (mut tk, entry) = entry_tk();
    tk.sim_click(20.0, 10.0);
    tk.sim_ime_preedit("にほん", Some((0, 9)));
    tk.sim_ime_commit("日本");
    assert_eq!(tk.entry_get(entry).as_deref(), Some("日本"));
    assert!(tk.ime_preedit().is_none());
}

/// A focus change implicitly cancels an active composition.
#[test]
fn focus_change_clears_preedit() {
    let (mut tk, entry) = entry_tk();
    let root = tk.root();
    let other = Entry::new(&mut tk, root).expect("entry2");
    tk.pack(other, PackOpts { side: Side::Top, fill: rtk::Fill::X, ..PackOpts::default() });
    tk.sim_resize(240, 120);
    let first_rect = tk.widget_rect(entry).expect("first rect");
    let other_rect = tk.widget_rect(other).expect("other rect");
    tk.sim_click(first_rect.x + 20.0, first_rect.y + first_rect.height / 2.0);
    assert_eq!(tk.focused_widget(), Some(entry));
    tk.sim_ime_preedit("abc", None);
    assert!(tk.ime_preedit().is_some());
    tk.sim_click(other_rect.x + 20.0, other_rect.y + other_rect.height / 2.0);
    assert!(tk.ime_preedit().is_none());
    assert_eq!(tk.focused_widget(), Some(other));
}

/// `ime_wanted` is false for a focused Button and true for Entry and Text.
#[test]
fn ime_wanted_tracks_focus_kind() {
    let mut tk = Tk::new();
    let root = tk.root();
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.pack(entry, PackOpts { side: Side::Top, fill: rtk::Fill::X, ..PackOpts::default() });
    let button = Button::new(&mut tk, root, "ok").expect("button");
    tk.pack(button, PackOpts { side: Side::Top, ..PackOpts::default() });
    let text = Text::new(&mut tk, root, 20, 3).expect("text");
    tk.pack(text, PackOpts { side: Side::Top, ..PackOpts::default() });
    tk.sim_resize(240, 200);

    tk.sim_click(20.0, 10.0);
    assert_eq!(tk.focused_widget(), Some(entry));
    assert!(tk.ime_wanted());

    let brect = tk.widget_rect(button).expect("button rect");
    tk.sim_click(brect.x + 5.0, brect.y + 5.0);
    assert_eq!(tk.focused_widget(), Some(button));
    assert!(!tk.ime_wanted());

    let trect = tk.widget_rect(text).expect("text rect");
    tk.sim_click(trect.x + 5.0, trect.y + 5.0);
    assert_eq!(tk.focused_widget(), Some(text));
    assert!(tk.ime_wanted());
}

/// A preedit on a Text widget followed by a commit inserts into the Text.
#[test]
fn text_preedit_then_commit() {
    let mut tk = Tk::new();
    let root = tk.root();
    let text = Text::new(&mut tk, root, 20, 3).expect("text");
    tk.pack(text, PackOpts { side: Side::Top, ..PackOpts::default() });
    tk.sim_resize(240, 120);
    let trect = tk.widget_rect(text).expect("text rect");
    tk.sim_click(trect.x + 5.0, trect.y + 5.0);
    assert!(tk.ime_wanted());
    tk.sim_ime_preedit("にほん", Some((0, 9)));
    assert_eq!(tk.text_get(text).as_deref(), Some(""));
    tk.sim_ime_commit("日本");
    assert_eq!(tk.text_get(text).as_deref(), Some("日本"));
    assert!(tk.ime_preedit().is_none());
}

/// The preedit cursor range is clamped to char boundaries.
#[test]
fn preedit_cursor_clamped_to_char_boundaries() {
    let (mut tk, _entry) = entry_tk();
    tk.sim_click(20.0, 10.0);
    // "日本語" is 9 bytes; ask for a range that lands mid-char.
    tk.sim_ime_preedit("日本語", Some((1, 4)));
    let (_, text, cursor) = tk.ime_preedit().expect("preedit");
    assert_eq!(text, "日本語");
    let (start, end) = cursor.expect("cursor");
    assert!(text.is_char_boundary(start));
    assert!(text.is_char_boundary(end));
    assert!(start <= end);
    // Out-of-range values are clamped to the string length.
    tk.sim_ime_preedit("日本語", Some((100, 200)));
    let (_, _, cursor) = tk.ime_preedit().expect("preedit");
    assert_eq!(cursor, Some((9, 9)));
}

/// `ime_cursor_area` is Some for a focused Entry and moves right after typing.
#[test]
fn ime_cursor_area_moves_with_caret() {
    let (mut tk, _entry) = entry_tk();
    tk.sim_click(20.0, 10.0);
    let first = tk.ime_cursor_area().expect("cursor area");
    tk.sim_text("abc");
    let second = tk.ime_cursor_area().expect("cursor area");
    assert!(second.x > first.x);
    assert!(second.height > 0.0);
}

/// Rendering with an active preedit differs from rendering without it.
#[test]
fn preedit_changes_render() {
    let (mut tk, _entry) = entry_tk();
    tk.sim_click(20.0, 10.0);
    let without = tk.render_to_pixmap(240, 60).expect("render");
    tk.sim_ime_preedit("xyz", None);
    let with = tk.render_to_pixmap(240, 60).expect("render");
    let (diff, _) = compare_pixmaps(&without, &with, 0);
    assert!(diff > 0, "preedit should change the rendered image");
}

/// Golden image of an Entry with an active preedit (ASCII to avoid CJK fallback).
#[test]
fn golden_ime_preedit_light_1x() {
    let mut tk = Tk::new();
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Name").expect("label");
    tk.pack(label, PackOpts { side: Side::Top, ..PackOpts::default() });
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.pack(entry, PackOpts { side: Side::Top, fill: rtk::Fill::X, ..PackOpts::default() });
    tk.sim_resize(240, 60);
    let erect = tk.widget_rect(entry).expect("entry rect");
    tk.sim_click(erect.x + 10.0, erect.y + erect.height / 2.0);
    assert_eq!(tk.focused_widget(), Some(entry));
    tk.sim_text("abc");
    tk.sim_ime_preedit("xyz", None);
    assert!(tk.ime_preedit().is_some());
    assert_golden(&mut tk, golden_dir(), "ime_preedit_light_1x", 240, 60, GoldenOpts::default());
}
