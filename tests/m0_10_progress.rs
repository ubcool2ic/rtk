//! Behaviour and golden tests for the milestone 0.10 ProgressRing and the
//! polished linear ProgressBar.

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{Frame, PackOpts, ProgressBar, ProgressRing, RingMode, Theme, Tk};

/// Builds a small scene with a percent ring (with text) and a spinner.
fn build_ring_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 8.0, ..PackOpts::default() });

    let var = tk.f64_var(0.65);
    let ring = ProgressRing::new(tk, frame, 80.0, var).expect("ring");
    tk.ring_set_show_text(ring, true);
    tk.pack(ring, PackOpts::default());

    let svar = tk.f64_var(0.0);
    let spinner = ProgressRing::new(tk, frame, 48.0, svar).expect("spinner");
    tk.ring_set_mode(spinner, RingMode::Spinner);
    tk.pack(spinner, PackOpts::default());
}

/// Builds a scene with a linear progress bar showing percent and a label.
fn build_bar_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(frame, PackOpts { fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 8.0, ..PackOpts::default() });

    let var = tk.int_var(40);
    let bar = ProgressBar::new(tk, frame, 200.0, var).expect("bar");
    tk.progress_set_show_percent(bar, true);
    tk.progress_set_label(bar, Some("Loading".into()));
    tk.pack(bar, PackOpts::default());
}

/// Percent ring clamps out-of-range values and maps NaN to zero.
#[test]
fn ring_clamps_and_handles_nan() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.5);
    let ring = ProgressRing::new(&mut tk, root, 64.0, var).expect("ring");
    assert_eq!(tk.ring_get(ring), Some(0.5));
    assert!(tk.ring_set(ring, 2.0));
    assert_eq!(tk.ring_get(ring), Some(1.0));
    assert!(tk.ring_set(ring, -1.0));
    assert_eq!(tk.ring_get(ring), Some(0.0));
    assert!(tk.ring_set(ring, f64::NAN));
    assert_eq!(tk.ring_get(ring), Some(0.0));
    assert_eq!(var.get(&tk), Some(0.0));
}

/// Ring mode defaults to Percent and can be switched to Spinner.
#[test]
fn ring_mode_roundtrip() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.3);
    let ring = ProgressRing::new(&mut tk, root, 64.0, var).expect("ring");
    assert_eq!(tk.ring_mode(ring), Some(RingMode::Percent));
    assert!(tk.ring_set_mode(ring, RingMode::Spinner));
    assert_eq!(tk.ring_mode(ring), Some(RingMode::Spinner));
}

/// Spinner animates while visible and stops when switched back to Percent.
#[test]
fn ring_spinner_animation() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let ring = ProgressRing::new(&mut tk, root, 64.0, var).expect("ring");
    tk.pack(ring, PackOpts::default());
    tk.sim_resize(320, 200);
    assert!(!tk.is_animating());
    tk.ring_set_mode(ring, RingMode::Spinner);
    assert!(tk.is_animating());
    tk.sim_advance_time(250);
    assert!(tk.is_animating());
    tk.ring_set_mode(ring, RingMode::Percent);
    assert!(!tk.is_animating());
}

/// Progress polish APIs report the current fraction and toggle text.
#[test]
fn progress_polish_apis() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(40);
    let bar = ProgressBar::new(&mut tk, root, 200.0, var).expect("bar");
    assert_eq!(tk.progress_get(bar), Some(0.4));
    assert!(tk.progress_set_show_percent(bar, true));
    assert!(tk.progress_set_label(bar, Some("Load".into())));
    assert!(tk.progress_set_smooth(bar, true));
    assert!(tk.progress_set_label(bar, None));
}

/// Smooth mode eases toward the target and settles within 300 ms.
#[test]
fn progress_smooth_settles() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let int_var = tk.int_var(0);
    let bar = ProgressBar::new(&mut tk, root, 200.0, int_var).expect("bar");
    assert!(tk.progress_bind(bar, var));
    tk.pack(bar, PackOpts::default());
    tk.sim_resize(320, 200);
    tk.sim_advance_time(16);
    assert!(tk.progress_set_smooth(bar, true));
    var.set(&mut tk, 1.0);
    assert!(tk.is_animating());
    tk.sim_advance_time(300);
    assert!(!tk.is_animating());
    assert_eq!(tk.progress_get(bar), Some(1.0));
}

/// Golden images for the ring scene across themes and scales.
#[test]
fn golden_ring() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_ring_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("ring_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}

/// Golden images for the linear progress bar scene across themes and scales.
#[test]
fn golden_progress_bar() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_bar_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            assert_golden(
                &mut tk,
                golden_dir(),
                &format!("progressbar_{theme:?}_{scale}x").to_lowercase(),
                (320.0 * scale) as u32,
                (200.0 * scale) as u32,
                GoldenOpts::default(),
            );
        }
    }
}
