//! Behaviour, accessibility and golden tests for the tri-state Checkbutton.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{CheckState, Checkbutton, Frame, Key, Opt, PackOpts, Theme, Tk};

/// Builds a scene with four tri-state check boxes: unchecked, checked, mixed
/// and a disabled mixed box.
fn build_tristate_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(
        frame,
        PackOpts { fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 8.0, ..PackOpts::default() },
    );

    let unchecked_var = tk.bool_var(false);
    let unchecked = Checkbutton::new(tk, frame, "Unchecked", unchecked_var).expect("unchecked");
    tk.checkbutton_set_tristate(unchecked, true);
    tk.pack(unchecked, PackOpts::default());

    let checked_var = tk.bool_var(true);
    let checked = Checkbutton::new(tk, frame, "Checked", checked_var).expect("checked");
    tk.checkbutton_set_tristate(checked, true);
    tk.pack(checked, PackOpts::default());

    let mixed_var = tk.bool_var(false);
    let mixed = Checkbutton::new(tk, frame, "Mixed", mixed_var).expect("mixed");
    tk.checkbutton_set_tristate(mixed, true);
    tk.checkbutton_set_state(mixed, CheckState::Mixed);
    tk.pack(mixed, PackOpts::default());

    let disabled_var = tk.bool_var(false);
    let disabled = Checkbutton::new(tk, frame, "Mixed (disabled)", disabled_var).expect("disabled");
    tk.checkbutton_set_tristate(disabled, true);
    tk.checkbutton_set_state(disabled, CheckState::Mixed);
    tk.config(disabled, Opt::Disabled(true));
    tk.pack(disabled, PackOpts::default());
}

/// A default check box is not tri-state and reports Checked/Unchecked only.
#[test]
fn default_checkbutton_unaffected() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Plain", var).expect("checkbutton");
    tk.pack(cb, PackOpts::default());
    tk.sim_resize(320, 200);
    assert!(!tk.checkbutton_is_tristate(cb));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Unchecked));
    assert!(!tk.checkbutton_set_state(cb, CheckState::Mixed));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Unchecked));
    let rect = tk.widget_rect(cb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(var.get(&tk), Some(true));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
}

/// Setting Mixed is rejected while tri-state mode is off.
#[test]
fn set_state_mixed_rejected_when_tristate_off() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(true);
    let cb = Checkbutton::new(&mut tk, root, "Plain", var).expect("checkbutton");
    assert!(!tk.checkbutton_set_state(cb, CheckState::Mixed));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
    assert_eq!(var.get(&tk), Some(true));
}

/// Programmatic state transitions update the variable and mixed flag.
#[test]
fn set_state_transitions() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    assert!(tk.checkbutton_set_tristate(cb, true));
    assert!(tk.checkbutton_is_tristate(cb));
    assert!(tk.checkbutton_set_state(cb, CheckState::Mixed));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Mixed));
    assert!(tk.checkbutton_set_state(cb, CheckState::Checked));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
    assert_eq!(var.get(&tk), Some(true));
    assert!(tk.checkbutton_set_state(cb, CheckState::Unchecked));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Unchecked));
    assert_eq!(var.get(&tk), Some(false));
}

/// Clicking a mixed box moves it to Checked and runs the command once.
#[test]
fn click_from_mixed_goes_checked_and_runs_command() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    tk.checkbutton_set_tristate(cb, true);
    tk.checkbutton_set_state(cb, CheckState::Mixed);
    tk.pack(cb, PackOpts::default());
    tk.sim_resize(320, 200);
    let runs = Rc::new(Cell::new(0));
    let r = runs.clone();
    tk.set_command(cb, move |_| r.set(r.get() + 1));
    let rect = tk.widget_rect(cb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
    assert_eq!(var.get(&tk), Some(true));
    assert_eq!(runs.get(), 1);
}

/// Space toggles a focused tri-state box.
#[test]
fn space_toggles_when_focused() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    tk.checkbutton_set_tristate(cb, true);
    tk.pack(cb, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(cb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
    tk.sim_key(Key::Space);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Unchecked));
}

/// With cycle_mixed on, three clicks walk Unchecked -> Checked -> Mixed -> Unchecked.
#[test]
fn cycle_mixed_order_over_three_clicks() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    tk.checkbutton_set_tristate(cb, true);
    tk.checkbutton_set_cycle_mixed(cb, true);
    tk.pack(cb, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(cb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Mixed));
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Unchecked));
}

/// `checkbutton_set_state` does not run the command.
#[test]
fn set_state_does_not_run_command() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    tk.checkbutton_set_tristate(cb, true);
    let runs = Rc::new(Cell::new(0));
    let r = runs.clone();
    tk.set_command(cb, move |_| r.set(r.get() + 1));
    assert!(tk.checkbutton_set_state(cb, CheckState::Checked));
    assert!(tk.checkbutton_set_state(cb, CheckState::Mixed));
    assert!(tk.checkbutton_set_state(cb, CheckState::Unchecked));
    assert_eq!(runs.get(), 0);
}

/// Disabled tri-state boxes ignore clicks.
#[test]
fn disabled_ignores_clicks() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    tk.checkbutton_set_tristate(cb, true);
    tk.checkbutton_set_state(cb, CheckState::Mixed);
    tk.config(cb, Opt::Disabled(true));
    tk.pack(cb, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(cb).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Mixed));
    assert_eq!(var.get(&tk), Some(false));
}

/// Turning tri-state off clears the mixed flag.
#[test]
fn turning_tristate_off_clears_mixed() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(true);
    let cb = Checkbutton::new(&mut tk, root, "Tri", var).expect("checkbutton");
    tk.checkbutton_set_tristate(cb, true);
    tk.checkbutton_set_state(cb, CheckState::Mixed);
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Mixed));
    assert!(tk.checkbutton_set_tristate(cb, false));
    assert!(!tk.checkbutton_is_tristate(cb));
    assert_eq!(tk.checkbutton_state(cb), Some(CheckState::Checked));
}

/// The accessibility tree reports `mixed` for a mixed box and false otherwise.
#[test]
fn a11y_reports_mixed() {
    let mut tk = Tk::new();
    let root = tk.root();
    let mixed_var = tk.bool_var(false);
    let mixed = Checkbutton::new(&mut tk, root, "Mixed", mixed_var).expect("mixed");
    tk.checkbutton_set_tristate(mixed, true);
    tk.checkbutton_set_state(mixed, CheckState::Mixed);
    tk.pack(mixed, PackOpts::default());
    let plain_var = tk.bool_var(true);
    let plain = Checkbutton::new(&mut tk, root, "Plain", plain_var).expect("plain");
    tk.pack(plain, PackOpts::default());
    tk.sim_resize(320, 200);
    let tree = tk.accessibility_tree();
    let mixed_node = tree.iter().find(|node| node.id == mixed).expect("mixed node");
    assert!(mixed_node.mixed);
    assert_eq!(mixed_node.toggled, Some(false));
    let plain_node = tree.iter().find(|node| node.id == plain).expect("plain node");
    assert!(!plain_node.mixed);
    assert_eq!(plain_node.toggled, Some(true));
}

/// Golden images for the tri-state scene across themes and scales.
#[test]
fn golden_tristate() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_tristate_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("tristate_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}
