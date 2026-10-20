//! Headless input simulation, hit testing, focus traversal and activation.

#[allow(unused_imports)]
use super::*;

/// Largest char boundary `<= index` in `s` (stable replacement for the
/// unstable `str::floor_char_boundary`). Prevents mid-UTF-8 slicing (review F2).
pub(crate) fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut i = index;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

impl Tk {
    /// Headless harness: sets the DPI scale factor (0.1..=16). Returns false if rejected.
    #[doc(hidden)]
    pub fn sim_scale_factor(&mut self, factor: f64) -> bool {
        if !factor.is_finite() || !(0.1..=16.0).contains(&factor) {
            return false;
        }
        self.scale_factor = factor as f32;
        self.dirty = true;
        self.layout();
        true
    }

    /// Returns the frontmost interactive widget at a logical pointer position.
    pub(crate) fn hit_test(&self, x: f32, y: f32) -> Option<WidgetId> {
        self.paint_order.iter().rev().copied().find(|id| {
            if let Some(modal_id) = self.modal {
                if !self.is_descendant_of(*id, modal_id) {
                    return false;
                }
            }
            self.node(*id).is_some_and(|node| {
                let interactive = node.kind.focusable()
                    || matches!(node.kind, Kind::Canvas | Kind::Scrollbar | Kind::Notebook | Kind::ScrollArea)
                    || node.tooltip.is_some();
                node.visible && interactive && node.clip.contains(x, y)
            })
        })
    }


    /// Headless harness: moves the pointer to logical `(x, y)`, updating hover,
    /// drags and tooltips.
    #[doc(hidden)]
    pub fn sim_mouse_move(&mut self, x: f32, y: f32) {
        self.ensure_layout();
        let moved = (self.pointer.0 - x).abs() > 2.0 || (self.pointer.1 - y).abs() > 2.0;
        self.pointer = (x, y);
        if self.menu_pointer_move(x, y) {
            return;
        }
        if let Some(pid) = self.pressed {
            if let Some(rx) = self.node(pid).filter(|n| n.kind == Kind::Table).map(|n| n.rect.x) {
                self.table_mouse_move(pid, x - rx);
            }
        }
        let hovered = self.hit_test(x, y);
        if self.hovered != hovered {
            self.hover_timer_ms = 0;
            self.tooltip_visible = false;
            let prev = self.hovered;
            self.hovered = hovered;
            if let Some(prev_id) = prev {
                // It was hovered until a moment ago.
                self.check_state_transition(prev_id, self.widget_state(prev_id) | State::HOVER);
                self.invalidate_widget(prev_id);
            }
            if let Some(new_id) = hovered {
                self.check_state_transition(new_id, self.widget_state(new_id).without(State::HOVER));
                self.invalidate_widget(new_id);
            }
            // Hover changes only repaint the affected widgets; a full redraw is
            // requested by `invalidate_widget` itself for overlay widgets.
        } else if moved && !self.tooltip_visible {
            self.hover_timer_ms = 0;
        }

        if let Some(canvas_id) = self.drag_active {
            if let Some(node) = self.node(canvas_id) {
                let cx = x - node.rect.x + node.view.0;
                let cy = y - node.rect.y + node.view.1;
                self.dispatch_canvas_drag(canvas_id, cx, cy, DragPhase::Move);
            }
        }

        if let Some((scrollbar_id, start_my, _)) = self.scrollbar_drag {
            let horizontal = self
                .node(scrollbar_id)
                .and_then(|n| n.inputs.as_ref())
                .is_some_and(|s| s.scrollbar_horizontal);
            if horizontal {
                let bar_w = self.node(scrollbar_id).map(|n| n.rect.width).unwrap_or(0.0);
                if let Some(target_id) = self.node(scrollbar_id).and_then(|n| n.scroll_target) {
                    if let Some((total, visible, _)) = self.scroll_metrics_x(target_id) {
                        let thumb_w = (bar_w * (visible / total)).max(16.0).min(bar_w);
                        let avail = bar_w - thumb_w;
                        if avail > 0.0 {
                            let start = self.scrollbar_drag_start;
                            let pos = start + (x - start_my) / avail * (total - visible);
                            self.set_scroll_pos_x(target_id, pos);
                        }
                    }
                }
            } else {
                let bar_h = self.node(scrollbar_id).map(|n| n.rect.height).unwrap_or(0.0);
                if let Some(target_id) = self.node(scrollbar_id).and_then(|n| n.scroll_target) {
                    if let Some((total, visible, _)) = self.scroll_metrics(target_id) {
                        let thumb_h = (bar_h * (visible / total)).max(16.0).min(bar_h);
                        let avail = bar_h - thumb_h;
                        if avail > 0.0 {
                            let start = self.scrollbar_drag_start;
                            let pos = start + (y - start_my) / avail * (total - visible);
                            self.set_scroll_pos(target_id, pos);
                        }
                    }
                }
            }
        }

        if let Some(id) = self.pressed {
            if self.node(id).is_some_and(|node| node.kind == Kind::Scale && !node.disabled) {
                self.drag_scale(id, x);
            } else if self.node(id).is_some_and(|node| node.kind == Kind::Slider && !node.disabled) {
                if let Some(value) = self.slider_value_at(id, x, y) {
                    self.slider_user_set(id, value);
                }
            } else if self.node(id).is_some_and(|node| {
                (matches!(node.kind, Kind::Entry | Kind::Text)
                    || (node.kind == Kind::Combobox && node.combo_editable))
                    && !node.disabled
            }) {
                self.drag_text_selection(id, x, y);
            }
        }
    }

    /// Extends the selection of an Entry/Text while the pointer is dragged.
    fn drag_text_selection(&mut self, id: WidgetId, x: f32, y: f32) {
        let Some(node) = self.node(id) else { return };
        let kind = node.kind;
        let rect = node.rect;
        let scroll_x = node.scroll_x;
        let font = self.resolved_style(id).unwrap_or_default();
        let text = node.text.clone();
        let cursor = match kind {
            Kind::Entry | Kind::Combobox => {
                let target = (x - rect.x - 6.0 + scroll_x).max(0.0);
                let boundaries = clusters(&text);
                let (mut low, mut high) = (0, boundaries.len());
                while low < high {
                    let mid = low + (high - low) / 2;
                    let end = boundaries.get(mid).copied().unwrap_or(text.len());
                    let width = self.measure_with(text.get(..end).unwrap_or_default(), &font).width;
                    if width < target {
                        low = mid + 1;
                    } else {
                        high = mid;
                    }
                }
                let right = boundaries.get(low).copied().unwrap_or(text.len());
                let left = low.checked_sub(1).and_then(|i| boundaries.get(i)).copied().unwrap_or(0);
                let rw = self.measure_with(text.get(..right).unwrap_or_default(), &font).width;
                let lw = self.measure_with(text.get(..left).unwrap_or_default(), &font).width;
                if (rw - target).abs() < (lw - target).abs() { right } else { left }
            }
            Kind::Text => {
                let line_h = (font.font_size * 1.3 + 4.0).max(16.0);
                let row = node.text_scroll_row + ((y - rect.y - 4.0) / line_h).max(0.0) as usize;
                let lines: Vec<&str> = text.split('\n').collect();
                let target_line = row.min(lines.len().saturating_sub(1));
                lines.iter().take(target_line).map(|l| l.len() + 1).sum::<usize>()
            }
            _ => return,
        };
        if let Some(node) = self.node_mut(id) {
            if node.selection_anchor.is_none() {
                node.selection_anchor = Some(node.cursor);
            }
            node.cursor = cursor.min(node.text.len());
        }
        self.dirty = true;
    }

    /// Headless harness: presses the primary mouse button at the current pointer.
    #[doc(hidden)]
    pub fn sim_mouse_down(&mut self) {
        self.ensure_layout();
        self.tooltip_visible = false;
        self.hover_timer_ms = 0;
        let (x, y) = self.pointer;
        if self.menu_click(x, y) {
            return;
        }
        if self.toast_click(x, y) {
            return;
        }
        if let Some((bar, index)) = self.menubar_hit(x, y) {
            self.menubar_open(bar, index);
            return;
        }
        // Combobox closed by this very click; must not be re-opened below.
        let mut just_closed: Option<WidgetId> = None;

        // Check if click is on open combobox dropdown
        if let Some(combo_id) = self.open_combobox {
            let rows = self.combo_rows(combo_id);
            if let Some(cnode) = self.node(combo_id) {
                let item_h = (cnode.style.font_size * 1.3 + 8.0).max(20.0);
                let drop_h = (rows.len() as f32 * item_h).max(item_h).min(240.0);
                let drop_rect = Rect::new(cnode.rect.x, cnode.rect.y + cnode.rect.height, cnode.rect.width, drop_h);
                if drop_rect.contains(x, y) {
                    let pos = ((y - drop_rect.y) / item_h).floor() as usize;
                    if let Some(&idx) = rows.get(pos) {
                        self.combo_select_value(combo_id, idx);
                    }
                    if let Some(cn) = self.node_mut(combo_id) {
                        cn.combo_open = false;
                        cn.combo_highlight = None;
                    }
                    self.open_combobox = None;
                    self.dirty = true;
                    return;
                }
            }
            if let Some(cn) = self.node_mut(combo_id) {
                cn.combo_open = false;
                cn.combo_highlight = None;
            }
            self.open_combobox = None;
            just_closed = Some(combo_id);
            self.dirty = true;
        }

        let hit = self.hit_test(x, y);
        let is_disabled = hit.and_then(|id| self.node(id)).map(|n| n.disabled).unwrap_or(false);

        if is_disabled {
            self.hovered = hit;
            return;
        }

        self.hovered = hit;
        let prev_pressed = self.pressed;
        self.pressed = hit;
        if let Some(prev) = prev_pressed {
            self.check_state_transition(prev, self.widget_state(prev) | State::PRESSED);
            self.invalidate_widget(prev);
        }
        if let Some(cur) = hit {
            self.check_state_transition(cur, self.widget_state(cur).without(State::PRESSED));
            self.invalidate_widget(cur);
        }
        // Pressed-state changes are covered by the per-widget invalidations
        // above; the full redraw below is still needed for focus/overlay state.

        if hit.is_some_and(|id| self.is_focusable(id)) {
            let prev_focus = self.focused;
            self.focused = hit;
            if prev_focus != hit {
                if let Some(pf) = prev_focus {
                    self.check_state_transition(pf, self.widget_state(pf) | State::FOCUSED);
                    self.invalidate_widget(pf);
                }
                if let Some(nf) = hit {
                    self.check_state_transition(nf, self.widget_state(nf).without(State::FOCUSED));
                    self.invalidate_widget(nf);
                }
            }
        }
        if let Some(id) = hit {
            let Some(node) = self.node(id) else { return };
            match node.kind {
                Kind::Entry => {
                    let target = (x - node.rect.x - 6.0 + node.scroll_x).max(0.0);
                    let text = self.widget_text(id).unwrap_or_default();
                    let font = self.resolved_style(id).unwrap_or_default();
                    let boundaries = clusters(&text);
                    let (mut low, mut high) = (0, boundaries.len());
                    while low < high {
                        let mid = low + (high - low) / 2;
                        let end = boundaries.get(mid).copied().unwrap_or(text.len());
                        let width = self.measure_with(text.get(..end).unwrap_or_default(), &font).width;
                        if width < target {
                            low = mid + 1;
                        } else {
                            high = mid;
                        }
                    }
                    let right = boundaries.get(low).copied().unwrap_or(text.len());
                    let left = low.checked_sub(1).and_then(|i| boundaries.get(i)).copied().unwrap_or(0);
                    let rw = self.measure_with(text.get(..right).unwrap_or_default(), &font).width;
                    let lw = self.measure_with(text.get(..left).unwrap_or_default(), &font).width;
                    let cursor = if (rw - target).abs() < (lw - target).abs() { right } else { left };
                    let shift = self.mods.shift;
                    if let Some(node) = self.node_mut(id) {
                        if shift {
                            if node.selection_anchor.is_none() {
                                node.selection_anchor = Some(node.cursor);
                            }
                        } else {
                            node.selection_anchor = Some(cursor);
                        }
                        node.cursor = cursor;
                    }
                }
                Kind::Scale => self.drag_scale(id, x),
                Kind::Slider => {
                    if let Some(value) = self.slider_value_at(id, x, y) {
                        self.slider_user_set(id, value);
                    }
                }
                Kind::DoubleSpinbox => {
                    let rx = x - node.rect.x;
                    let rw = node.rect.width;
                    let ry = y - node.rect.y;
                    let rh = node.rect.height;
                    if rx >= rw - 20.0 {
                        let direction = if ry < rh / 2.0 { 1 } else { -1 };
                        self.dspin_step(id, direction);
                    }
                }
                Kind::Listbox => {
                    let line_h = (node.style.font_size * 1.3 + 4.0).max(16.0);
                    let local_y = y - node.rect.y;
                    let row_idx = node.list_scroll + (local_y / line_h).floor() as usize;
                    if row_idx < node.list_items.len() {
                        // Double-click detection: a second press on the same row
                        // within 400 ms activates it (in addition to selecting).
                        let now = self.now_ms();
                        let prev = self
                            .node(id)
                            .map(|n| (n.list_last_click_ms, n.list_last_click_row))
                            .unwrap_or((0, None));
                        let is_double = prev.1 == Some(row_idx) && now.saturating_sub(prev.0) <= 400;
                        if let Some(n) = self.node_mut(id) {
                            n.list_last_click_ms = now;
                            n.list_last_click_row = Some(row_idx);
                        }
                        self.listbox_select(id, Some(row_idx));
                        if is_double {
                            self.dispatch_list_activate(id, row_idx);
                        }
                    }
                }
                Kind::Spinbox => {
                    let rx = x - node.rect.x;
                    let rw = node.rect.width;
                    let ry = y - node.rect.y;
                    let rh = node.rect.height;
                    if rx >= rw - 20.0 {
                        let (from, to) = node.range;
                        let step = if to >= from { 1 } else { -1 };
                        let old = node.scale.and_then(|v| v.get(self)).unwrap_or(from);
                        let val = if ry < rh / 2.0 {
                            old.saturating_add(step)
                        } else {
                            old.saturating_sub(step)
                        };
                        let val = val.clamp(from.min(to), from.max(to));
                        if let Some(var) = node.scale {
                            var.set(self, val);
                        }
                    }
                }
                Kind::Scrollbar => {
                    let horizontal = node
                        .inputs
                        .as_ref()
                        .is_some_and(|s| s.scrollbar_horizontal);
                    if horizontal {
                        let rx = x - node.rect.x;
                        let bar_w = node.rect.width;
                        if let Some(target_id) = node.scroll_target {
                            if let Some((total, visible, pos)) = self.scroll_metrics_x(target_id) {
                                let thumb_w = (bar_w * (visible / total)).max(16.0).min(bar_w);
                                let avail = bar_w - thumb_w;
                                let max = total - visible;
                                let thumb_x = if max > 0.0 { avail * (pos / max) } else { 0.0 };
                                if rx < thumb_x {
                                    self.set_scroll_pos_x(target_id, pos - visible);
                                } else if rx > thumb_x + thumb_w {
                                    self.set_scroll_pos_x(target_id, pos + visible);
                                } else {
                                    self.scrollbar_drag = Some((id, x, 0));
                                    self.scrollbar_drag_start = pos;
                                }
                            }
                        }
                    } else {
                        let ry = y - node.rect.y;
                        let bar_h = node.rect.height;
                        if let Some(target_id) = node.scroll_target {
                            if let Some((total, visible, pos)) = self.scroll_metrics(target_id) {
                                let thumb_h = (bar_h * (visible / total)).max(16.0).min(bar_h);
                                let avail = bar_h - thumb_h;
                                let max = total - visible;
                                let thumb_y = if max > 0.0 { avail * (pos / max) } else { 0.0 };
                                if ry < thumb_y {
                                    self.set_scroll_pos(target_id, pos - visible);
                                } else if ry > thumb_y + thumb_h {
                                    self.set_scroll_pos(target_id, pos + visible);
                                } else {
                                    self.scrollbar_drag = Some((id, y, 0));
                                    self.scrollbar_drag_start = pos;
                                }
                            }
                        }
                    }
                }
                Kind::Canvas => {
                    let (local_x, local_y) = (x - node.rect.x + node.view.0, y - node.rect.y + node.view.1);
                    if node.scrollregion.is_some() {
                        self.focused = Some(id);
                    }
                    self.drag_active = Some(id);
                    self.drag_start = (local_x, local_y);
                    self.dispatch_canvas_drag(id, local_x, local_y, DragPhase::Press);
                }
                Kind::Text => {
                    let line_h = (node.style.font_size * 1.3 + 4.0).max(16.0);
                    let row = node.text_scroll_row + ((y - node.rect.y - 4.0) / line_h).max(0.0) as usize;
                    let lines: Vec<&str> = node.text.split('\n').collect();
                    let target_line = row.min(lines.len().saturating_sub(1));
                    let byte_offset = lines.iter().take(target_line).map(|l| l.len() + 1).sum::<usize>();
                    let shift = self.mods.shift;
                    if let Some(node) = self.node_mut(id) {
                        if shift {
                            if node.selection_anchor.is_none() {
                                node.selection_anchor = Some(node.cursor);
                            }
                        } else {
                            node.selection_anchor = Some(byte_offset.min(node.text.len()));
                        }
                        node.cursor = byte_offset.min(node.text.len());
                    }
                }
                Kind::Combobox if node.combo_editable => {
                    let arrow_x = node.rect.x + node.rect.width - 22.0;
                    if x >= arrow_x {
                        let open = !node.combo_open && just_closed != Some(id);
                        if open {
                            self.combobox_open(id);
                        } else {
                            self.combobox_close(id);
                        }
                    } else {
                        // Place the caret in the text area like an Entry.
                        let target = (x - node.rect.x - 6.0 + node.scroll_x).max(0.0);
                        let text = node.text.clone();
                        let font = self.resolved_style(id).unwrap_or_default();
                        let boundaries = clusters(&text);
                        let (mut low, mut high) = (0, boundaries.len());
                        while low < high {
                            let mid = low + (high - low) / 2;
                            let end = boundaries.get(mid).copied().unwrap_or(text.len());
                            let width = self.measure_with(text.get(..end).unwrap_or_default(), &font).width;
                            if width < target {
                                low = mid + 1;
                            } else {
                                high = mid;
                            }
                        }
                        let right = boundaries.get(low).copied().unwrap_or(text.len());
                        let left = low.checked_sub(1).and_then(|i| boundaries.get(i)).copied().unwrap_or(0);
                        let rw = self.measure_with(text.get(..right).unwrap_or_default(), &font).width;
                        let lw = self.measure_with(text.get(..left).unwrap_or_default(), &font).width;
                        let cursor = if (rw - target).abs() < (lw - target).abs() { right } else { left };
                        let shift = self.mods.shift;
                        if let Some(node) = self.node_mut(id) {
                            if shift {
                                if node.selection_anchor.is_none() {
                                    node.selection_anchor = Some(node.cursor);
                                }
                            } else {
                                node.selection_anchor = Some(cursor);
                            }
                            node.cursor = cursor;
                        }
                    }
                }
                Kind::Combobox => {
                    let open = !node.combo_open && just_closed != Some(id);
                    if let Some(cn) = self.node_mut(id) {
                        cn.combo_open = open;
                    }
                    self.open_combobox = if open { Some(id) } else { None };
                }
                Kind::Table => {
                    let local_x = x - node.rect.x;
                    let local_y = y - node.rect.y;
                    self.table_mouse_down(id, local_x, local_y);
                }
                Kind::TreeView => {
                    let local_x = x - node.rect.x;
                    let local_y = y - node.rect.y;
                    self.tree_mouse_down(id, local_x, local_y);
                }
                Kind::Notebook if y < node.rect.y + 32.0 => {
                    let mut cur_x = node.rect.x;
                    let pages = node.notebook_pages.clone();
                    // Same metrics as `update_requested`, so hit areas match drawn tabs.
                    let tab_style = self.resolved_style(id).unwrap_or_default();
                    for (idx, (_, title)) in pages.iter().enumerate() {
                        let m = self.measure_with(title, &tab_style);
                        let tab_w = m.width + 24.0;
                        if x >= cur_x && x < cur_x + tab_w {
                            self.notebook_select(id, idx);
                            break;
                        }
                        cur_x += tab_w;
                    }
                }
                _ => {}
            }
        }
        self.dirty = true;
    }

    /// Headless harness: releases the primary mouse button, activating the
    /// pressed widget if the pointer is still over it.
    #[doc(hidden)]
    pub fn sim_mouse_up(&mut self) {
        self.ensure_layout();
        let hit = self.hit_test(self.pointer.0, self.pointer.1);
        if let Some(canvas_id) = self.drag_active.take() {
            if let Some(node) = self.node(canvas_id) {
                let cx = self.pointer.0 - node.rect.x + node.view.0;
                let cy = self.pointer.1 - node.rect.y + node.view.1;
                self.dispatch_canvas_drag(canvas_id, cx, cy, DragPhase::Release);
            }
        }
        self.scrollbar_drag = None;
        if let Some(id) = self.pressed {
            if self.node(id).is_some_and(|n| n.kind == Kind::Table) {
                self.table_mouse_up(id);
            }
        }
        if let Some(id) = self.pressed.take() {
            let is_disabled = self.node(id).map(|n| n.disabled).unwrap_or(false);
            if hit == Some(id) && !is_disabled {
                self.activate(id);
            }
            // It was pressed until `take()` above.
            self.check_state_transition(id, self.widget_state(id) | State::PRESSED);
            self.invalidate_widget(id);
        }
        self.hovered = hit.filter(|id| self.node(*id).is_some());
        // Activation may have changed focus, variables or overlays; keep the
        // full redraw request here.
        self.dirty = true;
    }

    /// Headless harness: move + press + release at logical `(x, y)`.
    #[doc(hidden)]
    pub fn sim_click(&mut self, x: f32, y: f32) {
        self.sim_mouse_move(x, y);
        self.sim_mouse_down();
        self.sim_mouse_up();
    }


    /// Runs the widget's command callback unless it is disabled or already executing, restoring the command afterward.
    pub(crate) fn run_command(&mut self, id: WidgetId) {
        let Some(node) = self.node_mut(id) else { return };
        if node.disabled || node.command_running {
            return;
        }
        let Some(mut command) = node.command.take() else { return };
        node.command_running = true;
        command(self);
        if let Some(node) = self.node_mut(id) {
            node.command_running = false;
            if node.command.is_none() {
                node.command = Some(command);
            }
        }
    }

    /// Activates a widget, toggling checkbutton or switch variables and invoking the command when enabled.
    pub(crate) fn activate(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        let (kind, check) = (node.kind, node.check);
        let radio_var = node.radio_var;
        let radio_value = node.radio_value;
        if kind == Kind::Checkbutton && self.tristate_activate(id) {
            self.run_command(id);
            return;
        }
        if kind == Kind::Checkbutton || kind == Kind::Switch {
            if let Some(var) = check {
                if let Some(value) = var.get(self) {
                    var.set(self, !value);
                }
            }
        } else if kind == Kind::Radiobutton {
            if let Some(var) = radio_var {
                var.set(self, radio_value);
            }
        }
        if matches!(kind, Kind::Button | Kind::Checkbutton | Kind::Radiobutton | Kind::Switch) {
            self.run_command(id);
        }
    }

    /// Returns the scale widget's current value as a clamped fraction between 0.0 and 1.0 of its range.
    pub(crate) fn scale_fraction(&self, id: WidgetId) -> f32 {
        let Some(node) = self.node(id) else { return 0.0 };
        let (from, to) = node.range;
        let value = node.scale.and_then(|var| var.get(self)).unwrap_or(from);
        if from == to {
            0.0
        } else {
            ((value as f64 - from as f64) / (to as f64 - from as f64)).clamp(0.0, 1.0) as f32
        }
    }

    /// Sets a scale widget's value, clamped to its range, updating the variable and running the command on change.
    pub(crate) fn set_scale(&mut self, id: WidgetId, value: i64) {
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        let Some(var) = node.scale else { return };
        let (from, to) = node.range;
        let value = value.clamp(from.min(to), from.max(to));
        if var.get(self).is_some_and(|old| old != value) {
            var.set(self, value);
            self.run_command(id);
        }
    }

    /// Updates a scale widget's value from a horizontal drag position mapped onto its track.
    pub(crate) fn drag_scale(&mut self, id: WidgetId, x: f32) {
        if !x.is_finite() {
            return;
        }
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        let track = node.rect.inset(12.0, 0.0);
        let fraction = if track.width > 0.0 {
            ((x - track.x) / track.width).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (from, to) = node.range;
        let value = (from as f64 + f64::from(fraction) * (to as f64 - from as f64)).round() as i64;
        self.set_scale(id, value);
    }

    /// Moves keyboard focus to the next or previous focusable widget in paint order, honoring modal scope.
    pub(crate) fn traverse_focus(&mut self, backwards: bool) {
        let order: Vec<_> = self.paint_order.iter().copied()
            .filter(|id| {
                if let Some(modal_id) = self.modal {
                    if !self.is_descendant_of(*id, modal_id) {
                        return false;
                    }
                }
                self.is_focusable(*id)
            }).collect();
        if order.is_empty() {
            self.focused = None;
            return;
        }
        let current = self.focused.and_then(|id| order.iter().position(|other| *other == id));
        let next = match (current, backwards) {
            (Some(0), true) | (None, true) => order.len() - 1,
            (Some(index), true) => index - 1,
            (Some(index), false) => (index + 1) % order.len(),
            (None, false) => 0,
        };
        self.focused = order.get(next).copied();
        self.dirty = true;
    }

    /// Headless harness: delivers a non-text key to the focused widget (or to the
    /// modal/combobox/focus-traversal handlers that intercept it).
    ///
    /// Equivalent to [`Tk::sim_key_with`] with no modifiers held.
    #[doc(hidden)]
    pub fn sim_key(&mut self, key: Key) {
        self.sim_key_with(key, crate::shortcut::Modifiers::NONE);
    }

    /// Delivers a non-text key to the focused widget, ignoring bound shortcuts.
    ///
    /// This is the internal dispatch path used by [`Tk::sim_key_with`] once
    /// shortcut matching has been ruled out.
    pub(crate) fn sim_key_raw(&mut self, key: Key) {
        self.ensure_layout();
        if self.menu_key(key) {
            return;
        }
        if key == Key::Escape {
            if let Some(combo_id) = self.open_combobox.take() {
                if let Some(node) = self.node_mut(combo_id) {
                    node.combo_open = false;
                    node.combo_highlight = None;
                }
                self.dirty = true;
                return;
            }
            if self.modal.is_some() {
                if let Some(mut cancel) = self.modal_cancel.take() {
                    cancel(self);
                    if self.modal.is_some() && self.modal_cancel.is_none() {
                        self.modal_cancel = Some(cancel);
                    }
                    return;
                }
            }
        }
        if key == Key::Enter && self.modal.is_some() {
            let on_button = self.focused.is_some_and(|f| {
                self.node(f).is_some_and(|n| n.kind == Kind::Button)
            });
            // A focused Listbox with a selection and an activate binding takes
            // precedence over the modal accept handler (e.g. directory chooser
            // navigation into a sub-directory).
            let listbox_activates = self.focused.is_some_and(|f| {
                self.node(f).is_some_and(|n| {
                    n.kind == Kind::Listbox
                        && n.list_selection.is_some()
                        && n.list_activate_cb.is_some()
                })
            });
            if !on_button && !listbox_activates {
                if let Some(mut accept) = self.modal_accept.take() {
                    accept(self);
                    // If the dialog rejected the input (still open, no new handler
                    // installed), restore the handler so a later Enter can accept.
                    if self.modal.is_some() && self.modal_accept.is_none() {
                        self.modal_accept = Some(accept);
                    }
                    return;
                }
            }
        }
        if matches!(key, Key::Tab | Key::ShiftTab) {
            self.traverse_focus(key == Key::ShiftTab);
            if let Some(focused) = self.focused {
                self.scrollarea_reveal(focused);
            }
            return;
        }
        if let Some(modal_id) = self.modal {
            if let Some(f) = self.focused {
                if !self.is_descendant_of(f, modal_id) {
                    self.traverse_focus(false);
                }
            }
        }
        let Some(id) = self.focused else { return };
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        if node.kind == Kind::Canvas && node.scrollregion.is_some() {
            match key {
                Key::PageUp => self.yview_scroll(id, -1, ScrollUnit::Pages),
                Key::PageDown | Key::Space => self.yview_scroll(id, 1, ScrollUnit::Pages),
                Key::Up => self.yview_scroll(id, -1, ScrollUnit::Units),
                Key::Down => self.yview_scroll(id, 1, ScrollUnit::Units),
                Key::Left => self.xview_scroll(id, -1, ScrollUnit::Units),
                Key::Right => self.xview_scroll(id, 1, ScrollUnit::Units),
                Key::Home => self.yview_moveto(id, 0.0),
                Key::End => self.yview_moveto(id, 1.0),
                _ => {}
            }
            return;
        }
        let extend = self.mods.shift;
        match node.kind {
            Kind::Entry => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else if key == Key::Enter {
                    // Submit callback from `entry_bind_submit` (no-op if none).
                    self.run_command(id);
                } else {
                    self.edit_entry_ext(id, Some(key), "", extend);
                }
            }
            Kind::Text => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else {
                    self.edit_text_ext(id, Some(key), "", extend);
                }
            }
            Kind::Combobox if node.combo_editable => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else {
                    self.combo_key(id, key, extend);
                }
            }
            Kind::Combobox => {
                if matches!(key, Key::Up | Key::Down) && !node.combo_values.is_empty() {
                    let old = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
                    let max_idx = node.combo_values.len().saturating_sub(1) as i64;
                    let new_val = if key == Key::Up { old.saturating_sub(1) } else { old.saturating_add(1) }.clamp(0, max_idx);
                    if let Some(var) = node.combo_var {
                        var.set(self, new_val);
                    }
                    self.dirty = true;
                }
            }
            Kind::Button | Kind::Checkbutton | Kind::Radiobutton | Kind::Switch if matches!(key, Key::Enter | Key::Space) => {
                self.activate(id);
            }
            Kind::Scale if matches!(key, Key::Left | Key::Right) => {
                let (from, to) = node.range;
                let old = node.scale.and_then(|var| var.get(self)).unwrap_or(from);
                let increasing = (key == Key::Right) == (to >= from);
                self.set_scale(id, if increasing { old.saturating_add(1) } else { old.saturating_sub(1) });
            }
            Kind::Listbox => {
                let total = node.list_items.len();
                if total > 0 {
                    let sel = node.list_selection;
                    let new_sel = match key {
                        Key::Up => Some(sel.unwrap_or(0).saturating_sub(1)),
                        Key::Down => Some((sel.map_or(0, |s| s + 1)).min(total - 1)),
                        Key::Home => Some(0),
                        Key::End => Some(total - 1),
                        _ => sel,
                    };
                    if new_sel != sel {
                        self.listbox_select(id, new_sel);
                    }
                    // Enter/Return activates the selected row (does not fire on
                    // arrow-key selection changes).
                    if matches!(key, Key::Enter) {
                        if let Some(idx) = self.node(id).and_then(|n| n.list_selection) {
                            self.dispatch_list_activate(id, idx);
                        }
                    }
                }
            }
            Kind::Slider => {
                self.slider_key(id, key);
            }
            Kind::DoubleSpinbox => {
                self.dspin_key(id, key);
            }
            Kind::Table => {
                self.table_key(id, key);
            }
            Kind::TreeView => {
                self.tree_key(id, key);
            }
            Kind::Spinbox if matches!(key, Key::Up | Key::Down) => {
                let (from, to) = node.range;
                let step = if to >= from { 1 } else { -1 };
                let old = node.scale.and_then(|v| v.get(self)).unwrap_or(from);
                let val = if key == Key::Up {
                    old.saturating_add(step)
                } else {
                    old.saturating_sub(step)
                };
                let val = val.clamp(from.min(to), from.max(to));
                if let Some(var) = node.scale {
                    var.set(self, val);
                }
            }
            _ => {}
        }
    }

    /// Headless harness: types `text` into the focused Entry or Text widget.
    #[doc(hidden)]
    pub fn sim_text(&mut self, text: &str) {
        self.ensure_layout();
        if let Some(id) = self.focused {
            if self.node(id).is_some_and(|node| node.kind == Kind::Entry && !node.disabled) {
                self.edit_entry(id, None, text);
            } else if self.node(id).is_some_and(|node| node.kind == Kind::Text && !node.disabled && !node.text_read_only) {
                self.edit_text(id, None, text);
            } else if self.node(id).is_some_and(|node| node.kind == Kind::Combobox && node.combo_editable && !node.disabled) {
                self.edit_entry(id, None, text);
            } else if self.node(id).is_some_and(|node| node.kind == Kind::DoubleSpinbox && !node.disabled) {
                self.dspin_type(id, text);
            }
        }
    }


    /// Clears all transient input state such as focus, press, hover and drag, and marks the toolkit dirty.
    pub(crate) fn clear_input(&mut self) {
        self.focused = None;
        self.pressed = None;
        self.hovered = None;
        self.drag_active = None;
        self.scrollbar_drag = None;
        self.ime_preedit = None;
        if let Some(combo_id) = self.open_combobox.take() {
            if let Some(node) = self.node_mut(combo_id) {
                node.combo_open = false;
            }
        }
        self.dirty = true;
    }

}
