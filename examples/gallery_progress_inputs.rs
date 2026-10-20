//! The 0.10 "Progress & inputs" gallery page.
//!
//! This example demonstrates the progress and input widgets added in milestone
//! 0.10: the polished linear `ProgressBar` (percent text, label, smooth easing,
//! indeterminate mode), the `ProgressRing` (percent and spinner modes), the
//! animated `Switch`, the floating-point `Slider` (horizontal and vertical),
//! the `DoubleSpinbox`, the `ScrollArea` with attached scrollbars, and a
//! scrollable `Canvas` with a linked horizontal scrollbar.
//!
//! The page is built by [`build_page`], which is kept separate from [`main`] so
//! the 1.0 gallery can embed it as a single page.

use rtk::{
    Button, Canvas, CheckState, Checkbutton, DoubleSpinbox, Label, Opt, Orient, PackOpts,
    ProgressBar, ProgressRing, RingMode, ScrollArea, Scrollbar, Slider, Switch, Tk, WidgetId,
};

/// Builds the "Progress & inputs" page under `parent`.
///
/// Creates every milestone 0.10 widget, wires the sliders to the progress
/// widgets, and installs a timer that advances the demo values.
fn build_page(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn std::error::Error>> {
    // --- Theme switch button row ------------------------------------------
    let theme_row = rtk::Frame::new(tk, parent)?;
    tk.pack(theme_row, PackOpts::default());
    let themes = [
        ("Light", rtk::Theme::Light),
        ("Dark", rtk::Theme::Dark),
        ("Fluent", rtk::Theme::Fluent),
        ("Material", rtk::Theme::Material),
        ("High contrast", rtk::Theme::HighContrast),
        ("Modern dark", rtk::Theme::ModernDark),
        ("Modern light", rtk::Theme::ModernLight),
    ];
    for (name, theme) in themes {
        let button = Button::new(tk, theme_row, name)?;
        tk.pack(button, PackOpts::default());
        tk.set_command(button, move |tk| tk.set_theme(theme));
    }

    // --- Linear progress bars ---------------------------------------------
    let progress_var = tk.f64_var(0.35);
    let progress_int_var = tk.int_var(35);
    let progress = ProgressBar::new(tk, parent, 360.0, progress_int_var)?;
    tk.pack(progress, PackOpts::default());
    tk.progress_bind(progress, progress_var);
    tk.progress_set_show_percent(progress, true);
    tk.progress_set_label(progress, Some("Downloading".into()));
    tk.progress_set_smooth(progress, true);

    let busy_var = tk.f64_var(0.0);
    let busy_int_var = tk.int_var(0);
    let busy = ProgressBar::new(tk, parent, 360.0, busy_int_var)?;
    tk.pack(busy, PackOpts::default());
    tk.progress_bind(busy, busy_var);
    tk.progress_set_indeterminate(busy, true);

    // --- Progress rings ---------------------------------------------------
    let ring_var = tk.f64_var(0.35);
    let ring = ProgressRing::new(tk, parent, 96.0, ring_var)?;
    tk.pack(ring, PackOpts::default());
    tk.ring_set_mode(ring, RingMode::Percent);
    tk.ring_set_show_text(ring, true);

    let spinner_var = tk.f64_var(0.0);
    let spinner = ProgressRing::new(tk, parent, 64.0, spinner_var)?;
    tk.pack(spinner, PackOpts::default());
    tk.ring_set_mode(spinner, RingMode::Spinner);

    // --- Switches ---------------------------------------------------------
    let busy_switch_var = tk.bool_var(true);
    let busy_switch = Switch::new(tk, parent, "Indeterminate bar", busy_switch_var)?;
    tk.pack(busy_switch, PackOpts::default());
    tk.set_command(busy_switch, move |tk| {
        let on = tk.switch_get(busy_switch).unwrap_or(false);
        tk.progress_set_indeterminate(busy, on);
    });

    let disabled_var = tk.bool_var(false);
    let disabled_switch = Switch::new(tk, parent, "Disabled switch", disabled_var)?;
    tk.pack(disabled_switch, PackOpts::default());
    tk.config(disabled_switch, Opt::Disabled(true));

    // --- Sliders ----------------------------------------------------------
    let slider_var = tk.f64_var(0.35);
    let slider = Slider::new(tk, parent, 0.0, 1.0, slider_var)?;
    tk.pack(slider, PackOpts::default());
    tk.slider_set_orient(slider, Orient::Horizontal);
    tk.slider_set_step(slider, 0.05);
    tk.slider_set_ticks(slider, Some(0.1));
    tk.slider_set_length(slider, 360.0);
    tk.set_command(slider, move |tk| {
        let value = tk.slider_get(slider).unwrap_or(0.0);
        tk.ring_set(ring, value);
        progress_var.set(tk, value);
    });

    let vslider_var = tk.f64_var(0.5);
    let vslider = Slider::new(tk, parent, 0.0, 1.0, vslider_var)?;
    tk.pack(vslider, PackOpts::default());
    tk.slider_set_orient(vslider, Orient::Vertical);
    tk.slider_set_step(vslider, 0.05);
    tk.slider_set_length(vslider, 160.0);

    // --- Double spin boxes ------------------------------------------------
    let voltage_var = tk.f64_var(3.3);
    let voltage = DoubleSpinbox::new(tk, parent, 0.0, 12.0, voltage_var)?;
    tk.pack(voltage, PackOpts::default());
    tk.dspin_set_decimals(voltage, 2);
    tk.dspin_set_step(voltage, 0.05);
    tk.dspin_set_suffix(voltage, " V");

    let count_var = tk.f64_var(7.0);
    let count = DoubleSpinbox::new(tk, parent, 0.0, 100.0, count_var)?;
    tk.pack(count, PackOpts::default());
    tk.dspin_set_decimals(count, 0);
    tk.dspin_set_step(count, 1.0);

    // --- Tri-state checkbox -----------------------------------------------
    let select_all_var = tk.bool_var(false);
    let select_all = Checkbutton::new(tk, parent, "Select all", select_all_var)?;
    tk.checkbutton_set_tristate(select_all, true);
    tk.pack(select_all, PackOpts::default());
    let child_a_var = tk.bool_var(false);
    let child_a = Checkbutton::new(tk, parent, "  Option A", child_a_var)?;
    tk.pack(child_a, PackOpts::default());
    let child_b_var = tk.bool_var(false);
    let child_b = Checkbutton::new(tk, parent, "  Option B", child_b_var)?;
    tk.pack(child_b, PackOpts::default());
    tk.set_command(child_a, move |tk| {
        let a = child_a_var.get(tk).unwrap_or(false);
        let b = child_b_var.get(tk).unwrap_or(false);
        let state = if a && b {
            CheckState::Checked
        } else if !a && !b {
            CheckState::Unchecked
        } else {
            CheckState::Mixed
        };
        tk.checkbutton_set_state(select_all, state);
    });
    tk.set_command(child_b, move |tk| {
        let a = child_a_var.get(tk).unwrap_or(false);
        let b = child_b_var.get(tk).unwrap_or(false);
        let state = if a && b {
            CheckState::Checked
        } else if !a && !b {
            CheckState::Unchecked
        } else {
            CheckState::Mixed
        };
        tk.checkbutton_set_state(select_all, state);
    });
    tk.set_command(select_all, move |tk| {
        let on = select_all_var.get(tk).unwrap_or(false);
        child_a_var.set(tk, on);
        child_b_var.set(tk, on);
    });

    // --- ScrollArea with 20 labelled rows ---------------------------------
    let area = ScrollArea::new(tk, parent, 360.0, 160.0)?;
    tk.pack(area, PackOpts::default());
    if let Some(content) = tk.scrollarea_content(area) {
        for row in 0..20 {
            let label = Label::new(tk, content, format!("Row {row}"))?;
            tk.pack(label, PackOpts::default());
        }
    }
    let vbar = Scrollbar::new(tk, parent)?;
    tk.pack(vbar, PackOpts::default());
    let hbar = Scrollbar::horizontal(tk, parent)?;
    tk.pack(hbar, PackOpts::default());
    tk.scrollarea_attach_scrollbars(area, Some(vbar), Some(hbar));

    // --- Scrollable Canvas with a linked horizontal scrollbar -------------
    let canvas = Canvas::new(tk, parent, 360.0, 120.0)?;
    tk.pack(canvas, PackOpts::default());
    tk.canvas_scrollregion(canvas, 1200.0, 120.0);
    let accent = tk.tokens().palette.accent;
    for i in 0..12 {
        let x = 20.0 + i as f32 * 96.0;
        tk.canvas_create_round_rect(canvas, x, 20.0, x + 72.0, 100.0, 8.0, accent);
    }
    let canvas_bar = Scrollbar::horizontal(tk, parent)?;
    tk.pack(canvas_bar, PackOpts::default());
    tk.link_scrollbar(canvas_bar, canvas);

    // --- Live demo feed ---------------------------------------------------
    let mut tick: u64 = 0;
    tk.every(500, move |tk| {
        tick = tick.wrapping_add(1);
        let t = tick as f64;
        let value = 0.5 + 0.5 * (t * 0.2).sin();
        progress_var.set(tk, value);
        tk.ring_set(ring, value);
    });

    Ok(())
}

/// Runs the "Progress & inputs" gallery page.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("rtk 0.10 - progress & inputs");
    tk.geometry(900, 900);
    let root = tk.root();
    build_page(&mut tk, root)?;
    tk.mainloop()
}
