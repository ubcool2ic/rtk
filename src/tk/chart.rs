//! Chart widget APIs: series management, live append and axis configuration.

#[allow(unused_imports)]
use super::*;

impl Tk {
    /// Returns a mutable reference to the chart state of `id`, if it is a live chart.
    fn chart_state_mut(&mut self, id: WidgetId) -> Option<&mut crate::chart::ChartState> {
        let node = self.node_mut(id)?;
        if node.kind != Kind::Chart {
            return None;
        }
        node.chart.as_deref_mut()
    }

    /// Returns a shared reference to the chart state of `id`, if it is a live chart.
    fn chart_state(&self, id: WidgetId) -> Option<&crate::chart::ChartState> {
        let node = self.node(id)?;
        if node.kind != Kind::Chart {
            return None;
        }
        node.chart.as_deref()
    }

    /// Sets the visual form of a chart.
    pub fn chart_set_kind(&mut self, id: WidgetId, kind: crate::chart::ChartKind) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        state.kind = kind;
        self.dirty = true;
        true
    }

    /// Appends a new named series, returning its index.
    pub fn chart_add_series(&mut self, id: WidgetId, name: &str) -> Option<usize> {
        let state = self.chart_state_mut(id)?;
        state.series.push(crate::chart::SeriesData::new(name.to_string()));
        let index = state.series.len() - 1;
        self.dirty = true;
        Some(index)
    }

    /// Sets an explicit colour for a series, overriding the palette.
    pub fn chart_set_series_color(&mut self, id: WidgetId, series: usize, color: Color) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        let Some(s) = state.series.get_mut(series) else { return false };
        s.color = Some(color);
        self.dirty = true;
        true
    }

    /// Replaces the data of a series, keeping only the last `max_points`.
    pub fn chart_set_data(&mut self, id: WidgetId, series: usize, points: &[(f64, f64)]) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        let max_points = state.max_points.max(1);
        let Some(s) = state.series.get_mut(series) else { return false };
        s.points.clear();
        let start = points.len().saturating_sub(max_points);
        for &p in &points[start..] {
            s.points.push_back(p);
        }
        self.dirty = true;
        true
    }

    /// Appends a point to a series, dropping the oldest beyond `max_points`.
    pub fn chart_append(&mut self, id: WidgetId, series: usize, x: f64, y: f64) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        let max_points = state.max_points.max(1);
        let Some(s) = state.series.get_mut(series) else { return false };
        s.points.push_back((x, y));
        while s.points.len() > max_points {
            s.points.pop_front();
        }
        self.dirty = true;
        true
    }

    /// Sets the maximum retained points per series (at least 1) and trims existing data.
    pub fn chart_set_max_points(&mut self, id: WidgetId, n: usize) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        let n = n.max(1);
        state.max_points = n;
        for s in &mut state.series {
            while s.points.len() > n {
                s.points.pop_front();
            }
        }
        self.dirty = true;
        true
    }

    /// Enables or disables UTC time formatting on the x axis.
    pub fn chart_set_time_axis(&mut self, id: WidgetId, time_axis: bool) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        state.time_axis = time_axis;
        self.dirty = true;
        true
    }

    /// Shows or hides the legend row.
    pub fn chart_set_legend(&mut self, id: WidgetId, legend: bool) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        state.legend = legend;
        self.dirty = true;
        true
    }

    /// Sets an explicit y range, or `None` to derive it from the data.
    ///
    /// Rejects non-finite bounds or `min >= max`.
    pub fn chart_set_y_range(&mut self, id: WidgetId, range: Option<(f64, f64)>) -> bool {
        if let Some((lo, hi)) = range {
            if !lo.is_finite() || !hi.is_finite() || lo >= hi {
                return false;
            }
        }
        let Some(state) = self.chart_state_mut(id) else { return false };
        state.y_range = range;
        self.dirty = true;
        true
    }

    /// Clears the data of a series.
    pub fn chart_clear(&mut self, id: WidgetId, series: usize) -> bool {
        let Some(state) = self.chart_state_mut(id) else { return false };
        let Some(s) = state.series.get_mut(series) else { return false };
        s.points.clear();
        self.dirty = true;
        true
    }

    /// Number of series in a chart.
    pub fn chart_series_count(&self, id: WidgetId) -> Option<usize> {
        self.chart_state(id).map(|s| s.series.len())
    }

    /// Number of points in a series.
    pub fn chart_series_len(&self, id: WidgetId, series: usize) -> Option<usize> {
        self.chart_state(id).and_then(|s| s.series.get(series)).map(|s| s.points.len())
    }

    /// Returns the point at `index` in a series.
    pub fn chart_point(&self, id: WidgetId, series: usize, index: usize) -> Option<(f64, f64)> {
        self.chart_state(id)
            .and_then(|s| s.series.get(series))
            .and_then(|s| s.points.get(index).copied())
    }

    /// Computes the data bounds `(xmin, xmax, ymin, ymax)` of a chart.
    pub fn chart_bounds(&self, id: WidgetId) -> Option<(f64, f64, f64, f64)> {
        self.chart_state(id).and_then(|s| s.bounds())
    }
}

