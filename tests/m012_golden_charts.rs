//! Golden-image tests for the Chart widget across themes and scale factors.

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{
    Chart, ChartKind, Frame, PackOpts, Sparkline, Theme, Tk,
};

/// Builds a scene with a line chart (time axis, two series, legend), an area
/// chart, a bar chart and two sparklines.
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

    let line = Chart::new(tk, frame, 300.0, 140.0, ChartKind::Line).expect("line");
    let a = tk.chart_add_series(line, "alpha").expect("series a");
    let b = tk.chart_add_series(line, "beta").expect("series b");
    let mut pa = Vec::new();
    let mut pb = Vec::new();
    for i in 0..20 {
        let x = i as f64 * 3600.0;
        pa.push((x, (i as f64 * 0.5).sin() * 10.0 + 20.0));
        pb.push((x, (i as f64 * 0.3).cos() * 8.0 + 15.0));
    }
    assert!(tk.chart_set_data(line, a, &pa));
    assert!(tk.chart_set_data(line, b, &pb));
    assert!(tk.chart_set_time_axis(line, true));
    tk.pack(line, PackOpts::default());

    let area = Chart::new(tk, frame, 300.0, 100.0, ChartKind::Area).expect("area");
    let c = tk.chart_add_series(area, "area").expect("series c");
    assert!(tk.chart_set_data(area, c, &[(0.0, 3.0), (1.0, 7.0), (2.0, 4.0), (3.0, 9.0)]));
    tk.pack(area, PackOpts::default());

    let bar = Chart::new(tk, frame, 300.0, 100.0, ChartKind::Bar).expect("bar");
    let d = tk.chart_add_series(bar, "bars").expect("series d");
    assert!(tk.chart_set_data(bar, d, &[(0.0, 4.0), (1.0, 8.0), (2.0, 6.0), (3.0, 10.0)]));
    tk.pack(bar, PackOpts::default());

    let spark1 = Sparkline::new(tk, frame, 120.0, 24.0).expect("spark1");
    assert!(tk.chart_set_data(spark1, 0, &[(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (3.0, 5.0)]));
    tk.pack(spark1, PackOpts::default());

    let spark2 = Sparkline::new(tk, frame, 120.0, 24.0).expect("spark2");
    assert!(tk.chart_set_data(spark2, 0, &[(0.0, 5.0), (1.0, 2.0), (2.0, 4.0), (3.0, 1.0)]));
    tk.pack(spark2, PackOpts::default());
}

/// Renders the chart scene for each theme and scale factor against goldens.
#[test]
fn golden_charts_across_themes_and_scales() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast] {
        for scale in [1.0_f64, 2.0_f64] {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (640.0 * scale) as u32;
            let h = (480.0 * scale) as u32;
            let name = format!("charts_{theme:?}_{scale}x").to_lowercase();
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

