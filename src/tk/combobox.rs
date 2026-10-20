//! Combobox values and selection.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Combobox API
    // -----------------------------------------------------------------------

    pub fn combobox_set_values(&mut self, id: WidgetId, values: Vec<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Combobox { return false; }
        node.combo_values = values;
        self.dirty = true;
        true
    }

    pub fn combobox_get(&self, id: WidgetId) -> Option<(i64, String)> {
        let node = self.node(id)?;
        if node.kind != Kind::Combobox { return None; }
        let idx = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
        let s = node.combo_values.get(idx as usize).cloned().unwrap_or_default();
        Some((idx, s))
    }

}
