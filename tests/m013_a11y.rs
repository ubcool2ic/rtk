//! Headless integration tests for the dependency-free accessibility tree.

use rtk::{
    A11yAction, A11yRole, Button, Checkbutton, Entry, Frame, Label, ProgressBar, Scale, Tk,
};

/// Builds a scene with a representative set of widgets and returns the toolkit.
fn scene() -> (Tk, rtk::WidgetId, rtk::WidgetId, rtk::WidgetId, rtk::WidgetId, rtk::WidgetId, rtk::WidgetId, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();
    let label = Label::new(&mut tk, frame, "Name").unwrap();
    let button = Button::new(&mut tk, frame, "Submit").unwrap();
    let entry = Entry::new(&mut tk, frame).unwrap();
    let check_var = tk.bool_var(true);
    let check = Checkbutton::new(&mut tk, frame, "Enabled", check_var).unwrap();
    let scale_var = tk.int_var(5);
    let scale = Scale::new(&mut tk, frame, 0, 10, scale_var).unwrap();
    let prog_var = tk.int_var(50);
    let progress = ProgressBar::new(&mut tk, frame, 100.0, prog_var).unwrap();
    tk.pack(frame, rtk::PackOpts::default());
    for id in [label, button, entry, check, scale, progress] {
        tk.pack(id, rtk::PackOpts::default());
    }
    tk.sim_resize(640, 480);
    (tk, frame, label, button, entry, check, scale, progress)
}

/// The tree starts with the root and contains the expected roles and labels.
#[test]
fn tree_has_expected_roles_and_labels() {
    let (tk, _frame, label, button, entry, check, scale, progress) = scene();
    let tree = tk.accessibility_tree();
    assert_eq!(tree.first().map(|n| n.id), Some(tk.root()));
    assert_eq!(tree.first().map(|n| n.role), Some(A11yRole::Window));
    let find = |id| tree.iter().find(|n| n.id == id).cloned().unwrap();
    assert_eq!(find(label).role, A11yRole::Label);
    assert_eq!(find(label).label.as_deref(), Some("Name"));
    assert_eq!(find(button).role, A11yRole::Button);
    assert_eq!(find(button).label.as_deref(), Some("Submit"));
    assert_eq!(find(entry).role, A11yRole::TextInput);
    assert_eq!(find(check).role, A11yRole::CheckBox);
    assert_eq!(find(check).toggled, Some(true));
    assert_eq!(find(scale).role, A11yRole::Slider);
    assert_eq!(find(scale).numeric, Some((5.0, 0.0, 10.0)));
    assert_eq!(find(progress).role, A11yRole::ProgressIndicator);
    assert_eq!(find(progress).numeric, Some((50.0, 0.0, 100.0)));
}

/// `a11y_set_label` overrides the derived label.
#[test]
fn set_label_overrides() {
    let (mut tk, _frame, _label, button, _entry, _check, _scale, _progress) = scene();
    assert!(tk.a11y_set_label(button, "Send now"));
    let tree = tk.accessibility_tree();
    let node = tree.iter().find(|n| n.id == button).unwrap();
    assert_eq!(node.label.as_deref(), Some("Send now"));
}

/// The focused flag follows a click on the Entry.
#[test]
fn focused_flag_follows_click() {
    let (mut tk, _frame, _label, _button, entry, _check, _scale, _progress) = scene();
    let rect = tk.widget_rect(entry).unwrap();
    tk.sim_click(rect.x + 4.0, rect.y + 4.0);
    let tree = tk.accessibility_tree();
    let node = tree.iter().find(|n| n.id == entry).unwrap();
    assert!(node.focused);
}

/// Clicking a Button via the accessibility action runs its command.
#[test]
fn click_runs_command() {
    let (mut tk, _frame, _label, button, _entry, _check, _scale, _progress) = scene();
    let counter = tk.int_var(0);
    tk.set_command(button, move |tk| {
        let v = counter.get(tk).unwrap_or(0);
        counter.set(tk, v + 1);
    });
    assert!(tk.a11y_do_action(button, A11yAction::Click, None));
    assert_eq!(counter.get(&tk), Some(1));
}

/// Clicking a Checkbutton via the accessibility action toggles it.
#[test]
fn click_toggles_checkbutton() {
    let (mut tk, _frame, _label, _button, _entry, check, _scale, _progress) = scene();
    let before = tk.accessibility_tree().into_iter().find(|n| n.id == check).unwrap().toggled;
    assert!(tk.a11y_do_action(check, A11yAction::Click, None));
    let after = tk.accessibility_tree().into_iter().find(|n| n.id == check).unwrap().toggled;
    assert_ne!(before, after);
}

/// SetValue on an Entry changes its text.
#[test]
fn set_value_changes_entry() {
    let (mut tk, _frame, _label, _button, entry, _check, _scale, _progress) = scene();
    assert!(tk.a11y_do_action(entry, A11yAction::SetValue, Some("hello")));
    assert_eq!(tk.entry_get(entry).as_deref(), Some("hello"));
}

/// Increment on a Scale increases its bound variable.
#[test]
fn increment_scale() {
    let (mut tk, _frame, _label, _button, _entry, _check, scale, _progress) = scene();
    let before = tk.accessibility_tree().into_iter().find(|n| n.id == scale).unwrap().numeric.unwrap().0;
    assert!(tk.a11y_do_action(scale, A11yAction::Increment, None));
    let after = tk.accessibility_tree().into_iter().find(|n| n.id == scale).unwrap().numeric.unwrap().0;
    assert!(after > before);
}

/// `a11y_tree_changed` reports changes once and then stays quiet.
#[test]
fn tree_changed_reports_once() {
    let (mut tk, _frame, _label, button, _entry, _check, _scale, _progress) = scene();
    assert!(tk.a11y_tree_changed());
    assert!(!tk.a11y_tree_changed());
    tk.a11y_set_label(button, "Changed");
    assert!(tk.a11y_tree_changed());
    assert!(!tk.a11y_tree_changed());
}
