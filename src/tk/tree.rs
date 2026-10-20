//! Widget tree: add, destroy, config, commands, text and selection.

#[allow(unused_imports)]
use super::*;

impl Tk {
    pub(crate) fn add(
        &mut self,
        kind: Kind,
        parent: WidgetId,
        text: String,
    ) -> Result<WidgetId, RtkError> {
        if !self.node(parent).is_some_and(|node| matches!(node.kind, Kind::Frame | Kind::Notebook)) {
            return Err(RtkError::InvalidParent);
        }
        let mut node = WidgetNode::new(kind, Some(parent), text);
        let is_hc = self.theme == Theme::HighContrast;
        let is_dark = matches!(self.theme, Theme::Dark | Theme::ModernDark);
        if is_hc {
            node.style.bg = Color::BLACK;
            node.style.fg = Color::WHITE;
        } else if is_dark {
            let (bg, fg) = dark_default_colors(kind);
            node.style.bg = bg;
            node.style.fg = fg;
        }
        let id = if let Some(index) = self.free.pop() {
            let Some(slot) = self.nodes.get_mut(index as usize) else {
                return Err(RtkError::ArenaFull);
            };
            let id = WidgetId { index, gen: slot.gen };
            slot.node = Some(node);
            id
        } else {
            let index = u32::try_from(self.nodes.len()).map_err(|_| RtkError::ArenaFull)?;
            self.nodes.push(WidgetSlot { gen: 0, node: Some(node) });
            WidgetId { index, gen: 0 }
        };
        if let Some(parent) = self.node_mut(parent) {
            parent.children.push(id);
        }
        self.dirty = true;
        Ok(id)
    }

    /// Destroys a widget and its whole subtree, releasing their IDs. Destroying
    /// the root requests quit. Returns false if `id` is not live.
    pub fn destroy(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        let parent = node.parent;
        if let Some(parent) = parent.and_then(|parent| self.node_mut(parent)) {
            parent.children.retain(|child| *child != id);
            if parent.kind == Kind::Notebook {
                if let Some(pos) = parent.notebook_pages.iter().position(|(pid, _)| *pid == id) {
                    parent.notebook_pages.remove(pos);
                    if pos < parent.notebook_selected {
                        parent.notebook_selected -= 1;
                    }
                    let last = parent.notebook_pages.len().saturating_sub(1);
                    parent.notebook_selected = parent.notebook_selected.min(last);
                }
            }
        }
        if self.modal.is_some_and(|m| self.is_descendant_of(m, id)) {
            self.modal = None;
        }
        if self.scrollbar_drag.is_some_and(|(sb, _, _)| self.is_descendant_of(sb, id)) {
            self.scrollbar_drag = None;
        }
        let mut pending = vec![id];
        while let Some(current) = pending.pop() {
            let Some(slot) = self.nodes.get_mut(current.index as usize) else { continue };
            if slot.gen != current.gen {
                continue;
            }
            let Some(node) = slot.node.take() else { continue };
            pending.extend(node.children);
            if let Some(next) = slot.gen.checked_add(1) {
                slot.gen = next;
                if current.index != 0 {
                    self.free.push(current.index);
                }
            }
            if self.focused == Some(current) {
                self.focused = None;
            }
            if self.hovered == Some(current) {
                self.hovered = None;
            }
            if self.pressed == Some(current) {
                self.pressed = None;
            }
            if self.drag_active == Some(current) {
                self.drag_active = None;
            }
            if self.open_combobox == Some(current) {
                self.open_combobox = None;
            }
        }
        if id == self.root() {
            self.quit();
        }
        self.dirty = true;
        true
    }

    /// Changes one option of a widget (tkinter `widget.config(...)`). Returns
    /// false if the widget is gone or the option is invalid for its kind.
    pub fn config(&mut self, id: WidgetId, option: Opt) -> bool {
        let Some(kind) = self.node(id).map(|node| node.kind) else { return false };
        match option {
            Opt::Text(text) if kind == Kind::Entry => return self.entry_set(id, text),
            Opt::Text(text) if kind == Kind::Text => return self.text_set(id, text),
            Opt::Text(text) => {
                if let Some(node) = self.node_mut(id) {
                    node.text = text;
                }
            }
            Opt::Bg(color) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.bg = color;
                    node.style.bg_overridden = true;
                }
            }
            Opt::Fg(color) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.fg = color;
                    node.style.fg_overridden = true;
                }
            }
            Opt::Width(chars) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.width = Some(chars.min(100_000));
                }
            }
            Opt::FontSize(size) => {
                if !size.is_finite() || !(1.0..=512.0).contains(&size) {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    node.style.font_size = size;
                }
            }
            Opt::TextVariable(var) => {
                let Some(text) = var.get(self) else { return false };
                if let Some(node) = self.node_mut(id) {
                    node.binding = Some(Binding::Text(var));
                    node.cursor = clean_entry(&text).len();
                }
            }
            Opt::TextVariableInt(var) => {
                if kind == Kind::Entry || var.get(self).is_none() {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    node.binding = Some(Binding::Int(var));
                }
            }
            Opt::Command(command) => {
                if !kind.has_command() {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    node.command = Some(command);
                }
            }
            Opt::Style(style_name) => {
                if let Some(node) = self.node_mut(id) {
                    node.named_style = Some(style_name);
                }
            }
            Opt::StyleOverride(spec) => {
                if let Some(node) = self.node_mut(id) {
                    node.style_override = Some(spec);
                }
            }
            Opt::Disabled(disabled) => {
                if let Some(node) = self.node_mut(id) {
                    node.disabled = disabled;
                }
                if disabled && self.focused == Some(id) {
                    self.traverse_focus(false);
                }
            }
            Opt::Image(img_id) => {
                if let Some(node) = self.node_mut(id) {
                    node.image = Some(img_id);
                }
            }
            Opt::Compound(compound) => {
                if let Some(node) = self.node_mut(id) {
                    node.compound = compound;
                }
            }
            Opt::Monospace(mono) => {
                if let Some(node) = self.node_mut(id) {
                    node.monospace = mono;
                }
            }
        }
        self.dirty = true;
        true
    }

    /// Shorthand for `config(id, Opt::Command(...))`.
    pub fn set_command(&mut self, id: WidgetId, command: impl FnMut(&mut Tk) + 'static) -> bool {
        self.config(id, Opt::Command(Box::new(command)))
    }

    /// Runs `callback` when Enter is pressed in the single-line Entry `id`
    /// (tkinter `entry.bind("<Return>", ...)`), e.g. to submit a search box.
    /// Inside an open dialog, Enter still accepts the dialog first.
    /// Returns false if `id` is not a live Entry. Calling it again replaces the callback.
    pub fn entry_bind_submit(&mut self, id: WidgetId, callback: impl FnMut(&mut Tk) + 'static) -> bool {
        match self.node_mut(id) {
            Some(node) if node.kind == Kind::Entry => {
                node.command = Some(Box::new(callback));
                true
            }
            _ => false,
        }
    }

    /// Current text of an Entry (tkinter `entry.get()`); `None` for other kinds.
    pub fn entry_get(&self, id: WidgetId) -> Option<String> {
        if self.node(id)?.kind != Kind::Entry {
            return None;
        }
        self.widget_text(id)
    }

    /// Replaces an Entry's text (control characters stripped, capped at 64 KiB)
    /// and moves the cursor to the end.
    pub fn entry_set(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        if !self.node(id).is_some_and(|node| node.kind == Kind::Entry) {
            return false;
        }
        let text = clean_entry(&text.into());
        if let Some(node) = self.node_mut(id) {
            node.cursor = text.len();
        }
        self.write_entry(id, text);
        true
    }

    pub(crate) fn write_entry(&mut self, id: WidgetId, text: String) {
        let binding = self.node(id).and_then(|node| node.binding);
        let mut changed = false;
        if let Some(node) = self.node_mut(id) {
            if node.text != text {
                node.text.clone_from(&text);
                changed = true;
            }
        }
        if changed {
            // The entry's own contents changed; repaint only its area rather
            // than requesting a full redraw.
            self.invalidate_widget(id);
        }
        if let Some(Binding::Text(var)) = binding {
            var.set(self, text.clone());
        }
        if let Some(mut cb) = self.modal_entry_changed.take() {
            cb(self, id, &text);
            if self.modal_entry_changed.is_none() {
                self.modal_entry_changed = Some(cb);
            }
        }
    }

    /// Sets or clears the `SELECTED` state bit on a widget (e.g. for sidebar items).
    pub fn set_selected(&mut self, id: WidgetId, selected: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.selected != selected {
            node.selected = selected;
            self.dirty = true;
        }
        true
    }

    /// Displayed text of any widget, honouring a bound `textvariable`.
    pub fn widget_text(&self, id: WidgetId) -> Option<String> {
        let node = self.node(id)?;
        let text = match node.binding {
            Some(Binding::Text(var)) => var.get(self).unwrap_or_else(|| node.text.clone()),
            Some(Binding::Int(var)) => var.get(self).map(|value| value.to_string())
                .unwrap_or_else(|| node.text.clone()),
            None => node.text.clone(),
        };
        Some(if node.kind == Kind::Entry { clean_entry(&text) } else { text })
    }

}
