//! Keyboard shortcut binding and modifier-aware key dispatch.

#[allow(unused_imports)]
use super::*;

use crate::shortcut::{Modifiers, Shortcut, ShortcutError, ShortcutId};

impl Tk {
    /// Binds an accelerator string (e.g. `"Ctrl+S"`) to a callback.
    ///
    /// Returns a [`ShortcutId`] that can be passed to [`Tk::unbind_shortcut`].
    /// If the same chord is bound more than once, the most recently bound
    /// callback wins.
    pub fn bind_shortcut(
        &mut self,
        accel: &str,
        callback: impl FnMut(&mut Tk) + 'static,
    ) -> Result<ShortcutId, ShortcutError> {
        let shortcut = Shortcut::parse(accel)?;
        let id = ShortcutId(self.next_shortcut_id);
        self.next_shortcut_id = self.next_shortcut_id.wrapping_add(1);
        self.shortcuts.push((id, shortcut, Box::new(callback)));
        self.dirty = true;
        Ok(id)
    }

    /// Removes a previously bound shortcut. Returns true if it existed.
    pub fn unbind_shortcut(&mut self, id: ShortcutId) -> bool {
        let before = self.shortcuts.len();
        self.shortcuts.retain(|(sid, _, _)| *sid != id);
        let removed = self.shortcuts.len() != before;
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// Returns every currently bound shortcut, in binding order.
    pub fn shortcuts(&self) -> Vec<(ShortcutId, Shortcut)> {
        self.shortcuts.iter().map(|(id, sc, _)| (*id, *sc)).collect()
    }

    /// Returns the modifier set of the most recent key press.
    pub fn modifiers(&self) -> Modifiers {
        self.mods
    }

    /// Headless harness: delivers a key press with the given modifiers.
    ///
    /// Dispatch order:
    /// 1. If an Entry/Text has focus and the chord is a text-editing chord
    ///    (primary + a/c/x/v/z/y, primary+shift+z), it is left for the widget.
    /// 2. Otherwise a matching bound shortcut runs (last bound wins).
    /// 3. Otherwise falls through to [`Tk::sim_key`] for non-`Char` keys.
    #[doc(hidden)]
    pub fn sim_key_with(&mut self, key: Key, mods: Modifiers) {
        self.ensure_layout();
        self.mods = mods;

        // (a) Text-editing chords are handled by the focused Entry/Text widget
        // (or an editable Combobox, which behaves like an Entry).
        if self.is_text_editing_chord(key, mods) {
            if let Some(id) = self.focused {
                if self.node(id).is_some_and(|n| {
                    matches!(n.kind, Kind::Entry | Kind::Text)
                        || (n.kind == Kind::Combobox && n.combo_editable)
                }) {
                    self.handle_edit_chord(id, key, mods);
                    return;
                }
            }
        }

        // (b) Bound shortcut (last bound wins). Ignored while a modal dialog is
        // open unless the shortcut is Escape-free (i.e. does not use Escape).
        let modal_open = self.modal.is_some();
        let mut matched: Option<usize> = None;
        for (idx, (_, sc, _)) in self.shortcuts.iter().enumerate().rev() {
            if sc.key == key && sc.mods == mods {
                if modal_open && sc.key == Key::Escape {
                    continue;
                }
                matched = Some(idx);
                break;
            }
        }
        if let Some(idx) = matched {
            let mut entry = self.shortcuts.remove(idx);
            let mut cb = std::mem::replace(&mut entry.2, Box::new(|_| {}));
            cb(self);
            // If the callback did not unbind itself, put it back.
            let still_bound = self.shortcuts.iter().any(|(sid, _, _)| *sid == entry.0);
            if !still_bound {
                self.shortcuts.insert(idx.min(self.shortcuts.len()), entry);
            }
            self.dirty = true;
            return;
        }

        // (c) Fall through to existing handling for non-Char keys.
        if !matches!(key, Key::Char(_)) {
            self.sim_key_raw(key);
        }
    }

    /// Returns true if `(key, mods)` is a text-editing chord that should be
    /// handled by the focused Entry/Text widget rather than a shortcut.
    fn is_text_editing_chord(&self, key: Key, mods: Modifiers) -> bool {
        if !mods.is_primary() {
            return false;
        }
        matches!(key, Key::Char('a' | 'c' | 'x' | 'v' | 'z' | 'y'))
    }

    /// Handles a text-editing chord (select-all, copy, cut, paste, undo, redo)
    /// for the focused Entry/Text widget `id`.
    fn handle_edit_chord(&mut self, id: WidgetId, key: Key, mods: Modifiers) {
        let Some(node) = self.node(id) else { return };
        let is_text = node.kind == Kind::Text;
        let read_only = is_text && node.text_read_only;
        match key {
            Key::Char('a') => self.select_all(id),
            Key::Char('c') => {
                self.copy_selection(id);
            }
            Key::Char('x') => {
                if !read_only {
                    self.cut_selection(id);
                }
            }
            Key::Char('v') => {
                if !read_only {
                    self.paste_clipboard(id);
                }
            }
            Key::Char('z') => {
                if mods.shift {
                    self.edit_redo(id);
                } else {
                    self.edit_undo(id);
                }
            }
            Key::Char('y') => {
                self.edit_redo(id);
            }
            _ => {}
        }
    }
}
