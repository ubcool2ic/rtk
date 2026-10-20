//! The 0.12 "Charts & media" gallery page.
//!
//! This example demonstrates the chart, image, file drag-and-drop and (optionally)
//! system tray features added in milestone 0.12. The page is built by
//! [`build_page`], which is kept separate from [`main`] so the 1.0 gallery can
//! embed it as a single page.

#[cfg(feature = "tray")]
use std::cell::RefCell;

use rtk::{
    Button, Chart, ChartKind, Image, Label, Opt, PackOpts, Sparkline, Tk, WidgetId,
};

/// Builds the "Charts & media" page under `parent`.
///
/// Creates a live line chart (time axis, two series), an area chart, a bar
/// chart, a live sparkline, an SVG image and icon button (when the `svg`
/// feature is enabled), and a file drop target label.
fn build_page(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn std::error::Error>> {
    // --- Live line chart with a time axis and two series -------------------
    let line = Chart::new(tk, parent, 860.0, 220.0, ChartKind::Line)?;
    tk.pack(line, PackOpts::default());
    tk.chart_set_time_axis(line, true);
    tk.chart_set_legend(line, true);
    tk.chart_set_max_points(line, 120);
    let temp = tk.chart_add_series(line, "temp").unwrap_or(0);
    let humidity = tk.chart_add_series(line, "humidity").unwrap_or(1);

    // --- Area chart --------------------------------------------------------
    let area = Chart::new(tk, parent, 860.0, 160.0, ChartKind::Area)?;
    tk.pack(area, PackOpts::default());
    tk.chart_set_legend(area, true);
    let area_series = tk.chart_add_series(area, "load").unwrap_or(0);
    let area_points: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let x = i as f64;
            let y = 50.0 + 30.0 * (x * 0.3).sin();
            (x, y)
        })
        .collect();
    tk.chart_set_data(area, area_series, &area_points);

    // --- Bar chart ---------------------------------------------------------
    let bar = Chart::new(tk, parent, 860.0, 160.0, ChartKind::Bar)?;
    tk.pack(bar, PackOpts::default());
    tk.chart_set_legend(bar, true);
    let bar_series = tk.chart_add_series(bar, "sales").unwrap_or(0);
    let bar_points: Vec<(f64, f64)> = (0..8)
        .map(|i| (i as f64, 10.0 + (i as f64 * 7.0) % 40.0))
        .collect();
    tk.chart_set_data(bar, bar_series, &bar_points);

    // --- Live sparkline ----------------------------------------------------
    let spark = Sparkline::new(tk, parent, 860.0, 60.0)?;
    tk.pack(spark, PackOpts::default());
    tk.chart_set_max_points(spark, 120);

    // --- SVG image and icon button (only with the `svg` feature) -----------
    #[cfg(feature = "svg")]
    {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
            <circle cx="32" cy="32" r="28" fill="#0072B2"/>
            <polygon points="32,12 38,26 53,26 41,35 45,50 32,41 19,50 23,35 11,26 26,26" fill="#E69F00"/>
        </svg>"##;
        let image = Image::new(tk, parent, 64.0, 64.0)?;
        tk.pack(image, PackOpts::default());
        tk.image_set_svg(image, svg);

        let icon = tk.load_svg(svg, 16, 16);
        let icon_button = Button::new(tk, parent, "Icon button")?;
        tk.pack(icon_button, PackOpts::default());
        if let Some(icon) = icon {
            tk.config(icon_button, Opt::Image(icon));
        }
    }

    // --- File drop target --------------------------------------------------
    let drop_label = Label::new(tk, parent, "Drop files here")?;
    tk.pack(drop_label, PackOpts::default());
    tk.bind_file_drop(drop_label, move |tk, paths| {
        let names: Vec<String> = paths
            .iter()
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| p.display().to_string())
            })
            .collect();
        tk.config(drop_label, Opt::Text(names.join(", ")));
    });
    tk.bind_file_hover(drop_label, move |tk, phase| match phase {
        rtk::DropPhase::Enter => {
            tk.config(drop_label, Opt::Text("Release to drop".into()));
        }
        rtk::DropPhase::Leave => {
            tk.config(drop_label, Opt::Text("Drop files here".into()));
        }
        _ => {}
    });

    // --- Live data feed ----------------------------------------------------
    // x = seconds since 2026-01-01T00:00Z, advancing 60 s per tick.
    let mut tick: u64 = 0;
    tk.every(500, move |tk| {
        tick = tick.wrapping_add(1);
        let x = (tick * 60) as f64;
        let t = tick as f64;
        let temp_y = 20.0 + 5.0 * (t * 0.2).sin();
        let humidity_y = 50.0 + 20.0 * (t * 0.15).cos();
        tk.chart_append(line, temp, x, temp_y);
        tk.chart_append(line, humidity, x, humidity_y);
        tk.chart_append(spark, 0, x, temp_y);
    });

    Ok(())
}

/// Installs a system tray icon (only with the `tray` feature).
///
/// The tray is created from `tk.after_idle` so it runs on the UI thread once
/// the event loop is active (required on macOS). It is kept alive in a
/// thread-local holder so it is not dropped immediately.
#[cfg(feature = "tray")]
fn install_tray(tk: &mut Tk) {
    thread_local! {
        /// Keeps the tray icon alive for the lifetime of the UI thread.
        static TRAY: RefCell<Option<rtk::tray::Tray>> = const { RefCell::new(None) };
    }

    tk.after_idle(|tk| {
        // Generate a 16x16 RGBA icon: an Okabe-Ito blue square.
        let mut rgba = Vec::with_capacity(16 * 16 * 4);
        for _ in 0..(16 * 16) {
            rgba.extend_from_slice(&[0x00, 0x72, 0xB2, 0xFF]);
        }
        let opts = rtk::tray::TrayOptions {
            tooltip: "rtk gallery".into(),
            icon_rgba: rgba,
            icon_width: 16,
            icon_height: 16,
            menu_items: vec!["Show".into(), "Quit".into()],
        };
        let tray = rtk::tray::Tray::new(tk, opts, |tk, ev| {
            if let rtk::tray::TrayEvent::Menu(1) = ev {
                tk.quit();
            }
        });
        if let Ok(tray) = tray {
            TRAY.with(|slot| {
                *slot.borrow_mut() = Some(tray);
            });
        }
    });
}

/// Runs the "Charts & media" gallery page.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("rtk gallery: charts & media");
    tk.geometry(900, 700);
    let root = tk.root();
    build_page(&mut tk, root)?;
    #[cfg(feature = "tray")]
    install_tray(&mut tk);
    tk.mainloop()
}
