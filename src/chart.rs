//! Pure chart logic: series palette, axis tick generation, time-axis labels and
//! the display-list builder that turns a [`ChartState`] into canvas items.
//!
//! Nothing in this module touches [`crate::Tk`]; it is deterministic and unit
//! testable. Rendering happens in `render.rs` through the Canvas item path.

use std::collections::VecDeque;

use crate::style::Palette;
use crate::widgets::{CanvasItemKind, Color};

/// Okabe-Ito colour-blind-safe series palette in chart order (black omitted), taken from
/// [`crate::theme::OKABE_ITO`]: blue, orange, bluish green, vermillion, reddish purple,
/// sky blue, yellow.
pub const OKABE_ITO: [Color; 7] = {
    let p = crate::theme::OKABE_ITO;
    [p[5], p[1], p[3], p[6], p[7], p[2], p[4]]
};

/// Returns the palette colour for series `index`, cycling through [`OKABE_ITO`].
pub fn series_color(index: usize) -> Color {
    OKABE_ITO[index % OKABE_ITO.len()]
}

/// The visual form of a chart.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChartKind {
    /// Connected line per series (default).
    #[default]
    Line,
    /// Filled area under each line.
    Area,
    /// Grouped vertical bars per series.
    Bar,
}

/// A single named data series.
pub(crate) struct SeriesData {
    /// Legend label.
    pub name: String,
    /// Data points in `(x, y)` order.
    pub points: VecDeque<(f64, f64)>,
    /// Optional explicit colour overriding the palette.
    pub color: Option<Color>,
}

impl SeriesData {
    /// Creates an empty series with the given legend name.
    pub(crate) fn new(name: String) -> Self {
        Self { name, points: VecDeque::new(), color: None }
    }
}

/// Mutable state of a chart widget.
pub(crate) struct ChartState {
    /// Visual form.
    pub kind: ChartKind,
    /// Data series in draw order.
    pub series: Vec<SeriesData>,
    /// Whether the x axis is formatted as UTC time.
    pub time_axis: bool,
    /// Sparkline mode: no axes, grid, labels or legend.
    pub sparkline: bool,
    /// Whether to draw the legend row.
    pub legend: bool,
    /// Maximum retained points per series.
    pub max_points: usize,
    /// Explicit y range overriding the data-derived one.
    pub y_range: Option<(f64, f64)>,
}

impl Default for ChartState {
    /// Default chart: line, no series, legend on, 10 000 points per series.
    fn default() -> Self {
        Self {
            kind: ChartKind::Line,
            series: Vec::new(),
            time_axis: false,
            sparkline: false,
            legend: true,
            max_points: 10_000,
            y_range: None,
        }
    }
}

impl ChartState {
    /// Computes `(xmin, xmax, ymin, ymax)` over all finite points.
    ///
    /// `y_range` overrides the y bounds; Bar and Area charts include 0 in y.
    /// Degenerate ranges are widened by 1.0 on each side. Returns `None` when
    /// there are no finite points.
    pub(crate) fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        let mut ymin = f64::INFINITY;
        let mut ymax = f64::NEG_INFINITY;
        let mut any = false;
        for series in &self.series {
            for &(x, y) in &series.points {
                if !x.is_finite() || !y.is_finite() {
                    continue;
                }
                any = true;
                xmin = xmin.min(x);
                xmax = xmax.max(x);
                ymin = ymin.min(y);
                ymax = ymax.max(y);
            }
        }
        if !any {
            return None;
        }
        if let Some((lo, hi)) = self.y_range {
            ymin = lo;
            ymax = hi;
        } else if matches!(self.kind, ChartKind::Bar | ChartKind::Area) {
            ymin = ymin.min(0.0);
            ymax = ymax.max(0.0);
        }
        if xmin == xmax {
            xmin -= 1.0;
            xmax += 1.0;
        }
        if ymin == ymax {
            ymin -= 1.0;
            ymax += 1.0;
        }
        Some((xmin, xmax, ymin, ymax))
    }
}

/// Computes "nice" axis ticks at 1/2/5×10^n steps inside `[min, max]`.
///
/// Returns an empty vector when the inputs are non-finite, `max_ticks == 0`
/// or `min > max`. When `min == max` returns `vec![min]`.
pub fn nice_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || max_ticks == 0 || min > max {
        return Vec::new();
    }
    if min == max {
        return vec![min];
    }
    let raw = (max - min) / max_ticks as f64;
    let magnitude = 10f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    let step = nice * magnitude;
    if !step.is_finite() || step <= 0.0 {
        return Vec::new();
    }
    let eps = step * 1e-9;
    let first = (min / step).ceil() as i64;
    let last = (max / step).floor() as i64;
    let mut ticks = Vec::new();
    let mut k = first;
    while k <= last {
        let v = k as f64 * step;
        if v >= min - eps && v <= max + eps {
            ticks.push(v);
        }
        k += 1;
    }
    ticks
}

/// Candidate time-axis steps in seconds.
const TIME_STEPS: [f64; 20] = [
    1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0,
    10800.0, 21600.0, 43200.0, 86400.0, 172800.0, 604800.0,
];

/// Computes time-axis ticks `(unix_seconds, label)` inside `[min, max]`.
///
/// Chooses the smallest step from [`TIME_STEPS`] that yields at most
/// `max_ticks` ticks (falling back to the largest step). Same empty rules as
/// [`nice_ticks`].
pub fn time_ticks(min: f64, max: f64, max_ticks: usize) -> Vec<(f64, String)> {
    if !min.is_finite() || !max.is_finite() || max_ticks == 0 || min > max {
        return Vec::new();
    }
    if min == max {
        return vec![(min, format_time(min, 1.0))];
    }
    let mut step = *TIME_STEPS.last().unwrap_or(&604800.0);
    for candidate in TIME_STEPS {
        let count = ((max / candidate).floor() - (min / candidate).ceil() + 1.0).max(0.0);
        if count <= max_ticks as f64 {
            step = candidate;
            break;
        }
    }
    let eps = step * 1e-9;
    let first = (min / step).ceil() as i64;
    let last = (max / step).floor() as i64;
    let mut ticks = Vec::new();
    let mut k = first;
    while k <= last {
        let v = k as f64 * step;
        if v >= min - eps && v <= max + eps {
            ticks.push((v, format_time(v, step)));
        }
        k += 1;
    }
    ticks
}

/// Formats a Unix timestamp (UTC) for a time axis with the given step.
///
/// `step < 60` gives `HH:MM:SS`, `step < 86400` gives `HH:MM`, otherwise
/// `MM-DD`.
pub fn format_time(secs: f64, step: f64) -> String {
    let total = secs.floor() as i64;
    let days = total.div_euclid(86_400);
    let rem = total.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let _ = year;
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;
    let second = rem % 60;
    if step < 60.0 {
        format!("{hour:02}:{minute:02}:{second:02}")
    } else if step < 86_400.0 {
        format!("{hour:02}:{minute:02}")
    } else {
        format!("{month:02}-{day:02}")
    }
}

/// Converts days since the Unix epoch to a proleptic Gregorian `(year, month, day)`.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Formats a tick value, trimming a trailing `.0` from whole numbers.
fn format_tick(v: f64) -> String {
    let s = format!("{v}");
    if let Some(stripped) = s.strip_suffix(".0") {
        stripped.to_string()
    } else {
        s
    }
}

/// Estimated pixel width of `text` at `size` (0.6 × size per character).
fn estimate_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.6
}

/// Builds the display list for a chart in widget-local logical pixels.
///
/// Colours come only from `palette` and the series colours. Returns an empty
/// list when the plot area is smaller than 2 px in either direction or there
/// are no finite points (axes and legend are still drawn when not a sparkline).
pub(crate) fn build_items(
    state: &ChartState,
    w: f32,
    h: f32,
    palette: &Palette,
    font_size: f32,
) -> Vec<CanvasItemKind> {
    let mut items = Vec::new();
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return items;
    }

    if state.sparkline {
        let pad = 2.0;
        let plot = (w - 2.0 * pad, h - 2.0 * pad);
        if plot.0 < 2.0 || plot.1 < 2.0 {
            return items;
        }
        let Some((xmin, xmax, ymin, ymax)) = state.bounds() else {
            return items;
        };
        let map = |x: f64, y: f64| -> (f32, f32) {
            let px = pad + ((x - xmin) / (xmax - xmin)) as f32 * plot.0;
            let py = pad + (1.0 - ((y - ymin) / (ymax - ymin)) as f32) * plot.1;
            (px, py)
        };
        for (i, series) in state.series.iter().enumerate() {
            let color = series.color.unwrap_or_else(|| series_color(i));
            let mut points = Vec::new();
            for &(x, y) in &series.points {
                if !x.is_finite() || !y.is_finite() {
                    continue;
                }
                points.push(map(x, y));
            }
            if points.len() >= 2 {
                items.push(CanvasItemKind::Path { points, color, width: 1.5, filled: false });
            }
        }
        return items;
    }

    let label_size = (font_size * 0.85).max(1.0);
    let legend_h = if state.legend && !state.series.is_empty() { 18.0 } else { 0.0 };
    let left = 44.0;
    let right = 8.0;
    let top = 8.0 + legend_h;
    let bottom = 22.0;
    let plot_w = w - left - right;
    let plot_h = h - top - bottom;

    // Legend row at the top.
    if legend_h > 0.0 {
        let mut lx = left;
        let ly = 4.0;
        for (i, series) in state.series.iter().enumerate() {
            let color = series.color.unwrap_or_else(|| series_color(i));
            items.push(CanvasItemKind::Rect { x0: lx, y0: ly, x1: lx + 10.0, y1: ly + 10.0, color });
            let tx = lx + 14.0;
            items.push(CanvasItemKind::TextEx {
                x: tx,
                y: ly - 1.0,
                text: series.name.clone(),
                color: palette.fg,
                size: label_size,
                weight: 400,
            });
            lx = tx + estimate_width(&series.name, label_size) + 16.0;
        }
    }

    let Some((xmin, xmax, ymin, ymax)) = state.bounds() else {
        return items;
    };

    if plot_w < 2.0 || plot_h < 2.0 {
        return items;
    }

    let map_x = |x: f64| -> f32 { left + ((x - xmin) / (xmax - xmin)) as f32 * plot_w };
    let map_y = |y: f64| -> f32 { top + (1.0 - ((y - ymin) / (ymax - ymin)) as f32) * plot_h };

    // Horizontal grid lines and y labels, thinned to fit the plot height.
    let max_y = ((plot_h / (label_size * 2.2)) as usize).clamp(2, 6);
    for tick in nice_ticks(ymin, ymax, max_y) {
        let y = map_y(tick);
        items.push(CanvasItemKind::Line { x0: left, y0: y, x1: left + plot_w, y1: y, color: palette.border });
        let label = format_tick(tick);
        let tw = estimate_width(&label, label_size);
        items.push(CanvasItemKind::TextEx {
            x: left - 4.0 - tw,
            y: y - label_size * 0.65,
            text: label,
            color: palette.muted,
            size: label_size,
            weight: 400,
        });
    }

    let max_len = state.series.iter().map(|s| s.points.len()).max().unwrap_or(0);

    // X labels under the plot.
    if state.kind == ChartKind::Bar {
        // Category slots: one per point index, labelled from series 0.
        if max_len > 0 {
            let slot_w = plot_w / max_len as f32;
            let mut last_right = f32::NEG_INFINITY;
            let first_points = state.series.first().map(|s| &s.points);
            for k in 0..max_len {
                let Some(&(x, _)) = first_points.and_then(|p| p.get(k)) else { continue };
                if !x.is_finite() {
                    continue;
                }
                let label = if state.time_axis {
                    let step = if max_len > 1 {
                        ((xmax - xmin) / (max_len - 1) as f64).abs()
                    } else {
                        1.0
                    };
                    format_time(x, step)
                } else {
                    format_tick(x)
                };
                let tw = estimate_width(&label, label_size);
                let cx = left + (k as f32 + 0.5) * slot_w;
                let lx = cx - tw / 2.0;
                if lx < last_right + 40.0 {
                    continue;
                }
                last_right = lx + tw;
                items.push(CanvasItemKind::TextEx {
                    x: lx,
                    y: top + plot_h + 4.0,
                    text: label,
                    color: palette.muted,
                    size: label_size,
                    weight: 400,
                });
            }
        }
    } else {
        let x_labels: Vec<(f64, String)> = if state.time_axis {
            time_ticks(xmin, xmax, 6)
        } else {
            nice_ticks(xmin, xmax, 6).into_iter().map(|v| (v, format_tick(v))).collect()
        };
        for (v, label) in x_labels {
            let x = map_x(v);
            let tw = estimate_width(&label, label_size);
            items.push(CanvasItemKind::TextEx {
                x: x - tw / 2.0,
                y: top + plot_h + 4.0,
                text: label,
                color: palette.muted,
                size: label_size,
                weight: 400,
            });
        }
    }

    // Axis lines.
    items.push(CanvasItemKind::Line { x0: left, y0: top + plot_h, x1: left + plot_w, y1: top + plot_h, color: palette.muted });
    items.push(CanvasItemKind::Line { x0: left, y0: top, x1: left, y1: top + plot_h, color: palette.muted });

    let baseline_value = 0.0_f64.clamp(ymin, ymax);
    let baseline_y = map_y(baseline_value);

    for (i, series) in state.series.iter().enumerate() {
        let color = series.color.unwrap_or_else(|| series_color(i));
        match state.kind {
            ChartKind::Line | ChartKind::Area => {
                let mut points = Vec::new();
                for &(x, y) in &series.points {
                    if !x.is_finite() || !y.is_finite() {
                        continue;
                    }
                    points.push((map_x(x), map_y(y)));
                }
                if state.kind == ChartKind::Area && points.len() >= 2 {
                    let mut poly = Vec::with_capacity(points.len() + 2);
                    poly.push((points[0].0, baseline_y));
                    poly.extend(points.iter().copied());
                    poly.push((points[points.len() - 1].0, baseline_y));
                    items.push(CanvasItemKind::Path {
                        points: poly,
                        color: Color::rgba(color.r, color.g, color.b, 90),
                        width: 0.0,
                        filled: true,
                    });
                }
                if points.len() >= 2 {
                    items.push(CanvasItemKind::Path { points, color, width: 2.0, filled: false });
                }
            }
            ChartKind::Bar => {
                if max_len == 0 {
                    continue;
                }
                let n = state.series.len().max(1) as f32;
                let slot_w = plot_w / max_len as f32;
                let group_w = slot_w * 0.8;
                let bar_w = (group_w / n).max(0.5);
                for (k, &(x, y)) in series.points.iter().enumerate() {
                    if !x.is_finite() || !y.is_finite() {
                        continue;
                    }
                    let slot_left = left + k as f32 * slot_w;
                    let group_left = slot_left + (slot_w - group_w) / 2.0;
                    let x0 = group_left + i as f32 * bar_w;
                    let y = map_y(y);
                    let (y0, y1) = if y <= baseline_y { (y, baseline_y) } else { (baseline_y, y) };
                    items.push(CanvasItemKind::Rect { x0, y0, x1: x0 + bar_w, y1, color });
                }
            }
        }
    }

    items
}

