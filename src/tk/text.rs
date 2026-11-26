//! Text widget API and editing.

#[allow(unused_imports)]
use super::*;

/// Trims the text to at most the given number of lines, dropping the oldest lines first.
pub(crate) fn trim_lines(text: &mut String, max: usize) {
    // 0 means "unlimited", matching `text_set_max_lines`.
    if max == 0 {
        return;
    }
    let newlines = text.bytes().filter(|&b| b == b'\n').count();
    if newlines + 1 > max {
        let drop_count = newlines + 1 - max;
        if let Some((pos, _)) = text.match_indices('\n').nth(drop_count - 1) {
            text.drain(..=pos);
        }
    }
}

impl Tk {
    /// Edits a Text widget's contents in response to a key or inserted text, without extending the selection.
    pub(crate) fn edit_text(&mut self, id: WidgetId, key: Option<Key>, inserted: &str) {
        self.edit_text_ext(id, key, inserted, false);
    }

    /// Edits a Text widget, optionally extending the selection (Shift held).
    pub(crate) fn edit_text_ext(
        &mut self,
        id: WidgetId,
        key: Option<Key>,
        inserted: &str,
        extend: bool,
    ) {
        let selection = self.selection_of(id);
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Text {
            return;
        }
        let read_only = node.text_read_only;
        let vis_rows = node.text_dimensions.1;
        let mut cursor = floor_char_boundary(&node.text, node.cursor);

        enum Action {
            None,
            Move(usize),
            Extend(usize),
            MoveTo(usize),
        }

        let action = {
            let text = node.text.as_str();
            match key {
                Some(Key::Left) => {
                    if extend {
                        let target = if cursor > 0 {
                            text[..cursor]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        Action::Extend(target)
                    } else if let Some((s, _)) = selection {
                        Action::MoveTo(s)
                    } else {
                        let target = if cursor > 0 {
                            text[..cursor]
                                .char_indices()
                                .last()
                                .map(|(i, _)| i)
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        Action::Move(target)
                    }
                }
                Some(Key::Right) => {
                    if extend {
                        let target = if cursor < text.len() {
                            text[cursor..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| cursor + i)
                                .unwrap_or(text.len())
                        } else {
                            text.len()
                        };
                        Action::Extend(target)
                    } else if let Some((_, e)) = selection {
                        Action::MoveTo(e)
                    } else {
                        let target = if cursor < text.len() {
                            text[cursor..]
                                .char_indices()
                                .nth(1)
                                .map(|(i, _)| cursor + i)
                                .unwrap_or(text.len())
                        } else {
                            text.len()
                        };
                        Action::Move(target)
                    }
                }
                Some(Key::Home) => {
                    let start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                    if extend {
                        Action::Extend(start)
                    } else {
                        Action::Move(start)
                    }
                }
                Some(Key::End) => {
                    let end = text[cursor..].find('\n').map_or(text.len(), |i| cursor + i);
                    if extend {
                        Action::Extend(end)
                    } else {
                        Action::Move(end)
                    }
                }
                Some(Key::Up) => {
                    let line_start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                    let col = text[line_start..cursor].chars().count();
                    let mut target = cursor;
                    if line_start > 0 {
                        let prev_start = text[..line_start - 1].rfind('\n').map_or(0, |i| i + 1);
                        let prev_line = &text[prev_start..line_start - 1];
                        let mut new_c = prev_start;
                        for (i, ch) in prev_line.chars().enumerate() {
                            if i >= col {
                                break;
                            }
                            new_c += ch.len_utf8();
                        }
                        target = new_c;
                    }
                    if extend {
                        Action::Extend(target)
                    } else {
                        Action::Move(target)
                    }
                }
                Some(Key::Down) => {
                    let line_start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                    let col = text[line_start..cursor].chars().count();
                    let next_nl = text[cursor..].find('\n').map(|i| cursor + i);
                    let mut target = cursor;
                    if let Some(nl) = next_nl {
                        let next_start = nl + 1;
                        let next_end = text[next_start..]
                            .find('\n')
                            .map_or(text.len(), |i| next_start + i);
                        let next_line = &text[next_start..next_end];
                        let mut new_c = next_start;
                        for (i, ch) in next_line.chars().enumerate() {
                            if i >= col {
                                break;
                            }
                            new_c += ch.len_utf8();
                        }
                        target = new_c;
                    }
                    if extend {
                        Action::Extend(target)
                    } else {
                        Action::Move(target)
                    }
                }
                _ => Action::None,
            }
        };

        match action {
            Action::Extend(target) => {
                self.extend_selection_to(id, target);
                return;
            }
            Action::MoveTo(target) => {
                self.move_cursor_to(id, target);
                return;
            }
            Action::Move(target) => {
                if let Some(node) = self.node_mut(id) {
                    let moved_line = node.text[..target].bytes().filter(|&b| b == b'\n').count();
                    node.cursor = target;
                    node.selection_anchor = None;
                    if moved_line < node.text_scroll_row {
                        node.text_scroll_row = moved_line;
                    } else if vis_rows > 0 && moved_line >= node.text_scroll_row + vis_rows {
                        node.text_scroll_row = moved_line + 1 - vis_rows;
                    }
                }
                self.invalidate_widget(id);
                return;
            }
            Action::None => {}
        }

        let mut edited = false;
        match key {
            Some(Key::Enter) if !read_only => {
                if selection.is_some() {
                    self.replace_selection_with(id, "\n", false);
                    return;
                }
                self.push_undo(id, false);
                if let Some(node) = self.node_mut(id) {
                    node.text.insert(cursor, '\n');
                    cursor += 1;
                    node.cursor = cursor;
                    node.selection_anchor = None;
                }
                edited = true;
            }
            Some(Key::Backspace) if !read_only => {
                if selection.is_some() {
                    self.delete_selection(id);
                    return;
                }
                if cursor > 0 {
                    self.push_undo(id, false);
                    if let Some(node) = self.node_mut(id) {
                        let prev = node.text[..cursor]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        node.text.remove(prev);
                        cursor = prev;
                        node.cursor = cursor;
                        node.selection_anchor = None;
                    }
                    edited = true;
                }
            }
            Some(Key::Delete) if !read_only => {
                if selection.is_some() {
                    self.delete_selection(id);
                    return;
                }
                let len = self.node(id).map_or(0, |n| n.text.len());
                if cursor < len {
                    self.push_undo(id, false);
                    if let Some(node) = self.node_mut(id) {
                        node.text.remove(cursor);
                        node.cursor = cursor;
                        node.selection_anchor = None;
                    }
                    edited = true;
                }
            }
            None if !read_only && !inserted.is_empty() => {
                let typing = inserted.chars().count() == 1;
                self.replace_selection_with(id, inserted, typing);
                return;
            }
            _ => {}
        }

        if edited {
            self.sync_after_edit(id);
        }
        // Scrolling and text/caret changes are confined to the widget's own
        // rect, so a partial redraw of just this widget suffices.
        self.invalidate_widget(id);
    }

    // -----------------------------------------------------------------------
    // Text widget API
    // -----------------------------------------------------------------------

    /// Returns the full text contents (tkinter `get("1.0", "end")`).
    pub fn text_get(&self, id: WidgetId) -> Option<String> {
        self.node(id)
            .filter(|n| n.kind == Kind::Text)
            .map(|n| n.text.clone())
    }

    /// Replaces the contents, moves the cursor to the end and resets undo.
    pub fn text_set(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        if node.kind != Kind::Text {
            return false;
        }
        let mut s = text.into();
        if let Some(max) = node.text_max_lines {
            trim_lines(&mut s, max);
        }
        node.text = s;
        node.cursor = node.text.len();
        node.selection_anchor = None;
        let last_row = node.text.split('\n').count() - 1;
        node.text_scroll_row = node.text_scroll_row.min(last_row);
        self.dirty = true;
        self.edit_reset(id);
        true
    }

    /// Appends `text` at the end, trimming to the max line count and
    /// auto-scrolling to the bottom when enabled.
    pub fn text_insert_end(&mut self, id: WidgetId, text: &str) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        if node.kind != Kind::Text {
            return false;
        }
        node.text.push_str(text);
        if let Some(max) = node.text_max_lines {
            trim_lines(&mut node.text, max);
        }
        if node.text_autoscroll {
            let total = node.text.split('\n').count();
            let vis = node.text_dimensions.1;
            node.text_scroll_row = total.saturating_sub(vis);
        } else {
            let last_row = node.text.split('\n').count() - 1;
            node.text_scroll_row = node.text_scroll_row.min(last_row);
        }
        node.cursor = node.text.len();
        node.selection_anchor = None;
        self.dirty = true;
        true
    }

    /// Removes all text and resets the cursor and scroll position.
    pub fn text_clear(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        if node.kind != Kind::Text {
            return false;
        }
        node.text.clear();
        node.cursor = 0;
        node.selection_anchor = None;
        node.text_scroll_row = 0;
        self.dirty = true;
        true
    }

    /// Enables or disables user editing (tkinter `state="disabled"`).
    pub fn text_set_readonly(&mut self, id: WidgetId, read_only: bool) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        if node.kind != Kind::Text {
            return false;
        }
        node.text_read_only = read_only;
        true
    }

    /// Number of lines (newline-separated) in the text.
    pub fn text_line_count(&self, id: WidgetId) -> Option<usize> {
        self.node(id)
            .filter(|n| n.kind == Kind::Text)
            .map(|n| n.text.split('\n').count())
    }

    /// Caps the retained line count, trimming the oldest lines immediately.
    pub fn text_set_max_lines(&mut self, id: WidgetId, max_lines: usize) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        if node.kind != Kind::Text {
            return false;
        }
        node.text_max_lines = Some(max_lines);
        if max_lines > 0 {
            let old_len = node.text.len();
            trim_lines(&mut node.text, max_lines);
            let removed = old_len - node.text.len();
            node.cursor = node.cursor.saturating_sub(removed).min(node.text.len());
            node.selection_anchor = None;
            let last_row = node.text.split('\n').count() - 1;
            node.text_scroll_row = node.text_scroll_row.min(last_row);
        }
        self.dirty = true;
        true
    }
}
