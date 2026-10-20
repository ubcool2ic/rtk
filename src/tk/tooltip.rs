//! Tooltip text and visibility.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Tooltip API
    // -----------------------------------------------------------------------

    pub fn tooltip(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        node.tooltip = Some(text.into());
        true
    }

    pub fn tooltip_remove(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        node.tooltip = None;
        if self.hovered == Some(id) {
            self.tooltip_visible = false;
        }
        true
    }

    pub fn active_tooltip(&self) -> Option<String> {
        if self.tooltip_visible {
            self.hovered.and_then(|id| self.node(id)).and_then(|n| n.tooltip.clone())
        } else {
            None
        }
    }
}
