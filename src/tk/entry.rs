//! Entry editing, cursor movement and text cluster helpers.

#[allow(unused_imports)]
use super::*;

pub(crate) fn join_text(text: &str, start: usize, end: usize, value: &str) -> String {
    let mut result = String::new();
    result.push_str(text.get(..start).unwrap_or_default());
    result.push_str(value);
    result.push_str(text.get(end..).unwrap_or_default());
    result
}

pub(crate) fn clean_entry_limit(text: &str, limit: usize) -> String {
    let mut result = String::new();
    for ch in text.chars().filter(|ch| !ch.is_control()) {
        if result.len() + ch.len_utf8() > limit {
            break;
        }
        result.push(ch);
    }
    result
}

pub(crate) fn clean_entry(text: &str) -> String {
    clean_entry_limit(text, MAX_ENTRY_BYTES)
}

pub(crate) fn clusters(text: &str) -> Vec<usize> {
    let mut result = vec![0];
    let mut previous = '\0';
    for (index, ch) in text.char_indices() {
        let extending = matches!(
            ch,
            '\u{0300}'..='\u{036f}'
                | '\u{fe0e}'..='\u{fe0f}'
                | '\u{1f3fb}'..='\u{1f3ff}'
                | '\u{200d}'
        );
        if index > 0 && !extending && previous != '\u{200d}' {
            result.push(index);
        }
        previous = ch;
    }
    if !text.is_empty() {
        result.push(text.len());
    }
    result
}

pub(crate) fn cursor_boundary(text: &str, cursor: usize) -> usize {
    clusters(text).into_iter().take_while(|boundary| *boundary <= cursor).last().unwrap_or(0)
}

impl Tk {
    pub(crate) fn edit_entry(&mut self, id: WidgetId, key: Option<Key>, inserted: &str) {
        self.edit_entry_ext(id, key, inserted, false);
    }

    /// Edits an Entry, optionally extending the selection (Shift held).
    pub(crate) fn edit_entry_ext(&mut self, id: WidgetId, key: Option<Key>, inserted: &str, extend: bool) {
        let Some(text) = self.entry_get(id) else { return };
        let Some(node) = self.node(id) else { return };
        let boundaries = clusters(&text);
        let mut cursor = cursor_boundary(&text, node.cursor);
        let previous = boundaries.iter().copied().take_while(|b| *b < cursor).last().unwrap_or(0);
        let next = boundaries.iter().copied().find(|b| *b > cursor).unwrap_or(text.len());
        let selection = self.selection_of(id);

        // Plain cursor moves with an active selection collapse to its edge.
        let mut replacement = None;
        match key {
            Some(Key::Left) => {
                if extend {
                    self.extend_selection_to(id, previous);
                    self.dirty = true;
                    return;
                }
                if let Some((s, _)) = selection {
                    self.move_cursor_to(id, s);
                    return;
                }
                cursor = previous;
            }
            Some(Key::Right) => {
                if extend {
                    self.extend_selection_to(id, next);
                    self.dirty = true;
                    return;
                }
                if let Some((_, e)) = selection {
                    self.move_cursor_to(id, e);
                    return;
                }
                cursor = next;
            }
            Some(Key::Home) => {
                if extend {
                    self.extend_selection_to(id, 0);
                    self.dirty = true;
                    return;
                }
                cursor = 0;
            }
            Some(Key::End) => {
                if extend {
                    self.extend_selection_to(id, text.len());
                    self.dirty = true;
                    return;
                }
                cursor = text.len();
            }
            Some(Key::Backspace) => {
                if selection.is_some() {
                    self.delete_selection(id);
                    return;
                }
                if cursor > 0 {
                    self.push_undo(id, false);
                    replacement = Some(join_text(&text, previous, cursor, ""));
                    cursor = previous;
                }
            }
            Some(Key::Delete) => {
                if selection.is_some() {
                    self.delete_selection(id);
                    return;
                }
                if cursor < text.len() {
                    self.push_undo(id, false);
                    replacement = Some(join_text(&text, cursor, next, ""));
                }
            }
            None => {
                let value = clean_entry_limit(inserted, MAX_ENTRY_BYTES.saturating_sub(text.len()));
                if !value.is_empty() {
                    let typing = value.chars().count() == 1;
                    self.replace_selection_with(id, &value, typing);
                    return;
                }
            }
            _ => {}
        }
        if let Some(new_text) = replacement {
            cursor = cursor_boundary(&new_text, cursor);
            if let Some(node) = self.node_mut(id) {
                node.cursor = cursor;
                node.selection_anchor = None;
            }
            self.write_entry(id, new_text);
        } else if let Some(node) = self.node_mut(id) {
            node.cursor = cursor;
            node.selection_anchor = None;
        }
        self.dirty = true;
    }

}
