//! Accessibility tree construction and action dispatch.
//!
//! Builds a dependency-free, headless-testable snapshot of the widget tree
//! ([`A11yNode`]) and lets assistive technology drive widgets through
//! [`A11yAction`]s. The optional `accesskit` feature maps this snapshot onto
//! AccessKit's `TreeUpdate` in `src/a11y_accesskit.rs`.

#[allow(unused_imports)]
use super::*;

use crate::a11y::{A11yAction, A11yNode, A11yRole};

impl Tk {
    /// Maps a widget kind to its accessibility role.
    pub(crate) fn a11y_role(kind: Kind) -> A11yRole {
        match kind {
            Kind::Frame => A11yRole::Group,
            Kind::Label => A11yRole::Label,
            Kind::Button => A11yRole::Button,
            Kind::Checkbutton => A11yRole::CheckBox,
            Kind::Radiobutton => A11yRole::RadioButton,
            Kind::Entry => A11yRole::TextInput,
            Kind::Text => A11yRole::MultilineTextInput,
            Kind::Scale => A11yRole::Slider,
            Kind::Spinbox => A11yRole::SpinButton,
            Kind::ProgressBar => A11yRole::ProgressIndicator,
            Kind::Listbox => A11yRole::ListBox,
            Kind::Combobox => A11yRole::ComboBox,
            Kind::Notebook => A11yRole::TabList,
            Kind::Scrollbar => A11yRole::ScrollBar,
            Kind::Canvas | Kind::Chart | Kind::Image => A11yRole::Canvas,
        }
    }

    /// Builds the accessibility tree: root first, then depth-first in child
    /// order, including only mapped (visible) widgets.
    pub fn accessibility_tree(&self) -> Vec<A11yNode> {
        let mut out = Vec::new();
        let root = self.root();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            let Some(node) = self.node(id) else { continue };
            if !node.visible {
                continue;
            }
            let role = if id == root { A11yRole::Window } else { Self::a11y_role(node.kind) };
            let label = self.a11y_label(id);
            let value = self.a11y_value(id);
            let numeric = self.a11y_numeric(id);
            let toggled = self.a11y_toggled(id);
            let disabled = node.disabled;
            let focused = self.focused == Some(id);
            let bounds = node.rect;
            let children = node.children.clone();
            let actions = self.a11y_actions(id);
            out.push(A11yNode {
                id,
                role,
                label,
                value,
                numeric,
                toggled,
                disabled,
                focused,
                bounds,
                children: children.clone(),
                actions,
            });
            // Push children in reverse so they pop in document order.
            for child in children.iter().rev() {
                pending.push(*child);
            }
        }
        out
    }

    /// Computes the accessibility label for a widget, honouring overrides.
    pub(crate) fn a11y_label(&self, id: WidgetId) -> Option<String> {
        if let Some(label) = self.a11y_labels.get(&id) {
            return Some(label.clone());
        }
        let node = self.node(id)?;
        match node.kind {
            Kind::Notebook => {
                let titles: Vec<String> = node.notebook_pages.iter().map(|(_, t)| t.clone()).collect();
                if titles.is_empty() { None } else { Some(titles.join(", ")) }
            }
            Kind::Canvas => None,
            Kind::Frame => None,
            _ => {
                let text = self.widget_text(id).unwrap_or_default();
                if text.is_empty() { None } else { Some(text) }
            }
        }
    }

    /// Computes the accessibility value for a widget (text/selection).
    pub(crate) fn a11y_value(&self, id: WidgetId) -> Option<String> {
        let node = self.node(id)?;
        match node.kind {
            Kind::Entry => self.entry_get(id),
            Kind::Text => self.text_get(id),
            Kind::Combobox => self.combobox_get(id).map(|(_, s)| s),
            _ => None,
        }
    }

    /// Computes the numeric `(value, min, max)` triple for range widgets.
    pub(crate) fn a11y_numeric(&self, id: WidgetId) -> Option<(f64, f64, f64)> {
        let node = self.node(id)?;
        match node.kind {
            Kind::Scale | Kind::Spinbox => {
                let (from, to) = node.range;
                let value = node.scale.and_then(|v| v.get(self)).unwrap_or(from);
                Some((value as f64, from as f64, to as f64))
            }
            Kind::ProgressBar => {
                let value = if let Some(var) = node.progress_f64 {
                    var.get(self).unwrap_or(0.0)
                } else {
                    node.scale.and_then(|v| v.get(self)).map(|v| v as f64).unwrap_or(0.0)
                };
                Some((value, 0.0, 100.0))
            }
            _ => None,
        }
    }

    /// Computes the toggled state for check boxes and radio buttons.
    pub(crate) fn a11y_toggled(&self, id: WidgetId) -> Option<bool> {
        let node = self.node(id)?;
        match node.kind {
            Kind::Checkbutton => node.check.and_then(|v| v.get(self)),
            Kind::Radiobutton => {
                let var = node.radio_var?;
                let current = var.get(self)?;
                Some(current == node.radio_value)
            }
            _ => None,
        }
    }

    /// Computes the set of actions supported by a widget.
    pub(crate) fn a11y_actions(&self, id: WidgetId) -> Vec<A11yAction> {
        let Some(node) = self.node(id) else { return Vec::new() };
        if node.disabled {
            return Vec::new();
        }
        let mut actions = Vec::new();
        match node.kind {
            Kind::Button | Kind::Checkbutton | Kind::Radiobutton => actions.push(A11yAction::Click),
            Kind::Entry | Kind::Text => actions.push(A11yAction::SetValue),
            Kind::Scale | Kind::Spinbox => {
                actions.push(A11yAction::Increment);
                actions.push(A11yAction::Decrement);
            }
            _ => {}
        }
        if node.kind.focusable() || (node.kind == Kind::Canvas && node.scrollregion.is_some()) {
            actions.push(A11yAction::Focus);
        }
        actions
    }

    /// Sets an accessibility label override for a widget (e.g. icon-only buttons).
    /// Returns false if `id` is not a live widget.
    pub fn a11y_set_label(&mut self, id: WidgetId, label: impl Into<String>) -> bool {
        if self.node(id).is_none() {
            return false;
        }
        self.a11y_labels.insert(id, label.into());
        true
    }

    /// Performs an accessibility action on a widget. Returns false if the
    /// widget is dead, disabled, or does not support the action.
    pub fn a11y_do_action(&mut self, id: WidgetId, action: A11yAction, value: Option<&str>) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.disabled {
            return false;
        }
        let kind = node.kind;
        match action {
            A11yAction::Click => {
                if !matches!(kind, Kind::Button | Kind::Checkbutton | Kind::Radiobutton) {
                    return false;
                }
                self.focused = Some(id);
                self.activate(id);
                self.dirty = true;
                true
            }
            A11yAction::Focus => {
                if !self.is_focusable(id) {
                    return false;
                }
                self.focused = Some(id);
                self.dirty = true;
                true
            }
            A11yAction::SetValue => {
                let Some(value) = value else { return false };
                match kind {
                    Kind::Entry => self.entry_set(id, value),
                    Kind::Text => self.text_set(id, value),
                    _ => false,
                }
            }
            A11yAction::Increment | A11yAction::Decrement => {
                if !matches!(kind, Kind::Scale | Kind::Spinbox) {
                    return false;
                }
                self.focused = Some(id);
                let key = match (kind, action) {
                    (Kind::Scale, A11yAction::Increment) => Key::Right,
                    (Kind::Scale, A11yAction::Decrement) => Key::Left,
                    (_, A11yAction::Increment) => Key::Up,
                    (_, A11yAction::Decrement) => Key::Down,
                    _ => return false,
                };
                self.sim_key(key);
                true
            }
        }
    }

    /// Returns true (and resets the baseline) if the accessibility tree has
    /// changed since the last call.
    pub fn a11y_tree_changed(&mut self) -> bool {
        let current = self.accessibility_tree();
        let changed = self.a11y_last_tree.as_ref() != Some(&current);
        if changed {
            self.a11y_last_tree = Some(current);
        }
        changed
    }
}
