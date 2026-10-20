//! Dirty-rectangle (partial) redraw support.
//!
//! Widgets and the toolkit mark logical-pixel rectangles as damaged with
//! [`Tk::invalidate_rect`] / [`Tk::invalidate_widget`]. [`Tk::render_frame`]
//! then either performs a full render (returning [`FrameDamage::Full`]) or
//! repaints only the damaged regions into a scratch pixmap and copies them into
//! the cached frame (returning [`FrameDamage::Partial`]). Full renders always go
//! through [`Tk::render_to_pixmap`], so its output is unchanged.

#[allow(unused_imports)]
use super::*;

/// The outcome of [`Tk::render_frame`].
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum FrameDamage {
    /// The whole frame was repainted.
    Full,
    /// Only these device-pixel rectangles changed (integer-aligned, clamped to
    /// the frame).
    Partial(Vec<crate::layout::Rect>),
    /// Nothing needed repainting.
    None,
}

/// Maximum number of damaged rectangles kept before they are coalesced into
/// their union bounding box.
const MAX_DAMAGE_RECTS: usize = 16;

/// Logical-pixel padding added around an invalidated widget for focus rings and
/// shadows.
const WIDGET_INFLATE: f32 = 4.0;

impl Tk {
    /// Marks a logical-pixel rectangle as damaged, requesting a partial redraw.
    ///
    /// Empty or non-finite rectangles are ignored. When more than
    /// [`MAX_DAMAGE_RECTS`] rectangles accumulate they are replaced by their
    /// union bounding box.
    pub fn invalidate_rect(&mut self, rect: Rect) {
        if !rect.x.is_finite()
            || !rect.y.is_finite()
            || !rect.width.is_finite()
            || !rect.height.is_finite()
            || rect.width <= 0.0
            || rect.height <= 0.0
        {
            return;
        }
        self.damage.push(rect);
        self.partial_dirty = true;
        // Once the list reaches the cap, fold everything (including any new
        // rects) into a single union bounding box. While the list is already
        // coalesced (exactly one rect), keep merging new rects into that union
        // so it never grows past one rectangle.
        if self.damage.len() >= MAX_DAMAGE_RECTS || self.coalesced {
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            for r in &self.damage {
                min_x = min_x.min(r.x);
                min_y = min_y.min(r.y);
                max_x = max_x.max(r.x + r.width);
                max_y = max_y.max(r.y + r.height);
            }
            self.damage.clear();
            if min_x.is_finite() && min_y.is_finite() && max_x > min_x && max_y > min_y {
                self.damage.push(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y));
            }
            self.coalesced = true;
        }
    }

    /// Marks the visible area of widget `id` as damaged.
    ///
    /// For widgets without a shadow this uses `node.clip ∩ node.rect` inflated
    /// by [`WIDGET_INFLATE`] logical px to cover focus rings. For widgets whose
    /// resolved style carries a drop shadow the base is `node.rect` (the shadow
    /// is drawn without clipping) inflated by [`WIDGET_INFLATE`] plus the
    /// shadow's blur and offset so the whole shadow is repainted. Widgets that
    /// draw overlays outside their own rect (Combobox dropdowns, Notebook tabs,
    /// Canvas content) fall back to a full redraw.
    pub fn invalidate_widget(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        let kind = node.kind;
        let rect = node.rect;
        let clip = node.clip;
        if matches!(kind, Kind::Combobox | Kind::Notebook | Kind::Canvas) {
            self.dirty = true;
            return;
        }
        let shadow = self
            .resolved_style(id)
            .and_then(|style| style.shadow);
        if let Some(shadow) = shadow {
            // The shadow is drawn unclipped, offset by (dx, dy) and spreading up
            // to `blur` logical px, so damage the widget rect plus that reach.
            let reach = WIDGET_INFLATE
                + shadow.blur
                + shadow.dx.abs().max(shadow.dy.abs())
                + 1.0;
            if rect.width <= 0.0 || rect.height <= 0.0 {
                return;
            }
            self.invalidate_rect(Rect::new(
                rect.x - reach,
                rect.y - reach,
                rect.width + 2.0 * reach,
                rect.height + 2.0 * reach,
            ));
            return;
        }
        let area = rect.intersection(clip);
        if area.width <= 0.0 || area.height <= 0.0 {
            return;
        }
        self.invalidate_rect(Rect::new(
            area.x - WIDGET_INFLATE,
            area.y - WIDGET_INFLATE,
            area.width + 2.0 * WIDGET_INFLATE,
            area.height + 2.0 * WIDGET_INFLATE,
        ));
    }

    /// Returns true when a full or partial redraw is pending.
    pub fn needs_redraw(&self) -> bool {
        self.dirty || self.partial_dirty
    }

    /// Returns the last frame produced by [`Tk::render_frame`], if any.
    pub fn frame(&self) -> Option<&Pixmap> {
        self.frame.as_ref()
    }

    /// True when an overlay that is drawn outside widget rects is visible, which
    /// forces a full redraw.
    fn overlays_visible(&self) -> bool {
        self.open_combobox.is_some()
            || self.tooltip_visible
            || self.modal.is_some()
            || self.menu_is_open()
            || !self.toasts.list.is_empty()
    }

    /// Converts the pending logical damage rectangles to integer-aligned device
    /// rectangles clamped to a `width`×`height` frame.
    fn damage_device_rects(&self, width: u32, height: u32) -> Vec<Rect> {
        let scale = self.scale_factor;
        let mut out = Vec::with_capacity(self.damage.len());
        for r in &self.damage {
            let x0 = (r.x * scale).floor().clamp(0.0, width as f32);
            let y0 = (r.y * scale).floor().clamp(0.0, height as f32);
            let x1 = ((r.x + r.width) * scale).ceil().clamp(0.0, width as f32);
            let y1 = ((r.y + r.height) * scale).ceil().clamp(0.0, height as f32);
            if x1 > x0 && y1 > y0 {
                out.push(Rect::new(x0, y0, x1 - x0, y1 - y0));
            }
        }
        out
    }

    /// Renders the next frame, performing a full or partial redraw as needed.
    ///
    /// A full render happens when a full redraw is pending, no frame is cached,
    /// the frame size changed, an overlay is visible, or an animation is running
    /// on a widget that is not damaged. Otherwise only the damaged rectangles are
    /// repainted into a scratch pixmap and copied into the cached frame.
    pub fn render_frame(&mut self, width: u32, height: u32) -> Result<FrameDamage, RtkError> {
        let size_matches = self
            .frame
            .as_ref()
            .is_some_and(|frame| frame.width() == width && frame.height() == height);
        let animating_undamaged = self.is_animating()
            && self.nodes.iter().filter_map(|slot| slot.node.as_ref()).any(|node| {
                node.transition.is_some()
                    && node.visible
                    && !self.damage.iter().any(|r| r.intersection(node.rect).width > 0.0)
            });
        if self.dirty
            || !size_matches
            || self.overlays_visible()
            || animating_undamaged
        {
            let pixmap = self.render_to_pixmap(width, height)?;
            self.frame = Some(pixmap);
            self.damage.clear();
            self.partial_dirty = false;
            self.coalesced = false;
            return Ok(FrameDamage::Full);
        }
        if !self.partial_dirty {
            return Ok(FrameDamage::None);
        }
        let rects = self.damage_device_rects(width, height);
        if rects.is_empty() {
            self.damage.clear();
            self.partial_dirty = false;
            self.coalesced = false;
            return Ok(FrameDamage::None);
        }
        let needs_scratch = self
            .scratch
            .as_ref()
            .is_none_or(|s| s.width() != width || s.height() != height);
        if needs_scratch {
            self.scratch = Pixmap::new(width, height);
        }
        let Some(mut scratch) = self.scratch.take() else {
            // Scratch allocation failed; fall back to a full render.
            let pixmap = self.render_to_pixmap(width, height)?;
            self.frame = Some(pixmap);
            self.damage.clear();
            self.partial_dirty = false;
            return Ok(FrameDamage::Full);
        };
        let union = rects.iter().skip(1).fold(rects[0], |acc, r| {
            let x = acc.x.min(r.x);
            let y = acc.y.min(r.y);
            let right = (acc.x + acc.width).max(r.x + r.width);
            let bottom = (acc.y + acc.height).max(r.y + r.height);
            Rect::new(x, y, right - x, bottom - y)
        });
        self.paint_scene(&mut scratch, Some(union));
        if let Some(frame) = self.frame.as_mut() {
            let stride = width as usize * 4;
            let src = scratch.data();
            let dst = frame.data_mut();
            for r in &rects {
                let x0 = r.x as usize;
                let y0 = r.y as usize;
                let x1 = (r.x + r.width) as usize;
                let y1 = (r.y + r.height) as usize;
                for y in y0..y1 {
                    let start = y * stride + x0 * 4;
                    let end = y * stride + x1 * 4;
                    if let (Some(src_row), Some(dst_row)) =
                        (src.get(start..end), dst.get_mut(start..end))
                    {
                        dst_row.copy_from_slice(src_row);
                    }
                }
            }
        }
        self.scratch = Some(scratch);
        self.damage.clear();
        self.partial_dirty = false;
        self.coalesced = false;
        Ok(FrameDamage::Partial(rects))
    }
}
