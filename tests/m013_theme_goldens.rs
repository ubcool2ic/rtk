//! Milestone 0.13 step 2: golden-image tests for the semantic theme tokens,
//! status colours and the colourblind-safe palette across themes and scales.

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{Button, Canvas, Checkbutton, Entry, Fill, Frame, Label, PackOpts, ProgressBar, Theme, Tk};

/// Builds the shared theme scene under `tk`'s root.
///
/// The scene must be built *after* the theme and colourblind mode are set,
/// because canvas items capture their colours at creation time.
fn build_theme_scene(tk: &mut Tk) {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(
        frame,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );

    let title = Label::new(tk, frame, "Theme tokens").expect("title");
    tk.pack(title, PackOpts::default());

    let tokens = tk.semantic_tokens();
    let canvas = Canvas::new(tk, frame, 300.0, 60.0).expect("canvas");
    for (i, color) in tokens.chart.iter().enumerate() {
        let x = 4.0 + i as f32 * 36.0;
        tk.canvas_create_rect(canvas, x, 4.0, x + 30.0, 34.0, *color);
    }
    let statuses = [
        (tokens.colors.success, tokens.colors.on_success, "ok"),
        (tokens.colors.warning, tokens.colors.on_warning, "warn"),
        (tokens.colors.danger, tokens.colors.on_danger, "err"),
        (tokens.colors.info, tokens.colors.on_info, "info"),
    ];
    for (i, (fill, on, label)) in statuses.iter().enumerate() {
        let x = 4.0 + i as f32 * 72.0;
        tk.canvas_create_round_rect(canvas, x, 38.0, x + 64.0, 58.0, tokens.radii.md, *fill);
        tk.canvas_create_text_ex(
            canvas,
            x + 6.0,
            42.0,
            *label,
            *on,
            tokens.typography.caption,
            700,
        );
    }
    tk.pack(canvas, PackOpts::default());

    let button = Button::new(tk, frame, "Accent").expect("button");
    tk.pack(button, PackOpts::default());

    let entry = Entry::new(tk, frame).expect("entry");
    tk.entry_set(entry, "text");
    tk.pack(entry, PackOpts::default());

    let check_var = tk.bool_var(true);
    let check = Checkbutton::new(tk, frame, "Checked", check_var).expect("check");
    tk.pack(check, PackOpts::default());

    let progress_var = tk.int_var(60i64);
    let progress = ProgressBar::new(tk, frame, 120.0, progress_var).expect("progress");
    tk.pack(progress, PackOpts::default());
}

/// Renders the theme scene for `theme` at both 1x and 2x and compares goldens.
fn golden_for_theme(theme: Theme, name: &str) {
    for (scale, w, h) in [(1.0_f64, 360u32, 300u32), (2.0_f64, 720u32, 600u32)] {
        let mut tk = Tk::new();
        tk.set_theme(theme);
        build_theme_scene(&mut tk);
        assert!(tk.sim_scale_factor(scale));
        let suffix = if scale == 1.0 { "1x" } else { "2x" };
        let golden = format!("{name}_{suffix}");
        assert_golden(&mut tk, golden_dir(), &golden, w, h, GoldenOpts::default());
    }
}

/// Renders the theme scene for `theme` with colourblind-safe mode on at both scales.
fn golden_for_theme_cvd(theme: Theme, name: &str) {
    for (scale, w, h) in [(1.0_f64, 360u32, 300u32), (2.0_f64, 720u32, 600u32)] {
        let mut tk = Tk::new();
        tk.set_theme(theme);
        tk.set_colorblind_safe(true);
        build_theme_scene(&mut tk);
        assert!(tk.sim_scale_factor(scale));
        let suffix = if scale == 1.0 { "1x" } else { "2x" };
        let golden = format!("{name}_{suffix}");
        assert_golden(&mut tk, golden_dir(), &golden, w, h, GoldenOpts::default());
    }
}

/// Golden for the light theme.
#[test]
fn theme_light_golden() {
    golden_for_theme(Theme::Light, "theme_light");
}

/// Golden for the dark theme.
#[test]
fn theme_dark_golden() {
    golden_for_theme(Theme::Dark, "theme_dark");
}

/// Golden for the high-contrast theme.
#[test]
fn theme_highcontrast_golden() {
    golden_for_theme(Theme::HighContrast, "theme_highcontrast");
}

/// Golden for the modern light theme.
#[test]
fn theme_modernlight_golden() {
    golden_for_theme(Theme::ModernLight, "theme_modernlight");
}

/// Golden for the modern dark theme.
#[test]
fn theme_moderndark_golden() {
    golden_for_theme(Theme::ModernDark, "theme_moderndark");
}

/// Golden for the light theme with colourblind-safe mode enabled.
#[test]
fn theme_light_cvd_golden() {
    golden_for_theme_cvd(Theme::Light, "theme_light_cvd");
}

/// Golden for the dark theme with colourblind-safe mode enabled.
#[test]
fn theme_dark_cvd_golden() {
    golden_for_theme_cvd(Theme::Dark, "theme_dark_cvd");
}
