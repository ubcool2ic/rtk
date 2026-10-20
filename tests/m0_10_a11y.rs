//! Integration tests for accessibility of the milestone 0.10 widgets
//! (ProgressRing, Switch, Slider, DoubleSpinbox, ScrollArea).
//!
//! Verifies that the accessibility tree exposes the correct roles, toggled
//! state, numeric values and actions for these widgets.

use rtk::{A11yAction, A11yRole, DoubleSpinbox, Frame, ProgressRing, ScrollArea, Slider, Switch, Tk};

/// Creates a fresh headless toolkit with a packed root frame.
fn setup() -> (Tk, rtk::WidgetId) {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, rtk::PackOpts::default());
    (tk, frame)
}

/// Finds the accessibility node for `id` in the tree.
fn node_for(tk: &Tk, id: rtk::WidgetId) -> Option<rtk::A11yNode> {
    tk.accessibility_tree().into_iter().find(|n| n.id == id)
}

/// ProgressRing maps to `ProgressIndicator` with a `0..=1` numeric value.
#[test]
fn ring_role_and_numeric() {
    let (mut tk, frame) = setup();
    let var = tk.f64_var(0.5);
    let ring = ProgressRing::new(&mut tk, frame, 48.0, var).expect("ring");
    tk.pack(ring, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    let node = node_for(&tk, ring).expect("ring node");
    assert_eq!(node.role, A11yRole::ProgressIndicator);
    let (value, lo, hi) = node.numeric.expect("numeric");
    assert!((value - 0.5).abs() < 1e-9);
    assert!((lo - 0.0).abs() < 1e-9);
    assert!((hi - 1.0).abs() < 1e-9);
}

/// Switch maps to `CheckBox`, exposes its label and toggled state, and
/// supports `Click` + `Toggle` actions.
#[test]
fn switch_role_label_toggled() {
    let (mut tk, frame) = setup();
    let var = tk.bool_var(true);
    let switch = Switch::new(&mut tk, frame, "Enabled", var).expect("switch");
    tk.pack(switch, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    let node = node_for(&tk, switch).expect("switch node");
    assert_eq!(node.role, A11yRole::CheckBox);
    assert_eq!(node.label.as_deref(), Some("Enabled"));
    assert_eq!(node.toggled, Some(true));
    assert!(node.actions.contains(&A11yAction::Click));
    assert!(node.actions.contains(&A11yAction::Toggle));
}

/// Toggling a switch through `a11y_do_action` flips its bound value.
#[test]
fn switch_toggle_action() {
    let (mut tk, frame) = setup();
    let var = tk.bool_var(false);
    let switch = Switch::new(&mut tk, frame, "Flag", var).expect("switch");
    tk.pack(switch, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    assert!(tk.a11y_do_action(switch, A11yAction::Toggle, None));
    assert_eq!(tk.switch_get(switch), Some(true));
    let node = node_for(&tk, switch).expect("switch node");
    assert_eq!(node.toggled, Some(true));
}

/// Slider maps to `Slider` with the correct numeric range and value.
#[test]
fn slider_role_and_numeric() {
    let (mut tk, frame) = setup();
    let var = tk.f64_var(5.0);
    let slider = Slider::new(&mut tk, frame, 0.0, 10.0, var).expect("slider");
    tk.pack(slider, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    let node = node_for(&tk, slider).expect("slider node");
    assert_eq!(node.role, A11yRole::Slider);
    let (value, lo, hi) = node.numeric.expect("numeric");
    assert!((value - 5.0).abs() < 1e-9);
    assert!((lo - 0.0).abs() < 1e-9);
    assert!((hi - 10.0).abs() < 1e-9);
    assert!(node.actions.contains(&A11yAction::Increment));
    assert!(node.actions.contains(&A11yAction::Decrement));
}

/// Incrementing a slider through `a11y_do_action` raises its value.
#[test]
fn slider_increment_action() {
    let (mut tk, frame) = setup();
    let var = tk.f64_var(5.0);
    let slider = Slider::new(&mut tk, frame, 0.0, 10.0, var).expect("slider");
    tk.pack(slider, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    assert!(tk.a11y_do_action(slider, A11yAction::Increment, None));
    let value = tk.slider_get(slider).expect("value");
    assert!(value > 5.0);
}

/// DoubleSpinbox maps to `SpinButton` with a formatted value and numeric range.
#[test]
fn dspin_role_value_numeric() {
    let (mut tk, frame) = setup();
    let var = tk.f64_var(2.5);
    let dspin = DoubleSpinbox::new(&mut tk, frame, 0.0, 10.0, var).expect("dspin");
    tk.pack(dspin, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    let node = node_for(&tk, dspin).expect("dspin node");
    assert_eq!(node.role, A11yRole::SpinButton);
    assert_eq!(node.value.as_deref(), Some("2.50"));
    let (value, lo, hi) = node.numeric.expect("numeric");
    assert!((value - 2.5).abs() < 1e-9);
    assert!((lo - 0.0).abs() < 1e-9);
    assert!((hi - 10.0).abs() < 1e-9);
}

/// ScrollArea maps to `Group` and has no label.
#[test]
fn scrollarea_role() {
    let (mut tk, frame) = setup();
    let area = ScrollArea::new(&mut tk, frame, 200.0, 150.0).expect("scrollarea");
    tk.pack(area, rtk::PackOpts::default());
    tk.sim_resize(640, 480);
    let node = node_for(&tk, area).expect("scrollarea node");
    assert_eq!(node.role, A11yRole::Group);
    assert!(node.label.is_none());
}
