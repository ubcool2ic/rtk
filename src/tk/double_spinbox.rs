//! DoubleSpinbox widget: a floating-point spin box with editable text.
//!
//! Looks like the integer `Spinbox` (text field + up/down arrows) but binds a
//! `Var<f64>` and supports configurable step, decimals and suffix. All state
//! lives in [`crate::tk::inputs_state::InputsState`].

#[allow(unused_imports)]
use super::*;

/// Constructor namespace for floating-point spin boxes.
pub struct DoubleSpinbox;

#[allow(clippy::new_ret_no_self)]
impl DoubleSpinbox {
    /// Creates a spin box over `from..=to` bound to `var` (a `f64`).
    ///
    /// Non-finite bounds are replaced by `0.0..=1.0`. Fails with
    /// [`RtkError::InvalidVariable`] if `var` is not live in `tk`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        from: f64,
        to: f64,
        var: crate::Var<f64>,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let (from, to) = if from.is_finite() && to.is_finite() {
            (from, to)
        } else {
            (0.0, 1.0)
        };
        let id = tk.add(Kind::DoubleSpinbox, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.progress_f64 = Some(var);
            node.range_f64 = (from, to);
            let state = node.inputs.get_or_insert_with(Default::default);
            state.dspin_step = 1.0;
            state.dspin_decimals = 2;
            state.dspin_suffix = String::new();
            state.dspin_buffer = None;
        }
        Ok(id)
    }
}

impl Tk {
    /// Returns the inputs state of `id`, creating a default one if absent.
    /// Returns `None` if `id` is not a double spin box.
    fn dspin_state_mut(&mut self, id: WidgetId) -> Option<&mut crate::tk::inputs_state::InputsState> {
        let node = self.node_mut(id)?;
        if node.kind != Kind::DoubleSpinbox {
            return None;
        }
        Some(node.inputs.get_or_insert_with(Default::default))
    }

    /// Sets the step used by arrows/keys/wheel (non-finite or `<= 0` becomes `1.0`).
    /// Returns false for non-double-spin widgets.
    pub fn dspin_set_step(&mut self, id: WidgetId, step: f64) -> bool {
        let Some(state) = self.dspin_state_mut(id) else { return false };
        state.dspin_step = if step.is_finite() && step > 0.0 { step } else { 1.0 };
        self.dirty = true;
        true
    }

    /// Sets the number of decimals used for display (clamped to `0..=10`).
    /// Returns false for non-double-spin widgets.
    pub fn dspin_set_decimals(&mut self, id: WidgetId, decimals: u8) -> bool {
        let Some(state) = self.dspin_state_mut(id) else { return false };
        state.dspin_decimals = decimals.min(10);
        self.dirty = true;
        true
    }

    /// Sets the suffix appended to the displayed value (e.g. `" V"`).
    /// Returns false for non-double-spin widgets.
    pub fn dspin_set_suffix(&mut self, id: WidgetId, suffix: impl Into<String>) -> bool {
        let Some(state) = self.dspin_state_mut(id) else { return false };
        state.dspin_suffix = suffix.into();
        self.dirty = true;
        true
    }

    /// Returns the spin box's current value, or `None` if `id` is not a double spin box.
    pub fn dspin_get(&self, id: WidgetId) -> Option<f64> {
        let node = self.node(id).filter(|n| n.kind == Kind::DoubleSpinbox)?;
        let value = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(node.range_f64.0);
        Some(if value.is_nan() { node.range_f64.0 } else { value })
    }

    /// Sets the value (clamped to the range, rounded to `decimals`), writing the
    /// bound variable without running the command. Returns false for
    /// non-double-spin widgets or dead variables.
    pub fn dspin_set(&mut self, id: WidgetId, value: f64) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::DoubleSpinbox {
            return false;
        }
        let Some(var) = node.progress_f64 else { return false };
        if var.get(self).is_none() {
            return false;
        }
        let rounded = self.dspin_round(id, value);
        var.set(self, rounded);
        if let Some(state) = self.dspin_state_mut(id) {
            state.dspin_buffer = None;
        }
        self.dirty = true;
        true
    }

    /// Returns the formatted value without the suffix, used to seed the edit buffer.
    fn dspin_edit_seed(&self, id: WidgetId) -> String {
        let text = self.dspin_text(id).unwrap_or_default();
        let suffix = self
            .node(id)
            .and_then(|n| n.inputs.as_ref())
            .map(|s| s.dspin_suffix.clone())
            .unwrap_or_default();
        match text.strip_suffix(suffix.as_str()) {
            Some(stripped) => stripped.to_string(),
            None => text,
        }
    }

    /// Returns the formatted display text (value + suffix), or `None` if `id`
    /// is not a double spin box.
    pub fn dspin_text(&self, id: WidgetId) -> Option<String> {
        let node = self.node(id).filter(|n| n.kind == Kind::DoubleSpinbox)?;
        let value = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(node.range_f64.0);
        let value = if value.is_nan() { node.range_f64.0 } else { value };
        let (decimals, suffix) = node
            .inputs
            .as_ref()
            .map_or((2u8, String::new()), |s| (s.dspin_decimals, s.dspin_suffix.clone()));
        Some(format!("{:.*}{}", decimals as usize, value, suffix))
    }

    /// Rounds `value` to the spin box's decimals and clamps it to the range.
    pub(crate) fn dspin_round(&self, id: WidgetId, value: f64) -> f64 {
        let Some(node) = self.node(id).filter(|n| n.kind == Kind::DoubleSpinbox) else { return value };
        let (from, to) = node.range_f64;
        let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
        let clamped = if value.is_nan() { lo } else { value.clamp(lo, hi) };
        let decimals = node.inputs.as_ref().map_or(2u8, |s| s.dspin_decimals) as i32;
        let factor = 10f64.powi(decimals);
        let scaled = (clamped * factor).round() / factor;
        if !scaled.is_finite() {
            return clamped;
        }
        if scaled == 0.0 { 0.0 } else { scaled }
    }

    /// Applies a user-driven value change: rounds, writes the variable and runs
    /// the command if the value actually changed.
    pub(crate) fn dspin_user_set(&mut self, id: WidgetId, value: f64) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::DoubleSpinbox || node.disabled {
            return;
        }
        let Some(var) = node.progress_f64 else { return };
        let rounded = self.dspin_round(id, value);
        let old = var.get(self).unwrap_or(node.range_f64.0);
        if old.is_nan() || (old - rounded).abs() > f64::EPSILON {
            var.set(self, rounded);
            self.run_command(id);
        }
    }

    /// Steps the value by `direction * step` (direction is `+1` or `-1`).
    pub(crate) fn dspin_step(&mut self, id: WidgetId, direction: i32) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::DoubleSpinbox || node.disabled {
            return;
        }
        let step = node.inputs.as_ref().map_or(1.0, |s| s.dspin_step);
        let current = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(node.range_f64.0);
        let next = current + f64::from(direction) * step;
        if let Some(state) = self.dspin_state_mut(id) {
            state.dspin_buffer = None;
        }
        self.dspin_user_set(id, next);
        self.dirty = true;
    }

    /// Handles a key press on a focused double spin box. Returns true if consumed.
    pub(crate) fn dspin_key(&mut self, id: WidgetId, key: Key) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::DoubleSpinbox || node.disabled {
            return false;
        }
        match key {
            Key::Up => { self.dspin_step(id, 1); true }
            Key::Down => { self.dspin_step(id, -1); true }
            Key::Enter => { self.dspin_commit(id); true }
            Key::Escape => { self.dspin_revert(id); true }
            Key::Backspace => { self.dspin_backspace(id); true }
            _ => false,
        }
    }

    /// Appends typed text to the edit buffer (digits, `.`, `,`, `-` only).
    pub(crate) fn dspin_type(&mut self, id: WidgetId, text: &str) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::DoubleSpinbox || node.disabled {
            return;
        }
        let mut buffer = node
            .inputs
            .as_ref()
            .and_then(|s| s.dspin_buffer.clone())
            .unwrap_or_else(|| self.dspin_edit_seed(id));
        for ch in text.chars() {
            if ch.is_ascii_digit() || ch == '.' || ch == ',' || ch == '-' {
                buffer.push(ch);
            }
        }
        if let Some(state) = self.dspin_state_mut(id) {
            state.dspin_buffer = Some(buffer);
        }
        self.dirty = true;
    }

    /// Removes the last character from the edit buffer.
    pub(crate) fn dspin_backspace(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::DoubleSpinbox || node.disabled {
            return;
        }
        let mut buffer = node
            .inputs
            .as_ref()
            .and_then(|s| s.dspin_buffer.clone())
            .unwrap_or_else(|| self.dspin_edit_seed(id));
        buffer.pop();
        if let Some(state) = self.dspin_state_mut(id) {
            state.dspin_buffer = Some(buffer);
        }
        self.dirty = true;
    }

    /// Parses the edit buffer and commits it (invalid text reverts to the last value).
    pub(crate) fn dspin_commit(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::DoubleSpinbox {
            return;
        }
        let buffer = node.inputs.as_ref().and_then(|s| s.dspin_buffer.clone());
        if let Some(buffer) = buffer {
            let cleaned = buffer.trim().replace(',', ".");
            if let Ok(value) = cleaned.parse::<f64>() {
                self.dspin_user_set(id, value);
            }
        }
        if let Some(state) = self.dspin_state_mut(id) {
            state.dspin_buffer = None;
        }
        self.dirty = true;
    }

    /// Discards the edit buffer, reverting to the last committed value.
    pub(crate) fn dspin_revert(&mut self, id: WidgetId) {
        if let Some(state) = self.dspin_state_mut(id) {
            state.dspin_buffer = None;
        }
        self.dirty = true;
    }

    /// Returns the text currently displayed (edit buffer if present, else formatted value).
    pub(crate) fn dspin_display(&self, id: WidgetId) -> String {
        let Some(node) = self.node(id).filter(|n| n.kind == Kind::DoubleSpinbox) else { return String::new() };
        if let Some(buffer) = node.inputs.as_ref().and_then(|s| s.dspin_buffer.clone()) {
            return buffer;
        }
        self.dspin_text(id).unwrap_or_default()
    }

    /// Requested size of a double spin box: the widest of the formatted value
    /// (including suffix) and the range endpoints, plus the 20 px arrow column,
    /// 6 px text inset on each side and the resolved padding. The height fits
    /// the measured text line plus 12 px of vertical padding.
    pub(crate) fn dspin_requested(&mut self, id: WidgetId, resolved: &ResolvedStyle) -> Size {
        let (from, to) = self
            .node(id)
            .filter(|n| n.kind == Kind::DoubleSpinbox)
            .map_or((0.0, 1.0), |n| n.range_f64);
        let mut widest = self.dspin_text(id).unwrap_or_default();
        for endpoint in [from, to] {
            let candidate = self.dspin_format_value(id, endpoint);
            if self.measure_with(&candidate, resolved).width > self.measure_with(&widest, resolved).width {
                widest = candidate;
            }
        }
        let m = self.measure_with(&widest, resolved);
        let pad_w = resolved.padding.left + resolved.padding.right;
        let pad_h = resolved.padding.top + resolved.padding.bottom;
        let text_w = m.width.max(resolved.font_size * 4.0);
        let text_h = m.height.max(resolved.font_size * 1.3);
        Size::new(text_w + 12.0 + 20.0 + pad_w, text_h + 12.0 + pad_h)
    }

    /// Formats `value` like [`Tk::dspin_text`] would for `id` (decimals and
    /// suffix), without reading the bound variable.
    fn dspin_format_value(&self, id: WidgetId, value: f64) -> String {
        let node = self.node(id).filter(|n| n.kind == Kind::DoubleSpinbox);
        let (decimals, suffix) = node
            .and_then(|n| n.inputs.as_ref())
            .map_or((2u8, String::new()), |s| (s.dspin_decimals, s.dspin_suffix.clone()));
        let value = if value.is_nan() { 0.0 } else { value };
        format!("{:.*}{}", decimals as usize, value, suffix)
    }
}
