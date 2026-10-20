//! Progressbar variable binding and indeterminate mode.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Progressbar var & indeterminate mode
    // -----------------------------------------------------------------------

    pub fn progress_bind(&mut self, id: WidgetId, var: crate::Var<f64>) -> bool {
        if var.get(self).is_none() { return false; }
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressBar { return false; }
        node.progress_f64 = Some(var);
        self.dirty = true;
        true
    }

    pub fn progress_set_indeterminate(&mut self, id: WidgetId, indeterminate: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressBar { return false; }
        node.progress_indeterminate = indeterminate;
        self.dirty = true;
        true
    }

}
