//! Switch widget: an animated on/off toggle with a label.

#[allow(unused_imports)]
use super::*;

/// Constructor namespace for animated on/off switches.
pub struct Switch;

#[allow(clippy::new_ret_no_self)]
impl Switch {
    /// Creates a switch labelled `text` bound to `var`.
    ///
    /// Fails with [`RtkError::InvalidVariable`] if `var` is not live in `tk`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        text: impl Into<String>,
        var: crate::Var<bool>,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::Switch, parent, text.into())?;
        if let Some(node) = tk.node_mut(id) {
            node.check = Some(var);
        }
        Ok(id)
    }
}

impl Tk {
    /// Returns the switch's bound value, or `None` if `id` is not a switch.
    pub fn switch_get(&self, id: WidgetId) -> Option<bool> {
        let node = self.node(id).filter(|n| n.kind == Kind::Switch)?;
        node.check.and_then(|v| v.get(self))
    }

    /// Sets the switch's bound value without running its command. Returns false
    /// for non-switches or dead variables.
    pub fn switch_set(&mut self, id: WidgetId, value: bool) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Switch {
            return false;
        }
        let Some(var) = node.check else { return false };
        if var.get(self).is_none() {
            return false;
        }
        var.set(self, value);
        self.dirty = true;
        true
    }

    /// Requested size of a switch: pill (36x20) + gap (8) + label text, plus
    /// the style's horizontal padding so the label is never clipped at the
    /// widget's right edge. Height is `max(20, text height)` plus padding.
    pub(crate) fn switch_requested(&mut self, id: WidgetId, resolved: &ResolvedStyle) -> Size {
        let text = self.node(id).map(|n| n.text.clone()).unwrap_or_default();
        let measured = self.measure_with(&text, resolved);
        let pad_w = resolved.padding.left + resolved.padding.right;
        let pad_h = resolved.padding.top + resolved.padding.bottom;
        let width = 36.0 + 8.0 + measured.width + pad_w;
        let height = measured.height.max(20.0) + pad_h;
        Size::new(width, height)
    }

    /// Advances the switch knob animation toward the bound value. Returns true
    /// if the knob moved this step.
    pub(crate) fn switch_advance(&mut self, id: WidgetId, ms: u32) -> bool {
        // Read the target before taking a mutable borrow of the node.
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Switch || !node.visible {
            return false;
        }
        let target = if node.check.and_then(|v| v.get(self)).unwrap_or(false) { 1.0 } else { 0.0 };
        let Some(node) = self.node_mut(id) else { return false };
        let state = node.inputs.get_or_insert_with(Default::default);
        if !state.switch_initialised {
            state.switch_knob = target;
            state.switch_initialised = true;
            return false;
        }
        if (state.switch_knob - target).abs() < f32::EPSILON {
            return false;
        }
        // ~120 ms full travel: step by dt/120, clamped to the target.
        let step = (ms as f32) / 120.0;
        if state.switch_knob < target {
            state.switch_knob = (state.switch_knob + step).min(target);
        } else {
            state.switch_knob = (state.switch_knob - step).max(target);
        }
        self.dirty = true;
        true
    }

    /// True if `id` is a visible switch whose knob is still moving.
    pub(crate) fn switch_is_animating(&self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Switch || !node.visible {
            return false;
        }
        let target = if node.check.and_then(|v| v.get(self)).unwrap_or(false) { 1.0 } else { 0.0 };
        node.inputs
            .as_ref()
            .is_some_and(|s| s.switch_initialised && (s.switch_knob - target).abs() >= f32::EPSILON)
    }
}
