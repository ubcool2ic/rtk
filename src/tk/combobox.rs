//! Combobox values, selection, and editable (filter-as-you-type) behaviour.
//!
//! Read-only comboboxes keep the classic tkinter behaviour: a `Var<i64>`
//! index into `combo_values`, a click toggles the dropdown overlay and
//! Up/Down change the index. Editable comboboxes additionally keep free text
//! in `node.text` (with the usual cursor/selection byte offsets) and filter
//! the dropdown as the user types.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Combobox API
    // -----------------------------------------------------------------------

    /// Replaces the dropdown choices (tkinter `values`).
    ///
    /// Also recomputes the filter for editable comboboxes and clears the
    /// highlighted row.
    pub fn combobox_set_values(&mut self, id: WidgetId, values: Vec<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Combobox { return false; }
        node.combo_values = values;
        node.combo_highlight = None;
        self.dirty = true;
        self.combo_recompute_filter(id);
        let (editable, text, filtered_empty, var) = match self.node(id) {
            Some(n) => (
                n.combo_editable,
                n.text.clone(),
                n.combo_filtered.is_empty(),
                n.combo_var,
            ),
            None => return false,
        };
        if editable {
            // Re-sync the bound index to the value equal to the current text.
            let index = self
                .node(id)
                .and_then(|n| n.combo_values.iter().position(|v| *v == text))
                .map_or(-1, |i| i as i64);
            if let Some(var) = var {
                var.set(self, index);
            }
        }
        if filtered_empty {
            self.combobox_close(id);
        }
        true
    }

    /// Returns the selected `(index, text)` (tkinter `current()` / `get()`).
    ///
    /// For an editable combobox the text is the current free text; for a
    /// read-only combobox it is the value at the bound index.
    pub fn combobox_get(&self, id: WidgetId) -> Option<(i64, String)> {
        let node = self.node(id)?;
        if node.kind != Kind::Combobox { return None; }
        let idx = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
        let s = if node.combo_editable {
            node.text.clone()
        } else {
            node.combo_values.get(idx as usize).cloned().unwrap_or_default()
        };
        Some((idx, s))
    }

    /// Returns true if the combobox accepts free text.
    pub fn combobox_is_editable(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|n| n.kind == Kind::Combobox && n.combo_editable)
    }

    /// Returns the current text of a combobox.
    ///
    /// Editable comboboxes return their free text; read-only comboboxes return
    /// the value at the bound index.
    pub fn combobox_get_text(&self, id: WidgetId) -> Option<String> {
        let node = self.node(id)?;
        if node.kind != Kind::Combobox { return None; }
        if node.combo_editable {
            Some(node.text.clone())
        } else {
            let idx = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
            Some(node.combo_values.get(idx as usize).cloned().unwrap_or_default())
        }
    }

    /// Sets the free text of an editable combobox.
    ///
    /// Control characters are stripped (like an Entry), the cursor moves to the
    /// end, any selection is cleared, the filter is recomputed and the bound
    /// variable is set to the index of the first value equal to `text`, or `-1`
    /// when there is no exact match. The dropdown is not opened and no
    /// callbacks fire. Returns false for non-editable or non-combobox ids.
    pub fn combobox_set_text(&mut self, id: WidgetId, text: &str) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Combobox || !node.combo_editable { return false; }
        let cleaned = clean_entry(text);
        let var = node.combo_var;
        let values = node.combo_values.clone();
        if let Some(node) = self.node_mut(id) {
            node.text = cleaned.clone();
            node.cursor = cleaned.len();
            node.selection_anchor = None;
            node.combo_highlight = None;
        }
        let index = values.iter().position(|v| *v == cleaned).map_or(-1, |i| i as i64);
        if let Some(var) = var {
            var.set(self, index);
        }
        self.combo_recompute_filter(id);
        if self.node(id).is_some_and(|n| n.combo_filtered.is_empty()) {
            self.combobox_close(id);
        }
        self.dirty = true;
        true
    }

    /// Registers a callback fired when the user picks a value.
    ///
    /// The callback receives the toolkit, the index into `combo_values` and the
    /// selected text. Works for both editable and read-only comboboxes.
    pub fn combobox_on_select(
        &mut self,
        id: WidgetId,
        f: impl FnMut(&mut Tk, usize, &str) + 'static,
    ) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Combobox { return false; }
        node.combo_on_select = Some(Box::new(f));
        true
    }

    /// Registers a callback fired after every user edit of an editable
    /// combobox's text (typing, deletion, paste, cut, undo/redo, IME commit).
    ///
    /// Returns false for non-editable or non-combobox ids.
    pub fn combobox_on_text_change(
        &mut self,
        id: WidgetId,
        f: impl FnMut(&mut Tk, &str) + 'static,
    ) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Combobox || !node.combo_editable { return false; }
        node.combo_on_text = Some(Box::new(f));
        true
    }

    /// Returns the values currently shown in the dropdown.
    pub fn combobox_get_filtered(&self, id: WidgetId) -> Vec<String> {
        let Some(node) = self.node(id) else { return Vec::new() };
        if node.kind != Kind::Combobox { return Vec::new(); }
        self.combo_rows(id)
            .into_iter()
            .filter_map(|i| node.combo_values.get(i).cloned())
            .collect()
    }

    /// Returns the index into `combo_values` of the highlighted dropdown row.
    pub fn combobox_get_highlighted(&self, id: WidgetId) -> Option<usize> {
        let node = self.node(id)?;
        if node.kind != Kind::Combobox { return None; }
        let pos = node.combo_highlight?;
        node.combo_filtered.get(pos).copied()
    }

    /// Returns true if the dropdown overlay is open for this combobox.
    pub fn combobox_is_open(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|n| n.kind == Kind::Combobox && n.combo_open)
    }

    /// Opens the dropdown for a combobox.
    ///
    /// For editable comboboxes the filter is recomputed from the current text
    /// (all values when the text is empty or exactly equals a value); when no
    /// value matches, the dropdown stays closed.
    pub fn combobox_open(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Combobox { return; }
        if node.combo_editable {
            self.combo_recompute_filter(id);
            if self.node(id).is_some_and(|n| n.combo_filtered.is_empty()) {
                // Nothing matches the typed text: there is no list to show.
                return;
            }
        }
        self.combo_close_other(id);
        if let Some(node) = self.node_mut(id) {
            node.combo_open = true;
        }
        self.open_combobox = Some(id);
        self.dirty = true;
    }

    /// Closes any open combobox other than `keep`, clearing its highlight and
    /// the shared `open_combobox` overlay slot.
    fn combo_close_other(&mut self, keep: WidgetId) {
        if let Some(other) = self.open_combobox {
            if other != keep {
                if let Some(node) = self.node_mut(other) {
                    node.combo_open = false;
                    node.combo_highlight = None;
                }
                self.open_combobox = None;
            }
        }
    }

    /// Closes the dropdown for a combobox and clears the highlighted row.
    pub fn combobox_close(&mut self, id: WidgetId) {
        let Some(node) = self.node_mut(id) else { return };
        if node.kind != Kind::Combobox { return; }
        node.combo_open = false;
        node.combo_highlight = None;
        if self.open_combobox == Some(id) {
            self.open_combobox = None;
        }
        self.dirty = true;
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Returns the indices into `combo_values` currently shown in the dropdown.
    ///
    /// Editable comboboxes use the stored filter; read-only comboboxes show
    /// every value in order.
    pub(crate) fn combo_rows(&self, id: WidgetId) -> Vec<usize> {
        let Some(node) = self.node(id) else { return Vec::new() };
        if node.kind != Kind::Combobox { return Vec::new(); }
        if node.combo_editable {
            node.combo_filtered.clone()
        } else {
            (0..node.combo_values.len()).collect()
        }
    }

    /// Recomputes `combo_filtered` for an editable combobox from its text.
    ///
    /// An empty text, or a text that exactly equals one of the values, shows
    /// every value; otherwise a case-insensitive substring match is used.
    pub(crate) fn combo_recompute_filter(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Combobox || !node.combo_editable { return; }
        let text = node.text.clone();
        let values = node.combo_values.clone();
        let show_all = text.is_empty() || values.contains(&text);
        let needle = text.to_lowercase();
        let filtered: Vec<usize> = values
            .iter()
            .enumerate()
            .filter(|(_, v)| show_all || v.to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect();
        if let Some(node) = self.node_mut(id) {
            node.combo_filtered = filtered;
            if let Some(pos) = node.combo_highlight {
                if pos >= node.combo_filtered.len() {
                    node.combo_highlight = None;
                }
            }
        }
    }

    /// Post-edit hook for an editable combobox: recomputes the filter, resets
    /// the highlight, opens/closes the dropdown, updates the bound variable and
    /// fires the text-change callback.
    pub(crate) fn combo_text_edited(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Combobox || !node.combo_editable { return; }
        let text = node.text.clone();
        let values = node.combo_values.clone();
        let var = node.combo_var;
        self.combo_recompute_filter(id);
        let filtered_len = self.node(id).map(|n| n.combo_filtered.len()).unwrap_or(0);
        let should_open = !text.is_empty() && filtered_len > 0;
        if should_open {
            self.combo_close_other(id);
        }
        if let Some(node) = self.node_mut(id) {
            node.combo_highlight = None;
            node.combo_open = should_open;
        }
        if should_open {
            self.open_combobox = Some(id);
        } else if self.open_combobox == Some(id) {
            self.open_combobox = None;
        }
        let index = values.iter().position(|v| *v == text).map_or(-1, |i| i as i64);
        if let Some(var) = var {
            var.set(self, index);
        }
        self.dirty = true;
        self.combo_fire_text(id, &text);
    }

    /// Fires the text-change callback of an editable combobox, restoring it
    /// afterwards unless the callback replaced or removed it.
    fn combo_fire_text(&mut self, id: WidgetId, text: &str) {
        let Some(node) = self.node_mut(id) else { return };
        let Some(mut cb) = node.combo_on_text.take() else { return };
        cb(self, text);
        if let Some(node) = self.node_mut(id) {
            if node.combo_on_text.is_none() {
                node.combo_on_text = Some(cb);
            }
        }
    }

    /// Fires the select callback of a combobox, restoring it afterwards unless
    /// the callback replaced or removed it.
    fn combo_fire_select(&mut self, id: WidgetId, index: usize, text: &str) {
        let Some(node) = self.node_mut(id) else { return };
        let Some(mut cb) = node.combo_on_select.take() else { return };
        cb(self, index, text);
        if let Some(node) = self.node_mut(id) {
            if node.combo_on_select.is_none() {
                node.combo_on_select = Some(cb);
            }
        }
    }

    /// Selects the value at `index` in `combo_values` for a combobox.
    ///
    /// Editable comboboxes set their text (cursor to end, selection cleared),
    /// close the dropdown and update the bound variable; read-only comboboxes
    /// set the bound variable. Both fire the select callback.
    pub(crate) fn combo_select_value(&mut self, id: WidgetId, index: usize) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Combobox { return; }
        let Some(value) = node.combo_values.get(index).cloned() else { return };
        let editable = node.combo_editable;
        let var = node.combo_var;
        if editable {
            if let Some(node) = self.node_mut(id) {
                node.text = value.clone();
                node.cursor = value.len();
                node.selection_anchor = None;
            }
            self.combo_recompute_filter(id);
        }
        self.combobox_close(id);
        if let Some(var) = var {
            var.set(self, index as i64);
        }
        self.dirty = true;
        self.combo_fire_select(id, index, &value);
    }

    /// Handles a non-text key for an editable combobox.
    ///
    /// Left/Right/Home/End move the caret (Shift extends the selection),
    /// Backspace/Delete edit the text, Up/Down move the dropdown highlight and
    /// Enter selects the highlighted row (or closes the dropdown).
    pub(crate) fn combo_key(&mut self, id: WidgetId, key: Key, extend: bool) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Combobox || !node.combo_editable || node.disabled {
            return;
        }
        match key {
            Key::Left | Key::Right | Key::Home | Key::End | Key::Backspace | Key::Delete => {
                self.edit_entry_ext(id, Some(key), "", extend);
            }
            Key::Down => {
                let open = node.combo_open;
                if !open {
                    self.combobox_open(id);
                    if let Some(node) = self.node_mut(id) {
                        if !node.combo_filtered.is_empty() {
                            node.combo_highlight = Some(0);
                        }
                    }
                } else {
                    let len = node.combo_filtered.len();
                    if len > 0 {
                        let next = node.combo_highlight.map_or(0, |h| (h + 1).min(len - 1));
                        if let Some(node) = self.node_mut(id) {
                            node.combo_highlight = Some(next);
                        }
                    }
                }
                self.dirty = true;
            }
            Key::Up => {
                let open = node.combo_open;
                if !open {
                    self.combobox_open(id);
                    if let Some(node) = self.node_mut(id) {
                        let len = node.combo_filtered.len();
                        if len > 0 {
                            node.combo_highlight = Some(len - 1);
                        }
                    }
                } else {
                    let len = node.combo_filtered.len();
                    if len > 0 {
                        let prev = node.combo_highlight.map_or(0, |h| h.saturating_sub(1));
                        if let Some(node) = self.node_mut(id) {
                            node.combo_highlight = Some(prev);
                        }
                    }
                }
                self.dirty = true;
            }
            Key::Enter => {
                let open = node.combo_open;
                let highlight = node.combo_highlight;
                if open {
                    if let Some(pos) = highlight {
                        if let Some(&index) = node.combo_filtered.get(pos) {
                            self.combo_select_value(id, index);
                            return;
                        }
                    }
                }
                self.combobox_close(id);
            }
            _ => {}
        }
    }
}
