//! Behaviour and golden tests for the milestone 0.10 Switch widget.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{Frame, Key, PackOpts, Switch, Theme, Tk};

/// Builds a scene with an on switch, an off switch, and a disabled switch.
fn build_switch_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 8.0, ..PackOpts::default() });

    let on_var = tk.bool_var(true);
    let on = Switch::new(tk, frame, "On", on_var).expect("on");
    tk.pack(on, PackOpts::default());

    let off_var = tk.bool_var(false);
    let off = Switch::new(tk, frame, "Off", off_var).expect("off");
    tk.pack(off, PackOpts::default());

    let dis_var = tk.bool_var(true);
    let dis = Switch::new(tk, frame, "Disabled", dis_var).expect("disabled");
    tk.config(dis, rtk::Opt::Disabled(true));
    tk.pack(dis, PackOpts::default());
}

/// Clicking a switch toggles its variable and runs its command.
#[test]
fn switch_click_toggles_and_runs_command() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let sw = Switch::new(&mut tk, root, "Toggle", var).expect("switch");
    tk.pack(sw, PackOpts::default());
    tk.sim_resize(320, 200);
    let runs = Rc::new(Cell::new(0));
    let r = runs.clone();
    tk.set_command(sw, move |_| r.set(r.get() + 1));
    let rect = tk.widget_rect(sw).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(var.get(&tk), Some(true));
    assert_eq!(runs.get(), 1);
}

/// Space toggles a focused switch.
#[test]
fn switch_space_toggles_when_focused() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let sw = Switch::new(&mut tk, root, "Toggle", var).expect("switch");
    tk.pack(sw, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(sw).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(var.get(&tk), Some(true));
    tk.sim_key(Key::Space);
    assert_eq!(var.get(&tk), Some(false));
}

/// Disabled switches ignore clicks.
#[test]
fn switch_disabled_ignores_input() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let sw = Switch::new(&mut tk, root, "Toggle", var).expect("switch");
    tk.config(sw, rtk::Opt::Disabled(true));
    tk.pack(sw, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(sw).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(var.get(&tk), Some(false));
}

/// `switch_set` changes the value without running the command.
#[test]
fn switch_set_does_not_run_command() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let sw = Switch::new(&mut tk, root, "Toggle", var).expect("switch");
    let runs = Rc::new(Cell::new(0));
    let r = runs.clone();
    tk.set_command(sw, move |_| r.set(r.get() + 1));
    assert!(tk.switch_set(sw, true));
    assert_eq!(var.get(&tk), Some(true));
    assert_eq!(runs.get(), 0);
    assert_eq!(tk.switch_get(sw), Some(true));
}

/// The knob animation settles and `is_animating` becomes false.
#[test]
fn switch_animation_settles() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.bool_var(false);
    let sw = Switch::new(&mut tk, root, "Toggle", var).expect("switch");
    tk.pack(sw, PackOpts::default());
    tk.sim_resize(320, 200);
    tk.sim_advance_time(16);
    assert!(!tk.is_animating());
    assert!(tk.switch_set(sw, true));
    assert!(tk.is_animating());
    tk.sim_advance_time(200);
    assert!(!tk.is_animating());
}

/// Golden images for the switch scene across themes and scales.
#[test]
fn golden_switch() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_switch_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("switch_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}
