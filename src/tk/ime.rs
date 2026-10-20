//! Input-method (IME) composition support for Entry and Text widgets.
//!
//! The real window backend forwards `winit::event::Ime` events here via the
//! `sim_ime_*` harness methods; the same methods are used by headless tests.
//! A composition ("preedit") is stored on the toolkit, never written into the
//! widget's value or bound variable, and is rendered inline at the caret.

#[allow(unused_imports)]
use super::*;

/// Borrowed view of the active IME composition: widget, preedit text, byte cursor range.
pub type ImePreeditRef<'a> = (WidgetId, &'a str, Option<(usize, usize)>);

impl Tk {
    /// Returns true when the focused widget accepts IME composition: an enabled
    /// `Entry`, or an enabled, non-read-only `Text`.
    pub fn ime_wanted(&self) -> bool {
        let Some(id) = self.focused else { return false };
        let Some(node) = self.node(id) else { return false };
        if node.disabled {
            return false;
        }
        match node.kind {
            Kind::Entry => true,
            Kind::Text => !node.text_read_only,
            Kind::Combobox => node.combo_editable,
            _ => false,
        }
    }

    /// Returns the active composition as `(widget, text, cursor)`.
    ///
    /// A stored composition whose widget is no longer focused is treated as
    /// absent, so focus changes implicitly cancel it.
    pub fn ime_preedit(&self) -> Option<ImePreeditRef<'_>> {
        let (id, text, cursor) = self.ime_preedit.as_ref()?;
        if Some(*id) != self.focused {
            return None;
        }
        Some((*id, text.as_str(), *cursor))
    }

    /// Drops a stored composition that no longer belongs to the focused widget.
    fn drop_stale_preedit(&mut self) {
        if let Some((id, _, _)) = &self.ime_preedit {
            if Some(*id) != self.focused {
                self.ime_preedit = None;
            }
        }
    }

    /// Headless harness: applies an IME preedit (composition) update.
    ///
    /// Mirrors `winit::event::Ime::Preedit`. An empty `text` clears the
    /// composition. The byte cursor range is clamped to char boundaries. The
    /// widget's value and any bound variable are left untouched.
    #[doc(hidden)]
    pub fn sim_ime_preedit(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        self.ensure_layout();
        self.drop_stale_preedit();
        if !self.ime_wanted() {
            self.ime_preedit = None;
            return;
        }
        let Some(id) = self.focused else { return };
        if text.is_empty() {
            self.ime_preedit = None;
            self.dirty = true;
            return;
        }
        let cursor = cursor.map(|(start, end)| {
            let start = floor_char_boundary(text, start.min(text.len()));
            let end = floor_char_boundary(text, end.min(text.len()));
            (start.min(end), start.max(end))
        });
        self.ime_preedit = Some((id, text.to_owned(), cursor));
        self.dirty = true;
    }

    /// Headless harness: applies an IME commit, inserting `text` at the caret.
    ///
    /// Mirrors `winit::event::Ime::Commit`. The composition is cleared first and
    /// the text is inserted through the same helpers as `sim_text`, so variable
    /// bindings, line limits and callbacks behave identically.
    #[doc(hidden)]
    pub fn sim_ime_commit(&mut self, text: &str) {
        self.ensure_layout();
        self.drop_stale_preedit();
        self.ime_preedit = None;
        if text.is_empty() {
            self.dirty = true;
            return;
        }
        let Some(id) = self.focused else { return };
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        match node.kind {
            Kind::Entry => self.edit_entry(id, None, text),
            Kind::Text if !node.text_read_only => self.edit_text(id, None, text),
            Kind::Combobox if node.combo_editable => self.edit_entry(id, None, text),
            _ => {}
        }
        self.dirty = true;
    }

    /// Returns the logical-pixel rectangle of the caret of the focused
    /// Entry/Text, for positioning the IME candidate window.
    ///
    /// The rectangle is approximate: the widget's left padding plus the measured
    /// width of the text before the caret, with a height of `font_size * 1.3`.
    /// Returns `None` when [`Tk::ime_wanted`] is false.
    pub fn ime_cursor_area(&mut self) -> Option<crate::layout::Rect> {
        if !self.ime_wanted() {
            return None;
        }
        self.ensure_layout();
        let id = self.focused?;
        let node = self.node(id)?;
        let kind = node.kind;
        let rect = node.rect;
        let cursor = node.cursor;
        let scroll_x = node.scroll_x;
        let scroll_row = node.text_scroll_row;
        let value = node.text.clone();
        let resolved = self.resolved_style(id).unwrap_or_default();
        let line_h = (resolved.font_size * 1.3).max(1.0);
        let caret = floor_char_boundary(&value, cursor);
        match kind {
            Kind::Entry | Kind::Combobox => {
                let before = value.get(..caret).unwrap_or_default();
                let width = self.measure_with(before, &resolved).width;
                let x = rect.x + 6.0 + width - scroll_x.max(0.0);
                let y = rect.y + (rect.height - line_h) / 2.0;
                Some(crate::layout::Rect::new(x, y, 1.0, line_h))
            }
            Kind::Text => {
                let before = value.get(..caret).unwrap_or_default();
                let line_index = before.matches('\n').count();
                let line_start = before.rfind('\n').map_or(0, |i| i + 1);
                let col = before.get(line_start..).unwrap_or_default();
                let width = self.measure_with(col, &resolved).width;
                let row = line_index.saturating_sub(scroll_row);
                let x = rect.x + 4.0 + width;
                let y = rect.y + 4.0 + row as f32 * (line_h + 4.0).max(16.0);
                Some(crate::layout::Rect::new(x, y, 1.0, line_h))
            }
            _ => None,
        }
    }
}
