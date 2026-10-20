//! Tests for shared event handling across widgets, covering clicks, focus, and resize behavior.

use rtk::{Button, Checkbutton, Entry, Key, Label, Opt, PackOpts, Scale, Tk, WidgetId};

/// Simulates a click at the center of the given widget.
fn click(tk: &mut Tk, id: WidgetId) {
    let rect = tk.widget_rect(id).expect("visible widget");
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
}

/// Verifies that shared event handlers correctly update widget-bound variables for widgets, focus changes, and resize events.
#[test]
fn shared_event_handlers_cover_widgets_focus_and_resize() {
    let mut tk = Tk::new();
    let root = tk.root();
    let count = tk.int_var(0);
    let text = tk.string_var("");
    let checked = tk.bool_var(false);
    let level = tk.int_var(5);
    let label = Label::new(&mut tk, root, "").expect("label");
    tk.config(label, Opt::TextVariableInt(count));
    let button = Button::new(&mut tk, root, "increment").expect("button");
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.config(entry, Opt::TextVariable(text));
    let check = Checkbutton::new(&mut tk, root, "enabled", checked).expect("check");
    let scale = Scale::new(&mut tk, root, 0, 10, level).expect("scale");
    for id in [label, button, entry, check, scale] {
        tk.pack(id, PackOpts { pady: 4.0, ..PackOpts::default() });
    }
    tk.set_command(button, move |tk| {
        count.set(tk, count.get(tk).unwrap_or(0) + 1);
    });
    tk.sim_resize(400, 320);
    click(&mut tk, button);
    assert_eq!(tk.widget_text(label).as_deref(), Some("1"));
    assert_eq!(tk.focused_widget(), Some(button));
    tk.sim_key(Key::Space);
    tk.sim_key(Key::Enter);
    assert_eq!(count.get(&tk), Some(3));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(entry));
    tk.sim_text("hello 世界");
    assert_eq!(text.get(&tk).as_deref(), Some("hello 世界"));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(check));
    tk.sim_key(Key::Space);
    assert_eq!(checked.get(&tk), Some(true));
    click(&mut tk, check);
    assert_eq!(checked.get(&tk), Some(false));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(scale));
    tk.sim_key(Key::Right);
    assert_eq!(level.get(&tk), Some(6));
    tk.sim_key(Key::Left);
    assert_eq!(level.get(&tk), Some(5));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(button));
    tk.sim_key(Key::ShiftTab);
    assert_eq!(tk.focused_widget(), Some(scale));

    let rect = tk.widget_rect(scale).expect("scale rect");
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(rect.x + rect.width + 100.0, rect.y);
    tk.sim_mouse_up();
    assert_eq!(level.get(&tk), Some(10));

    tk.sim_resize(0, 0);
    assert!(tk.render_to_pixmap(0, 0).is_err());
    tk.sim_click(0.0, 0.0);
    assert_eq!(count.get(&tk), Some(3));
    tk.sim_resize(400, 320);
    assert!(tk.render_to_pixmap(400, 320).is_ok());
    click(&mut tk, button);
    assert_eq!(count.get(&tk), Some(4));
}

/// Ensures that releasing the mouse outside a button after pressing inside does not activate the button's command.
#[test]
fn release_outside_button_does_not_activate() {
    let mut tk = Tk::new();
    let root = tk.root();
    let count = tk.int_var(0);
    let button = Button::new(&mut tk, root, "click").expect("button");
    tk.pack(button, PackOpts::default());
    tk.set_command(button, move |tk| {
        count.set(tk, 1);
    });
    let rect = tk.widget_rect(button).expect("rect");
    tk.sim_mouse_move(rect.x + 2.0, rect.y + 2.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(-100.0, -100.0);
    tk.sim_mouse_up();
    assert_eq!(count.get(&tk), Some(0));
    // Control: the same button must activate when released inside, so the
    // negative assertion above is not vacuous.
    click(&mut tk, button);
    assert_eq!(count.get(&tk), Some(1));
}

/// Checks that checkbutton and scale commands observe the updated variable values when triggered.
#[test]
fn check_and_scale_commands_observe_updated_values() {
    let mut tk = Tk::new();
    let root = tk.root();
    let checked = tk.bool_var(false);
    let number = tk.int_var(0);
    let observed = tk.int_var(0);
    let check = Checkbutton::new(&mut tk, root, "check", checked).expect("check");
    let scale = Scale::new(&mut tk, root, -2, 2, number).expect("scale");
    tk.pack(check, PackOpts::default());
    tk.pack(scale, PackOpts::default());
    tk.set_command(check, move |tk| {
        assert_eq!(checked.get(tk), Some(true));
        observed.set(tk, 1);
    });
    tk.set_command(scale, move |tk| {
        observed.set(tk, number.get(tk).unwrap_or(0) + 10);
    });
    click(&mut tk, check);
    assert_eq!(observed.get(&tk), Some(1));
    tk.sim_key(Key::Tab);
    tk.sim_key(Key::Right);
    assert_eq!(observed.get(&tk), Some(11));
}
