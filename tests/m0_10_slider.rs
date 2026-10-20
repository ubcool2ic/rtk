//! Behaviour and golden tests for the milestone 0.10 Slider widget.

use std::cell::Cell;
use std::rc::Rc;

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{Frame, Key, Orient, PackOpts, Slider, Theme, Tk};

/// Builds a scene with a horizontal slider with ticks, a vertical slider, and
/// a disabled slider.
fn build_slider_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 8.0, ..PackOpts::default() });

    let h_var = tk.f64_var(0.4);
    let h = Slider::new(tk, frame, 0.0, 1.0, h_var).expect("h");
    tk.slider_set_ticks(h, Some(0.2));
    tk.pack(h, PackOpts::default());

    let v_var = tk.f64_var(0.7);
    let v = Slider::new(tk, frame, 0.0, 1.0, v_var).expect("v");
    tk.slider_set_orient(v, Orient::Vertical);
    tk.slider_set_length(v, 80.0);
    tk.pack(v, PackOpts::default());

    let d_var = tk.f64_var(0.5);
    let d = Slider::new(tk, frame, 0.0, 1.0, d_var).expect("d");
    tk.config(d, rtk::Opt::Disabled(true));
    tk.pack(d, PackOpts::default());
}

/// `slider_set` clamps and snaps to the step.
#[test]
fn slider_set_snaps_and_clamps() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let s = Slider::new(&mut tk, root, 0.0, 10.0, var).expect("slider");
    assert!(tk.slider_set_step(s, 2.0));
    assert!(tk.slider_set(s, 5.0));
    assert_eq!(tk.slider_get(s), Some(6.0));
    assert!(tk.slider_set(s, 100.0));
    assert_eq!(tk.slider_get(s), Some(10.0));
    assert!(tk.slider_set(s, -5.0));
    assert_eq!(tk.slider_get(s), Some(0.0));
}

/// Inverted ranges work: `from > to`.
#[test]
fn slider_inverted_range() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(10.0);
    let s = Slider::new(&mut tk, root, 10.0, 0.0, var).expect("slider");
    assert_eq!(tk.slider_get(s), Some(10.0));
    assert!(tk.slider_set(s, 3.0));
    assert_eq!(tk.slider_get(s), Some(3.0));
}

/// Dragging a horizontal slider sets the value from the pointer position.
#[test]
fn slider_horizontal_drag() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let s = Slider::new(&mut tk, root, 0.0, 1.0, var).expect("slider");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_mouse_down();
    tk.sim_mouse_up();
    let v = var.get(&tk).unwrap_or(-1.0);
    assert!((v - 0.5).abs() < 0.1, "value {v}");
}

/// Vertical slider: top = max, bottom = min.
#[test]
fn slider_vertical_direction() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let s = Slider::new(&mut tk, root, 0.0, 1.0, var).expect("slider");
    tk.slider_set_orient(s, Orient::Vertical);
    tk.slider_set_length(s, 100.0);
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    // Click near the top -> high value.
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + 10.0);
    let top = var.get(&tk).unwrap_or(0.0);
    // Click near the bottom -> low value.
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height - 10.0);
    let bottom = var.get(&tk).unwrap_or(0.0);
    assert!(top > bottom, "top {top} bottom {bottom}");
}

/// Keys move the slider by one step (or 1% of range if step is 0).
#[test]
fn slider_keys() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.5);
    let s = Slider::new(&mut tk, root, 0.0, 1.0, var).expect("slider");
    tk.slider_set_step(s, 0.1);
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_key(Key::Right);
    let v = var.get(&tk).unwrap_or(0.0);
    assert!((v - 0.6).abs() < 1e-9, "v {v}");
    tk.sim_key(Key::Left);
    let v = var.get(&tk).unwrap_or(0.0);
    assert!((v - 0.5).abs() < 1e-9, "v {v}");
    tk.sim_key(Key::Home);
    assert_eq!(var.get(&tk), Some(0.0));
    tk.sim_key(Key::End);
    assert_eq!(var.get(&tk), Some(1.0));
}

/// Wheel over a slider changes the value by one step.
#[test]
fn slider_wheel() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.5);
    let s = Slider::new(&mut tk, root, 0.0, 1.0, var).expect("slider");
    tk.slider_set_step(s, 0.1);
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_wheel(1.0);
    let v = var.get(&tk).unwrap_or(0.0);
    assert!((v - 0.6).abs() < 1e-9, "v {v}");
}

/// Disabled sliders ignore input.
#[test]
fn slider_disabled_ignores_input() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.5);
    let s = Slider::new(&mut tk, root, 0.0, 1.0, var).expect("slider");
    tk.config(s, rtk::Opt::Disabled(true));
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + rect.width - 5.0, rect.y + rect.height / 2.0);
    assert_eq!(var.get(&tk), Some(0.5));
}

/// The command runs once per user change.
#[test]
fn slider_command_runs_once() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let s = Slider::new(&mut tk, root, 0.0, 1.0, var).expect("slider");
    tk.pack(s, PackOpts::default());
    tk.sim_resize(320, 200);
    let runs = Rc::new(Cell::new(0));
    let r = runs.clone();
    tk.set_command(s, move |_| r.set(r.get() + 1));
    let rect = tk.widget_rect(s).expect("rect");
    tk.sim_click(rect.x + rect.width - 5.0, rect.y + rect.height / 2.0);
    assert_eq!(runs.get(), 1);
}

/// Golden images for the slider scene across themes and scales.
#[test]
fn golden_slider() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_slider_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("slider_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}
