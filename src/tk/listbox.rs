//! Listbox items, selection and callbacks.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // Listbox methods
    /// Inserts an item at `index` (appends when out of range), shifting the selection.
    pub fn listbox_insert(&mut self, id: WidgetId, index: usize, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        let item = text.into();
        if index >= node.list_items.len() {
            node.list_items.push(item);
        } else {
            node.list_items.insert(index, item);
            if let Some(sel) = node.list_selection {
                if index <= sel {
                    node.list_selection = Some(sel + 1);
                }
            }
        }
        self.dirty = true;
        true
    }

    /// Deletes items `first..=last` (clamped), adjusting selection and scroll.
    pub fn listbox_delete(&mut self, id: WidgetId, first: usize, last: usize) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox || first >= node.list_items.len() {
            return false;
        }
        let end = (last.saturating_add(1)).min(node.list_items.len());
        if first < end {
            node.list_items.drain(first..end);
            if let Some(sel) = node.list_selection {
                if sel >= first && sel < end {
                    node.list_selection = None;
                } else if sel >= end {
                    node.list_selection = Some(sel - (end - first));
                }
            }
            let total = node.list_items.len();
            let vis = node.list_height.max(1);
            let max_scroll = total.saturating_sub(vis);
            node.list_scroll = node.list_scroll.min(max_scroll);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// All items of a Listbox (empty for other kinds).
    pub fn listbox_get(&self, id: WidgetId) -> Vec<String> {
        self.node(id)
            .filter(|n| n.kind == Kind::Listbox)
            .map(|n| n.list_items.clone())
            .unwrap_or_default()
    }

    /// Index of the selected row, if any (tkinter `curselection()`).
    pub fn listbox_curselection(&self, id: WidgetId) -> Option<usize> {
        self.node(id)
            .filter(|n| n.kind == Kind::Listbox)
            .and_then(|n| n.list_selection)
    }

    /// Selects row `index` (or clears with `None`), scrolls it into view, fires
    /// the select callback on change and the command on every call.
    pub fn listbox_select(&mut self, id: WidgetId, index: Option<usize>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        let old_sel = node.list_selection;
        if let Some(idx) = index {
            if idx >= node.list_items.len() {
                return false;
            }
            node.list_selection = Some(idx);
            let vis = node.list_height.max(1);
            if idx < node.list_scroll {
                node.list_scroll = idx;
            } else if idx >= node.list_scroll + vis {
                node.list_scroll = idx + 1 - vis;
            }
        } else {
            node.list_selection = None;
        }
        let new_sel = node.list_selection;
        self.dirty = true;
        if new_sel != old_sel {
            self.dispatch_list_select(id, new_sel);
        }
        self.run_command(id);
        true
    }

    /// Binds a callback invoked with the new selection when it changes.
    pub fn listbox_bind_select(&mut self, id: WidgetId, cb: impl FnMut(&mut Tk, Option<usize>) + 'static) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        node.list_select_cb = Some(Box::new(cb));
        true
    }

    /// Binds a callback invoked with the row index when a Listbox row is
    /// *activated*: Enter/Return pressed while the listbox is focused and has a
    /// selection, or a double-click on a row (second mouse-down on the same row
    /// within 400 ms). Unlike [`Tk::listbox_bind_select`] this does not fire on
    /// plain selection changes (single clicks, arrow keys).
    pub fn listbox_bind_activate(&mut self, id: WidgetId, cb: impl FnMut(&mut Tk, usize) + 'static) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        node.list_activate_cb = Some(Box::new(cb));
        true
    }

    /// Invokes the listbox activation callback with `index`, guarding against reentrancy.
    pub(crate) fn dispatch_list_activate(&mut self, id: WidgetId, index: usize) {
        let Some(node) = self.node_mut(id) else { return };
        if node.list_activate_running { return; }
        let Some(mut cb) = node.list_activate_cb.take() else { return };
        node.list_activate_running = true;
        cb(self, index);
        if let Some(node) = self.node_mut(id) {
            node.list_activate_running = false;
            if node.list_activate_cb.is_none() {
                node.list_activate_cb = Some(cb);
            }
        }
    }

    /// Invokes the listbox selection callback with the new selection, guarding against reentrancy.
    pub(crate) fn dispatch_list_select(&mut self, id: WidgetId, sel: Option<usize>) {
        let Some(node) = self.node_mut(id) else { return };
        if node.list_select_running { return; }
        let Some(mut cb) = node.list_select_cb.take() else { return };
        node.list_select_running = true;
        cb(self, sel);
        if let Some(node) = self.node_mut(id) {
            node.list_select_running = false;
            if node.list_select_cb.is_none() {
                node.list_select_cb = Some(cb);
            }
        }
    }

}
