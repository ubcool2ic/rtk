//! Notebook pages and change callbacks.

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Notebook and Geometry show/hide API
    // -----------------------------------------------------------------------

    /// Unmaps a widget managed by pack (tkinter `pack_forget`).
    pub fn pack_forget(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.manager.is_some() {
            node.manager = None;
            node.visible = false;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Unmaps a widget managed by grid (tkinter `grid_forget`).
    pub fn grid_forget(&mut self, id: WidgetId) -> bool {
        self.pack_forget(id)
    }


    /// Adds a child Frame as a tab page with the given title (tkinter `add`).
    pub fn notebook_add(&mut self, nb: WidgetId, page: WidgetId, title: impl Into<String>) -> bool {
        let is_nb = self.node(nb).is_some_and(|n| n.kind == Kind::Notebook);
        let is_page = self.node(page).is_some_and(|n| n.kind == Kind::Frame && n.parent == Some(nb));
        if !is_nb || !is_page { return false; }
        let title = title.into();
        if let Some(nb_node) = self.node_mut(nb) {
            nb_node.notebook_pages.push((page, title));
            if nb_node.notebook_pages.len() == 1 {
                nb_node.notebook_selected = 0;
            }
        }
        self.dirty = true;
        true
    }

    /// Selects the page at `index`, firing the change callback if it changed.
    pub fn notebook_select(&mut self, nb: WidgetId, index: usize) -> bool {
        let Some(nb_node) = self.node(nb) else { return false };
        if nb_node.kind != Kind::Notebook || index >= nb_node.notebook_pages.len() {
            return false;
        }
        let old = nb_node.notebook_selected;
        if let Some(n) = self.node_mut(nb) {
            n.notebook_selected = index;
        }
        self.dirty = true;
        if old != index {
            self.dispatch_notebook_changed(nb, index);
        }
        true
    }

    /// Index of the currently selected page (tkinter `index("current")`).
    pub fn notebook_selected(&self, nb: WidgetId) -> Option<usize> {
        self.node(nb).filter(|n| n.kind == Kind::Notebook).map(|n| n.notebook_selected)
    }

    /// Binds a callback invoked with the new page index when selection changes.
    pub fn notebook_bind_changed(&mut self, nb: WidgetId, cb: impl FnMut(&mut Tk, usize) + 'static) -> bool {
        let Some(node) = self.node_mut(nb) else { return false };
        if node.kind != Kind::Notebook { return false; }
        node.notebook_changed_cb = Some(Box::new(cb));
        true
    }

    /// Invokes the notebook's changed callback with the new tab index, guarding against reentrancy.
    pub(crate) fn dispatch_notebook_changed(&mut self, nb: WidgetId, index: usize) {
        let Some(node) = self.node_mut(nb) else { return };
        if node.notebook_changed_running { return; }
        let Some(mut cb) = node.notebook_changed_cb.take() else { return };
        node.notebook_changed_running = true;
        cb(self, index);
        if let Some(node) = self.node_mut(nb) {
            node.notebook_changed_running = false;
            if node.notebook_changed_cb.is_none() {
                node.notebook_changed_cb = Some(cb);
            }
        }
    }

}
