//! TreeView widget: hierarchical, virtualized tree with expand/collapse.
//!
//! All tree state lives in [`TreeState`], stored on the widget node as
//! `tree: Option<Box<TreeState>>`. Nodes are kept in a `Vec` with free-slot
//! keys (generational handles are not needed here because keys are stable
//! indices into the node vector). A cached flattened visible list
//! (`visible: Vec<(key, depth)>`) is rebuilt only when the structure or the
//! open state changes (dirty flag), never per frame. Rendering and hit-testing
//! only touch the visible window of rows (`top..top+visible+1`).

use super::*;

/// Callback fired when the selected node of a TreeView changes.
pub(crate) type TreeSelectCallback = Box<dyn FnMut(&mut Tk, Option<usize>)>;

/// One node in a [`TreeState`].
pub(crate) struct TreeNode {
    /// Display text.
    pub(crate) text: String,
    /// Parent key, or `None` for a root node.
    pub(crate) parent: Option<usize>,
    /// Child keys in insertion order.
    pub(crate) children: Vec<usize>,
    /// Whether the node is expanded (children shown).
    pub(crate) open: bool,
}

/// Mutable state of a TreeView widget.
pub(crate) struct TreeState {
    /// Node storage; `None` marks a freed slot.
    pub(crate) nodes: Vec<Option<TreeNode>>,
    /// Free slot keys available for reuse.
    pub(crate) free: Vec<usize>,
    /// Cached flattened visible list as `(key, depth)`.
    pub(crate) visible: Vec<(usize, usize)>,
    /// Set when `visible` must be rebuilt.
    pub(crate) dirty: bool,
    /// Selected node key.
    pub(crate) selected: Option<usize>,
    /// Display row shown at the top of the body.
    pub(crate) top: usize,
    /// Selection-change callback.
    pub(crate) on_select: Option<TreeSelectCallback>,
    /// Re-entrancy guard for the selection callback.
    pub(crate) on_select_running: bool,
}

impl TreeState {
    /// Creates an empty tree.
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
            visible: Vec::new(),
            dirty: true,
            selected: None,
            top: 0,
            on_select: None,
            on_select_running: false,
        }
    }

    /// Returns the node stored at `key`, if live.
    pub(crate) fn get(&self, key: usize) -> Option<&TreeNode> {
        self.nodes.get(key)?.as_ref()
    }

    /// Returns the node stored at `key` mutably, if live.
    pub(crate) fn get_mut(&mut self, key: usize) -> Option<&mut TreeNode> {
        self.nodes.get_mut(key)?.as_mut()
    }

    /// Allocates a slot for a new node and returns its key.
    pub(crate) fn alloc(&mut self, node: TreeNode) -> usize {
        if let Some(key) = self.free.pop() {
            self.nodes[key] = Some(node);
            key
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        }
    }

    /// Frees the slot at `key` (no-op if already free).
    pub(crate) fn free_slot(&mut self, key: usize) {
        if let Some(slot) = self.nodes.get_mut(key) {
            if slot.take().is_some() {
                self.free.push(key);
            }
        }
    }

    /// Rebuilds the flattened visible list from the current structure and open state.
    pub(crate) fn rebuild_visible(&mut self) {
        self.visible.clear();
        // Roots are nodes with no parent, in key order.
        let roots: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(key, slot)| {
                let node = slot.as_ref()?;
                node.parent.is_none().then_some(key)
            })
            .collect();
        for root in roots {
            self.push_visible(root, 0);
        }
        self.dirty = false;
    }

    /// Rebuilds `visible` if it is dirty *or* if the cached list is empty while
    /// the tree still holds live nodes.
    ///
    /// The second condition guards against a stale cache: a tree built while
    /// its page is hidden may have had `dirty` cleared by an early layout pass
    /// before any node existed, leaving `visible` empty even though nodes were
    /// inserted afterwards. Rebuilding whenever the cache is empty but nodes
    /// exist is cheap (it only happens once) and makes rendering robust.
    pub(crate) fn ensure_visible_robust(&mut self) {
        let has_nodes = self.nodes.iter().any(|slot| slot.is_some());
        if self.visible.is_empty() && has_nodes {
            self.dirty = true;
        }
        self.ensure_visible();
    }

    /// Appends `key` and (if open) its descendants to `visible`.
    fn push_visible(&mut self, key: usize, depth: usize) {
        let Some(node) = self.get(key) else { return };
        let open = node.open;
        let children = node.children.clone();
        self.visible.push((key, depth));
        if !open {
            return;
        }
        for child in children {
            self.push_visible(child, depth + 1);
        }
    }

    /// Rebuilds `visible` if it is dirty.
    pub(crate) fn ensure_visible(&mut self) {
        if self.dirty {
            self.rebuild_visible();
        }
    }

    /// Display position of `key` in the visible list, if shown.
    pub(crate) fn display_of(&self, key: usize) -> Option<usize> {
        self.visible.iter().position(|&(k, _)| k == key)
    }

    /// Collects `key` and all of its descendants into `out`.
    fn collect_subtree(&self, key: usize, out: &mut Vec<usize>) {
        let Some(node) = self.get(key) else { return };
        out.push(key);
        for &child in &node.children {
            self.collect_subtree(child, out);
        }
    }
}

impl Tk {
    /// Returns a shared reference to the tree state of `id`, if it is a TreeView.
    pub(crate) fn tree_state(&self, id: WidgetId) -> Option<&TreeState> {
        self.node(id).filter(|n| n.kind == Kind::TreeView)?.tree.as_deref()
    }

    /// Returns a mutable reference to the tree state of `id`, if it is a TreeView.
    pub(crate) fn tree_state_mut(&mut self, id: WidgetId) -> Option<&mut TreeState> {
        self.node_mut(id).filter(|n| n.kind == Kind::TreeView)?.tree.as_deref_mut()
    }

    /// Row height in logical px derived from the resolved body font size.
    pub(crate) fn tree_row_height_for(&self, id: WidgetId) -> f32 {
        let size = self.resolved_style(id).map(|s| s.font_size).unwrap_or(14.0);
        (size * 1.6).round().max(8.0)
    }

    /// Number of fully visible body rows for the current widget rect.
    ///
    /// Falls back to the requested height when the widget has not been laid
    /// out yet (e.g. it lives on a notebook page that is not selected), so
    /// callers never see a bogus zero row count.
    pub(crate) fn tree_visible_rows(&self, id: WidgetId) -> usize {
        let Some(node) = self.node(id) else { return 0 };
        let row_h = self.tree_row_height_for(id);
        if row_h <= 0.0 {
            return 0;
        }
        let height = if node.rect.height > 0.0 {
            node.rect.height
        } else {
            node.requested.height
        };
        (height / row_h).floor() as usize
    }

    /// Clamps `top` so the view stays within the visible row count.
    pub(crate) fn tree_clamp_top(&mut self, id: WidgetId) {
        self.ensure_layout();
        let visible = self.tree_visible_rows(id);
        let total = self.tree_state(id).map(|s| s.visible.len()).unwrap_or(0);
        let max_top = total.saturating_sub(visible.max(1));
        if let Some(state) = self.tree_state_mut(id) {
            state.top = state.top.min(max_top);
        }
    }

    /// Inserts a node under `parent` (or as a root when `None`), returning its key.
    ///
    /// Returns `None` if `id` is not a TreeView or `parent` is not a live node.
    pub fn tree_insert(&mut self, id: WidgetId, parent: Option<usize>, text: &str) -> Option<usize> {
        let state = self.tree_state_mut(id)?;
        if let Some(p) = parent {
            state.get(p)?;
        }
        let key = state.alloc(TreeNode {
            text: text.to_string(),
            parent,
            children: Vec::new(),
            open: false,
        });
        if let Some(p) = parent {
            if let Some(pnode) = state.get_mut(p) {
                pnode.children.push(key);
            }
        }
        state.dirty = true;
        self.dirty = true;
        Some(key)
    }

    /// Removes the subtree rooted at `key`. Returns true if the node existed.
    pub fn tree_remove(&mut self, id: WidgetId, key: usize) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        if state.get(key).is_none() {
            return false;
        }
        let parent = state.get(key).and_then(|n| n.parent);
        let mut subtree = Vec::new();
        state.collect_subtree(key, &mut subtree);
        if let Some(p) = parent {
            if let Some(pnode) = state.get_mut(p) {
                pnode.children.retain(|&c| c != key);
            }
        }
        for k in subtree {
            state.free_slot(k);
        }
        if state.selected.is_some_and(|s| state.get(s).is_none()) {
            state.selected = None;
        }
        state.dirty = true;
        self.dirty = true;
        true
    }

    /// Removes every node from the tree.
    pub fn tree_clear(&mut self, id: WidgetId) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        state.nodes.clear();
        state.free.clear();
        state.visible.clear();
        state.selected = None;
        state.top = 0;
        state.dirty = true;
        self.dirty = true;
        true
    }

    /// Sets the display text of node `key`. Returns true if the node exists.
    pub fn tree_set_text(&mut self, id: WidgetId, key: usize, text: &str) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        let Some(node) = state.get_mut(key) else { return false };
        node.text = text.to_string();
        self.dirty = true;
        true
    }

    /// Returns the display text of node `key`, if it exists.
    pub fn tree_text(&self, id: WidgetId, key: usize) -> Option<String> {
        self.tree_state(id)?.get(key).map(|n| n.text.clone())
    }

    /// Returns the child keys of `parent` (or the root keys when `None`).
    pub fn tree_children(&self, id: WidgetId, parent: Option<usize>) -> Vec<usize> {
        let Some(state) = self.tree_state(id) else { return Vec::new() };
        match parent {
            Some(p) => state.get(p).map(|n| n.children.clone()).unwrap_or_default(),
            None => state
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(key, slot)| {
                    let node = slot.as_ref()?;
                    node.parent.is_none().then_some(key)
                })
                .collect(),
        }
    }

    /// Returns the parent key of node `key`, if any.
    pub fn tree_parent(&self, id: WidgetId, key: usize) -> Option<usize> {
        self.tree_state(id)?.get(key)?.parent
    }

    /// Sets the open (expanded) state of node `key`. Returns true if it exists.
    pub fn tree_set_open(&mut self, id: WidgetId, key: usize, open: bool) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        let Some(node) = state.get_mut(key) else { return false };
        if node.open != open {
            node.open = open;
            state.dirty = true;
            self.dirty = true;
        }
        true
    }

    /// Returns whether node `key` is open (expanded).
    pub fn tree_is_open(&self, id: WidgetId, key: usize) -> bool {
        self.tree_state(id).and_then(|s| s.get(key)).is_some_and(|n| n.open)
    }

    /// Expands every node that has children.
    pub fn tree_expand_all(&mut self, id: WidgetId) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        for node in state.nodes.iter_mut().flatten() {
            if !node.children.is_empty() {
                node.open = true;
            }
        }
        state.dirty = true;
        self.dirty = true;
        true
    }

    /// Collapses every node.
    pub fn tree_collapse_all(&mut self, id: WidgetId) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        for node in state.nodes.iter_mut().flatten() {
            node.open = false;
        }
        state.dirty = true;
        self.dirty = true;
        true
    }

    /// Returns the selected node key, if any.
    pub fn tree_selection(&self, id: WidgetId) -> Option<usize> {
        self.tree_state(id)?.selected
    }

    /// Selects node `key` (or clears with `None`), expanding ancestors and
    /// scrolling it into view. Returns false if `key` is not a live node.
    pub fn tree_select(&mut self, id: WidgetId, key: Option<usize>) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        if let Some(k) = key {
            if state.get(k).is_none() {
                return false;
            }
            // Expand every ancestor so the node becomes visible.
            let mut ancestor = state.get(k).and_then(|n| n.parent);
            while let Some(a) = ancestor {
                if let Some(node) = state.get_mut(a) {
                    node.open = true;
                    ancestor = node.parent;
                } else {
                    break;
                }
            }
            state.dirty = true;
        }
        let old = state.selected;
        state.selected = key;
        self.dirty = true;
        if let Some(k) = key {
            self.tree_see(id, k);
        }
        if old != key {
            self.dispatch_tree_select(id, key);
        }
        true
    }

    /// Registers a callback fired when the selected node changes.
    pub fn tree_on_select(
        &mut self,
        id: WidgetId,
        cb: impl FnMut(&mut Tk, Option<usize>) + 'static,
    ) -> bool {
        let Some(state) = self.tree_state_mut(id) else { return false };
        state.on_select = Some(Box::new(cb));
        true
    }

    /// Number of rows currently shown (respecting open state).
    pub fn tree_visible_count(&mut self, id: WidgetId) -> usize {
        let Some(state) = self.tree_state_mut(id) else { return 0 };
        state.ensure_visible_robust();
        state.visible.len()
    }

    /// Display row currently shown at the top of the body.
    pub fn tree_first_visible(&mut self, id: WidgetId) -> usize {
        let Some(state) = self.tree_state_mut(id) else { return 0 };
        state.ensure_visible_robust();
        state.top
    }

    /// Scrolls so node `key` is visible.
    pub fn tree_see(&mut self, id: WidgetId, key: usize) -> bool {
        self.ensure_layout();
        let visible = self.tree_visible_rows(id).max(1);
        let Some(state) = self.tree_state_mut(id) else { return false };
        state.ensure_visible_robust();
        let Some(display) = state.display_of(key) else { return false };
        let top = state.top;
        let new_top = if display < top {
            display
        } else if display >= top + visible {
            display + 1 - visible
        } else {
            top
        };
        if let Some(state) = self.tree_state_mut(id) {
            state.top = new_top;
        }
        self.tree_clamp_top(id);
        self.dirty = true;
        true
    }

    /// Row height in logical px.
    pub fn tree_row_height(&self, id: WidgetId) -> f32 {
        self.tree_row_height_for(id)
    }

    /// Fires the selection callback (re-entrancy guarded).
    pub(crate) fn dispatch_tree_select(&mut self, id: WidgetId, sel: Option<usize>) {
        let Some(state) = self.tree_state_mut(id) else { return };
        if state.on_select_running {
            return;
        }
        let Some(mut cb) = state.on_select.take() else { return };
        state.on_select_running = true;
        cb(self, sel);
        if let Some(state) = self.tree_state_mut(id) {
            state.on_select_running = false;
            if state.on_select.is_none() {
                state.on_select = Some(cb);
            }
        }
    }

    /// Handles a mouse press at widget-local `(x, y)`. Returns true if consumed.
    pub(crate) fn tree_mouse_down(&mut self, id: WidgetId, x: f32, y: f32) -> bool {
        let row_h = self.tree_row_height_for(id);
        if row_h <= 0.0 {
            return false;
        }
        let Some(state) = self.tree_state_mut(id) else { return false };
        state.ensure_visible_robust();
        let top = state.top;
        let display = top + (y / row_h).floor().max(0.0) as usize;
        let Some(&(key, depth)) = state.visible.get(display) else { return true };
        let has_children = state.get(key).is_some_and(|n| !n.children.is_empty());
        let indent = depth as f32 * 16.0;
        // Click on the disclosure triangle area toggles open state.
        if has_children && x >= indent && x < indent + 16.0 {
            let open = state.get(key).is_some_and(|n| n.open);
            if let Some(node) = state.get_mut(key) {
                node.open = !open;
            }
            state.dirty = true;
            self.dirty = true;
            return true;
        }
        self.tree_select(id, Some(key));
        true
    }

    /// Scrolls the tree by `lines` rows (positive scrolls down).
    pub(crate) fn tree_scroll(&mut self, id: WidgetId, lines: f32) {
        let Some(state) = self.tree_state_mut(id) else { return };
        state.ensure_visible_robust();
        let delta = lines.round() as i64;
        let new_top = (state.top as i64 + delta).max(0) as usize;
        state.top = new_top;
        self.tree_clamp_top(id);
        self.dirty = true;
    }

    /// Handles a key press on a focused tree. Returns true if consumed.
    pub(crate) fn tree_key(&mut self, id: WidgetId, key: Key) -> bool {
        let visible = self.tree_visible_rows(id).max(1);
        let Some(state) = self.tree_state_mut(id) else { return false };
        state.ensure_visible_robust();
        let total = state.visible.len();
        if total == 0 {
            return false;
        }
        let cur_display = state
            .selected
            .and_then(|k| state.display_of(k))
            .unwrap_or(0);
        let cur_key = state.visible.get(cur_display).map(|&(k, _)| k);
        match key {
            Key::Up => {
                let new_display = cur_display.saturating_sub(1);
                if let Some(&(k, _)) = state.visible.get(new_display) {
                    self.tree_select(id, Some(k));
                }
            }
            Key::Down => {
                let new_display = (cur_display + 1).min(total - 1);
                if let Some(&(k, _)) = state.visible.get(new_display) {
                    self.tree_select(id, Some(k));
                }
            }
            Key::PageUp => {
                let new_display = cur_display.saturating_sub(visible);
                if let Some(&(k, _)) = state.visible.get(new_display) {
                    self.tree_select(id, Some(k));
                }
            }
            Key::PageDown => {
                let new_display = (cur_display + visible).min(total - 1);
                if let Some(&(k, _)) = state.visible.get(new_display) {
                    self.tree_select(id, Some(k));
                }
            }
            Key::Home => {
                if let Some(&(k, _)) = state.visible.first() {
                    self.tree_select(id, Some(k));
                }
            }
            Key::End => {
                if let Some(&(k, _)) = state.visible.last() {
                    self.tree_select(id, Some(k));
                }
            }
            Key::Right => {
                let Some(k) = cur_key else { return true };
                let (open, has_children, first_child) = {
                    let Some(node) = state.get(k) else { return true };
                    (node.open, !node.children.is_empty(), node.children.first().copied())
                };
                if has_children && !open {
                    if let Some(node) = state.get_mut(k) {
                        node.open = true;
                    }
                    state.dirty = true;
                    self.dirty = true;
                } else if open {
                    if let Some(child) = first_child {
                        self.tree_select(id, Some(child));
                    }
                }
            }
            Key::Left => {
                let Some(k) = cur_key else { return true };
                let (open, has_children, parent) = {
                    let Some(node) = state.get(k) else { return true };
                    (node.open, !node.children.is_empty(), node.parent)
                };
                if has_children && open {
                    if let Some(node) = state.get_mut(k) {
                        node.open = false;
                    }
                    state.dirty = true;
                    self.dirty = true;
                } else if let Some(p) = parent {
                    self.tree_select(id, Some(p));
                }
            }
            Key::Enter | Key::Space => {
                let Some(k) = cur_key else { return true };
                let has_children = state.get(k).is_some_and(|n| !n.children.is_empty());
                if has_children {
                    let open = state.get(k).is_some_and(|n| n.open);
                    if let Some(node) = state.get_mut(k) {
                        node.open = !open;
                    }
                    state.dirty = true;
                    self.dirty = true;
                }
            }
            _ => return false,
        }
        true
    }
}
