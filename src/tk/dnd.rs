//! File drag-and-drop: binding callbacks, hit testing and headless simulation.

#[allow(unused_imports)]
use super::*;
use std::path::PathBuf;

use crate::dnd::{DropPhase, FileDropCallback, FileHoverCallback};

impl Tk {
    /// Binds `cb` to receive files dropped on `id` (any live widget, including
    /// the root). Replaces an existing drop binding. Returns false if `id` is
    /// not live.
    pub fn bind_file_drop(
        &mut self,
        id: WidgetId,
        cb: impl FnMut(&mut Tk, &[PathBuf]) + 'static,
    ) -> bool {
        if self.node(id).is_none() {
            return false;
        }
        let cb: FileDropCallback = Box::new(cb);
        if let Some(slot) = self.dnd.drop.iter_mut().find(|(wid, _)| *wid == id) {
            slot.1 = Some(cb);
        } else {
            self.dnd.drop.push((id, Some(cb)));
        }
        true
    }

    /// Removes the drop binding of `id`. Returns false if there was none.
    pub fn unbind_file_drop(&mut self, id: WidgetId) -> bool {
        let before = self.dnd.drop.len();
        self.dnd.drop.retain(|(wid, _)| *wid != id);
        self.dnd.drop.len() != before
    }

    /// Binds `cb` to receive hover phase changes (Enter/Leave/Drop) for a file
    /// drag over `id`. Replaces an existing hover binding. Returns false if
    /// `id` is not live.
    pub fn bind_file_hover(
        &mut self,
        id: WidgetId,
        cb: impl FnMut(&mut Tk, DropPhase) + 'static,
    ) -> bool {
        if self.node(id).is_none() {
            return false;
        }
        let cb: FileHoverCallback = Box::new(cb);
        if let Some(slot) = self.dnd.hover.iter_mut().find(|(wid, _)| *wid == id) {
            slot.1 = Some(cb);
        } else {
            self.dnd.hover.push((id, Some(cb)));
        }
        true
    }

    /// Returns the deepest live, visible widget whose clip contains logical
    /// `(x, y)`, walking `paint_order` in reverse. Unlike `hit_test` this does
    /// not require the widget to be interactive, so Frames, Labels, Images and
    /// Charts are found too. When a modal is active, only the modal and its
    /// descendants are considered.
    fn deepest_widget_at(&self, x: f32, y: f32) -> Option<WidgetId> {
        for &id in self.paint_order.iter().rev() {
            let Some(node) = self.node(id) else { continue };
            if !node.visible || !node.clip.contains(x, y) {
                continue;
            }
            if let Some(modal_id) = self.modal {
                if id != modal_id && !self.is_descendant_of(id, modal_id) {
                    continue;
                }
            }
            return Some(id);
        }
        None
    }

    /// Returns the nearest widget at logical `(x, y)` that has a drop binding,
    /// walking up from the deepest widget under the pointer to the root. Falls
    /// back to the root when the root itself is bound.
    pub fn drop_target_at(&self, x: f32, y: f32) -> Option<WidgetId> {
        let mut current = self.deepest_widget_at(x, y);
        while let Some(id) = current {
            if self.node(id).is_some() && self.has_dnd_binding(id) {
                return Some(id);
            }
            current = self.node(id).and_then(|node| node.parent);
        }
        let root = self.root();
        if self.node(root).is_some() && self.has_dnd_binding(root) {
            return Some(root);
        }
        None
    }

    /// Returns true when `id` has a file-drop or file-hover binding.
    fn has_dnd_binding(&self, id: WidgetId) -> bool {
        self.dnd.drop.iter().any(|(wid, _)| *wid == id) || self.dnd.hover.iter().any(|(wid, _)| *wid == id)
    }

    /// Returns the widget currently under a file drag, if any (for highlight styling).
    pub fn file_hover_target(&self) -> Option<WidgetId> {
        self.dnd.target
    }

    /// Returns the paths currently being dragged over the window.
    pub fn hovered_files(&self) -> &[PathBuf] {
        &self.dnd.hovered
    }

    /// Headless harness: reports that a file drag is hovering at logical
    /// `(x, y)` with `path`. Fires `Leave` on the previous target and `Enter`
    /// on the new one when the target changes.
    #[doc(hidden)]
    pub fn sim_file_hover(&mut self, x: f32, y: f32, path: impl Into<PathBuf>) {
        self.ensure_layout();
        let path = path.into();
        if !self.dnd.hovered.contains(&path) {
            self.dnd.hovered.push(path);
        }
        let new_target = self.drop_target_at(x, y);
        if new_target == self.dnd.target {
            self.dirty = true;
            return;
        }
        let old_target = self.dnd.target;
        self.dnd.target = new_target;
        if let Some(old) = old_target {
            self.fire_file_hover(old, DropPhase::Leave);
        }
        if let Some(new) = new_target {
            self.fire_file_hover(new, DropPhase::Enter);
        }
        self.dirty = true;
    }

    /// Headless harness: reports that the file drag left the window or was
    /// cancelled. Fires `Leave` on the current target and clears hover state.
    #[doc(hidden)]
    pub fn sim_file_hover_cancel(&mut self) {
        let old_target = self.dnd.target.take();
        self.dnd.hovered.clear();
        if let Some(old) = old_target {
            self.fire_file_hover(old, DropPhase::Leave);
        }
        self.dirty = true;
    }

    /// Headless harness: drops `paths` at logical `(x, y)`. Calls the drop
    /// callback of the nearest bound widget and fires `Drop` on its hover
    /// callback, then clears hover state. Returns whether a bound target was found.
    #[doc(hidden)]
    pub fn sim_file_drop(&mut self, x: f32, y: f32, paths: &[PathBuf]) -> bool {
        self.ensure_layout();
        let target = self.drop_target_at(x, y);
        self.dnd.target = None;
        self.dnd.hovered.clear();
        self.dirty = true;
        let Some(target) = target else { return false };
        self.fire_file_drop(target, paths);
        self.fire_file_hover(target, DropPhase::Drop);
        true
    }

    /// Runs the drop callback bound to `id`, if any, using the take-call-restore
    /// pattern so the callback may rebind or unbind itself.
    fn fire_file_drop(&mut self, id: WidgetId, paths: &[PathBuf]) -> bool {
        if self.node(id).is_none() {
            return false;
        }
        let Some(slot) = self.dnd.drop.iter_mut().find(|(wid, _)| *wid == id) else {
            return false;
        };
        let Some(mut cb) = slot.1.take() else { return false };
        cb(self, paths);
        if let Some(slot) = self.dnd.drop.iter_mut().find(|(wid, _)| *wid == id) {
            if slot.1.is_none() {
                slot.1 = Some(cb);
            }
        }
        true
    }

    /// Runs the hover callback bound to `id`, if any, using the take-call-restore
    /// pattern so the callback may rebind or unbind itself.
    fn fire_file_hover(&mut self, id: WidgetId, phase: DropPhase) {
        if self.node(id).is_none() {
            return;
        }
        let Some(slot) = self.dnd.hover.iter_mut().find(|(wid, _)| *wid == id) else {
            return;
        };
        let Some(mut cb) = slot.1.take() else { return };
        cb(self, phase);
        if let Some(slot) = self.dnd.hover.iter_mut().find(|(wid, _)| *wid == id) {
            if slot.1.is_none() {
                slot.1 = Some(cb);
            }
        }
    }
}
