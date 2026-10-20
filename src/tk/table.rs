//! Table / DataGrid widget: virtualized rows, sortable and resizable columns.
//!
//! All table state lives in [`TableState`], stored on the widget node as
//! `table: Option<Box<TableState>>`. Rendering and hit-testing only touch the
//! visible window of rows (`top..top+visible+1`), so 10,000-row tables stay
//! smooth. Sorting permutes an index vector (`order`) and never moves row data.

use std::cmp::Ordering;

use super::*;

/// Callback fired when the selected row of a Table changes.
pub(crate) type TableSelectCallback = Box<dyn FnMut(&mut Tk, Option<usize>)>;

/// Mutable state of a Table widget.
pub(crate) struct TableState {
    /// Column titles.
    pub(crate) columns: Vec<String>,
    /// Column widths in logical px.
    pub(crate) widths: Vec<f32>,
    /// Row data in model order.
    pub(crate) rows: Vec<Vec<String>>,
    /// Display -> model permutation.
    pub(crate) order: Vec<usize>,
    /// Active sort: (column, ascending).
    pub(crate) sort: Option<(usize, bool)>,
    /// Selected model row.
    pub(crate) selected: Option<usize>,
    /// Display row shown at the top of the body.
    pub(crate) top: usize,
    /// Active column-resize drag: (column, start x, start width).
    pub(crate) resize_drag: Option<(usize, f32, f32)>,
    /// Selection-change callback.
    pub(crate) on_select: Option<TableSelectCallback>,
    /// Re-entrancy guard for the selection callback.
    pub(crate) on_select_running: bool,
}

impl TableState {
    /// Creates an empty table with the given column titles and default widths.
    pub(crate) fn new(columns: &[&str]) -> Self {
        Self {
            columns: columns.iter().map(|c| (*c).to_string()).collect(),
            widths: vec![120.0; columns.len()],
            rows: Vec::new(),
            order: Vec::new(),
            sort: None,
            selected: None,
            top: 0,
            resize_drag: None,
            on_select: None,
            on_select_running: false,
        }
    }

    /// Rebuilds the display permutation, applying the active sort if any.
    pub(crate) fn rebuild_order(&mut self) {
        self.order = (0..self.rows.len()).collect();
        if let Some((col, asc)) = self.sort {
            self.apply_sort(col, asc);
        }
    }

    /// Stable-sorts the permutation by column `col`; precomputes keys once.
    pub(crate) fn apply_sort(&mut self, col: usize, asc: bool) {
        if col >= self.columns.len() {
            return;
        }
        // Precompute comparison keys once per sort (no per-compare allocation).
        let keys: Vec<(f64, String)> = self
            .rows
            .iter()
            .map(|row| {
                let cell = row.get(col).map(String::as_str).unwrap_or("");
                let num = cell.trim().parse::<f64>().ok();
                (num.unwrap_or(f64::NAN), cell.to_lowercase())
            })
            .collect();
        let order = &mut self.order;
        order.sort_by(|&a, &b| {
            let (na, sa) = &keys[a];
            let (nb, sb) = &keys[b];
            let ord = match (na.is_nan(), nb.is_nan()) {
                (false, false) => na.partial_cmp(nb).unwrap_or(Ordering::Equal),
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                (true, true) => sa.cmp(sb),
            };
            if asc { ord } else { ord.reverse() }
        });
    }

    /// Model row shown at display position `display`.
    pub(crate) fn model_at(&self, display: usize) -> Option<usize> {
        self.order.get(display).copied()
    }

    /// Display position of model row `model`.
    pub(crate) fn display_of(&self, model: usize) -> Option<usize> {
        self.order.iter().position(|&m| m == model)
    }
}

impl Tk {
    /// Returns a shared reference to the table state of `id`, if it is a Table.
    pub(crate) fn table_state(&self, id: WidgetId) -> Option<&TableState> {
        self.node(id).filter(|n| n.kind == Kind::Table)?.table.as_deref()
    }

    /// Returns a mutable reference to the table state of `id`, if it is a Table.
    pub(crate) fn table_state_mut(&mut self, id: WidgetId) -> Option<&mut TableState> {
        self.node_mut(id).filter(|n| n.kind == Kind::Table)?.table.as_deref_mut()
    }

    /// Row height in logical px derived from the resolved body font size.
    pub(crate) fn table_row_height_for(&self, id: WidgetId) -> f32 {
        let size = self.resolved_style(id).map(|s| s.font_size).unwrap_or(14.0);
        (size * 1.6).round().max(8.0)
    }

    /// Header height in logical px (same as the row height).
    pub(crate) fn table_header_height(&self, id: WidgetId) -> f32 {
        self.table_row_height_for(id)
    }

    /// Number of fully visible body rows for the current widget rect.
    pub(crate) fn table_visible_rows(&self, id: WidgetId) -> usize {
        let Some(node) = self.node(id) else { return 0 };
        let row_h = self.table_row_height_for(id);
        let header = self.table_header_height(id);
        let body = (node.rect.height - header).max(0.0);
        if row_h <= 0.0 {
            return 0;
        }
        (body / row_h).floor() as usize
    }

    /// Clamps `top` so the view stays within the row count.
    pub(crate) fn table_clamp_top(&mut self, id: WidgetId) {
        let visible = self.table_visible_rows(id);
        let total = self.table_state(id).map(|s| s.order.len()).unwrap_or(0);
        let max_top = total.saturating_sub(visible.max(1));
        if let Some(state) = self.table_state_mut(id) {
            state.top = state.top.min(max_top);
        }
    }

    /// Replaces all rows, re-applying the active sort and clamping the view.
    pub fn table_set_rows(&mut self, id: WidgetId, rows: Vec<Vec<String>>) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        state.rows = rows;
        state.rebuild_order();
        if let Some(sel) = state.selected {
            if sel >= state.rows.len() {
                state.selected = None;
            }
        }
        self.table_clamp_top(id);
        self.dirty = true;
        true
    }

    /// Inserts a row at model `index` (appends when out of range), keeping the sort.
    pub fn table_insert_row(&mut self, id: WidgetId, index: usize, row: Vec<String>) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        let at = index.min(state.rows.len());
        state.rows.insert(at, row);
        // Selection tracks the model row it pointed at.
        if let Some(sel) = state.selected {
            if at <= sel {
                state.selected = Some(sel + 1);
            }
        }
        state.rebuild_order();
        self.table_clamp_top(id);
        self.dirty = true;
        true
    }

    /// Removes every row and clears the selection and sort.
    pub fn table_clear(&mut self, id: WidgetId) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        state.rows.clear();
        state.order.clear();
        state.selected = None;
        state.top = 0;
        self.dirty = true;
        true
    }

    /// Number of model rows.
    pub fn table_row_count(&self, id: WidgetId) -> usize {
        self.table_state(id).map(|s| s.rows.len()).unwrap_or(0)
    }

    /// Cell text at model `(row, col)`, if present.
    pub fn table_cell(&self, id: WidgetId, row: usize, col: usize) -> Option<String> {
        self.table_state(id)?.rows.get(row)?.get(col).cloned()
    }

    /// Number of columns.
    pub fn table_column_count(&self, id: WidgetId) -> usize {
        self.table_state(id).map(|s| s.columns.len()).unwrap_or(0)
    }

    /// Sets the width of column `col` in logical px (minimum 24).
    pub fn table_set_column_width(&mut self, id: WidgetId, col: usize, width: f32) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        let Some(w) = state.widths.get_mut(col) else { return false };
        *w = width.max(24.0);
        self.dirty = true;
        true
    }

    /// Width of column `col` in logical px.
    pub fn table_column_width(&self, id: WidgetId, col: usize) -> Option<f32> {
        self.table_state(id)?.widths.get(col).copied()
    }

    /// Sorts by column `col` (stable, numeric when both cells parse as f64).
    pub fn table_sort_by(&mut self, id: WidgetId, col: usize, ascending: bool) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        if col >= state.columns.len() {
            return false;
        }
        state.sort = Some((col, ascending));
        state.rebuild_order();
        self.table_clamp_top(id);
        self.dirty = true;
        true
    }

    /// Active sort as `(column, ascending)`.
    pub fn table_sort_state(&self, id: WidgetId) -> Option<(usize, bool)> {
        self.table_state(id)?.sort
    }

    /// Clears the sort, restoring model order.
    pub fn table_clear_sort(&mut self, id: WidgetId) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        state.sort = None;
        state.rebuild_order();
        self.table_clamp_top(id);
        self.dirty = true;
        true
    }

    /// Maps a display row to its model row.
    pub fn table_display_to_model(&self, id: WidgetId, display_row: usize) -> Option<usize> {
        self.table_state(id)?.model_at(display_row)
    }

    /// Selected model row, if any.
    pub fn table_selection(&self, id: WidgetId) -> Option<usize> {
        self.table_state(id)?.selected
    }

    /// Selects model row `row` (or clears with `None`), scrolling it into view.
    pub fn table_select(&mut self, id: WidgetId, row: Option<usize>) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        if let Some(r) = row {
            if r >= state.rows.len() {
                return false;
            }
        }
        let old = state.selected;
        state.selected = row;
        if let Some(r) = row {
            self.table_see(id, r);
        }
        self.dirty = true;
        if old != row {
            self.dispatch_table_select(id, row);
        }
        true
    }

    /// Registers a callback fired when the selected row changes.
    pub fn table_on_select(
        &mut self,
        id: WidgetId,
        cb: impl FnMut(&mut Tk, Option<usize>) + 'static,
    ) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        state.on_select = Some(Box::new(cb));
        true
    }

    /// Display row currently shown at the top of the body.
    pub fn table_first_visible(&self, id: WidgetId) -> usize {
        self.table_state(id).map(|s| s.top).unwrap_or(0)
    }

    /// Scrolls so model row `row` is visible.
    pub fn table_see(&mut self, id: WidgetId, row: usize) -> bool {
        let Some(state) = self.table_state(id) else { return false };
        let _ = state;
        let Some(display) = state.display_of(row) else { return false };
        let visible = self.table_visible_rows(id).max(1);
        let top = state.top;
        let new_top = if display < top {
            display
        } else if display >= top + visible {
            display + 1 - visible
        } else {
            top
        };
        if let Some(state) = self.table_state_mut(id) {
            state.top = new_top;
        }
        self.table_clamp_top(id);
        self.dirty = true;
        true
    }

    /// Row height in logical px.
    pub fn table_row_height(&self, id: WidgetId) -> f32 {
        self.table_row_height_for(id)
    }

    /// Fires the selection callback (re-entrancy guarded).
    pub(crate) fn dispatch_table_select(&mut self, id: WidgetId, sel: Option<usize>) {
        let Some(state) = self.table_state_mut(id) else { return };
        if state.on_select_running {
            return;
        }
        let Some(mut cb) = state.on_select.take() else { return };
        state.on_select_running = true;
        cb(self, sel);
        if let Some(state) = self.table_state_mut(id) {
            state.on_select_running = false;
            if state.on_select.is_none() {
                state.on_select = Some(cb);
            }
        }
    }

    /// Column index whose header cell contains logical x `x` (relative to widget).
    pub(crate) fn table_column_at(&self, id: WidgetId, x: f32) -> Option<usize> {
        let state = self.table_state(id)?;
        let mut cur = 0.0_f32;
        for (idx, w) in state.widths.iter().enumerate() {
            if x >= cur && x < cur + *w {
                return Some(idx);
            }
            cur += *w;
        }
        None
    }

    /// Handles a mouse press at widget-local `(x, y)`. Returns true if consumed.
    pub(crate) fn table_mouse_down(&mut self, id: WidgetId, x: f32, y: f32) -> bool {
        let Some(state) = self.table_state(id) else { return false };
        let header = self.table_header_height(id);
        let row_h = self.table_row_height_for(id);
        if y < header {
            // Header: resize handle or sort toggle.
            let mut cur = 0.0_f32;
            for (idx, w) in state.widths.iter().enumerate() {
                let right = cur + *w;
                if (x - right).abs() <= 4.0 {
                    let start_w = *w;
                    if let Some(state) = self.table_state_mut(id) {
                        state.resize_drag = Some((idx, x, start_w));
                    }
                    return true;
                }
                cur = right;
            }
            if let Some(col) = self.table_column_at(id, x) {
                let current = self.table_state(id).and_then(|s| s.sort);
                let asc = !matches!(current, Some((c, true)) if c == col);
                self.table_sort_by(id, col, asc);
            }
            return true;
        }
        // Body: select the clicked row.
        let display = state.top + ((y - header) / row_h).floor().max(0.0) as usize;
        let model = self.table_state(id).and_then(|s| s.model_at(display));
        if let Some(model) = model {
            self.table_select(id, Some(model));
        }
        true
    }

    /// Handles a mouse move at widget-local `(x, _y)` during a resize drag.
    pub(crate) fn table_mouse_move(&mut self, id: WidgetId, x: f32) -> bool {
        let Some(state) = self.table_state(id) else { return false };
        let Some((col, start_x, start_w)) = state.resize_drag else { return false };
        let new_w = (start_w + (x - start_x)).max(24.0);
        if let Some(state) = self.table_state_mut(id) {
            if let Some(w) = state.widths.get_mut(col) {
                *w = new_w;
            }
        }
        self.dirty = true;
        true
    }

    /// Ends a column-resize drag, if one is active. Returns true if it ended one.
    pub(crate) fn table_mouse_up(&mut self, id: WidgetId) -> bool {
        let Some(state) = self.table_state_mut(id) else { return false };
        state.resize_drag.take().is_some()
    }

    /// Scrolls the table by `lines` rows (positive scrolls down).
    pub(crate) fn table_scroll(&mut self, id: WidgetId, lines: f32) {
        let Some(state) = self.table_state_mut(id) else { return };
        let delta = lines.round() as i64;
        let new_top = (state.top as i64 + delta).max(0) as usize;
        state.top = new_top;
        self.table_clamp_top(id);
        self.dirty = true;
    }

    /// Handles a key press on a focused table. Returns true if consumed.
    pub(crate) fn table_key(&mut self, id: WidgetId, key: Key) -> bool {
        let Some(state) = self.table_state(id) else { return false };
        let total = state.rows.len();
        if total == 0 {
            return false;
        }
        let visible = self.table_visible_rows(id).max(1);
        let cur_display = state
            .selected
            .and_then(|m| state.display_of(m))
            .unwrap_or(0);
        let new_display = match key {
            Key::Up => cur_display.saturating_sub(1),
            Key::Down => (cur_display + 1).min(total - 1),
            Key::PageUp => cur_display.saturating_sub(visible),
            Key::PageDown => (cur_display + visible).min(total - 1),
            Key::Home => 0,
            Key::End => total - 1,
            _ => return false,
        };
        let model = self.table_state(id).and_then(|s| s.model_at(new_display));
        if let Some(model) = model {
            self.table_select(id, Some(model));
        }
        true
    }
}
