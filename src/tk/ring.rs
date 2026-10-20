//! ProgressRing widget: determinate percent arcs and indeterminate spinners.

#[allow(unused_imports)]
use super::*;

use crate::tk::inputs_state::RingMode;

/// Constructor namespace for circular progress indicators (percent or spinner).
pub struct ProgressRing;

#[allow(clippy::new_ret_no_self)]
impl ProgressRing {
    /// Creates a circular progress ring of `diameter` logical px (min 16) bound
    /// to `var`, which holds a fraction in `0.0..=1.0`.
    ///
    /// Fails with [`RtkError::InvalidVariable`] if `var` is not live in `tk`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        diameter: f32,
        var: crate::Var<f64>,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::ProgressRing, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.progress_length = if diameter.is_finite() { diameter.max(16.0) } else { 48.0 };
            node.progress_f64 = Some(var);
        }
        Ok(id)
    }
}

impl Tk {
    /// Returns the inputs state of `id`, creating a default one if absent.
    fn ring_state_mut(&mut self, id: WidgetId) -> Option<&mut crate::tk::inputs_state::InputsState> {
        let node = self.node_mut(id)?;
        if node.kind != Kind::ProgressRing {
            return None;
        }
        Some(node.inputs.get_or_insert_with(Default::default))
    }

    /// Sets the ring mode (percent or spinner). Returns false for non-rings.
    pub fn ring_set_mode(&mut self, id: WidgetId, mode: RingMode) -> bool {
        let Some(state) = self.ring_state_mut(id) else { return false };
        state.ring_mode = mode;
        self.dirty = true;
        true
    }

    /// Returns the current ring mode, or `None` if `id` is not a ring.
    pub fn ring_mode(&self, id: WidgetId) -> Option<RingMode> {
        let node = self.node(id).filter(|n| n.kind == Kind::ProgressRing)?;
        Some(node.inputs.as_ref().map_or(RingMode::Percent, |s| s.ring_mode))
    }

    /// Sets the ring fraction (clamped to `0.0..=1.0`, NaN becomes 0) and writes
    /// the bound variable. Returns false for non-rings or dead variables.
    pub fn ring_set(&mut self, id: WidgetId, fraction: f64) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::ProgressRing {
            return false;
        }
        let Some(var) = node.progress_f64 else { return false };
        if var.get(self).is_none() {
            return false;
        }
        let value = if fraction.is_nan() { 0.0 } else { fraction.clamp(0.0, 1.0) };
        var.set(self, value);
        self.dirty = true;
        true
    }

    /// Returns the ring fraction in `0.0..=1.0`, or `None` if `id` is not a ring.
    pub fn ring_get(&self, id: WidgetId) -> Option<f64> {
        let node = self.node(id).filter(|n| n.kind == Kind::ProgressRing)?;
        let value = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(0.0);
        Some(if value.is_nan() { 0.0 } else { value.clamp(0.0, 1.0) })
    }

    /// Toggles the centered "NN%" text in percent mode. Returns false for non-rings.
    pub fn ring_set_show_text(&mut self, id: WidgetId, show: bool) -> bool {
        let Some(state) = self.ring_state_mut(id) else { return false };
        state.ring_show_text = show;
        self.dirty = true;
        true
    }

    /// Sets the stroke thickness in logical px (clamped to `1.0..=diameter/2`).
    /// Non-finite values store 0. Returns false for non-rings.
    pub fn ring_set_thickness(&mut self, id: WidgetId, px: f32) -> bool {
        let diameter = self.node(id).map(|n| n.progress_length).unwrap_or(48.0);
        let max = (diameter / 2.0).max(1.0);
        let Some(state) = self.ring_state_mut(id) else { return false };
        state.ring_thickness = if px.is_finite() { px.clamp(1.0, max) } else { 0.0 };
        self.dirty = true;
        true
    }

    /// Requested size of a ring: a square of `diameter` plus padding.
    pub(crate) fn ring_requested(&mut self, id: WidgetId, resolved: &ResolvedStyle) -> Size {
        let diameter = self.node(id).map(|n| n.progress_length).unwrap_or(48.0);
        let pad_w = resolved.padding.left + resolved.padding.right;
        let pad_h = resolved.padding.top + resolved.padding.bottom;
        Size::new(diameter + pad_w, diameter + pad_h)
    }

    /// Advances ring spinner phase and reports whether a ring is animating.
    pub(crate) fn ring_advance(&mut self, id: WidgetId, ms: u32) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressRing || !node.visible {
            return false;
        }
        let state = node.inputs.get_or_insert_with(Default::default);
        if state.ring_mode != RingMode::Spinner {
            return false;
        }
        state.ring_phase = (state.ring_phase + (ms as f32) / 1000.0) % 1.0;
        self.dirty = true;
        true
    }

    /// True if `id` is a visible ring in spinner mode.
    pub(crate) fn ring_is_animating(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|n| {
            n.kind == Kind::ProgressRing
                && n.visible
                && n.inputs.as_ref().is_some_and(|s| s.ring_mode == RingMode::Spinner)
        })
    }
}
