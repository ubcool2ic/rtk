//! Canvas items, drag binding and scroll region.

#[allow(unused_imports)]
use super::*;

impl Tk {
    /// Sets the scrollable content size of a Canvas (tkinter `scrollregion=(0, 0, w, h)`).
    /// The current view is clamped to the new region.
    pub fn canvas_scrollregion(&mut self, id: WidgetId, width: f32, height: f32) -> bool {
        self.ensure_layout();
        let Some(n) = self.node_mut(id) else { return false };
        if n.kind != Kind::Canvas { return false; }
        let sane = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        n.scrollregion = Some((sane(width), sane(height)));
        let v = n.view;
        self.canvas_set_view(id, v.0, v.1);
        self.dirty = true;
        true
    }

    /// Current scroll offset of a Canvas in logical px (x, y).
    pub fn canvas_view(&self, id: WidgetId) -> Option<(f32, f32)> {
        self.node(id).filter(|n| n.kind == Kind::Canvas).map(|n| n.view)
    }

    pub(crate) fn dispatch_canvas_drag(&mut self, id: WidgetId, cx: f32, cy: f32, phase: DragPhase) {
        let Some(node) = self.node_mut(id) else { return };
        if node.canvas_drag_running {
            return;
        }
        let Some(mut cb) = node.canvas_drag.take() else { return };
        node.canvas_drag_running = true;
        cb(self, cx, cy, phase);
        if let Some(node) = self.node_mut(id) {
            node.canvas_drag_running = false;
            if node.canvas_drag.is_none() {
                node.canvas_drag = Some(cb);
            }
        }
    }


    // Canvas methods
    /// Creates a filled rectangle from `(x0, y0)` to `(x1, y1)` in canvas coordinates.
    pub fn canvas_create_rect(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Rect { x0, y0, x1, y1, color })
    }

    /// Creates a filled ellipse inscribed in the box `(x0, y0)`–`(x1, y1)`.
    pub fn canvas_create_oval(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Oval { x0, y0, x1, y1, color })
    }

    /// Creates a straight line segment from `(x0, y0)` to `(x1, y1)`.
    pub fn canvas_create_line(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Line { x0, y0, x1, y1, color })
    }

    /// Creates a text item anchored at `(x, y)` using the default font.
    pub fn canvas_create_text(&mut self, id: WidgetId, x: f32, y: f32, text: impl Into<String>, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Text { x, y, text: text.into(), color })
    }

    pub(crate) fn canvas_add_item(&mut self, id: WidgetId, kind: CanvasItemKind) -> Option<CanvasItem> {
        const MAX_CANVAS_ITEMS: usize = 1 << 20;
        let node = self.node_mut(id)?;
        if node.kind != Kind::Canvas {
            return None;
        }
        let items = &mut node.canvas_items;
        let seq = items.iter().map(|slot| slot.seq).max().map_or(0, |max| max.saturating_add(1));
        let free = items.iter().position(|slot| slot.item.is_none() && slot.gen < u32::MAX);
        let item = if let Some(index) = free {
            let slot = items.get_mut(index)?;
            slot.item = Some(kind);
            slot.seq = seq;
            CanvasItem { index: u32::try_from(index).ok()?, gen: slot.gen }
        } else {
            if items.len() >= MAX_CANVAS_ITEMS {
                return None;
            }
            let index = u32::try_from(items.len()).ok()?;
            items.push(CanvasItemSlot { gen: 0, seq, item: Some(kind) });
            CanvasItem { index, gen: 0 }
        };
        self.dirty = true;
        Some(item)
    }

    /// Deletes every item on the canvas (tkinter `delete("all")`).
    pub fn canvas_clear(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        for slot in node.canvas_items.iter_mut().filter(|slot| slot.item.is_some()) {
            slot.item = None;
            slot.gen = slot.gen.saturating_add(1);
        }
        self.dirty = true;
        true
    }

    /// Deletes a single item; returns false if the handle is stale.
    pub fn canvas_delete(&mut self, id: WidgetId, item: CanvasItem) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        let Some(slot) = node.canvas_items.get_mut(item.index as usize) else { return false };
        if slot.gen != item.gen || slot.item.is_none() {
            return false;
        }
        slot.item = None;
        slot.gen = slot.gen.saturating_add(1);
        self.dirty = true;
        true
    }

    /// Translates an item by `(dx, dy)` logical px (tkinter `move`).
    pub fn canvas_move(&mut self, id: WidgetId, item: CanvasItem, dx: f32, dy: f32) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        let Some(slot) = node.canvas_items.get_mut(item.index as usize) else { return false };
        if slot.gen != item.gen {
            return false;
        }
        let Some(kind) = slot.item.as_mut() else { return false };
        kind.translate(dx, dy);
        self.dirty = true;
        true
    }

    /// Returns the item's bounding box as `[x0, y0, x1, y1]` in canvas coordinates.
    pub fn canvas_coords(&self, id: WidgetId, item: CanvasItem) -> Option<[f32; 4]> {
        let node = self.node(id)?;
        if node.kind != Kind::Canvas {
            return None;
        }
        let slot = node.canvas_items.get(item.index as usize)?;
        if slot.gen != item.gen {
            return None;
        }
        match slot.item.as_ref()? {
            CanvasItemKind::Rect { x0, y0, x1, y1, .. }
            | CanvasItemKind::Oval { x0, y0, x1, y1, .. }
            | CanvasItemKind::Line { x0, y0, x1, y1, .. } => Some([*x0, *y0, *x1, *y1]),
            CanvasItemKind::Text { x, y, .. } | CanvasItemKind::TextEx { x, y, .. } => Some([*x, *y, *x, *y]),
            CanvasItemKind::RoundRect { x0, y0, x1, y1, .. } | CanvasItemKind::Ring { x0, y0, x1, y1, .. } => Some([*x0, *y0, *x1, *y1]),
            CanvasItemKind::Image { x, y, w, h, .. } => Some([*x, *y, *x + *w, *y + *h]),
            CanvasItemKind::Path { points, .. } => {
                let (mut a, mut b, mut c, mut d) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for (x, y) in points { a = a.min(*x); b = b.min(*y); c = c.max(*x); d = d.max(*y); }
                if points.is_empty() { None } else { Some([a, b, c, d]) }
            }
        }
    }

    /// Filled rounded rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_round_rect(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, radius: f32, fill: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::RoundRect { x0, y0, x1, y1, radius, fill: Some(fill), outline: None })
    }

    /// Rounded rectangle with optional fill and an outline of `width` logical pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_round_rect_outline(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, radius: f32, fill: Option<Color>, outline: Color, width: f32) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::RoundRect { x0, y0, x1, y1, radius, fill, outline: Some((outline, width)) })
    }

    /// Stroked polyline through `points` (e.g. sparklines, icon strokes). Needs at least two points.
    pub fn canvas_create_polyline(&mut self, id: WidgetId, points: &[(f32, f32)], color: Color, width: f32) -> Option<CanvasItem> {
        if points.len() < 2 { return None; }
        self.canvas_add_item(id, CanvasItemKind::Path { points: points.to_vec(), color, width, filled: false })
    }

    /// Filled closed polygon. Needs at least three points.
    pub fn canvas_create_polygon(&mut self, id: WidgetId, points: &[(f32, f32)], color: Color) -> Option<CanvasItem> {
        if points.len() < 3 { return None; }
        self.canvas_add_item(id, CanvasItemKind::Path { points: points.to_vec(), color, width: 0.0, filled: true })
    }

    /// Ellipse outline inside the box, stroked `width` logical pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_ring(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Ring { x0, y0, x1, y1, color, width })
    }

    /// Text with its own pixel size and weight; `(x, y)` is the top-left.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_text_ex(&mut self, id: WidgetId, x: f32, y: f32, text: impl Into<String>, color: Color, size: f32, weight: u16) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::TextEx { x, y, text: text.into(), color, size: size.max(1.0), weight: weight.clamp(100, 900) })
    }

    /// Measures text in logical pixels (width, height) for layout/centering on a Canvas.
    pub fn text_size(&mut self, text: &str, size: f32, weight: u16) -> (f32, f32) {
        let family = self.style_engine.tokens().font_family.clone();
        let s = self.text_engine.measure_styled(text, size, family.as_deref(), weight.clamp(100, 900), false);
        (s.width, s.height)
    }

    /// Number of live items on the canvas.
    pub fn canvas_items(&self, id: WidgetId) -> usize {
        self.node(id)
            .filter(|n| n.kind == Kind::Canvas)
            .map(|n| n.canvas_items.iter().filter(|s| s.item.is_some()).count())
            .unwrap_or(0)
    }

    /// Binds a drag callback receiving canvas coordinates and the drag phase.
    pub fn canvas_bind_drag(&mut self, id: WidgetId, cb: impl FnMut(&mut Tk, f32, f32, DragPhase) + 'static) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        node.canvas_drag = Some(Box::new(cb));
        true
    }

    /// Creates a raw RGBA image item scaled to `w` × `h` logical pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_image_rgba(&mut self, id: WidgetId, x: f32, y: f32, w: f32, h: f32, src_w: u32, src_h: u32, rgba: &[u8], smooth: bool) -> Option<CanvasItem> {
        if src_w == 0 || src_h == 0 || rgba.len() != (src_w as usize) * (src_h as usize) * 4 {
            return None;
        }
        self.canvas_add_item(id, CanvasItemKind::Image { x, y, w, h, src_w, src_h, rgba: rgba.to_vec(), smooth })
    }

    /// Updates the RGBA pixel data for a canvas image item.
    pub fn canvas_update_image(&mut self, id: WidgetId, item: CanvasItem, src_w: u32, src_h: u32, rgba: &[u8]) -> bool {
        if src_w == 0 || src_h == 0 || rgba.len() != (src_w as usize) * (src_h as usize) * 4 {
            return false;
        }
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas { return false; }
        let Some(slot) = node.canvas_items.get_mut(item.index as usize) else { return false };
        if slot.gen != item.gen { return false; }
        let Some(kind) = slot.item.as_mut() else { return false };
        if let CanvasItemKind::Image { src_w: sw, src_h: sh, rgba: buf, .. } = kind {
            *sw = src_w;
            *sh = src_h;
            *buf = rgba.to_vec();
            self.dirty = true;
            true
        } else {
            false
        }
    }

}
