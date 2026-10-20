//! Text widget API and editing.

#[allow(unused_imports)]
use super::*;

pub(crate) fn trim_lines(text: &mut String, max: usize) {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.len() > max {
        let drop_count = lines.len() - max;
        *text = lines[drop_count..].join("\n");
    }
}

impl Tk {
    pub(crate) fn edit_text(&mut self, id: WidgetId, key: Option<Key>, inserted: &str) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Text { return; }
        let read_only = node.text_read_only;
        let mut text = node.text.clone();
        // The stored cursor may be stale after programmatic text changes; never slice mid-char.
        let mut cursor = floor_char_boundary(&text, node.cursor);
        let vis_rows = node.text_dimensions.1;

        match key {
            Some(Key::Left) => {
                if cursor > 0 {
                    let mut prev = 0;
                    for (i, _) in text.char_indices() {
                        if i >= cursor { break; }
                        prev = i;
                    }
                    cursor = prev;
                }
            }
            Some(Key::Right) => {
                if cursor < text.len() {
                    cursor = text[cursor..].char_indices().nth(1).map(|(i, _)| cursor + i).unwrap_or(text.len());
                }
            }
            Some(Key::Home) => {
                let start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                cursor = start;
            }
            Some(Key::End) => {
                let end = text[cursor..].find('\n').map_or(text.len(), |i| cursor + i);
                cursor = end;
            }
            Some(Key::Up) => {
                let line_start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                let col = text[line_start..cursor].chars().count();
                if line_start > 0 {
                    let prev_start = text[..line_start - 1].rfind('\n').map_or(0, |i| i + 1);
                    let prev_line = &text[prev_start..line_start - 1];
                    let mut new_c = prev_start;
                    for (i, _) in prev_line.chars().enumerate() {
                        if i >= col { break; }
                        new_c += prev_line[new_c - prev_start..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                    }
                    cursor = new_c;
                }
            }
            Some(Key::Down) => {
                let line_start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                let col = text[line_start..cursor].chars().count();
                let next_nl = text[cursor..].find('\n').map(|i| cursor + i);
                if let Some(nl) = next_nl {
                    let next_start = nl + 1;
                    let next_end = text[next_start..].find('\n').map_or(text.len(), |i| next_start + i);
                    let next_line = &text[next_start..next_end];
                    let mut new_c = next_start;
                    for (i, _) in next_line.chars().enumerate() {
                        if i >= col { break; }
                        new_c += next_line[new_c - next_start..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                    }
                    cursor = new_c;
                }
            }
            Some(Key::Enter) if !read_only => {
                text.insert(cursor, '\n');
                cursor += 1;
            }
            Some(Key::Backspace) if !read_only && cursor > 0 => {
                let mut prev = 0;
                for (i, _) in text.char_indices() {
                    if i >= cursor { break; }
                    prev = i;
                }
                text.remove(prev);
                cursor = prev;
            }
            Some(Key::Delete) if !read_only && cursor < text.len() => {
                text.remove(cursor);
            }
            None if !read_only && !inserted.is_empty() => {
                text.insert_str(cursor, inserted);
                cursor += inserted.len();
            }
            _ => {}
        }

        let cur_line = text[..cursor].matches('\n').count();
        if let Some(node) = self.node_mut(id) {
            node.text = text;
            node.cursor = cursor;
            if cur_line < node.text_scroll_row {
                node.text_scroll_row = cur_line;
            } else if cur_line >= node.text_scroll_row + vis_rows {
                node.text_scroll_row = cur_line + 1 - vis_rows;
            }
        }
        self.dirty = true;
    }


    // -----------------------------------------------------------------------
    // Text widget API
    // -----------------------------------------------------------------------

    pub fn text_get(&self, id: WidgetId) -> Option<String> {
        self.node(id).filter(|n| n.kind == Kind::Text).map(|n| n.text.clone())
    }

    pub fn text_set(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        let mut s = text.into();
        if let Some(max) = node.text_max_lines {
            trim_lines(&mut s, max);
        }
        node.text = s;
        node.cursor = node.text.len();
        self.dirty = true;
        true
    }

    pub fn text_insert_end(&mut self, id: WidgetId, text: &str) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text.push_str(text);
        if let Some(max) = node.text_max_lines {
            trim_lines(&mut node.text, max);
        }
        if node.text_autoscroll {
            let total = node.text.split('\n').count();
            let vis = node.text_dimensions.1;
            node.text_scroll_row = total.saturating_sub(vis);
        }
        node.cursor = node.text.len();
        self.dirty = true;
        true
    }

    pub fn text_clear(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text.clear();
        node.cursor = 0;
        node.text_scroll_row = 0;
        self.dirty = true;
        true
    }

    pub fn text_set_readonly(&mut self, id: WidgetId, read_only: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text_read_only = read_only;
        true
    }

    pub fn text_line_count(&self, id: WidgetId) -> Option<usize> {
        self.node(id).filter(|n| n.kind == Kind::Text).map(|n| n.text.split('\n').count())
    }

    pub fn text_set_max_lines(&mut self, id: WidgetId, max_lines: usize) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text_max_lines = Some(max_lines);
        if max_lines > 0 {
            trim_lines(&mut node.text, max_lines);
            node.cursor = node.cursor.min(node.text.len());
        }
        self.dirty = true;
        true
    }

}
