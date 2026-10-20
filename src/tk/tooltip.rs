//! Tooltip text and visibility.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Tooltip API
    // -----------------------------------------------------------------------

    /// Attaches hover tooltip text to a widget.
    pub fn tooltip(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        node.tooltip = Some(text.into());
        true
    }

    /// Removes a widget's tooltip and hides it if currently shown.
    pub fn tooltip_remove(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        node.tooltip = None;
        if self.hovered == Some(id) {
            self.tooltip_visible = false;
        }
        true
    }

    /// Returns the tooltip text currently displayed, if any.
    pub fn active_tooltip(&self) -> Option<String> {
        if self.tooltip_visible {
            self.hovered.and_then(|id| self.node(id)).and_then(|n| n.tooltip.clone())
        } else {
            None
        }
    }
}
