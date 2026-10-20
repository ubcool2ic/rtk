//! Tri-state (checked / unchecked / mixed) support for the existing
//! [`crate::Checkbutton`].
//!
//! All state lives in [`crate::tk::inputs_state::InputsState`] so that
//! `WidgetNode` needs no new fields. The default (non-tri-state) check box is
//! completely unaffected: every method here is a no-op unless tri-state mode
//! has been enabled with [`Tk::checkbutton_set_tristate`].

#[allow(unused_imports)]
use super::*;

use crate::tk::inputs_state::CheckState;

impl Tk {
    /// Returns true if `id` is a live `Checkbutton`.
    fn is_checkbutton(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| node.kind == Kind::Checkbutton)
    }

    /// Enables or disables tri-state mode on a check box.
    ///
    /// Turning tri-state off also clears the mixed flag so the box falls back
    /// to the plain checked/unchecked look. Returns false if `id` is not a
    /// `Checkbutton`.
    pub fn checkbutton_set_tristate(&mut self, id: WidgetId, on: bool) -> bool {
        if !self.is_checkbutton(id) {
            return false;
        }
        let Some(node) = self.node_mut(id) else { return false };
        let state = node.inputs.get_or_insert_with(Default::default);
        state.check_tristate = on;
        if !on {
            state.check_mixed = false;
            state.check_cycle_mixed = false;
        }
        self.dirty = true;
        true
    }

    /// Returns true if `id` is a check box with tri-state mode enabled.
    pub fn checkbutton_is_tristate(&self, id: WidgetId) -> bool {
        self.node(id)
            .filter(|node| node.kind == Kind::Checkbutton)
            .and_then(|node| node.inputs.as_ref())
            .is_some_and(|state| state.check_tristate)
    }

    /// Returns the current state of a check box, or `None` if `id` is not a
    /// `Checkbutton`.
    ///
    /// Reports [`CheckState::Mixed`] when tri-state mode is on and the mixed
    /// flag is set; otherwise derives `Checked`/`Unchecked` from the bound
    /// `Var<bool>`.
    pub fn checkbutton_state(&self, id: WidgetId) -> Option<CheckState> {
        let node = self.node(id).filter(|node| node.kind == Kind::Checkbutton)?;
        if self.checkbutton_is_mixed(id) {
            return Some(CheckState::Mixed);
        }
        let checked = node.check.and_then(|var| var.get(self)).unwrap_or(false);
        Some(if checked { CheckState::Checked } else { CheckState::Unchecked })
    }

    /// Sets the state of a check box programmatically without running its
    /// command.
    ///
    /// `Mixed` requires tri-state mode to be enabled; otherwise it returns
    /// false and changes nothing. `Checked`/`Unchecked` clear the mixed flag
    /// and write the bound variable. Returns false if `id` is not a
    /// `Checkbutton` or the bound variable is dead.
    pub fn checkbutton_set_state(&mut self, id: WidgetId, state: CheckState) -> bool {
        if !self.is_checkbutton(id) {
            return false;
        }
        match state {
            CheckState::Mixed => {
                if !self.checkbutton_is_tristate(id) {
                    return false;
                }
                let Some(node) = self.node_mut(id) else { return false };
                let inputs = node.inputs.get_or_insert_with(Default::default);
                inputs.check_mixed = true;
                self.dirty = true;
                true
            }
            CheckState::Checked | CheckState::Unchecked => {
                let Some(var) = self.node(id).and_then(|node| node.check) else { return false };
                let value = state == CheckState::Checked;
                if !var.set(self, value) {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    let inputs = node.inputs.get_or_insert_with(Default::default);
                    inputs.check_mixed = false;
                }
                self.dirty = true;
                true
            }
        }
    }

    /// Enables or disables cycling through the mixed state on user activation.
    ///
    /// When on, clicking a tri-state box cycles
    /// `Unchecked -> Checked -> Mixed -> Unchecked`. When off (the default),
    /// activating a mixed box goes to `Checked` and otherwise toggles normally.
    /// Returns false if `id` is not a `Checkbutton`.
    pub fn checkbutton_set_cycle_mixed(&mut self, id: WidgetId, on: bool) -> bool {
        if !self.is_checkbutton(id) {
            return false;
        }
        let Some(node) = self.node_mut(id) else { return false };
        let state = node.inputs.get_or_insert_with(Default::default);
        state.check_cycle_mixed = on;
        self.dirty = true;
        true
    }

    /// Returns true if `id` is a tri-state check box currently showing the
    /// mixed (indeterminate) state.
    pub(crate) fn checkbutton_is_mixed(&self, id: WidgetId) -> bool {
        self.node(id)
            .filter(|node| node.kind == Kind::Checkbutton)
            .and_then(|node| node.inputs.as_ref())
            .is_some_and(|state| state.check_tristate && state.check_mixed)
    }

    /// Handles user activation of a tri-state check box.
    ///
    /// Returns true when tri-state mode is on and the activation was handled
    /// here (the bound variable and mixed flag are updated and the toolkit is
    /// marked dirty); the caller is then responsible for running the command.
    /// Returns false for non-check boxes and plain check boxes so the normal
    /// toggle path runs unchanged.
    pub(crate) fn tristate_activate(&mut self, id: WidgetId) -> bool {
        if !self.checkbutton_is_tristate(id) {
            return false;
        }
        let Some(node) = self.node(id) else { return false };
        let Some(var) = node.check else { return false };
        let cycle = node
            .inputs
            .as_ref()
            .is_some_and(|state| state.check_cycle_mixed);
        let mixed = self.checkbutton_is_mixed(id);
        let checked = var.get(self).unwrap_or(false);
        let (new_checked, new_mixed) = if mixed {
            if cycle {
                (false, false)
            } else {
                (true, false)
            }
        } else if checked {
            if cycle {
                (true, true)
            } else {
                (false, false)
            }
        } else {
            (true, false)
        };
        if !var.set(self, new_checked) {
            return false;
        }
        if let Some(node) = self.node_mut(id) {
            let inputs = node.inputs.get_or_insert_with(Default::default);
            inputs.check_mixed = new_mixed;
        }
        self.dirty = true;
        true
    }
}
