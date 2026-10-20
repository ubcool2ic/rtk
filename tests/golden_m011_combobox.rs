//! Golden-image tests for the editable Combobox (milestone 0.11).
//!
//! Scenes are rendered across the same theme × scale matrix used by
//! `tests/golden_m011.rs` and compared against checked-in PNGs under
//! `tests/golden`. Set `RTK_BLESS=1` to (re)write the golden images.

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{Combobox, Key, Label, PackOpts, Theme, Tk};

/// The themes exercised by every golden scene.
const THEMES: [Theme; 3] = [Theme::Light, Theme::Dark, Theme::HighContrast];

/// The scale factors exercised by every golden scene.
const SCALES: [f64; 2] = [1.0, 2.0];

/// Returns the golden name for `prefix` at `theme` and `scale`.
fn golden_name(prefix: &str, theme: Theme, scale: f64) -> String {
    format!("{prefix}_{theme:?}_{scale}x").to_lowercase()
}

/// The fruit values used by the scenes.
fn fruits() -> Vec<String> {
    ["Apple", "Apricot", "Banana", "Blackberry", "Cherry", "Grape", "Mango", "Papaya"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Builds the open editable scene: focused, "ap" typed, Apricot highlighted.
fn build_editable_scene(tk: &mut Tk) {
    let root = tk.root();
    let label = Label::new(tk, root, "Fruit").expect("label");
    tk.pack(label, PackOpts::default());
    let var = tk.int_var(-1);
    let combo = Combobox::editable(tk, root, fruits(), var).expect("combo");
    tk.pack(
        combo,
        PackOpts {
            fill: rtk::Fill::X,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );
    tk.sim_resize(320, 240);
    let rect = tk.widget_rect(combo).expect("rect");
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Down);
}

/// Builds the closed editable scene with the text "Cherry".
fn build_closed_scene(tk: &mut Tk) {
    let root = tk.root();
    let label = Label::new(tk, root, "Fruit").expect("label");
    tk.pack(label, PackOpts::default());
    let var = tk.int_var(0);
    let combo = Combobox::editable(tk, root, fruits(), var).expect("combo");
    tk.pack(
        combo,
        PackOpts {
            fill: rtk::Fill::X,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );
    tk.combobox_set_text(combo, "Cherry");
    tk.sim_resize(320, 120);
}

/// Renders the open editable scene across themes and scales.
#[test]
fn golden_m011_combobox_editable() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_editable_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (320.0 * scale) as u32;
            let h = (240.0 * scale) as u32;
            let name = golden_name("m011_combobox_editable", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Renders the closed editable scene across themes and scales.
#[test]
fn golden_m011_combobox_closed() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_closed_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (320.0 * scale) as u32;
            let h = (120.0 * scale) as u32;
            let name = golden_name("m011_combobox_closed", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Ensures both scenes render without panicking at 1x.
#[test]
fn golden_m011_combobox_scenes_render() {
    let mut tk = Tk::new();
    build_editable_scene(&mut tk);
    assert!(tk.render_to_pixmap(320, 240).is_ok());

    let mut tk = Tk::new();
    build_closed_scene(&mut tk);
    assert!(tk.render_to_pixmap(320, 120).is_ok());
}
