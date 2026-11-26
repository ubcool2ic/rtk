//! WP1 Input/keyboard regression tests (B2, B3, B6, B7, B8, B15 events, B17, P12).
//!
//! Probe cases for B6, B7, B3 and B8 are ported from
//! `/workspace/rtk-audit-notes/zz_audit_probe.rs`. B3 is adapted to call
//! `Tk::sim_modifiers` after Shift+Arrow, which is what a real window sends as
//! `ModifiersChanged` on Shift release (the original probe left that implicit
//! and would stay red after the correct fix).

use std::cell::Cell;
use std::rc::Rc;

use rtk::{
    Button, Color, Combobox, Entry, Key, Modifiers, Opt, Shortcut, Spinbox, State, StyleSpec, Text,
    Tk,
};

fn click_xy(tk: &mut Tk, id: rtk::WidgetId, xfrac: f32, yfrac: f32) {
    let r = tk.widget_rect(id).expect("visible widget");
    tk.sim_click(r.x + r.width * xfrac, r.y + r.height * yfrac);
}

/// B6: clicking near the right edge of a one-line Text must place the caret
/// at the end, not column 0.
#[test]
fn text_click_respects_x() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    tk.text_set(t, "hello world");
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(t).unwrap();
    tk.sim_click(r.x + r.width - 5.0, r.y + 10.0);
    tk.sim_text("X");
    let got = tk.text_get(t).unwrap();
    assert_eq!(got, "hello worldX");
}

/// B7: `bind_shortcut("Shift+Tab")` must fire for the chord the window actually
/// delivers (`Key::ShiftTab` + SHIFT).
#[test]
fn shift_tab_shortcut_fires() {
    let mut tk = Tk::new();
    let hit = Rc::new(Cell::new(false));
    let h = hit.clone();
    tk.bind_shortcut("Shift+Tab", move |_| h.set(true)).unwrap();
    tk.sim_key_with(Key::ShiftTab, Modifiers::SHIFT);
    assert!(hit.get(), "Shift+Tab shortcut did not fire");
}

/// B7 (parse): `Shift+Tab` and `ShiftTab` must both parse as `Key::ShiftTab` + SHIFT
/// so they match the real event.
#[test]
fn shift_tab_shortcut_parses_as_shift_tab_key() {
    let a = Shortcut::parse("Shift+Tab").unwrap();
    let b = Shortcut::parse("ShiftTab").unwrap();
    assert_eq!(a.key, Key::ShiftTab);
    assert_eq!(b.key, Key::ShiftTab);
    assert!(a.mods.shift);
    assert!(b.mods.shift);
}

/// B3: after Shift+Arrow, a Shift release (ModifiersChanged / sim_modifiers)
/// must clear Tk.mods so a plain click does not extend the selection.
#[test]
fn shift_is_not_sticky_after_shift_arrow() {
    let mut tk = Tk::new();
    let root = tk.root();
    let e = Entry::new(&mut tk, root).unwrap();
    tk.pack(e, Default::default());
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(e).unwrap();
    tk.sim_click(r.x + 5.0, r.y + r.height / 2.0);
    tk.sim_text("hello world");
    tk.sim_key_with(Key::Left, Modifiers::SHIFT);
    // Real windows: releasing Shift only sends ModifiersChanged.
    tk.sim_modifiers(Modifiers::NONE);
    tk.sim_click(r.x + 3.0, r.y + r.height / 2.0);
    tk.sim_text("X");
    let got = tk.widget_text(e).unwrap();
    assert_eq!(got, "Xhello world");
}

/// B3: Shift+click must extend the selection even if Shift was pressed with no
/// other key (ModifiersChanged only).
#[test]
fn shift_click_extends_after_modifiers_changed() {
    let mut tk = Tk::new();
    let root = tk.root();
    let e = Entry::new(&mut tk, root).unwrap();
    tk.pack(e, Default::default());
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(e).unwrap();
    tk.sim_click(r.x + 5.0, r.y + r.height / 2.0);
    tk.sim_text("hello world");
    // Caret is at the end. Press Shift (no key), click near the start.
    tk.sim_modifiers(Modifiers::SHIFT);
    tk.sim_click(r.x + 3.0, r.y + r.height / 2.0);
    tk.sim_text("X");
    let got = tk.widget_text(e).unwrap();
    assert_eq!(got, "X");
}

/// B3: losing window focus must clear Tk.mods so a later click is not sticky.
#[test]
fn focus_loss_clears_shift_mods() {
    let mut tk = Tk::new();
    let root = tk.root();
    let e = Entry::new(&mut tk, root).unwrap();
    tk.pack(e, Default::default());
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(e).unwrap();
    tk.sim_click(r.x + 5.0, r.y + r.height / 2.0);
    tk.sim_text("hello world");
    tk.sim_key_with(Key::Left, Modifiers::SHIFT);
    tk.sim_window_focus_lost();
    assert_eq!(tk.modifiers(), Modifiers::NONE);
    // Click again to focus and place the caret at the start, then type.
    tk.sim_click(r.x + 3.0, r.y + r.height / 2.0);
    tk.sim_text("X");
    let got = tk.widget_text(e).unwrap();
    assert_eq!(got, "Xhello world");
}

/// B8: `text_set_max_lines` must be enforced for typing and Enter, not only
/// `text_set` / `text_insert_end`.
#[test]
fn text_max_lines_enforced_on_typing() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    tk.text_set_max_lines(t, 3);
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(t).unwrap();
    tk.sim_click(r.x + 5.0, r.y + 10.0);
    for _ in 0..10 {
        tk.sim_text("x");
        tk.sim_key(Key::Enter);
    }
    let n = tk.text_get(t).unwrap().split('\n').count();
    assert!(n <= 3, "lines after typing 10 lines with max 3: {n}");
}

/// B8: paste and undo must also honour `text_set_max_lines`.
#[test]
fn text_max_lines_enforced_on_paste_and_undo() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    tk.text_set_max_lines(t, 3);
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(t).unwrap();
    tk.sim_click(r.x + 5.0, r.y + 10.0);
    tk.clipboard_clear();
    tk.clipboard_append("a\nb\nc\nd\ne");
    tk.sim_key_with(Key::Char('v'), Modifiers::primary());
    let pasted = tk.text_get(t).unwrap();
    assert!(pasted.split('\n').count() <= 3, "lines after paste with max 3: {}", pasted.split('\n').count());
    assert_eq!(pasted, "c\nd\ne");

    tk.sim_key_with(Key::Char('z'), Modifiers::primary());
    assert_eq!(tk.text_get(t).as_deref(), Some(""));

    tk.sim_key_with(Key::Char('y'), Modifiers::primary());
    let redone = tk.text_get(t).unwrap();
    assert_eq!(redone, pasted);
    assert!(redone.split('\n').count() <= 3, "lines after redo with max 3: {}", redone.split('\n').count());
}

/// B8: trimming leading lines must shift cursor (and a drained anchor) so a
/// mid-buffer Enter then type does not land in the wrong line.
#[test]
fn text_max_lines_shifts_cursor_after_mid_buffer_enter() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    tk.text_set(t, "aaaa\nbbbb\ncccc");
    tk.text_set_max_lines(t, 3);
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(t).unwrap();
    tk.sim_click(r.x + 5.0, r.y + 10.0);
    // Caret at the start of "bbbb".
    tk.text_select_range(t, 5, 5);
    tk.sim_key(Key::Enter);
    tk.sim_text("X");
    // After Enter, trim drops "aaaa\n" and leaves "\nbbbb\ncccc"; a shifted
    // caret is at byte 1, so X is inserted after the leftover leading newline.
    // Unfixed, the caret stays at 6 and this becomes "\nbbbb\nXcccc".
    assert_eq!(tk.text_get(t).as_deref(), Some("\nXbbbb\ncccc"));
}

/// B8: an undo that restores a snapshot whose cursor/anchor sit in a line
/// that `text_max_lines` then drains must clamp both to 0, not leave them
/// pointing into the kept text.
#[test]
fn text_max_lines_clamps_anchor_inside_drained_prefix() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 30, 4).unwrap();
    tk.pack(t, Default::default());
    tk.text_set(t, "aaaa\nbbbb\ncccc\ndddd");
    tk.render_frame(400, 200).unwrap();
    let r = tk.widget_rect(t).unwrap();
    tk.sim_click(r.x + 5.0, r.y + 10.0);
    // Snapshot will record cursor/anchor inside the first line.
    tk.text_select_range(t, 2, 4);
    tk.sim_text("Z");
    tk.text_set_max_lines(t, 3);
    tk.sim_key_with(Key::Char('z'), Modifiers::primary());
    // Undo restores the 4-line text; sync_after_edit drains "aaaa\n" and
    // must clamp the restored cursor (4) and anchor (2) to 0.
    assert_eq!(tk.text_get(t).as_deref(), Some("bbbb\ncccc\ndddd"));
    assert_eq!(tk.text_selection(t), None);
    tk.sim_text("X");
    assert_eq!(tk.text_get(t).as_deref(), Some("Xbbbb\ncccc\ndddd"));
}

/// B2: AltGr (Ctrl+Alt) / Option characters must be inserted, not dropped.
#[test]
fn altgr_option_text_is_inserted() {
    let mut tk = Tk::new();
    let root = tk.root();
    let e = Entry::new(&mut tk, root).unwrap();
    tk.pack(e, Default::default());
    tk.render_frame(400, 200).unwrap();
    click_xy(&mut tk, e, 0.1, 0.5);
    // What App delivers for AltGr '@' or Option '€': a Char key that no
    // shortcut consumes, plus the composed text.
    let altgr = Modifiers {
        ctrl: true,
        alt: true,
        ..Modifiers::NONE
    };
    tk.sim_key_or_text(Some(Key::Char('@')), "@", altgr);
    tk.sim_key_or_text(Some(Key::Char('€')), "€", Modifiers::ALT);
    assert_eq!(tk.widget_text(e).as_deref(), Some("@€"));
}

/// B2: a plain Control character (not AltGr) must not be inserted as text
/// when no shortcut matches.
#[test]
fn plain_control_char_is_not_inserted() {
    let mut tk = Tk::new();
    let root = tk.root();
    let e = Entry::new(&mut tk, root).unwrap();
    tk.pack(e, Default::default());
    tk.render_frame(400, 200).unwrap();
    click_xy(&mut tk, e, 0.1, 0.5);
    tk.sim_text("ab");
    tk.sim_key_or_text(Some(Key::Char('s')), "s", Modifiers::CTRL);
    assert_eq!(tk.widget_text(e).as_deref(), Some("ab"));
}

/// B15: Combobox Up/Down must set the var and fire the select callback/command.
#[test]
fn combobox_arrow_keys_fire_select() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(0);
    let hits = Rc::new(Cell::new(0));
    let last = Rc::new(Cell::new(usize::MAX));
    let combo =
        Combobox::new(&mut tk, root, vec!["a".into(), "b".into(), "c".into()], var).unwrap();
    let h = hits.clone();
    let l = last.clone();
    tk.combobox_on_select(combo, move |_, idx, _| {
        h.set(h.get() + 1);
        l.set(idx);
    });
    tk.pack(combo, Default::default());
    tk.render_frame(400, 200).unwrap();
    click_xy(&mut tk, combo, 0.3, 0.5);
    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(1));
    assert_eq!(hits.get(), 1, "Up/Down did not fire combobox_on_select");
    assert_eq!(last.get(), 1);
    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(2));
    assert_eq!(hits.get(), 2);
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(1));
    assert_eq!(hits.get(), 3);
}

/// B15: Spinbox arrow keys and the on-widget arrow clicks must run the command.
#[test]
fn spinbox_keys_and_arrows_run_command() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(5);
    let spin = Spinbox::new(&mut tk, root, 0, 10, var).unwrap();
    let hits = Rc::new(Cell::new(0));
    let h = hits.clone();
    assert!(tk.spinbox_on_change(spin, move |_| h.set(h.get() + 1)));
    tk.pack(spin, Default::default());
    tk.render_frame(400, 200).unwrap();
    click_xy(&mut tk, spin, 0.3, 0.5);
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(hits.get(), 1, "Spinbox Up did not run command");
    tk.sim_key(Key::Down);
    assert_eq!(var.get(&tk), Some(5));
    assert_eq!(hits.get(), 2, "Spinbox Down did not run command");

    // Click the up-arrow zone on the right edge (events.rs uses width-20, top half).
    let r = tk.widget_rect(spin).unwrap();
    tk.sim_click(r.x + r.width - 5.0, r.y + r.height * 0.25);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(hits.get(), 3, "Spinbox arrow click did not run command");
}

/// B17: Tab traversal must start focus transitions (same path as a click).
#[test]
fn tab_traversal_starts_focus_transition() {
    let mut tk = Tk::new();
    tk.geometry(240, 200);
    let root = tk.root();
    let a = Button::new(&mut tk, root, "A").unwrap();
    let b = Button::new(&mut tk, root, "B").unwrap();
    tk.pack(a, Default::default());
    tk.pack(b, Default::default());
    tk.config(
        a,
        Opt::StyleOverride(StyleSpec {
            bg: Some(Color::rgb(0, 0, 0)),
            transition_ms: Some(120),
            ..Default::default()
        }),
    );
    tk.config(
        b,
        Opt::StyleOverride(StyleSpec {
            bg: Some(Color::rgb(0, 0, 0)),
            transition_ms: Some(120),
            ..Default::default()
        }),
    );
    tk.style_map(
        "TButton",
        State::FOCUSED,
        StyleSpec {
            bg: Some(Color::rgb(100, 100, 100)),
            ..Default::default()
        },
    );
    tk.render_frame(240, 200).unwrap();
    click_xy(&mut tk, a, 0.5, 0.5);
    // Finish any click/hover/focus transition from the press.
    for _ in 0..10 {
        if !tk.is_animating() {
            break;
        }
        tk.sim_advance_time(60);
    }
    assert!(!tk.is_animating(), "setup transition should have finished");
    assert_eq!(tk.focused_widget(), Some(a));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(b));
    assert!(
        tk.is_animating(),
        "Tab traversal skipped check_state_transition"
    );
}

/// B17: `clear_input` / window-focus loss must run focus transitions too.
#[test]
fn clear_input_starts_focus_transition() {
    let mut tk = Tk::new();
    tk.geometry(240, 200);
    let root = tk.root();
    let a = Button::new(&mut tk, root, "A").unwrap();
    tk.pack(a, Default::default());
    tk.config(
        a,
        Opt::StyleOverride(StyleSpec {
            bg: Some(Color::rgb(0, 0, 0)),
            transition_ms: Some(120),
            ..Default::default()
        }),
    );
    tk.style_map(
        "TButton",
        State::FOCUSED,
        StyleSpec {
            bg: Some(Color::rgb(100, 100, 100)),
            ..Default::default()
        },
    );
    tk.render_frame(240, 200).unwrap();
    click_xy(&mut tk, a, 0.5, 0.5);
    for _ in 0..10 {
        if !tk.is_animating() {
            break;
        }
        tk.sim_advance_time(60);
    }
    assert!(!tk.is_animating());
    tk.sim_window_focus_lost();
    assert_eq!(tk.focused_widget(), None);
    assert!(
        tk.is_animating(),
        "clear_input skipped check_state_transition"
    );
}

/// P12: movement and drag-select on a multi-line Text still place the caret
/// and selection correctly after the no-clone / single-pass line lookup work.
#[test]
fn text_arrows_and_drag_select_stay_correct() {
    let mut tk = Tk::new();
    let root = tk.root();
    let t = Text::new(&mut tk, root, 40, 6).unwrap();
    tk.pack(t, Default::default());
    tk.text_set(t, "alpha\nbeta gamma\ndelta");
    tk.render_frame(400, 240).unwrap();
    let r = tk.widget_rect(t).unwrap();
    // Click at the start of the first line, then End + Down should land on
    // the second line (and typing should insert there).
    tk.sim_click(r.x + 6.0, r.y + 8.0);
    tk.sim_key(Key::End);
    tk.sim_key(Key::Down);
    tk.sim_text("X");
    // End of "alpha" is column 5; Down keeps the column, so the caret sits
    // after "beta " (5 chars) on the second line.
    let got = tk.text_get(t).unwrap();
    assert_eq!(got, "alpha\nbeta Xgamma\ndelta");

    // Drag-select across the first line should replace that range, not the
    // whole document.
    tk.text_set(t, "hello world\nsecond");
    tk.render_frame(400, 240).unwrap();
    let r = tk.widget_rect(t).unwrap();
    tk.sim_mouse_move(r.x + 6.0, r.y + 8.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(r.x + r.width - 8.0, r.y + 8.0);
    tk.sim_mouse_up();
    tk.sim_text("Z");
    let got = tk.text_get(t).unwrap();
    assert_eq!(
        got, "Z\nsecond",
        "drag-select replaced the wrong range: {got:?}"
    );
}

/// A single Spinbox Up key must run the command exactly once.
#[test]
fn spinbox_key_fires_command_exactly_once() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(5);
    let spin = Spinbox::new(&mut tk, root, 0, 10, var).unwrap();
    let hits = Rc::new(Cell::new(0));
    let h = hits.clone();
    assert!(tk.spinbox_on_change(spin, move |_| h.set(h.get() + 1)));
    tk.pack(spin, Default::default());
    tk.render_frame(400, 200).unwrap();
    click_xy(&mut tk, spin, 0.3, 0.5);
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(hits.get(), 1, "Up must fire the command exactly once");
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(7));
    assert_eq!(hits.get(), 2, "a second Up must fire exactly once more");
}

/// `spinbox_on_change` and `set_command` share one `node.command` slot; last setter wins.
#[test]
fn spinbox_setters_share_one_command_slot() {
    fn make() -> (Tk, rtk::WidgetId, rtk::Var<i64>) {
        let mut tk = Tk::new();
        let root = tk.root();
        let var = tk.int_var(5);
        let spin = Spinbox::new(&mut tk, root, 0, 10, var).unwrap();
        tk.pack(spin, Default::default());
        tk.render_frame(400, 200).unwrap();
        click_xy(&mut tk, spin, 0.3, 0.5);
        (tk, spin, var)
    }

    // Two spinbox_on_change calls: last wins.
    let (mut tk, spin, var) = make();
    let hits = Rc::new(Cell::new(0));
    let h1 = hits.clone();
    let h2 = hits.clone();
    assert!(tk.spinbox_on_change(spin, move |_| h1.set(h1.get() + 1)));
    assert!(tk.spinbox_on_change(spin, move |_| h2.set(h2.get() + 10)));
    tk.sim_key(Key::Up);
    assert_eq!(var.get(&tk), Some(6));
    assert_eq!(
        hits.get(),
        10,
        "second spinbox_on_change must replace the first"
    );

    // spinbox_on_change then set_command.
    let (mut tk, spin, var) = make();
    let hits = Rc::new(Cell::new(0));
    let h1 = hits.clone();
    let h2 = hits.clone();
    assert!(tk.spinbox_on_change(spin, move |_| h1.set(h1.get() + 1)));
    if tk.set_command(spin, move |_| h2.set(h2.get() + 10)) {
        tk.sim_key(Key::Up);
        assert_eq!(var.get(&tk), Some(6));
        assert_eq!(
            hits.get(),
            10,
            "set_command after spinbox_on_change must replace the callback"
        );
    } else {
        // Pre-WP2: set_command still rejects Spinbox, so the first callback remains.
        tk.sim_key(Key::Up);
        assert_eq!(var.get(&tk), Some(6));
        assert_eq!(
            hits.get(),
            1,
            "rejected set_command must not add a second callback"
        );
    }

    // set_command then spinbox_on_change (only when set_command accepts Spinbox).
    let (mut tk, spin, var) = make();
    let hits = Rc::new(Cell::new(0));
    let h1 = hits.clone();
    let h2 = hits.clone();
    if tk.set_command(spin, move |_| h1.set(h1.get() + 10)) {
        assert!(tk.spinbox_on_change(spin, move |_| h2.set(h2.get() + 1)));
        tk.sim_key(Key::Up);
        assert_eq!(var.get(&tk), Some(6));
        assert_eq!(
            hits.get(),
            1,
            "spinbox_on_change after set_command must replace the callback"
        );
    }
}
