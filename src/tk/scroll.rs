//! Scroll metrics, view fractions and scrollbar linking.

#[allow(unused_imports)]
use super::*;

use crate::tk::inputs_state::Orient;

impl Tk {
    /// Mouse wheel in lines (positive = up/away). Listboxes scroll by rows, canvases by 48 px per line.
    #[doc(hidden)]
    pub fn sim_wheel(&mut self, dy_lines: f32) {
        self.sim_wheel_xy(0.0, dy_lines, false);
    }

    /// Mouse wheel with Shift held: scrolls horizontally (positive = left).
    #[doc(hidden)]
    pub fn sim_wheel_shift(&mut self, dy_lines: f32) {
        self.sim_wheel_xy(dy_lines, 0.0, false);
    }

    /// Wheel/touchpad scroll. `pixels` = true for precise touchpad deltas (logical px), else lines.
    /// Positive values scroll up/left (content moves down/right), as winit reports them.
    #[doc(hidden)]
    pub fn sim_wheel_xy(&mut self, dx: f32, dy: f32, pixels: bool) {
        self.ensure_layout();
        let (x, y) = self.pointer;
        let Some(mut id) = self.hit_test(x, y) else { return };
        // A Scrollbar forwards wheel events to its target.
        if let Some(t) = self.node(id).filter(|n| n.kind == Kind::Scrollbar).and_then(|n| n.scroll_target) {
            id = t;
        }
        let Some(node) = self.node(id) else { return };
        // A widget inside a ScrollArea scrolls the nearest area unless it
        // consumes the wheel itself (Listbox, Canvas, Text, Slider, ...).
        if !matches!(node.kind, Kind::Listbox | Kind::Canvas | Kind::Text | Kind::Slider | Kind::DoubleSpinbox | Kind::Combobox) {
            if let Some(area) = self.nearest_scroll_area(id) {
                let k = if pixels { 1.0 } else { WHEEL_LINE_PX };
                self.scrollarea_scroll_by(area, -dx * k, -dy * k);
                return;
            }
        }
        match node.kind {
            Kind::Listbox => {
                let rows = if pixels { dy / 20.0 } else { dy };
                if let Some((_, _, pos)) = self.scroll_metrics(id) {
                    self.set_scroll_pos(id, pos - rows.round());
                }
            }
            Kind::Canvas if node.scrollregion.is_some() => {
                let k = if pixels { 1.0 } else { WHEEL_LINE_PX };
                let (vx, vy) = node.view;
                self.canvas_set_view(id, vx - dx * k, vy - dy * k);
            }
            Kind::ScrollArea => {
                let k = if pixels { 1.0 } else { WHEEL_LINE_PX };
                self.scrollarea_scroll_by(id, -dx * k, -dy * k);
            }
            Kind::Text => {
                let lines = if pixels { dy / 16.0 } else { dy };
                if let Some((_, _, pos)) = self.scroll_metrics(id) {
                    self.set_scroll_pos(id, pos - lines.round());
                }
            }
            Kind::Slider => {
                let step = node.inputs.as_ref().map_or(0.0, |s| s.slider_step);
                let (from, to) = node.range_f64;
                let unit = if step > 0.0 { step } else { (to - from).abs() * 0.01 };
                let current = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(from);
                let direction = if dy > 0.0 { 1.0 } else { -1.0 };
                self.slider_user_set(id, current + direction * unit);
            }
            Kind::DoubleSpinbox => {
                let direction = if dy > 0.0 { 1 } else { -1 };
                self.dspin_step(id, direction);
            }
            Kind::Combobox if !node.combo_open => {
                let old = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
                let max_idx = node.combo_values.len().saturating_sub(1) as i64;
                let step = if dy > 0.0 { -1 } else { 1 };
                let new_val = (old + step).clamp(0, max_idx);
                if let Some(var) = node.combo_var {
                    var.set(self, new_val);
                }
                self.dirty = true;
            }
            _ => {}
        }
    }

    /// Scroll state of a Listbox (rows) or scrollable Canvas (px, vertical): (total, visible, position).
    /// None when nothing overflows.
    pub(crate) fn scroll_metrics(&self, id: WidgetId) -> Option<(f32, f32, f32)> {
        let n = self.node(id)?;
        let (total, visible, pos) = match n.kind {
            Kind::Listbox => (n.list_items.len() as f32, n.list_height.max(1) as f32, n.list_scroll as f32),
            Kind::Canvas => (n.scrollregion?.1, n.rect.height, n.view.1),
            Kind::Text => (n.text.split('\n').count() as f32, n.text_dimensions.1.max(1) as f32, n.text_scroll_row as f32),
            Kind::ScrollArea => {
                let content = n.inputs.as_ref().and_then(|s| s.scroll_area_content)?;
                let (ox, oy) = n.inputs.as_ref().map_or((0.0, 0.0), |s| s.scroll_area_offset);
                let _ = ox;
                let ch = self.node(content).map(|c| c.rect.height).unwrap_or(n.rect.height);
                (ch, n.rect.height, oy)
            }
            _ => return None,
        };
        (total > visible && visible > 0.0).then_some((total, visible, pos))
    }

    /// Horizontal scroll state of a Canvas (px) or ScrollArea: (total, visible, position).
    /// None when nothing overflows.
    pub(crate) fn scroll_metrics_x(&self, id: WidgetId) -> Option<(f32, f32, f32)> {
        let n = self.node(id)?;
        let (total, visible, pos) = match n.kind {
            Kind::Canvas => (n.scrollregion?.0, n.rect.width, n.view.0),
            Kind::ScrollArea => {
                let content = n.inputs.as_ref().and_then(|s| s.scroll_area_content)?;
                let (ox, _) = n.inputs.as_ref().map_or((0.0, 0.0), |s| s.scroll_area_offset);
                let cw = self.node(content).map(|c| c.rect.width).unwrap_or(n.rect.width);
                (cw, n.rect.width, ox)
            }
            _ => return None,
        };
        (total > visible && visible > 0.0).then_some((total, visible, pos))
    }

    pub(crate) fn set_scroll_pos(&mut self, id: WidgetId, pos: f32) {
        let Some(kind) = self.node(id).map(|n| n.kind) else { return };
        match kind {
            Kind::Listbox => {
                if let Some((total, visible, _)) = self.scroll_metrics(id) {
                    let v = pos.round().clamp(0.0, total - visible) as usize;
                    if let Some(n) = self.node_mut(id) { n.list_scroll = v; }
                    self.dirty = true;
                }
            }
            Kind::Canvas => {
                let vx = self.node(id).map(|n| n.view.0).unwrap_or(0.0);
                self.canvas_set_view(id, vx, pos);
            }
            Kind::Text => {
                if let Some((total, visible, _)) = self.scroll_metrics(id) {
                    let v = pos.round().clamp(0.0, total - visible) as usize;
                    if let Some(n) = self.node_mut(id) { n.text_scroll_row = v; }
                    self.dirty = true;
                }
            }
            Kind::ScrollArea => {
                let (x, _) = self.scrollarea_offset(id).unwrap_or((0.0, 0.0));
                self.scrollarea_scroll_to(id, x, pos);
            }
            _ => {}
        }
    }

    /// Sets the horizontal scroll position of a Canvas or ScrollArea.
    pub(crate) fn set_scroll_pos_x(&mut self, id: WidgetId, pos: f32) {
        let Some(kind) = self.node(id).map(|n| n.kind) else { return };
        match kind {
            Kind::Canvas => {
                let vy = self.node(id).map(|n| n.view.1).unwrap_or(0.0);
                self.canvas_set_view(id, pos, vy);
            }
            Kind::ScrollArea => {
                let (_, y) = self.scrollarea_offset(id).unwrap_or((0.0, 0.0));
                self.scrollarea_scroll_to(id, pos, y);
            }
            _ => {}
        }
    }

    pub(crate) fn canvas_set_view(&mut self, id: WidgetId, x: f32, y: f32) {
        let Some(n) = self.node(id) else { return };
        let (rw, rh) = n.scrollregion.unwrap_or((n.rect.width, n.rect.height));
        let nx = x.clamp(0.0, (rw - n.rect.width).max(0.0));
        let ny = y.clamp(0.0, (rh - n.rect.height).max(0.0));
        if (nx, ny) != n.view {
            if let Some(n) = self.node_mut(id) { n.view = (nx, ny); }
            self.dirty = true;
        }
    }



    pub(crate) fn view_fractions(&mut self, id: WidgetId, vertical: bool) -> (f32, f32) {
        self.ensure_layout();
        let Some(n) = self.node(id) else { return (0.0, 1.0) };
        let (total, visible, pos) = match (n.kind, vertical) {
            (Kind::Listbox, true) => (n.list_items.len() as f32, n.list_height.max(1) as f32, n.list_scroll as f32),
            (Kind::Text, true) => (n.text.split('\n').count() as f32, n.text_dimensions.1.max(1) as f32, n.text_scroll_row as f32),
            (Kind::Canvas, true) => (n.scrollregion.map_or(n.rect.height, |r| r.1), n.rect.height, n.view.1),
            (Kind::Canvas, false) => (n.scrollregion.map_or(n.rect.width, |r| r.0), n.rect.width, n.view.0),
            (Kind::ScrollArea, true) => {
                let content = n.inputs.as_ref().and_then(|s| s.scroll_area_content);
                let (_, oy) = n.inputs.as_ref().map_or((0.0, 0.0), |s| s.scroll_area_offset);
                let ch = content.and_then(|c| self.node(c)).map(|c| c.rect.height).unwrap_or(n.rect.height);
                (ch, n.rect.height, oy)
            }
            (Kind::ScrollArea, false) => {
                let content = n.inputs.as_ref().and_then(|s| s.scroll_area_content);
                let (ox, _) = n.inputs.as_ref().map_or((0.0, 0.0), |s| s.scroll_area_offset);
                let cw = content.and_then(|c| self.node(c)).map(|c| c.rect.width).unwrap_or(n.rect.width);
                (cw, n.rect.width, ox)
            }
            _ => return (0.0, 1.0),
        };
        if total <= 0.0 || visible >= total { return (0.0, 1.0); }
        ((pos / total).clamp(0.0, 1.0), ((pos + visible) / total).clamp(0.0, 1.0))
    }

    /// tkinter `yview()`: visible fraction (first, last) of a Listbox, Text or Canvas.
    pub fn yview(&mut self, id: WidgetId) -> (f32, f32) { self.view_fractions(id, true) }
    /// tkinter `xview()` for a Canvas.
    pub fn xview(&mut self, id: WidgetId) -> (f32, f32) { self.view_fractions(id, false) }

    /// tkinter `yview_moveto(fraction)`: scrolls so `fraction` of the content is above the view.
    pub fn yview_moveto(&mut self, id: WidgetId, fraction: f32) {
        self.ensure_layout();
        let Some(n) = self.node(id) else { return };
        match n.kind {
            Kind::Listbox => { let t = n.list_items.len() as f32; self.set_scroll_pos(id, fraction * t); }
            Kind::Text => { let t = n.text.split('\n').count() as f32; self.set_scroll_pos(id, fraction * t); }
            Kind::Canvas => { let (vx, t) = (n.view.0, n.scrollregion.map_or(0.0, |r| r.1)); self.canvas_set_view(id, vx, fraction * t); }
            Kind::ScrollArea => {
                let (x, _) = self.scrollarea_offset(id).unwrap_or((0.0, 0.0));
                let content = n.inputs.as_ref().and_then(|s| s.scroll_area_content);
                let ch = content.and_then(|c| self.node(c)).map(|c| c.rect.height).unwrap_or(0.0);
                self.scrollarea_scroll_to(id, x, fraction * ch);
            }
            _ => {}
        }
    }

    /// tkinter `xview_moveto(fraction)` for a Canvas.
    pub fn xview_moveto(&mut self, id: WidgetId, fraction: f32) {
        self.ensure_layout();
        let Some(n) = self.node(id) else { return };
        match n.kind {
            Kind::Canvas => {
                let (t, vy) = (n.scrollregion.map_or(0.0, |r| r.0), n.view.1);
                self.canvas_set_view(id, fraction * t, vy);
            }
            Kind::ScrollArea => {
                let (_, y) = self.scrollarea_offset(id).unwrap_or((0.0, 0.0));
                let content = n.inputs.as_ref().and_then(|s| s.scroll_area_content);
                let cw = content.and_then(|c| self.node(c)).map(|c| c.rect.width).unwrap_or(0.0);
                self.scrollarea_scroll_to(id, fraction * cw, y);
            }
            _ => {}
        }
    }

    /// tkinter `yview_scroll(n, "units"|"pages")`. Units are rows (Listbox, Text) or 48 px (Canvas).
    pub fn yview_scroll(&mut self, id: WidgetId, n: i32, what: ScrollUnit) {
        self.ensure_layout();
        let Some(node) = self.node(id) else { return };
        let (unit, page, pos) = match node.kind {
            Kind::Listbox => (1.0, node.list_height.max(1) as f32, node.list_scroll as f32),
            Kind::Text => (1.0, node.text_dimensions.1.max(1) as f32, node.text_scroll_row as f32),
            Kind::Canvas => (WHEEL_LINE_PX, (node.rect.height - WHEEL_LINE_PX).max(WHEEL_LINE_PX), node.view.1),
            Kind::ScrollArea => {
                let (_, oy) = node.inputs.as_ref().map_or((0.0, 0.0), |s| s.scroll_area_offset);
                (WHEEL_LINE_PX, (node.rect.height - WHEEL_LINE_PX).max(WHEEL_LINE_PX), oy)
            }
            _ => return,
        };
        let step = n as f32 * if what == ScrollUnit::Pages { page } else { unit };
        if node.kind == Kind::Canvas {
            let vx = node.view.0;
            self.canvas_set_view(id, vx, pos + step);
        } else {
            self.set_scroll_pos(id, pos + step);
        }
    }

    /// tkinter `xview_scroll(n, "units"|"pages")` for a Canvas.
    pub fn xview_scroll(&mut self, id: WidgetId, n: i32, what: ScrollUnit) {
        self.ensure_layout();
        let Some(node) = self.node(id) else { return };
        let step = n as f32 * if what == ScrollUnit::Pages { (node.rect.width - WHEEL_LINE_PX).max(WHEEL_LINE_PX) } else { WHEEL_LINE_PX };
        match node.kind {
            Kind::Canvas => {
                let (vx, vy) = node.view;
                self.canvas_set_view(id, vx + step, vy);
            }
            Kind::ScrollArea => {
                let (ox, oy) = self.scrollarea_offset(id).unwrap_or((0.0, 0.0));
                self.scrollarea_scroll_to(id, ox + step, oy);
            }
            _ => {}
        }
    }


    // Scrollbar methods
    /// Returns the orientation of a Scrollbar widget, or `None` if `id` is not
    /// a live Scrollbar. A scrollbar without input state is vertical.
    pub fn scrollbar_orient(&self, id: WidgetId) -> Option<Orient> {
        let node = self.node(id).filter(|n| n.kind == Kind::Scrollbar)?;
        let horizontal = node
            .inputs
            .as_ref()
            .is_some_and(|s| s.scrollbar_horizontal);
        Some(if horizontal { Orient::Horizontal } else { Orient::Vertical })
    }

    /// Links a vertical Scrollbar to a Listbox, scrollable Canvas, or Text widget.
    pub fn link_scrollbar(&mut self, scrollbar: WidgetId, listbox: WidgetId) -> bool {
        let is_sb = self.node(scrollbar).is_some_and(|n| n.kind == Kind::Scrollbar);
        let is_lb = self.node(listbox).is_some_and(|n| matches!(n.kind, Kind::Listbox | Kind::Canvas | Kind::Text | Kind::ScrollArea));
        if !is_sb || !is_lb {
            return false;
        }
        if let Some(node) = self.node_mut(scrollbar) {
            node.scroll_target = Some(listbox);
        }
        self.dirty = true;
        true
    }

}
