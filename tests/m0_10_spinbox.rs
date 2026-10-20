//! Behaviour and golden tests for the milestone 0.10 DoubleSpinbox widget.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{DoubleSpinbox, Frame, Key, PackOpts, Theme, Tk};

/// Builds a scene with two double spin boxes, one with a suffix.
fn build_dspin_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 8.0, ..PackOpts::default() });

    let a_var = tk.f64_var(1.5);
    let a = DoubleSpinbox::new(tk, frame, 0.0, 10.0, a_var).expect("a");
    tk.pack(a, PackOpts::default());

    let b_var = tk.f64_var(3.25);
    let b = DoubleSpinbox::new(tk, frame, 0.0, 10.0, b_var).expect("b");
    tk.dspin_set_suffix(b, " V");
    tk.pack(b, PackOpts::default());
}

/// `dspin_text` formats with the configured decimals and suffix.
#[test]
fn dspin_formatting() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(1.5);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    assert_eq!(tk.dspin_text(s).as_deref(), Some("1.50"));
    assert!(tk.dspin_set_decimals(s, 3));
    assert_eq!(tk.dspin_text(s).as_deref(), Some("1.500"));
    assert!(tk.dspin_set_suffix(s, " V"));
    assert_eq!(tk.dspin_text(s).as_deref(), Some("1.500 V"));
}

/// Stepping by 0.1 three times gives exactly "0.30" (no float drift).
#[test]
fn dspin_no_drift() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    tk.dspin_set_step(s, 0.1);
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + rect.width - 10.0, rect.y + 5.0);
    tk.sim_click(rect.x + rect.width - 10.0, rect.y + 5.0);
    tk.sim_click(rect.x + rect.width - 10.0, rect.y + 5.0);
    assert_eq!(tk.dspin_text(s).as_deref(), Some("0.30"));
}

/// Clicking the up/down arrows steps the value and runs the command.
#[test]
fn dspin_arrows_step_and_run_command() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(1.0);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let runs = Rc::new(Cell::new(0));
    let r = runs.clone();
    tk.set_command(s, move |_| r.set(r.get() + 1));
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + rect.width - 10.0, rect.y + 5.0);
    assert_eq!(tk.dspin_get(s), Some(2.0));
    assert_eq!(runs.get(), 1);
    tk.sim_click(rect.x + rect.width - 10.0, rect.y + rect.height - 5.0);
    assert_eq!(tk.dspin_get(s), Some(1.0));
    assert_eq!(runs.get(), 2);
}

/// Up/Down keys step the value.
#[test]
fn dspin_keys_step() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(1.0);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + rect.height / 2.0);
    tk.sim_key(Key::Up);
    assert_eq!(tk.dspin_get(s), Some(2.0));
    tk.sim_key(Key::Down);
    assert_eq!(tk.dspin_get(s), Some(1.0));
}

/// Typing digits and pressing Enter commits the value.
#[test]
fn dspin_text_entry_commits() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(1.0);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + rect.height / 2.0);
    // Clear the buffer and type a new value.
    for _ in 0..8 {
        tk.sim_key(Key::Backspace);
    }
    tk.sim_text("4.25");
    tk.sim_key(Key::Enter);
    assert_eq!(tk.dspin_get(s), Some(4.25));
}

/// Invalid text reverts to the last value.
#[test]
fn dspin_invalid_text_reverts() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(2.0);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + rect.height / 2.0);
    for _ in 0..8 {
        tk.sim_key(Key::Backspace);
    }
    tk.sim_text("abc");
    tk.sim_key(Key::Enter);
    assert_eq!(tk.dspin_get(s), Some(2.0));
}

/// A comma is accepted as a decimal separator.
#[test]
fn dspin_comma_separator() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let s = DoubleSpinbox::new(&mut tk, root, 0.0, 10.0, var).expect("dspin");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + rect.height / 2.0);
    for _ in 0..8 {
        tk.sim_key(Key::Backspace);
    }
    tk.sim_text("3,5");
    tk.sim_key(Key::Enter);
    assert_eq!(tk.dspin_get(s), Some(3.5));
}

/// Golden images for the double spin box scene across themes and scales.
#[test]
fn golden_dspin() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_dspin_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("dspin_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}
