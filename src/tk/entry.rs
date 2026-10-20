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
        let Some(text) = self.entry_get(id) else { return };
        let Some(node) = self.node(id) else { return };
        let boundaries = clusters(&text);
        let mut cursor = cursor_boundary(&text, node.cursor);
        let previous = boundaries.iter().copied().take_while(|b| *b < cursor).last().unwrap_or(0);
        let next = boundaries.iter().copied().find(|b| *b > cursor).unwrap_or(text.len());
        let mut replacement = None;
        match key {
            Some(Key::Left) => cursor = previous,
            Some(Key::Right) => cursor = next,
            Some(Key::Home) => cursor = 0,
            Some(Key::End) => cursor = text.len(),
            Some(Key::Backspace) if cursor > 0 => {
                replacement = Some(join_text(&text, previous, cursor, ""));
                cursor = previous;
            }
            Some(Key::Delete) if cursor < text.len() => {
                replacement = Some(join_text(&text, cursor, next, ""));
            }
            None => {
                let value = clean_entry_limit(inserted, MAX_ENTRY_BYTES.saturating_sub(text.len()));
                if !value.is_empty() {
                    replacement = Some(join_text(&text, cursor, cursor, &value));
                    cursor += value.len();
                }
            }
            _ => {}
        }
        if let Some(new_text) = replacement {
            cursor = cursor_boundary(&new_text, cursor);
            if let Some(node) = self.node_mut(id) {
                node.cursor = cursor;
            }
            self.write_entry(id, new_text);
        } else if let Some(node) = self.node_mut(id) {
            node.cursor = cursor;
        }
        self.dirty = true;
    }

}
