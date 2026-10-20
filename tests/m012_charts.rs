//! Unit and integration tests for the Chart widget and pure chart logic.

use rtk::chart::{format_time, nice_ticks, series_color, time_ticks, ChartKind, OKABE_ITO};
use rtk::{Chart, Color, Frame, PackOpts, Sparkline, Tk};

/// `nice_ticks(0, 10, 5)` yields 0, 2, 4, 6, 8, 10.
#[test]
fn nice_ticks_zero_to_ten() {
    let ticks = nice_ticks(0.0, 10.0, 5);
    assert_eq!(ticks, vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
}

/// `nice_ticks` returns empty for degenerate inputs.
#[test]
fn nice_ticks_empty_cases() {
    assert!(nice_ticks(f64::NAN, 10.0, 5).is_empty());
    assert!(nice_ticks(0.0, f64::INFINITY, 5).is_empty());
    assert!(nice_ticks(0.0, 10.0, 0).is_empty());
    assert!(nice_ticks(10.0, 0.0, 5).is_empty());
    assert_eq!(nice_ticks(3.0, 3.0, 5), vec![3.0]);
}

/// Time ticks over a minute produce HH:MM:SS labels.
#[test]
fn time_ticks_seconds_labels() {
    let ticks = time_ticks(0.0, 60.0, 6);
    assert!(!ticks.is_empty());
    assert_eq!(ticks[0].1, "00:00:00");
    assert!(ticks.iter().all(|(_, label)| label.len() == 8));
}

/// Time ticks over several days produce MM-DD labels.
#[test]
fn time_ticks_day_labels() {
    let ticks = time_ticks(0.0, 86_400.0 * 3.0, 6);
    assert!(!ticks.is_empty());
    assert_eq!(ticks[0].1, "01-01");
    assert!(ticks.iter().all(|(_, label)| label.len() == 5));
}

/// `format_time` honours the step thresholds.
#[test]
fn format_time_steps() {
    assert_eq!(format_time(0.0, 1.0), "00:00:00");
    assert_eq!(format_time(3661.0, 60.0), "01:01");
    assert_eq!(format_time(86_400.0, 86_400.0), "01-02");
}

/// The Okabe-Ito palette entries are distinct and `series_color` cycles.
#[test]
fn okabe_ito_distinct() {
    for (i, a) in OKABE_ITO.iter().enumerate() {
        for b in OKABE_ITO.iter().skip(i + 1) {
            assert_ne!(a, b);
        }
    }
    assert_eq!(series_color(0), OKABE_ITO[0]);
    assert_eq!(series_color(OKABE_ITO.len()), OKABE_ITO[0]);
}

/// Builds a chart under a fresh toolkit root.
fn make_chart(tk: &mut Tk, kind: ChartKind) -> rtk::WidgetId {
    let root = tk.root();
    let frame = Frame::new(tk, root).expect("frame");
    tk.pack(frame, PackOpts::default());
    let chart = Chart::new(tk, frame, 200.0, 120.0, kind).expect("chart");
    tk.pack(chart, PackOpts::default());
    chart
}

/// `chart_append` trims to `max_points` and keeps the newest points.
#[test]
fn append_trims_to_max_points() {
    let mut tk = Tk::new();
    let chart = make_chart(&mut tk, ChartKind::Line);
    let s = tk.chart_add_series(chart, "a").expect("series");
    assert!(tk.chart_set_max_points(chart, 3));
    for i in 0..10 {
        assert!(tk.chart_append(chart, s, i as f64, i as f64));
    }
    assert_eq!(tk.chart_series_len(chart, s), Some(3));
    assert_eq!(tk.chart_point(chart, s, 0), Some((7.0, 7.0)));
    assert_eq!(tk.chart_point(chart, s, 2), Some((9.0, 9.0)));
}

/// `chart_set_y_range` rejects invalid ranges.
#[test]
fn y_range_validation() {
    let mut tk = Tk::new();
    let chart = make_chart(&mut tk, ChartKind::Line);
    assert!(!tk.chart_set_y_range(chart, Some((1.0, 1.0))));
    assert!(!tk.chart_set_y_range(chart, Some((2.0, 1.0))));
    assert!(!tk.chart_set_y_range(chart, Some((f64::NAN, 1.0))));
    assert!(tk.chart_set_y_range(chart, Some((0.0, 10.0))));
    assert!(tk.chart_set_y_range(chart, None));
}

/// A sparkline starts with one series named `value`.
#[test]
fn sparkline_has_one_series() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    let spark = Sparkline::new(&mut tk, frame, 80.0, 20.0).expect("sparkline");
    assert_eq!(tk.chart_series_count(spark), Some(1));
}

/// Invalid widget ids return false/None.
#[test]
fn invalid_ids() {
    let mut tk = Tk::new();
    let root = tk.root();
    let label = rtk::Label::new(&mut tk, root, "x").expect("label");
    assert!(!tk.chart_set_kind(label, ChartKind::Bar));
    assert!(tk.chart_add_series(label, "a").is_none());
    assert!(tk.chart_series_count(label).is_none());
    assert!(tk.chart_bounds(label).is_none());
    let bogus = rtk::WidgetId { index: 9999, gen: 0 };
    assert!(!tk.chart_set_legend(bogus, true));
    assert!(tk.chart_point(bogus, 0, 0).is_none());
}

/// Bar chart bounds include 0 in y.
#[test]
fn bar_bounds_include_zero() {
    let mut tk = Tk::new();
    let chart = make_chart(&mut tk, ChartKind::Bar);
    let s = tk.chart_add_series(chart, "a").expect("series");
    assert!(tk.chart_set_data(chart, s, &[(0.0, 5.0), (1.0, 8.0)]));
    let (_, _, ymin, ymax) = tk.chart_bounds(chart).expect("bounds");
    assert!(ymin <= 0.0);
    assert!(ymax >= 8.0);
}

/// Rendering a chart produces more than one distinct colour.
#[test]
fn chart_render_not_flat() {
    let mut tk = Tk::new();
    let chart = make_chart(&mut tk, ChartKind::Line);
    let s = tk.chart_add_series(chart, "a").expect("series");
    assert!(tk.chart_set_data(chart, s, &[(0.0, 0.0), (1.0, 5.0), (2.0, 2.0)]));
    assert!(tk.chart_set_series_color(chart, s, Color::rgb(200, 30, 30)));
    let pixmap = tk.render_to_pixmap(320, 240).expect("render");
    let data = pixmap.data();
    let first = &data[0..4];
    let distinct = data.chunks_exact(4).any(|px| px != first);
    assert!(distinct);
}

