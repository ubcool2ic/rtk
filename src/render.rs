use std::error::Error;
use std::path::Path;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::layout::{Rect, Size};
use crate::tk::cursor_boundary;
use crate::widgets::{CanvasItemKind, Kind, Theme};
use crate::{Color, RtkError, Tk};

pub(crate) struct TextEngine {
    fonts: FontSystem,
    cache: SwashCache,
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            fonts: FontSystem::new(),
            cache: SwashCache::new(),
        }
    }

    fn has_fonts(&self) -> bool {
        self.fonts.db().faces().next().is_some()
    }

    fn buffer(&mut self, text: &str, size: f32) -> Option<Buffer> {
        if !self.has_fonts() || !size.is_finite() || size <= 0.0 {
            return None;
        }
        let mut buffer = Buffer::new(&mut self.fonts, Metrics::new(size, size * 1.3));
        buffer.set_size(&mut self.fonts, None, None);
        buffer.set_wrap(&mut self.fonts, Wrap::None);
        buffer.set_text(
            &mut self.fonts,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.fonts, false);
        Some(buffer)
    }

    pub fn measure(&mut self, text: &str, size: f32) -> Size {
        let width = if let Some(buffer) = self.buffer(text, size) {
            buffer.layout_runs().map(|run| run.line_w).fold(0.0_f32, f32::max)
        } else {
            text.split('\n').map(|line| line.chars().count()).max().unwrap_or(0) as f32 * size * 0.6
        };
        Size::new(width, text.split('\n').count() as f32 * size * 1.3)
    }

    fn draw(
        &mut self,
        pixmap: &mut Pixmap,
        text: &str,
        appearance: (f32, Color),
        origin: (f32, f32),
        clip: Rect,
    ) {
        let (size, color) = appearance;
        let Some(buffer) = self.buffer(text, size) else { return };
        let origin = (origin.0.round(), origin.1.round());
        buffer.draw(
            &mut self.fonts,
            &mut self.cache,
            cosmic_text::Color::rgba(color.r, color.g, color.b, color.a),
            |x, y, width, height, color| {
                fill(
                    pixmap,
                    Rect::new(origin.0 + x as f32, origin.1 + y as f32, width as f32, height as f32),
                    clip,
                    Color::rgba(color.r(), color.g(), color.b(), color.a()),
                );
            },
        );
    }
}

fn fill(pixmap: &mut Pixmap, rect: Rect, clip: Rect, color: Color) {
    let rect = rect.intersection(clip);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    if let Some(rect) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = false;
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn outline(pixmap: &mut Pixmap, rect: Rect, clip: Rect, width: f32, colors: (Color, Color)) {
    let width = width.min(rect.width / 2.0).min(rect.height / 2.0);
    let (top, bottom) = colors;
    fill(pixmap, Rect::new(rect.x, rect.y, rect.width, width), clip, top);
    fill(pixmap, Rect::new(rect.x, rect.y, width, rect.height), clip, top);
    fill(
        pixmap,
        Rect::new(rect.x, rect.y + rect.height - width, rect.width, width),
        clip, bottom,
    );
    fill(
        pixmap,
        Rect::new(rect.x + rect.width - width, rect.y, width, rect.height),
        clip, bottom,
    );
}

fn fill_circle(pixmap: &mut Pixmap, cx: f32, cy: f32, r: f32, clip: Rect, color: Color) {
    let mut pb = PathBuilder::new();
    let d = r * 0.552_284_8;
    pb.move_to(cx, cy - r);
    pb.cubic_to(cx + d, cy - r, cx + r, cy - d, cx + r, cy);
    pb.cubic_to(cx + r, cy + d, cx + d, cy + r, cx, cy + r);
    pb.cubic_to(cx - d, cy + r, cx - r, cy + d, cx - r, cy);
    pb.cubic_to(cx - r, cy - d, cx - d, cy - r, cx, cy - r);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        if let Some(clip_rect) = tiny_skia::Rect::from_xywh(clip.x, clip.y, clip.width, clip.height) {
            let mut clip_mask = tiny_skia::Mask::new(pixmap.width(), pixmap.height());
            if let Some(mask) = clip_mask.as_mut() {
                mask.fill_path(&tiny_skia::PathBuilder::from_rect(clip_rect), tiny_skia::FillRule::Winding, false, Transform::identity());
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), Some(mask));
            } else {
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
    }
}

impl Tk {
    pub fn render_to_pixmap(&mut self, width: u32, height: u32) -> Result<Pixmap, RtkError> {
        self.sim_resize(width, height);
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 16_777_216 {
            self.dirty = false;
            return Err(RtkError::InvalidImageSize);
        }
        let mut pixmap = Pixmap::new(width, height).ok_or(RtkError::InvalidImageSize)?;
        let default_bg = if self.theme == Theme::Dark {
            Color::rgb(45, 45, 45)
        } else {
            Color::BG
        };
        let bg = self.node(self.root()).map(|node| node.style.bg).unwrap_or(default_bg);
        pixmap.fill(tiny_skia::Color::from_rgba8(bg.r, bg.g, bg.b, bg.a));
        let scale = self.scale_factor;

        let dark_mode = self.theme == Theme::Dark;
        let border_light = if dark_mode { Color::rgb(70, 70, 70) } else { Color::rgb(250, 250, 250) };
        let border_dark = if dark_mode { Color::rgb(20, 20, 20) } else { Color::rgb(110, 110, 110) };
        let active_bg = if dark_mode { Color::rgb(60, 60, 60) } else { Color::ACTIVE };

        for id in self.paint_order.clone() {
            let Some(node) = self.node(id) else { continue };
            let (kind, logical, logical_clip, style) = (node.kind, node.rect, node.clip, node.style);
            let (cursor, old_scroll, check) = (node.cursor, node.scroll_x, node.check);
            let rect = logical.scaled(scale);
            let clip = logical_clip.scaled(scale);
            if clip.width <= 0.0 || clip.height <= 0.0 {
                continue;
            }
            let pressed = self.pressed == Some(id) && self.hovered == Some(id);
            let bg = if kind.has_command() && self.hovered == Some(id) {
                active_bg
            } else {
                style.bg
            };
            fill(&mut pixmap, rect, clip, bg);

            if matches!(kind, Kind::Button | Kind::Entry | Kind::Spinbox) || (kind == Kind::Frame && id != self.root()) {
                let raised = kind == Kind::Button && !pressed;
                outline(
                    &mut pixmap, rect, clip, scale,
                    if raised { (border_light, border_dark) } else { (border_dark, border_light) },
                );
            }
            if self.focused == Some(id) && kind.focusable() {
                let color = Color::rgb(40, 95, 190);
                outline(&mut pixmap, rect.inset(2.0 * scale, 2.0 * scale), clip, scale, (color, color));
            }
            if kind == Kind::Frame {
                continue;
            }

            match kind {
                Kind::Scale => {
                    let track = logical.inset(12.0, 0.0);
                    let y = logical.y + logical.height / 2.0;
                    fill(
                        &mut pixmap,
                        Rect::new(track.x, y - 2.0, track.width, 4.0).scaled(scale),
                        clip, border_dark,
                    );
                    let x = track.x + track.width * self.scale_fraction(id);
                    fill(
                        &mut pixmap,
                        Rect::new(x - 5.0, y - 9.0, 10.0, 18.0).scaled(scale),
                        clip, Color::rgb(65, 105, 175),
                    );
                }
                Kind::ProgressBar => {
                    outline(&mut pixmap, rect, clip, scale, (border_dark, border_light));
                    let val = node.scale.and_then(|v| v.get(self)).unwrap_or(0).clamp(0, 100);
                    let frac = (val as f32) / 100.0;
                    let inner = logical.inset(2.0, 2.0);
                    let filled_w = inner.width * frac;
                    if filled_w > 0.0 {
                        let fill_rect = Rect::new(inner.x, inner.y, filled_w, inner.height).scaled(scale);
                        fill(&mut pixmap, fill_rect, clip, Color::rgb(40, 140, 220));
                    }
                }
                Kind::Entry => {
                    let inner = logical.inset(6.0, 5.0);
                    let text = self.widget_text(id).unwrap_or_default();
                    let cursor = cursor_boundary(&text, cursor);
                    let caret = self.measure(text.get(..cursor).unwrap_or_default(), style.font_size).width;
                    let available = (inner.width - 1.0).max(0.0);
                    let mut scroll = old_scroll.max(0.0);
                    if caret < scroll {
                        scroll = caret;
                    } else if caret > scroll + available {
                        scroll = caret - available;
                    }
                    let total = self.measure(&text, style.font_size).width;
                    scroll = scroll.min((total - available).max(0.0));
                    if let Some(node) = self.node_mut(id) {
                        node.scroll_x = scroll;
                        node.cursor = cursor;
                    }
                    let origin = (
                        (inner.x - scroll) * scale,
                        (logical.y + (logical.height - style.font_size * 1.3) / 2.0) * scale,
                    );
                    let text_clip = clip.intersection(inner.scaled(scale));
                    self.text_engine.draw(
                        &mut pixmap, &text, (style.font_size * scale, style.fg), origin, text_clip,
                    );
                    if self.focused == Some(id) {
                        fill(
                            &mut pixmap,
                            Rect::new(inner.x - scroll + caret, inner.y, 1.0, inner.height).scaled(scale),
                            text_clip, style.fg,
                        );
                    }
                }
                Kind::Spinbox => {
                    let inner = Rect::new(logical.x, logical.y, (logical.width - 20.0).max(0.0), logical.height);
                    let text = format!("{}", node.scale.and_then(|v| v.get(self)).unwrap_or(node.range.0));
                    let origin = (
                        (inner.x + 6.0) * scale,
                        (inner.y + (inner.height - style.font_size * 1.3) / 2.0) * scale,
                    );
                    let text_clip = clip.intersection(inner.scaled(scale));
                    self.text_engine.draw(
                        &mut pixmap, &text, (style.font_size * scale, style.fg), origin, text_clip,
                    );
                    // Buttons on the right
                    let btn_rect = Rect::new(logical.x + logical.width - 20.0, logical.y, 20.0, logical.height).scaled(scale);
                    fill(&mut pixmap, btn_rect, clip, if dark_mode { Color::rgb(55, 55, 55) } else { Color::BG });
                    outline(&mut pixmap, btn_rect, clip, scale, (border_light, border_dark));
                    let half_h = btn_rect.height / 2.0;
                    fill(&mut pixmap, Rect::new(btn_rect.x, btn_rect.y + half_h, btn_rect.width, scale), clip, border_dark);
                    // Arrow text
                    let up_text = "▲";
                    let dn_text = "▼";
                    let up_m = self.measure(up_text, 8.0);
                    let dn_m = self.measure(dn_text, 8.0);
                    self.text_engine.draw(
                        &mut pixmap, up_text, (8.0 * scale, style.fg),
                        (btn_rect.x + (btn_rect.width - up_m.width * scale) / 2.0, btn_rect.y + (half_h - up_m.height * scale) / 2.0),
                        clip,
                    );
                    self.text_engine.draw(
                        &mut pixmap, dn_text, (8.0 * scale, style.fg),
                        (btn_rect.x + (btn_rect.width - dn_m.width * scale) / 2.0, btn_rect.y + half_h + (half_h - dn_m.height * scale) / 2.0),
                        clip,
                    );
                }
                Kind::Listbox => {
                    outline(&mut pixmap, rect, clip, scale, (border_dark, border_light));
                    let line_h = (style.font_size * 1.3 + 4.0).max(16.0);
                    let start = node.list_scroll;
                    let count = node.list_height;
                    let items = node.list_items.clone();
                    let sel = node.list_selection;
                    for i in 0..count {
                        let item_idx = start + i;
                        if item_idx >= items.len() {
                            break;
                        }
                        let row_rect = Rect::new(
                            logical.x + 2.0,
                            logical.y + 2.0 + (i as f32) * line_h,
                            logical.width - 4.0,
                            line_h,
                        );
                        let is_selected = sel == Some(item_idx);
                        if is_selected {
                            fill(&mut pixmap, row_rect.scaled(scale), clip, Color::rgb(40, 95, 190));
                        }
                        let item_fg = if is_selected { Color::WHITE } else { style.fg };
                        let origin = (
                            (row_rect.x + 4.0) * scale,
                            (row_rect.y + (row_rect.height - style.font_size * 1.3) / 2.0) * scale,
                        );
                        self.text_engine.draw(
                            &mut pixmap, &items[item_idx], (style.font_size * scale, item_fg), origin, clip.intersection(row_rect.scaled(scale)),
                        );
                    }
                }
                Kind::Scrollbar => {
                    fill(&mut pixmap, rect, clip, if dark_mode { Color::rgb(35, 35, 35) } else { Color::rgb(230, 230, 230) });
                    outline(&mut pixmap, rect, clip, scale, (border_dark, border_light));
                    let target_info = node.scroll_target.and_then(|tid| self.node(tid)).map(|t| (t.list_items.len(), t.list_height, t.list_scroll));
                    if let Some((total, visible, scroll)) = target_info {
                        if total > visible {
                            let track_h = logical.height;
                            let thumb_h = (track_h * (visible as f32 / total as f32)).clamp(16.0, track_h);
                            let avail = track_h - thumb_h;
                            let max_scroll = total - visible;
                            let thumb_y = if max_scroll > 0 {
                                avail * (scroll as f32 / max_scroll as f32)
                            } else {
                                0.0
                            };
                            let thumb_rect = Rect::new(logical.x + 1.0, logical.y + thumb_y, logical.width - 2.0, thumb_h).scaled(scale);
                            let thumb_color = if dark_mode { Color::rgb(80, 80, 80) } else { Color::rgb(170, 170, 170) };
                            fill(&mut pixmap, thumb_rect, clip, thumb_color);
                            outline(&mut pixmap, thumb_rect, clip, scale, (border_light, border_dark));
                        }
                    }
                }
                Kind::Radiobutton => {
                    let circle_cx = (logical.x + 12.0) * scale;
                    let circle_cy = (logical.y + logical.height / 2.0) * scale;
                    let circle_r = 6.0 * scale;
                    fill_circle(&mut pixmap, circle_cx, circle_cy, circle_r, clip, if dark_mode { Color::rgb(35, 35, 35) } else { Color::WHITE });
                    // Outline
                    let mut pb = PathBuilder::new();
                    let d = circle_r * 0.552_284_8;
                    pb.move_to(circle_cx, circle_cy - circle_r);
                    pb.cubic_to(circle_cx + d, circle_cy - circle_r, circle_cx + circle_r, circle_cy - d, circle_cx + circle_r, circle_cy);
                    pb.cubic_to(circle_cx + circle_r, circle_cy + d, circle_cx + d, circle_cy + circle_r, circle_cx, circle_cy + circle_r);
                    pb.cubic_to(circle_cx - d, circle_cy + circle_r, circle_cx - circle_r, circle_cy + d, circle_cx - circle_r, circle_cy);
                    pb.cubic_to(circle_cx - circle_r, circle_cy - d, circle_cx - d, circle_cy - circle_r, circle_cx, circle_cy - circle_r);
                    pb.close();
                    if let Some(path) = pb.finish() {
                        let mut paint = Paint::default();
                        paint.set_color_rgba8(border_dark.r, border_dark.g, border_dark.b, border_dark.a);
                        paint.anti_alias = true;
                        pixmap.stroke_path(&path, &paint, &Stroke::default(), Transform::identity(), None);
                    }
                    let is_selected = node.radio_var.and_then(|v| v.get(self)) == Some(node.radio_value);
                    if is_selected {
                        fill_circle(&mut pixmap, circle_cx, circle_cy, 3.0 * scale, clip, Color::rgb(40, 95, 190));
                    }
                    let text = node.text.clone();
                    let text = text.as_str();
                    let measured = self.measure(text, style.font_size);
                    let origin = (
                        (logical.x + 24.0) * scale,
                        (logical.y + (logical.height - measured.height) / 2.0) * scale,
                    );
                    self.text_engine.draw(
                        &mut pixmap, text, (style.font_size * scale, style.fg), origin, clip,
                    );
                }
                Kind::Canvas => {
                    let w = rect.width.round() as u32;
                    let h = rect.height.round() as u32;
                    if w > 0 && h > 0 {
                        if let Some(mut sub) = Pixmap::new(w, h) {
                            sub.fill(tiny_skia::Color::from_rgba8(style.bg.r, style.bg.g, style.bg.b, style.bg.a));
                            let mut items = node.canvas_items.clone();
                            items.sort_by_key(|slot| slot.seq);
                            for slot in items {
                                let Some(item) = slot.item else { continue };
                                match item {
                                    CanvasItemKind::Rect { x0, y0, x1, y1, color } => {
                                        let min_x = x0.min(x1) * scale;
                                        let max_x = x0.max(x1) * scale;
                                        let min_y = y0.min(y1) * scale;
                                        let max_y = y0.max(y1) * scale;
                                        fill(&mut sub, Rect::new(min_x, min_y, max_x - min_x, max_y - min_y), Rect::new(0.0, 0.0, w as f32, h as f32), color);
                                    }
                                    CanvasItemKind::Oval { x0, y0, x1, y1, color } => {
                                        let min_x = x0.min(x1) * scale;
                                        let max_x = x0.max(x1) * scale;
                                        let min_y = y0.min(y1) * scale;
                                        let max_y = y0.max(y1) * scale;
                                        let cx = (min_x + max_x) / 2.0;
                                        let cy = (min_y + max_y) / 2.0;
                                        let rx = (max_x - min_x) / 2.0;
                                        let ry = (max_y - min_y) / 2.0;
                                        let mut pb = PathBuilder::new();
                                        let dx = rx * 0.552_284_8;
                                        let dy = ry * 0.552_284_8;
                                        pb.move_to(cx, cy - ry);
                                        pb.cubic_to(cx + dx, cy - ry, cx + rx, cy - dy, cx + rx, cy);
                                        pb.cubic_to(cx + rx, cy + dy, cx + dx, cy + ry, cx, cy + ry);
                                        pb.cubic_to(cx - dx, cy + ry, cx - rx, cy + dy, cx - rx, cy);
                                        pb.cubic_to(cx - rx, cy - dy, cx - dx, cy - ry, cx, cy - ry);
                                        pb.close();
                                        if let Some(path) = pb.finish() {
                                            let mut paint = Paint::default();
                                            paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                                            paint.anti_alias = true;
                                            sub.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                                        }
                                    }
                                    CanvasItemKind::Line { x0, y0, x1, y1, color } => {
                                        let mut pb = PathBuilder::new();
                                        pb.move_to(x0 * scale, y0 * scale);
                                        pb.line_to(x1 * scale, y1 * scale);
                                        if let Some(path) = pb.finish() {
                                            let mut paint = Paint::default();
                                            paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                                            paint.anti_alias = true;
                                            let stroke = Stroke { width: 1.5 * scale, ..Stroke::default() };
                                            sub.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                                        }
                                    }
                                    CanvasItemKind::Text { x, y, text, color } => {
                                        self.text_engine.draw(
                                            &mut sub, &text, (style.font_size * scale, color),
                                            (x * scale, y * scale),
                                            Rect::new(0.0, 0.0, w as f32, h as f32),
                                        );
                                    }
                                }
                            }
                            let int_clip = clip.intersection(rect);
                            if let Some(clip_rect) = tiny_skia::Rect::from_xywh(int_clip.x, int_clip.y, int_clip.width, int_clip.height) {
                                let mut mask = tiny_skia::Mask::new(pixmap.width(), pixmap.height());
                                if let Some(m) = mask.as_mut() {
                                    m.fill_path(&tiny_skia::PathBuilder::from_rect(clip_rect), tiny_skia::FillRule::Winding, false, Transform::identity());
                                    pixmap.draw_pixmap(
                                        rect.x.round() as i32,
                                        rect.y.round() as i32,
                                        sub.as_ref(),
                                        &tiny_skia::PixmapPaint::default(),
                                        Transform::identity(),
                                        Some(m),
                                    );
                                }
                            }
                        }
                    }
                    outline(&mut pixmap, rect, clip, scale, (border_dark, border_light));
                }
                _ => {
                    let text = self.widget_text(id).unwrap_or_default();
                    let measured = self.measure(&text, style.font_size);
                    let shift = if pressed { 1.0 } else { 0.0 };
                    let mut text_rect = logical;
                    if kind == Kind::Checkbutton {
                        let square = Rect::new(
                            logical.x + 7.0,
                            logical.y + (logical.height - 14.0) / 2.0,
                            14.0, 14.0,
                        ).within(logical);
                        fill(&mut pixmap, square.scaled(scale), clip, if dark_mode { Color::rgb(35, 35, 35) } else { Color::WHITE });
                        outline(&mut pixmap, square.scaled(scale), clip, scale, (border_dark, border_light));
                        if check.and_then(|var| var.get(self)).unwrap_or(false) {
                            fill(
                                &mut pixmap, square.inset(3.0, 3.0).scaled(scale),
                                clip, Color::rgb(40, 95, 190),
                            );
                        }
                        text_rect = Rect::new(
                            logical.x + 28.0, logical.y,
                            (logical.width - 28.0).max(0.0), logical.height,
                        ).within(logical);
                    }
                    self.text_engine.draw(
                        &mut pixmap,
                        &text,
                        (style.font_size * scale, style.fg),
                        (
                            (text_rect.x + (text_rect.width - measured.width) / 2.0 + shift) * scale,
                            (text_rect.y + (text_rect.height - measured.height) / 2.0 + shift) * scale,
                        ),
                        clip.intersection(text_rect.inset(4.0, 2.0).scaled(scale)),
                    );
                }
            }
        }
        self.dirty = false;
        Ok(pixmap)
    }

    pub fn save_png(
        &mut self,
        path: impl AsRef<Path>,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn Error>> {
        self.render_to_pixmap(width, height)?.save_png(path)?;
        Ok(())
    }
}
