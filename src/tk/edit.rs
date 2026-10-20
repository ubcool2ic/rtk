//! Selection, undo/redo and clipboard operations for Entry and Text widgets.
//!
//! Selection is stored as an optional anchor byte index on the widget node;
//! the live selection is `anchor..cursor` normalized to char/cluster
//! boundaries. Undo/redo keeps a bounded stack of [`EditSnapshot`]s.

#[allow(unused_imports)]
use super::*;

use crate::widgets::EditSnapshot;

/// Maximum number of undo snapshots retained per widget.
const UNDO_LIMIT: usize = 100;
/// Window (ms) within which consecutive single-character typing coalesces.
const COALESCE_MS: u64 = 1000;

impl Tk {
    /// Returns the current selection of an Entry as `(start, end)` byte offsets.
    pub fn entry_selection(&self, id: WidgetId) -> Option<(usize, usize)> {
        self.selection_of(id)
    }

    /// Selects the byte range `start..end` in an Entry (clamped to boundaries).
    pub fn entry_select_range(&mut self, id: WidgetId, start: usize, end: usize) {
        self.select_range(id, start, end);
    }

    /// Returns the selected text of an Entry, or `None` if nothing is selected.
    pub fn entry_selected_text(&self, id: WidgetId) -> Option<String> {
        self.selected_text_of(id)
    }

    /// Returns the current selection of a Text widget as `(start, end)`.
    pub fn text_selection(&self, id: WidgetId) -> Option<(usize, usize)> {
        self.selection_of(id)
    }

    /// Selects the byte range `start..end` in a Text widget.
    pub fn text_select_range(&mut self, id: WidgetId, start: usize, end: usize) {
        self.select_range(id, start, end);
    }

    /// Returns the selected text of a Text widget, or `None`.
    pub fn text_selected_text(&self, id: WidgetId) -> Option<String> {
        self.selected_text_of(id)
    }

    /// Clears any selection on `id` without moving the cursor.
    pub fn selection_clear(&mut self, id: WidgetId) {
        if let Some(node) = self.node_mut(id) {
            node.selection_anchor = None;
            self.dirty = true;
        }
    }

    /// Undoes the last edit on `id`. Returns true if anything changed.
    pub fn edit_undo(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        let Some(prev) = node.undo_stack.pop() else { return false };
        let current = EditSnapshot {
            text: node.text.clone(),
            cursor: node.cursor,
            anchor: node.selection_anchor,
        };
        node.redo_stack.push(current);
        if node.redo_stack.len() > UNDO_LIMIT {
            node.redo_stack.remove(0);
        }
        node.text = prev.text;
        node.cursor = prev.cursor.min(node.text.len());
        node.selection_anchor = prev.anchor.map(|a| a.min(node.text.len()));
        node.last_edit_typing = false;
        self.dirty = true;
        self.sync_after_edit(id);
        true
    }

    /// Redoes the last undone edit on `id`. Returns true if anything changed.
    pub fn edit_redo(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        let Some(next) = node.redo_stack.pop() else { return false };
        let current = EditSnapshot {
            text: node.text.clone(),
            cursor: node.cursor,
            anchor: node.selection_anchor,
        };
        node.undo_stack.push(current);
        if node.undo_stack.len() > UNDO_LIMIT {
            node.undo_stack.remove(0);
        }
        node.text = next.text;
        node.cursor = next.cursor.min(node.text.len());
        node.selection_anchor = next.anchor.map(|a| a.min(node.text.len()));
        node.last_edit_typing = false;
        self.dirty = true;
        self.sync_after_edit(id);
        true
    }

    /// Clears the undo/redo history of `id`.
    pub fn edit_reset(&mut self, id: WidgetId) {
        if let Some(node) = self.node_mut(id) {
            node.undo_stack.clear();
            node.redo_stack.clear();
            node.last_edit_typing = false;
        }
    }

    /// Returns the normalized selection `(start, end)` of `id`, if any.
    pub(crate) fn selection_of(&self, id: WidgetId) -> Option<(usize, usize)> {
        let node = self.node(id)?;
        let anchor = node.selection_anchor?;
        let cursor = node.cursor;
        if anchor == cursor {
            return None;
        }
        Some((anchor.min(cursor), anchor.max(cursor)))
    }

    /// Returns the selected substring of `id`, if any.
    pub(crate) fn selected_text_of(&self, id: WidgetId) -> Option<String> {
        let (start, end) = self.selection_of(id)?;
        let node = self.node(id)?;
        node.text.get(start..end).map(str::to_owned)
    }

    /// Sets the selection to `start..end` (clamped to char boundaries).
    pub(crate) fn select_range(&mut self, id: WidgetId, start: usize, end: usize) {
        let Some(node) = self.node_mut(id) else { return };
        let len = node.text.len();
        let s = floor_char_boundary(&node.text, start.min(len));
        let e = floor_char_boundary(&node.text, end.min(len));
        node.selection_anchor = Some(s);
        node.cursor = e;
        self.dirty = true;
    }

    /// Selects the entire text of `id`.
    pub(crate) fn select_all(&mut self, id: WidgetId) {
        let Some(node) = self.node_mut(id) else { return };
        node.selection_anchor = Some(0);
        node.cursor = node.text.len();
        self.dirty = true;
    }

    /// Copies the current selection of `id` to the clipboard. Returns true if
    /// there was a selection.
    pub(crate) fn copy_selection(&mut self, id: WidgetId) -> bool {
        if let Some(text) = self.selected_text_of(id) {
            self.clipboard_clear();
            self.clipboard_append(&text);
            true
        } else {
            false
        }
    }

    /// Cuts the current selection of `id` to the clipboard. Returns true if
    /// there was a selection and the widget is editable.
    pub(crate) fn cut_selection(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind == Kind::Text && node.text_read_only {
            return false;
        }
        let Some(text) = self.selected_text_of(id) else { return false };
        self.clipboard_clear();
        self.clipboard_append(&text);
        self.delete_selection(id);
        true
    }

    /// Pastes the clipboard into `id`, replacing any selection. Returns true if
    /// anything was inserted.
    pub(crate) fn paste_clipboard(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        let kind = node.kind;
        if node.kind == Kind::Text && node.text_read_only {
            return false;
        }
        let Some(raw) = self.clipboard_get() else { return false };
        let value = if kind == Kind::Entry {
            clean_entry(&raw)
        } else {
            raw
        };
        if value.is_empty() {
            return false;
        }
        self.replace_selection_with(id, &value, false);
        true
    }

    /// Deletes the current selection of `id`, if any. Returns true if deleted.
    pub(crate) fn delete_selection(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind == Kind::Text && node.text_read_only {
            return false;
        }
        let Some((start, end)) = self.selection_of(id) else { return false };
        self.push_undo(id, false);
        if let Some(node) = self.node_mut(id) {
            node.text.replace_range(start..end, "");
            node.cursor = start;
            node.selection_anchor = None;
        }
        self.dirty = true;
        self.sync_after_edit(id);
        true
    }

    /// Replaces the current selection (or inserts at the cursor) with `value`.
    ///
    /// `typing` marks a single-character insertion eligible for coalescing.
    pub(crate) fn replace_selection_with(&mut self, id: WidgetId, value: &str, typing: bool) {
        let Some(node) = self.node(id) else { return };
        let (start, end) = match self.selection_of(id) {
            Some((s, e)) => (s, e),
            None => (node.cursor, node.cursor),
        };
        self.push_undo(id, typing);
        if let Some(node) = self.node_mut(id) {
            let mut new_text = String::with_capacity(node.text.len() + value.len());
            new_text.push_str(node.text.get(..start).unwrap_or_default());
            new_text.push_str(value);
            new_text.push_str(node.text.get(end..).unwrap_or_default());
            node.text = new_text;
            node.cursor = start + value.len();
            node.selection_anchor = None;
        }
        self.dirty = true;
        self.sync_after_edit(id);
    }

    /// Pushes the current state onto the undo stack, coalescing typing edits.
    ///
    /// A single-character insertion within [`COALESCE_MS`] of the previous
    /// typing edit (and not preceded by whitespace) reuses the existing step.
    pub(crate) fn push_undo(&mut self, id: WidgetId, typing: bool) {
        let now = self.now_ms();
        let Some(node) = self.node_mut(id) else { return };
        let coalesce = typing
            && node.last_edit_typing
            && now.saturating_sub(node.last_edit_ms) <= COALESCE_MS
            && !node.text.ends_with(char::is_whitespace);
        if !coalesce {
            let snapshot = EditSnapshot {
                text: node.text.clone(),
                cursor: node.cursor,
                anchor: node.selection_anchor,
            };
            node.undo_stack.push(snapshot);
            if node.undo_stack.len() > UNDO_LIMIT {
                node.undo_stack.remove(0);
            }
        }
        node.redo_stack.clear();
        node.last_edit_typing = typing;
        node.last_edit_ms = now;
    }

    /// Returns the toolkit's monotonic time in milliseconds.
    pub(crate) fn now_ms(&self) -> u64 {
        crate::app::START
            .get()
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    /// After an edit, syncs a Var-bound Entry and clamps the cursor.
    pub(crate) fn sync_after_edit(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        let kind = node.kind;
        let text = node.text.clone();
        let cursor = node.cursor.min(text.len());
        if let Some(node) = self.node_mut(id) {
            node.cursor = cursor;
        }
        if kind == Kind::Entry {
            self.write_entry(id, text);
        }
    }

    /// Extends the selection of `id` to `new_cursor`, keeping the anchor.
    pub(crate) fn extend_selection_to(&mut self, id: WidgetId, new_cursor: usize) {
        let Some(node) = self.node_mut(id) else { return };
        if node.selection_anchor.is_none() {
            node.selection_anchor = Some(node.cursor);
        }
        node.cursor = new_cursor.min(node.text.len());
        self.dirty = true;
    }

    /// Moves the cursor of `id` to `new_cursor`, clearing any selection.
    pub(crate) fn move_cursor_to(&mut self, id: WidgetId, new_cursor: usize) {
        let Some(node) = self.node_mut(id) else { return };
        node.cursor = new_cursor.min(node.text.len());
        node.selection_anchor = None;
        self.dirty = true;
    }
}
