//! Slider widget: a continuous or stepped `f64` slider (horizontal or vertical).
//!
//! This is the floating-point replacement for the integer `Scale`. All state
//! lives in [`crate::tk::inputs_state::InputsState`]; this file only holds the
//! constructor namespace and the `Tk` methods that read/write that state.

#[allow(unused_imports)]
use super::*;

use crate::tk::inputs_state::Orient;

/// Constructor namespace for floating-point sliders.
pub struct Slider;

#[allow(clippy::new_ret_no_self)]
impl Slider {
    /// Creates a slider over `from..=to` bound to `var` (a `f64`).
    ///
    /// `from > to` is allowed and inverts the direction. Non-finite bounds are
    /// replaced by `0.0..=1.0`. Fails with [`RtkError::InvalidVariable`] if
    /// `var` is not live in `tk`.
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
        let id = tk.add(Kind::Slider, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.progress_f64 = Some(var);
            node.range_f64 = (from, to);
            let state = node.inputs.get_or_insert_with(Default::default);
            state.slider_orient = Orient::Horizontal;
            state.slider_step = 0.0;
            state.slider_ticks = None;
            state.slider_length = 160.0;
        }
        Ok(id)
    }
}

impl Tk {
    /// Returns the inputs state of `id`, creating a default one if absent.
    /// Returns `None` if `id` is not a slider.
    fn slider_state_mut(&mut self, id: WidgetId) -> Option<&mut crate::tk::inputs_state::InputsState> {
        let node = self.node_mut(id)?;
        if node.kind != Kind::Slider {
            return None;
        }
        Some(node.inputs.get_or_insert_with(Default::default))
    }

    /// Sets the slider orientation. Returns false for non-sliders.
    pub fn slider_set_orient(&mut self, id: WidgetId, orient: Orient) -> bool {
        let Some(state) = self.slider_state_mut(id) else { return false };
        state.slider_orient = orient;
        self.dirty = true;
        true
    }

    /// Returns the slider orientation, or `None` if `id` is not a slider.
    pub fn slider_orient(&self, id: WidgetId) -> Option<Orient> {
        let node = self.node(id).filter(|n| n.kind == Kind::Slider)?;
        Some(node.inputs.as_ref().map_or(Orient::Horizontal, |s| s.slider_orient))
    }

    /// Sets the step used for snapping and keyboard/wheel changes. `0.0` (or a
    /// non-finite value) means continuous. Returns false for non-sliders.
    pub fn slider_set_step(&mut self, id: WidgetId, step: f64) -> bool {
        let Some(state) = self.slider_state_mut(id) else { return false };
        state.slider_step = if step.is_finite() && step > 0.0 { step } else { 0.0 };
        self.dirty = true;
        true
    }

    /// Draws tick marks every `every` units along the track (`None` clears them).
    /// Returns false for non-sliders.
    pub fn slider_set_ticks(&mut self, id: WidgetId, every: Option<f64>) -> bool {
        let Some(state) = self.slider_state_mut(id) else { return false };
        state.slider_ticks = every.filter(|v| v.is_finite() && *v > 0.0);
        self.dirty = true;
        true
    }

    /// Sets the slider length in logical px (clamped to `20.0..=10_000.0`).
    /// Returns false for non-sliders.
    pub fn slider_set_length(&mut self, id: WidgetId, px: f32) -> bool {
        let Some(state) = self.slider_state_mut(id) else { return false };
        state.slider_length = if px.is_nan() { 160.0 } else { px.clamp(20.0, 10_000.0) };
        self.dirty = true;
        true
    }

    /// Returns the slider's current value, or `None` if `id` is not a slider.
    pub fn slider_get(&self, id: WidgetId) -> Option<f64> {
        let node = self.node(id).filter(|n| n.kind == Kind::Slider)?;
        let value = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(node.range_f64.0);
        Some(if value.is_nan() { node.range_f64.0 } else { value })
    }

    /// Sets the slider value (clamped to the range and snapped to the step),
    /// writing the bound variable without running the command. Returns false
    /// for non-sliders or dead variables.
    pub fn slider_set(&mut self, id: WidgetId, value: f64) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Slider {
            return false;
        }
        let Some(var) = node.progress_f64 else { return false };
        if var.get(self).is_none() {
            return false;
        }
        let snapped = self.slider_snap(id, value);
        var.set(self, snapped);
        self.dirty = true;
        true
    }

    /// Snaps `value` to the slider's range and step (no-op if `id` is not a slider).
    pub(crate) fn slider_snap(&self, id: WidgetId, value: f64) -> f64 {
        let Some(node) = self.node(id).filter(|n| n.kind == Kind::Slider) else { return value };
        let (from, to) = node.range_f64;
        let (lo, hi) = if from <= to { (from, to) } else { (to, from) };
        let clamped = if value.is_nan() { lo } else { value.clamp(lo, hi) };
        let step = node.inputs.as_ref().map_or(0.0, |s| s.slider_step);
        if step <= 0.0 {
            return clamped;
        }
        let k = ((clamped - from) / step).round();
        let snapped = from + k * step;
        snapped.clamp(lo, hi)
    }

    /// Applies a user-driven value change: snaps, writes the variable and runs
    /// the command if the value actually changed.
    pub(crate) fn slider_user_set(&mut self, id: WidgetId, value: f64) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Slider || node.disabled {
            return;
        }
        let Some(var) = node.progress_f64 else { return };
        let snapped = self.slider_snap(id, value);
        let old = var.get(self).unwrap_or(node.range_f64.0);
        if old != snapped {
            var.set(self, snapped);
            self.run_command(id);
        }
    }

    /// Returns the fraction of the slider's range that its current value
    /// represents (`0.0` at `from`, `1.0` at `to`).
    pub(crate) fn slider_fraction(&self, id: WidgetId) -> f32 {
        let Some(node) = self.node(id).filter(|n| n.kind == Kind::Slider) else { return 0.0 };
        let (from, to) = node.range_f64;
        if (to - from).abs() < f64::EPSILON {
            return 0.0;
        }
        let value = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(from);
        let ratio = (value - from) / (to - from);
        if ratio.is_nan() {
            return 0.0;
        }
        ratio.clamp(0.0, 1.0) as f32
    }

    /// Requested size of a slider: `length x 28` horizontal, `28 x length` vertical.
    pub(crate) fn slider_requested(&mut self, id: WidgetId, resolved: &ResolvedStyle) -> Size {
        let node = self.node(id);
        let length = node
            .and_then(|n| n.inputs.as_ref())
            .map_or(160.0, |s| s.slider_length);
        let orient = node
            .and_then(|n| n.inputs.as_ref())
            .map_or(Orient::Horizontal, |s| s.slider_orient);
        let pad_w = resolved.padding.left + resolved.padding.right;
        let pad_h = resolved.padding.top + resolved.padding.bottom;
        match orient {
            Orient::Horizontal => Size::new(length + pad_w, 28.0 + pad_h),
            Orient::Vertical => Size::new(28.0 + pad_w, length + pad_h),
        }
    }

    /// Maps a pointer position (logical px) inside the slider to a value.
    pub(crate) fn slider_value_at(&self, id: WidgetId, x: f32, y: f32) -> Option<f64> {
        let node = self.node(id).filter(|n| n.kind == Kind::Slider)?;
        let orient = node.inputs.as_ref().map_or(Orient::Horizontal, |s| s.slider_orient);
        let (from, to) = node.range_f64;
        let rect = node.rect;
        let fraction = match orient {
            Orient::Horizontal => {
                let track = rect.inset(12.0, 0.0);
                if track.width <= 0.0 { 0.0 } else { ((x - track.x) / track.width).clamp(0.0, 1.0) }
            }
            Orient::Vertical => {
                let track = rect.inset(0.0, 12.0);
                if track.height <= 0.0 { 0.0 } else { (1.0 - (y - track.y) / track.height).clamp(0.0, 1.0) }
            }
        };
        Some(from + f64::from(fraction) * (to - from))
    }

    /// Handles a key press on a focused slider. Returns true if the key was consumed.
    pub(crate) fn slider_key(&mut self, id: WidgetId, key: Key) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Slider || node.disabled {
            return false;
        }
        let (from, to) = node.range_f64;
        let step = node.inputs.as_ref().map_or(0.0, |s| s.slider_step);
        // Signed so that "increase" always moves toward `to`, even when inverted.
        let dir = if from <= to { 1.0 } else { -1.0 };
        let unit = dir * if step > 0.0 { step } else { (to - from).abs() * 0.01 };
        let current = node.progress_f64.and_then(|v| v.get(self)).unwrap_or(from);
        let next = match key {
            Key::Left | Key::Down => current - unit,
            Key::Right | Key::Up => current + unit,
            Key::Home => from,
            Key::End => to,
            Key::PageUp => current + unit * 10.0,
            Key::PageDown => current - unit * 10.0,
            _ => return false,
        };
        self.slider_user_set(id, next);
        true
    }
}
