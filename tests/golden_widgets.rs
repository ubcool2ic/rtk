//! Golden-image (screenshot) tests for the full widget set across themes and
//! scale factors, plus a unit check of the pixel comparison helper.

use rtk::testing::{assert_golden, compare_pixmaps, golden_dir, GoldenOpts};
use rtk::{
    Button, Canvas, Checkbutton, Combobox, Entry, Frame, Label, Listbox, Notebook, PackOpts,
    ProgressBar, Radiobutton, Scale, Spinbox, Text, Theme, Tk,
};

/// Builds a scene containing one of every current widget under `tk`'s root.
fn build_scene(tk: &mut Tk) {
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

    let label = Label::new(tk, frame, "Golden scene").expect("label");
    tk.pack(label, PackOpts::default());

    let button = Button::new(tk, frame, "Press").expect("button");
    tk.pack(button, PackOpts::default());

    let entry = Entry::new(tk, frame).expect("entry");
    tk.entry_set(entry, "hello");
    tk.pack(entry, PackOpts::default());

    let check_var = tk.bool_var(true);
    let check = Checkbutton::new(tk, frame, "Checked", check_var).expect("check");
    tk.pack(check, PackOpts::default());

    let radio_var = tk.int_var(1i64);
    let radio = Radiobutton::new(tk, frame, "Radio", radio_var, 1).expect("radio");
    tk.pack(radio, PackOpts::default());

    let scale_var = tk.int_var(50i64);
    let scale = Scale::new(tk, frame, 0, 100, scale_var).expect("scale");
    tk.pack(scale, PackOpts::default());

    let spin_var = tk.int_var(5i64);
    let spin = Spinbox::new(tk, frame, 0, 10, spin_var).expect("spin");
    tk.pack(spin, PackOpts::default());

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

    let notebook = Notebook::new(tk, frame).expect("notebook");
    let page1 = Frame::new(tk, notebook).expect("page1");
    let page2 = Frame::new(tk, notebook).expect("page2");
    tk.notebook_add(notebook, page1, "Tab 1");
    tk.notebook_add(notebook, page2, "Tab 2");
    tk.pack(notebook, PackOpts::default());

    let text = Text::new(tk, frame, 20, 2).expect("text");
    tk.text_set(text, "line one\nline two");
    tk.pack(text, PackOpts::default());

    let canvas = Canvas::new(tk, frame, 100.0, 30.0).expect("canvas");
    tk.canvas_create_rect(canvas, 4.0, 4.0, 40.0, 24.0, rtk::Color::rgb(200, 60, 60));
    tk.canvas_create_oval(canvas, 50.0, 4.0, 90.0, 24.0, rtk::Color::rgb(60, 160, 60));
    tk.canvas_create_polyline(
        canvas,
        &[(4.0, 28.0), (30.0, 20.0), (60.0, 28.0), (90.0, 20.0)],
        rtk::Color::rgb(60, 60, 200),
        1.5,
    );
    tk.pack(canvas, PackOpts::default());
}

/// Renders the full widget scene for each theme and scale factor and compares
/// against the stored golden images.
#[test]
fn golden_widgets_across_themes_and_scales() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (640.0 * scale) as u32;
            let h = (480.0 * scale) as u32;
            let name = format!("widgets_{theme:?}_{scale}x").to_lowercase();
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Verifies `compare_pixmaps` reports zero differences for identical images and
/// a positive count when a single pixel is altered.
#[test]
fn compare_pixmaps_detects_changes() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(
        frame,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    let a = tk.render_to_pixmap(64, 64).expect("image");

    let (count, delta) = compare_pixmaps(&a, &a, 0);
    assert_eq!(count, 0);
    assert_eq!(delta, 0);

    let mut b = a.clone();
    let idx = (32 * 64 + 32) * 4;
    b.data_mut()[idx] = 200;
    let (count, delta) = compare_pixmaps(&a, &b, 0);
    assert!(count > 0);
    assert!(delta > 0);
}
