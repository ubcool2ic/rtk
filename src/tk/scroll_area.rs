//! ScrollArea widget: a clipped viewport over an inner content frame.
//!
//! A [`ScrollArea`] owns exactly one child, its content `Frame`. Users add
//! widgets to that frame with the normal constructors and geometry managers;
//! the area scrolls the frame (and its whole subtree) inside its viewport and
//! clips everything to the viewport rectangle.
//!
//! All scroll state lives in [`crate::tk::inputs_state::InputsState`]; this
//! file holds the constructor namespace and the `Tk` methods that read and
//! write that state.

#[allow(unused_imports)]
use super::*;

/// Constructor namespace for scrollable viewport frames.
pub struct ScrollArea;

#[allow(clippy::new_ret_no_self)]
impl ScrollArea {
    /// Creates a scrollable viewport of `width` × `height` logical px under
    /// `parent`, together with an inner content `Frame` that receives the
    /// user's widgets (see [`Tk::scrollarea_content`]).
    ///
    /// Non-finite or negative sizes are clamped to `0.0`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        width: f32,
        height: f32,
    ) -> Result<WidgetId, RtkError> {
        // Stage the viewport as a Frame so the normal parent validation can
        // create its sole child. Once initialized, the ScrollArea itself is
        // deliberately not a valid parent for user-created widgets.
        let id = tk.add(Kind::Frame, parent, String::new())?;
        let content = match tk.add(Kind::Frame, id, String::new()) {
            Ok(content) => content,
            Err(error) => {
                tk.destroy(id);
                return Err(error);
            }
        };
        let sane = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        if let Some(node) = tk.node_mut(id) {
            node.kind = Kind::ScrollArea;
            node.canvas_size = (sane(width), sane(height));
            let state = node.inputs.get_or_insert_with(Default::default);
            state.scroll_area_content = Some(content);
            state.scroll_area_offset = (0.0, 0.0);
        }
        if let Some(node) = tk.node_mut(content) {
            node.scroll_area_owner = Some(id);
        }
        tk.dirty = true;
        Ok(id)
    }
}

impl Tk {
    /// Returns the inner content `Frame` of a ScrollArea, or `None` if `area`
    /// is not a live ScrollArea.
    pub fn scrollarea_content(&self, area: WidgetId) -> Option<WidgetId> {
        let node = self.node(area).filter(|n| n.kind == Kind::ScrollArea)?;
        node.inputs.as_ref().and_then(|s| s.scroll_area_content)
    }

    /// Returns the current scroll offset `(x, y)` of a ScrollArea in logical px.
    pub fn scrollarea_offset(&self, area: WidgetId) -> Option<(f32, f32)> {
        let node = self.node(area).filter(|n| n.kind == Kind::ScrollArea)?;
        Some(node.inputs.as_ref().map_or((0.0, 0.0), |s| s.scroll_area_offset))
    }

    /// Returns the content size `(width, height)` of a ScrollArea in logical px
    /// (the content frame's laid-out size, at least the viewport size).
    pub fn scrollarea_content_size(&mut self, area: WidgetId) -> Option<(f32, f32)> {
        self.ensure_layout();
        let node = self.node(area).filter(|n| n.kind == Kind::ScrollArea)?;
        let content = node.inputs.as_ref().and_then(|s| s.scroll_area_content)?;
        let cnode = self.node(content)?;
        Some((cnode.rect.width, cnode.rect.height))
    }

    /// Scrolls a ScrollArea to `(x, y)`, clamped to
    /// `0..=(content - viewport)` on each axis. Returns false if `area` is not
    /// a live ScrollArea.
    pub fn scrollarea_scroll_to(&mut self, area: WidgetId, x: f32, y: f32) -> bool {
        self.ensure_layout();
        let Some(node) = self.node(area).filter(|n| n.kind == Kind::ScrollArea) else {
            return false;
        };
        let Some(content) = node.inputs.as_ref().and_then(|s| s.scroll_area_content) else {
            return false;
        };
        let viewport = node.rect;
        let content_size = self
            .node(content)
            .map(|c| (c.rect.width, c.rect.height))
            .unwrap_or((viewport.width, viewport.height));
        let max_x = (content_size.0 - viewport.width).max(0.0);
        let max_y = (content_size.1 - viewport.height).max(0.0);
        let nx = if x.is_nan() { 0.0 } else { x.clamp(0.0, max_x) };
        let ny = if y.is_nan() { 0.0 } else { y.clamp(0.0, max_y) };
        let changed = self
            .node(area)
            .and_then(|n| n.inputs.as_ref())
            .is_none_or(|s| s.scroll_area_offset != (nx, ny));
        if changed {
            if let Some(state) = self.node_mut(area).and_then(|n| n.inputs.as_mut()) {
                state.scroll_area_offset = (nx, ny);
            }
            self.dirty = true;
        }
        true
    }

    /// Links existing Scrollbar widgets to a ScrollArea so they reflect and
    /// drive its offset. `vbar` should be a vertical scrollbar, `hbar` a
    /// horizontal one; either may be `None`. Returns false if `area` is not a
    /// live ScrollArea or a given widget is not a Scrollbar.
    pub fn scrollarea_attach_scrollbars(
        &mut self,
        area: WidgetId,
        vbar: Option<WidgetId>,
        hbar: Option<WidgetId>,
    ) -> bool {
        if !self.node(area).is_some_and(|n| n.kind == Kind::ScrollArea) {
            return false;
        }
        for bar in [vbar, hbar].into_iter().flatten() {
            if !self.node(bar).is_some_and(|n| n.kind == Kind::Scrollbar) {
                return false;
            }
        }
        if let Some(bar) = vbar {
            if let Some(node) = self.node_mut(bar) {
                node.scroll_target = Some(area);
            }
        }
        if let Some(bar) = hbar {
            if let Some(node) = self.node_mut(bar) {
                node.scroll_target = Some(area);
            }
        }
        self.dirty = true;
        true
    }

    /// Returns the nearest ScrollArea ancestor of `id` (including `id` itself),
    /// or `None` when the widget is not inside a scroll area.
    pub(crate) fn nearest_scroll_area(&self, id: WidgetId) -> Option<WidgetId> {
        let mut current = Some(id);
        while let Some(cursor) = current {
            let node = self.node(cursor)?;
            if node.kind == Kind::ScrollArea {
                return Some(cursor);
            }
            current = node.parent;
        }
        None
    }

    /// Scrolls `area` by `(dx, dy)` logical px, clamped to the content bounds.
    pub(crate) fn scrollarea_scroll_by(&mut self, area: WidgetId, dx: f32, dy: f32) {
        let (x, y) = self.scrollarea_offset(area).unwrap_or((0.0, 0.0));
        self.scrollarea_scroll_to(area, x + dx, y + dy);
    }

    /// Scrolls the nearest ScrollArea of `id` so that `id`'s rectangle is
    /// fully visible inside the viewport. No-op when `id` is not inside a
    /// scroll area or is already visible.
    pub(crate) fn scrollarea_reveal(&mut self, id: WidgetId) {
        self.ensure_layout();
        let Some(area) = self.nearest_scroll_area(id) else { return };
        let Some(area_rect) = self.node(area).map(|n| n.rect) else { return };
        let Some(target) = self.node(id).map(|n| n.rect) else { return };
        let (ox, oy) = self.scrollarea_offset(area).unwrap_or((0.0, 0.0));
        let mut nx = ox;
        let mut ny = oy;
        if target.x < area_rect.x {
            nx = ox - (area_rect.x - target.x);
        } else if target.x + target.width > area_rect.x + area_rect.width {
            nx = ox + (target.x + target.width - (area_rect.x + area_rect.width));
        }
        if target.y < area_rect.y {
            ny = oy - (area_rect.y - target.y);
        } else if target.y + target.height > area_rect.y + area_rect.height {
            ny = oy + (target.y + target.height - (area_rect.y + area_rect.height));
        }
        if (nx, ny) != (ox, oy) {
            self.scrollarea_scroll_to(area, nx, ny);
        }
    }

    /// Requested size of a ScrollArea: its viewport size plus padding.
    pub(crate) fn scrollarea_requested(&mut self, id: WidgetId, resolved: &ResolvedStyle) -> Size {
        let node = self.node(id);
        let (w, h) = node.map_or((100.0, 100.0), |n| n.canvas_size);
        let pad_w = resolved.padding.left + resolved.padding.right;
        let pad_h = resolved.padding.top + resolved.padding.bottom;
        Size::new(w + pad_w, h + pad_h)
    }

    /// Returns the width in logical px reserved for a visible vertical
    /// scrollbar attached to `area` (0 when none is attached or it is hidden).
    ///
    /// A scrollbar counts as attached when its `scroll_target` points at
    /// `area`; it may live anywhere in the widget tree (typically a sibling of
    /// the area under the same parent), not only as a direct child. The
    /// reserved width is the bar's laid-out width when available, falling back
    /// to its requested width (and finally to the 16 px default) before the
    /// first layout pass has run.
    pub(crate) fn scrollarea_vbar_width(&self, area: WidgetId) -> f32 {
        if !self.node(area).is_some_and(|n| n.kind == Kind::ScrollArea) {
            return 0.0;
        }
        let mut width = 0.0_f32;
        for slot in &self.nodes {
            let Some(bar) = slot.node.as_ref() else { continue };
            if bar.kind != Kind::Scrollbar || bar.scroll_target != Some(area) {
                continue;
            }
            let horizontal = bar.inputs.as_ref().is_some_and(|s| s.scrollbar_horizontal);
            if horizontal {
                continue;
            }
            let laid_out = bar.rect.width;
            let requested = bar.requested.width;
            let candidate = if laid_out > 0.0 {
                laid_out
            } else if requested > 0.0 {
                requested
            } else {
                16.0
            };
            width = width.max(candidate);
        }
        width
    }
}

