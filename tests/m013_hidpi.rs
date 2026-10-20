//! HiDPI verification tests (Milestone 0.13, step 6).
//!
//! These tests exercise the toolkit headlessly at several DPI scale factors
//! (100 %, 125 %, 150 %, 175 %, 200 %) and verify that:
//!
//! * rendering produces a pixmap of exactly the requested physical size;
//! * logical widget geometry is identical across scales;
//! * hit testing works in logical pixels at every scale;
//! * rendered text ink scales roughly with the scale factor;
//! * `sim_scale_factor` rejects invalid factors;
//! * a `ScaleFactorChanged` round-trip (1.0 → 1.5 → 1.0) is pixel-identical.
//!
//! Golden images for a subset of scales are also checked in.

use rtk::testing::{assert_golden, compare_pixmaps, golden_dir, GoldenOpts};
use rtk::{
    Button, Checkbutton, Combobox, Entry, Frame, Label, Listbox, PackOpts, ProgressBar, Rect,
    Scale, Theme, Tk, WidgetId,
};

/// The scale factors exercised by the geometry / hit-testing tests.
const SCALES: [f64; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];

/// Logical scene size in logical pixels.
const LOGICAL_W: f32 = 320.0;
/// Logical scene height in logical pixels.
const LOGICAL_H: f32 = 240.0;

/// Handles to the widgets of the shared test scene.
struct Scene {
    /// Root frame.
    frame: WidgetId,
    /// Static label.
    label: WidgetId,
    /// Push button with a counter command.
    button: WidgetId,
    /// Single-line entry.
    entry: WidgetId,
    /// Check box bound to a boolean variable.
    check: WidgetId,
    /// Slider bound to an integer variable.
    scale: WidgetId,
    /// Progress bar bound to an integer variable.
    progress: WidgetId,
    /// Listbox with three items.
    list: WidgetId,
    /// Drop-down selector.
    combo: WidgetId,
    /// Shared counter incremented by the button command.
    counter: std::rc::Rc<std::cell::Cell<u32>>,
    /// Boolean variable bound to the check box.
    check_var: rtk::Var<bool>,
}

/// Builds the shared scene under `tk`'s root and returns its handles.
fn build_scene(tk: &mut Tk) -> Scene {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(
        frame,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );

    let label = Label::new(tk, frame, "HiDPI").expect("label");
    tk.pack(label, PackOpts::default());

    let button = Button::new(tk, frame, "Press").expect("button");
    let counter = std::rc::Rc::new(std::cell::Cell::new(0u32));
    let counter_for_cb = counter.clone();
    tk.set_command(button, move |_tk| {
        counter_for_cb.set(counter_for_cb.get() + 1);
    });
    tk.pack(button, PackOpts::default());

    let entry = Entry::new(tk, frame).expect("entry");
    tk.entry_set(entry, "hello");
    tk.pack(entry, PackOpts::default());

    let check_var = tk.bool_var(true);
    let check = Checkbutton::new(tk, frame, "Checked", check_var).expect("check");
    tk.pack(check, PackOpts::default());

    let scale_var = tk.int_var(50i64);
    let scale = Scale::new(tk, frame, 0, 100, scale_var).expect("scale");
    tk.pack(scale, PackOpts::default());

    let progress_var = tk.int_var(40i64);
    let progress = ProgressBar::new(tk, frame, 120.0, progress_var).expect("progress");
    tk.pack(progress, PackOpts::default());

    let list = Listbox::new(tk, frame, 3).expect("listbox");
    for (index, item) in ["one", "two", "three"].into_iter().enumerate() {
        assert!(tk.listbox_insert(list, index, item));
    }
    tk.pack(list, PackOpts::default());

    let combo_var = tk.int_var(0i64);
    let combo = Combobox::new(
        tk,
        frame,
        vec!["alpha".into(), "beta".into(), "gamma".into()],
        combo_var,
    )
    .expect("combobox");
    tk.pack(combo, PackOpts::default());

    Scene {
        frame,
        label,
        button,
        entry,
        check,
        scale,
        progress,
        list,
        combo,
        counter,
        check_var,
    }
}

/// Returns the logical centre of `id`'s laid-out rectangle.
fn centre(tk: &mut Tk, id: WidgetId) -> (f32, f32) {
    let rect = tk.widget_rect(id).expect("widget rect");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

/// Renders the scene at `scale` and returns the pixmap plus the scene handles.
fn render_at(scale: f64) -> (Tk, Scene, tiny_skia::Pixmap) {
    let mut tk = Tk::new();
    let scene = build_scene(&mut tk);
    assert!(tk.sim_scale_factor(scale), "scale {scale} accepted");
    let w = (LOGICAL_W * scale as f32).round() as u32;
    let h = (LOGICAL_H * scale as f32).round() as u32;
    let pixmap = tk.render_to_pixmap(w, h).expect("render");
    (tk, scene, pixmap)
}

/// Verifies that rendering at each scale produces a pixmap of exactly the
/// requested physical size and does not panic.
#[test]
fn render_dimensions_match_requested_physical_size() {
    for scale in SCALES {
        let (_tk, _scene, pixmap) = render_at(scale);
        let w = (LOGICAL_W * scale as f32).round() as u32;
        let h = (LOGICAL_H * scale as f32).round() as u32;
        assert_eq!(pixmap.width(), w, "width at scale {scale}");
        assert_eq!(pixmap.height(), h, "height at scale {scale}");
    }
}

/// Verifies that every widget's logical geometry is identical across scales.
#[test]
fn logical_geometry_is_scale_invariant() {
    let mut baseline: Option<Vec<(WidgetId, Rect)>> = None;
    for scale in SCALES {
        let (mut tk, scene, _pixmap) = render_at(scale);
        let ids = [
            scene.frame,
            scene.label,
            scene.button,
            scene.entry,
            scene.check,
            scene.scale,
            scene.progress,
            scene.list,
            scene.combo,
        ];
        let rects: Vec<(WidgetId, Rect)> = ids
            .iter()
            .map(|id| (*id, tk.widget_rect(*id).expect("rect")))
            .collect();
        match &baseline {
            None => baseline = Some(rects),
            Some(base) => {
                for ((id, got), (_, want)) in rects.iter().zip(base.iter()) {
                    assert!(
                        (got.x - want.x).abs() < 0.5
                            && (got.y - want.y).abs() < 0.5
                            && (got.width - want.width).abs() < 0.5
                            && (got.height - want.height).abs() < 0.5,
                        "widget {id:?} geometry differs at scale {scale}: got {got:?}, want {want:?}"
                    );
                }
            }
        }
    }
}

/// Verifies that hit testing is in logical pixels at every scale: clicking the
/// logical centre of the button runs its command, and clicking the logical
/// centre of the check box toggles its variable.
#[test]
fn hit_testing_is_logical_at_every_scale() {
    for scale in SCALES {
        let (mut tk, scene, _pixmap) = render_at(scale);

        let (bx, by) = centre(&mut tk, scene.button);
        let before = scene.counter.get();
        tk.sim_click(bx, by);
        assert_eq!(
            scene.counter.get(),
            before + 1,
            "button command should run at scale {scale}"
        );

        let (cx, cy) = centre(&mut tk, scene.check);
        let before = scene.check_var.get(&tk).expect("check var");
        tk.sim_click(cx, cy);
        let after = scene.check_var.get(&tk).expect("check var");
        assert_ne!(before, after, "check box should toggle at scale {scale}");
    }
}

/// Samples the RGBA colour of the pixel at physical coordinates `(x, y)`,
/// clamping to the pixmap bounds.
fn sample_pixel(pixmap: &tiny_skia::Pixmap, x: f32, y: f32) -> [u8; 4] {
    let px = (x.max(0.0) as usize).min(pixmap.width().saturating_sub(1) as usize);
    let py = (y.max(0.0) as usize).min(pixmap.height().saturating_sub(1) as usize);
    let idx = py * pixmap.width() as usize * 4 + px * 4;
    let data = pixmap.data();
    [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]]
}

/// Measures contrast-weighted ink area inside `rect` (physical px) relative to `bg`.
///
/// Antialiased edge pixels contribute proportionally rather than as full pixels,
/// so their larger relative contribution at small scales does not bias the ratio.
fn ink_pixels(pixmap: &tiny_skia::Pixmap, rect: Rect, bg: [u8; 4]) -> f64 {
    let data = pixmap.data();
    let stride = pixmap.width() as usize * 4;
    let x0 = rect.x.max(0.0) as usize;
    let y0 = rect.y.max(0.0) as usize;
    let x1 = ((rect.x + rect.width) as usize).min(pixmap.width() as usize);
    let y1 = ((rect.y + rect.height) as usize).min(pixmap.height() as usize);
    let mut count = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let idx = y * stride + x * 4;
            let px = &data[idx..idx + 4];
            let contrast = px[0]
                .abs_diff(bg[0])
                .max(px[1].abs_diff(bg[1]))
                .max(px[2].abs_diff(bg[2]));
            count += f64::from(contrast) / 255.0;
        }
    }
    count
}

/// Verifies that the label's rendered ink bounding box scales roughly with the
/// scale factor (within ±20 % of the expected ratio relative to 1.0).
#[test]
fn label_ink_scales_with_scale_factor() {
    let mut baseline: Option<f64> = None;
    for scale in SCALES {
        let (mut tk, scene, pixmap) = render_at(scale);
        let logical = tk.widget_rect(scene.label).expect("label rect");
        let physical = Rect::new(
            logical.x * scale as f32,
            logical.y * scale as f32,
            logical.width * scale as f32,
            logical.height * scale as f32,
        );
        // Sample the label background from the rendered pixmap at the rect's
        // top-left inner pixel instead of assuming a hard-coded colour.
        let bg = sample_pixel(&pixmap, physical.x + 1.0, physical.y + 1.0);
        let ink = ink_pixels(&pixmap, physical, bg);
        let area = (physical.width as f64 * physical.height as f64).floor();
        assert!(ink > 0.0, "label rendered no ink at scale {scale}");
        assert!(
            ink < area,
            "label ink covers the whole rect at scale {scale}: background colour assumption is wrong"
        );
        match baseline {
            None => baseline = Some(ink),
            Some(base) => {
                let expected = base * scale * scale;
                let ratio = ink / expected;
                assert!(
                    (0.8..=1.2).contains(&ratio),
                    "label ink at scale {scale} = {ink} (expected ~{expected}, ratio {ratio})"
                );
            }
        }
    }
}

/// Verifies that `sim_scale_factor` rejects 0, negative, NaN and > 16.
#[test]
fn sim_scale_factor_rejects_invalid_values() {
    let mut tk = Tk::new();
    assert!(!tk.sim_scale_factor(0.0));
    assert!(!tk.sim_scale_factor(-1.0));
    assert!(!tk.sim_scale_factor(f64::NAN));
    assert!(!tk.sim_scale_factor(16.5));
    assert!(!tk.sim_scale_factor(f64::INFINITY));
    assert!(tk.sim_scale_factor(1.0));
}

/// Verifies that a `ScaleFactorChanged` round-trip (1.0 → 1.5 → 1.0) renders
/// pixel-identically at 1.0.
#[test]
fn scale_factor_round_trip_is_pixel_identical() {
    let mut tk = Tk::new();
    let _scene = build_scene(&mut tk);
    assert!(tk.sim_scale_factor(1.0));
    let first = tk.render_to_pixmap(320, 240).expect("first render");

    assert!(tk.sim_scale_factor(1.5));
    let _mid = tk.render_to_pixmap(480, 360).expect("mid render");

    assert!(tk.sim_scale_factor(1.0));
    let second = tk.render_to_pixmap(320, 240).expect("second render");

    let (diff, delta) = compare_pixmaps(&first, &second, 0);
    assert_eq!(diff, 0, "round-trip diff pixels (max delta {delta})");
}

/// Renders the shared scene at 1.25× (400×300) and compares against the golden.
#[test]
fn golden_hidpi_light_1_25x() {
    let (mut tk, _scene, _pixmap) = render_at(1.25);
    assert_golden(&mut tk, golden_dir(), "hidpi_light_1_25x", 400, 300, GoldenOpts::default());
}

/// Renders the shared scene at 1.5× (480×360) and compares against the golden.
#[test]
fn golden_hidpi_light_1_5x() {
    let (mut tk, _scene, _pixmap) = render_at(1.5);
    assert_golden(&mut tk, golden_dir(), "hidpi_light_1_5x", 480, 360, GoldenOpts::default());
}

/// Renders the shared scene at 1.5× in the Dark theme and compares against the golden.
#[test]
fn golden_hidpi_dark_1_5x() {
    let mut tk = Tk::new();
    tk.set_theme(Theme::Dark);
    let _scene = build_scene(&mut tk);
    assert!(tk.sim_scale_factor(1.5));
    assert_golden(&mut tk, golden_dir(), "hidpi_dark_1_5x", 480, 360, GoldenOpts::default());
}
