//! Progressbar variable binding, indeterminate mode, and milestone 0.10 polish
//! (percent/label text and smooth easing).

#[allow(unused_imports)]
use super::*;

impl Tk {
    // -----------------------------------------------------------------------
    // Progressbar var & indeterminate mode
    // -----------------------------------------------------------------------

    /// Binds a `f64` variable holding a fraction in `0.0..=1.0` to the progress
    /// bar `id`, replacing any integer binding. Returns false if `var` is not
    /// live in `tk` or `id` is not a progress bar.
    pub fn progress_bind(&mut self, id: WidgetId, var: crate::Var<f64>) -> bool {
        if var.get(self).is_none() { return false; }
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressBar { return false; }
        node.progress_f64 = Some(var);
        self.dirty = true;
        true
    }

    /// Switches the progress bar between determinate and indeterminate
    /// (animated marquee) mode. Returns false if `id` is not a progress bar.
    pub fn progress_set_indeterminate(&mut self, id: WidgetId, indeterminate: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressBar { return false; }
        node.progress_indeterminate = indeterminate;
        self.dirty = true;
        true
    }

    // -----------------------------------------------------------------------
    // Milestone 0.10 polish: percent text, label, smooth easing
    // -----------------------------------------------------------------------

    /// Returns the inputs state of `id`, creating a default one if absent.
    /// Returns `None` if `id` is not a progress bar.
    fn progress_state_mut(&mut self, id: WidgetId) -> Option<&mut crate::tk::inputs_state::InputsState> {
        let node = self.node_mut(id)?;
        if node.kind != Kind::ProgressBar {
            return None;
        }
        Some(node.inputs.get_or_insert_with(Default::default))
    }

    /// Toggles the centered "NN%" text drawn over the bar. Returns false for
    /// non-progress widgets.
    pub fn progress_set_show_percent(&mut self, id: WidgetId, show: bool) -> bool {
        let Some(state) = self.progress_state_mut(id) else { return false };
        state.progress_show_percent = show;
        self.dirty = true;
        true
    }

    /// Sets (or clears with `None`) the label drawn over the bar. Returns false
    /// for non-progress widgets.
    pub fn progress_set_label(&mut self, id: WidgetId, label: Option<String>) -> bool {
        let Some(state) = self.progress_state_mut(id) else { return false };
        state.progress_label = label;
        self.dirty = true;
        true
    }

    /// Enables smooth easing of the drawn fraction toward the bound value.
    ///
    /// When enabling, the drawn fraction is initialised to the bar's current
    /// fraction so that a subsequent change of the bound value is visibly
    /// eased (and [`Tk::is_animating`] reports true immediately). Returns
    /// false for non-progress widgets.
    pub fn progress_set_smooth(&mut self, id: WidgetId, smooth: bool) -> bool {
        let current = self.progress_get(id).unwrap_or(0.0) as f32;
        let Some(state) = self.progress_state_mut(id) else { return false };
        state.progress_smooth = smooth;
        if smooth {
            state.progress_drawn = current;
            state.progress_initialised = true;
        } else {
            state.progress_initialised = false;
        }
        self.dirty = true;
        true
    }

    /// Returns the current fraction of the progress bar in `0.0..=1.0`, or
    /// `None` if `id` is not a progress bar.
    pub fn progress_get(&self, id: WidgetId) -> Option<f64> {
        let node = self.node(id).filter(|n| n.kind == Kind::ProgressBar)?;
        let value = if let Some(v64) = node.progress_f64 {
            v64.get(self).unwrap_or(0.0)
        } else {
            node.scale.and_then(|v| v.get(self)).unwrap_or(0) as f64 / 100.0
        };
        Some(if value.is_nan() { 0.0 } else { value.clamp(0.0, 1.0) })
    }

    /// Requested size of a progress bar, growing in height when percent text or
    /// a label is shown so the text fits, and never narrower than the measured
    /// label text plus padding.
    pub(crate) fn progress_requested(&mut self, id: WidgetId, resolved: &ResolvedStyle) -> Size {
        let length = self.node(id).map(|n| n.progress_length).unwrap_or(100.0);
        let pad_w = resolved.padding.left + resolved.padding.right;
        let pad_h = resolved.padding.top + resolved.padding.bottom;
        let (show_percent, label) = self
            .node(id)
            .and_then(|n| n.inputs.as_ref())
            .map_or((false, None), |s| (s.progress_show_percent, s.progress_label.clone()));
        let show_text = show_percent || label.is_some();
        let height = if show_text {
            resolved.font_size * 1.3 + 6.0
        } else {
            20.0
        };
        let mut width = length;
        if show_text {
            // Measure the widest text the bar can display: the label with a
            // 100% suffix (the longest percent string) and the bare percent.
            let candidates = [
                label.as_ref().map(|l| format!("{l} 100%")),
                label.clone(),
                show_percent.then(|| "100%".to_string()),
            ];
            let mut text_w = 0.0_f32;
            let mut text_h = 0.0_f32;
            for candidate in candidates.into_iter().flatten() {
                let m = self.measure_with(&candidate, resolved);
                text_w = text_w.max(m.width);
                text_h = text_h.max(m.height);
            }
            width = width.max(text_w + 8.0);
            let height = height.max(text_h + 6.0);
            return Size::new(width + pad_w, height + pad_h);
        }
        Size::new(width + pad_w, height + pad_h)
    }

    /// Advances the smooth-easing fraction of a progress bar toward its bound
    /// value. Returns true if the drawn fraction changed this step.
    pub(crate) fn progress_advance(&mut self, id: WidgetId, ms: u32) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::ProgressBar || !node.visible {
            return false;
        }
        let target = if let Some(v64) = node.progress_f64 {
            v64.get(self).unwrap_or(0.0)
        } else {
            node.scale.and_then(|v| v.get(self)).unwrap_or(0) as f64 / 100.0
        };
        let target = if target.is_nan() { 0.0 } else { target.clamp(0.0, 1.0) } as f32;
        let Some(node) = self.node_mut(id) else { return false };
        let Some(state) = node.inputs.as_mut() else { return false };
        if !state.progress_smooth {
            return false;
        }
        if !state.progress_initialised {
            state.progress_drawn = target;
            state.progress_initialised = true;
            return false;
        }
        if (state.progress_drawn - target).abs() < 0.001 {
            if state.progress_drawn != target {
                state.progress_drawn = target;
                self.dirty = true;
                return true;
            }
            return false;
        }
        // Exponential approach with a 40 ms time constant: ~63% of the gap per
        // 40 ms, so the gap shrinks below 0.001 well within 300 ms. Composes
        // identically for many small steps and a single large step.
        let factor = 1.0 - (-(ms as f32) / 40.0).exp();
        let next = state.progress_drawn + (target - state.progress_drawn) * factor;
        let next = if (next - target).abs() < 0.001 { target } else { next };
        let changed = next != state.progress_drawn;
        state.progress_drawn = next;
        if changed {
            self.dirty = true;
        }
        changed
    }

    /// True if `id` is a visible progress bar whose smooth fraction is still
    /// easing toward its bound value.
    pub(crate) fn progress_is_animating(&self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::ProgressBar || !node.visible {
            return false;
        }
        let Some(state) = node.inputs.as_ref() else { return false };
        if !state.progress_smooth || !state.progress_initialised {
            return false;
        }
        let target = if let Some(v64) = node.progress_f64 {
            v64.get(self).unwrap_or(0.0)
        } else {
            node.scale.and_then(|v| v.get(self)).unwrap_or(0) as f64 / 100.0
        };
        let target = if target.is_nan() { 0.0 } else { target.clamp(0.0, 1.0) } as f32;
        (state.progress_drawn - target).abs() >= 0.001
    }
}
