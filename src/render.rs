use std::error::Error;
use std::path::Path;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap};
use tiny_skia::{Color as SkColor, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, SpreadMode, Stroke, Transform};

use crate::layout::{Rect, Size};
use crate::style::{build_rounded_rect_path, ResolvedStyle, State};
use crate::tk::cursor_boundary;
use crate::widgets::{CanvasItemKind, Kind, Theme};
use crate::{Color, RtkError, Tk};

/// Shaping-cache key: text, size bits, family, weight, italic.
type TextKey = (String, u32, Option<String>, u16, bool);

/// Upper bound on cached shaped strings; the cache is cleared when exceeded.
const TEXT_CACHE_LIMIT: usize = 2048;
/// Upper bound on cached text measurements.
const MEASURE_CACHE_LIMIT: usize = 65_536;

pub(crate) struct TextEngine {
    fonts: FontSystem,
    cache: SwashCache,
    /// Shaped buffers reused across frames so unchanged text is not reshaped.
    shaped: std::collections::HashMap<TextKey, Buffer>,
    /// Measured sizes; much smaller than buffers, so it can hold many more entries.
    measured: std::collections::HashMap<TextKey, Size>,
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            fonts: FontSystem::new(),
            cache: SwashCache::new(),
            shaped: std::collections::HashMap::new(),
            measured: std::collections::HashMap::new(),
        }
    }

    /// Returns a cached shaped buffer, shaping and caching it on a miss.
    fn shaped(
        &mut self,
        text: &str,
        size: f32,
        family_name: Option<&str>,
        weight: u16,
        italic: bool,
    ) -> Option<&Buffer> {
        let key: TextKey = (text.to_owned(), size.to_bits(), family_name.map(str::to_owned), weight, italic);
        if !self.shaped.contains_key(&key) {
            let buffer = self.buffer(text, size, family_name, weight, italic)?;
            if self.shaped.len() >= TEXT_CACHE_LIMIT {
                self.shaped.clear();
            }
            self.shaped.insert(key.clone(), buffer);
        }
        self.shaped.get(&key)
    }

    fn has_fonts(&self) -> bool {
        self.fonts.db().faces().next().is_some()
    }

    fn buffer(
        &mut self,
        text: &str,
        size: f32,
        family_name: Option<&str>,
        weight: u16,
        italic: bool,
    ) -> Option<Buffer> {
        if !self.has_fonts() || !size.is_finite() || size <= 0.0 {
            return None;
        }
        let mut buffer = Buffer::new(&mut self.fonts, Metrics::new(size, size * 1.3));
        buffer.set_size(&mut self.fonts, None, None);
        buffer.set_wrap(&mut self.fonts, Wrap::None);

        let mut attrs = Attrs::new();
        if let Some(name) = family_name {
            attrs = attrs.family(Family::Name(name));
        } else {
            attrs = attrs.family(Family::SansSerif);
        }
        attrs = attrs.weight(Weight(weight));
        if italic {
            attrs = attrs.style(Style::Italic);
        }

        buffer.set_text(
            &mut self.fonts,
            text,
            attrs,
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.fonts, false);
        Some(buffer)
    }

    pub fn measure_styled(
        &mut self,
        text: &str,
        size: f32,
        family: Option<&str>,
        weight: u16,
        italic: bool,
    ) -> Size {
        let key: TextKey = (text.to_owned(), size.to_bits(), family.map(str::to_owned), weight, italic);
        if let Some(size) = self.measured.get(&key) {
            return *size;
        }
        let width = if let Some(buffer) = self.buffer(text, size, family, weight, italic) {
            buffer.layout_runs().map(|run| run.line_w).fold(0.0_f32, f32::max)
        } else {
            text.split('\n').map(|line| line.chars().count()).max().unwrap_or(0) as f32 * size * 0.6
        };
        let result = Size::new(width, text.split('\n').count() as f32 * size * 1.3);
        if self.measured.len() >= MEASURE_CACHE_LIMIT {
            self.measured.clear();
        }
        self.measured.insert(key, result);
        result
    }

    pub fn measure(&mut self, text: &str, size: f32) -> Size {
        self.measure_styled(text, size, None, 400, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_styled(
        &mut self,
        pixmap: &mut Pixmap,
        text: &str,
        appearance: (f32, Color),
        family: Option<&str>,
        weight: u16,
        italic: bool,
        origin: (f32, f32),
        clip: Rect,
        opacity: f32,
    ) {
        let (size, color) = appearance;
        if clip.width <= 0.0 || clip.height <= 0.0 {
            return;
        }
        let origin = (origin.0.round(), origin.1.round());
        let effective_alpha = (color.a as f32 * opacity).round().clamp(0.0, 255.0) as u8;
        let draw_color = cosmic_text::Color::rgba(color.r, color.g, color.b, effective_alpha);
        let key: TextKey = (text.to_owned(), size.to_bits(), family.map(str::to_owned), weight, italic);
        if self.shaped(text, size, family, weight, italic).is_none() {
            return;
        }
        let Some(buffer) = self.shaped.get(&key) else { return };
        // Integer clip bounds in device pixels.
        let (width, height) = (pixmap.width() as i32, pixmap.height() as i32);
        let x0 = (clip.x.floor() as i32).clamp(0, width);
        let y0 = (clip.y.floor() as i32).clamp(0, height);
        let x1 = ((clip.x + clip.width).ceil() as i32).clamp(0, width);
        let y1 = ((clip.y + clip.height).ceil() as i32).clamp(0, height);
        let data = pixmap.data_mut();
        buffer.draw(&mut self.fonts, &mut self.cache, draw_color, |x, y, w, h, c| {
            let left = origin.0 as i32 + x;
            let top = origin.1 as i32 + y;
            for py in top.max(y0)..(top + h as i32).min(y1) {
                for px in left.max(x0)..(left + w as i32).min(x1) {
                    blend_pixel(data, (py * width + px) as usize, c);
                }
            }
        });
    }
}

/// Source-over blend of a straight-alpha colour onto a premultiplied RGBA8 pixel.
fn blend_pixel(data: &mut [u8], index: usize, color: cosmic_text::Color) {
    let alpha = u32::from(color.a());
    if alpha == 0 {
        return;
    }
    let Some(pixel) = data.get_mut(index * 4..index * 4 + 4) else { return };
    let inverse = 255 - alpha;
    let source = [color.r(), color.g(), color.b()];
    for (channel, value) in pixel.iter_mut().take(3).zip(source) {
        let premultiplied = u32::from(value) * alpha / 255;
        *channel = (premultiplied + u32::from(*channel) * inverse / 255).min(255) as u8;
    }
    pixel[3] = (alpha + u32::from(pixel[3]) * inverse / 255).min(255) as u8;
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

/// Renders a styled widget background: drop shadow, rounded rect, linear gradient, border.
fn render_styled_box(
    pixmap: &mut Pixmap,
    rect: Rect,
    clip: Rect,
    scale: f32,
    style: &ResolvedStyle,
    bg_override: Option<Color>,
    border_override: Option<Color>,
) {
    let rect_scaled = rect.scaled(scale);
    let clip_scaled = clip.scaled(scale);
    let int_clip = rect_scaled.intersection(clip_scaled);
    if int_clip.width <= 0.0 || int_clip.height <= 0.0 {
        return;
    }

    let radius_scaled = style.radius * scale;
    let opacity = style.opacity.clamp(0.0, 1.0);

    // 1. Drop shadow (draw offset expanding low-alpha rounded rects)
    if let Some(shadow) = style.shadow {
        if shadow.color.a > 0 {
            let blur_steps = 4;
            let max_blur = (shadow.blur * scale).max(1.0);
            for step in 1..=blur_steps {
                let factor = step as f32 / blur_steps as f32;
                let spread = factor * max_blur * 0.5;
                let alpha = ((shadow.color.a as f32 / blur_steps as f32) * 0.8 * opacity).round() as u8;
                if alpha == 0 {
                    continue;
                }
                let shadow_rect = Rect::new(
                    rect_scaled.x + shadow.dx * scale - spread,
                    rect_scaled.y + shadow.dy * scale - spread,
                    rect_scaled.width + 2.0 * spread,
                    rect_scaled.height + 2.0 * spread,
                );
                if let Some(shadow_path) = build_rounded_rect_path(shadow_rect, radius_scaled + spread) {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(shadow.color.r, shadow.color.g, shadow.color.b, alpha);
                    paint.anti_alias = true;
                    pixmap.fill_path(&shadow_path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
        }
    }

    // 2. Widget background (solid color or linear gradient)
    if let Some(path) = build_rounded_rect_path(rect_scaled, radius_scaled) {
        let mut paint = Paint { anti_alias: true, ..Paint::default() };

        if let Some(grad) = &style.gradient {
            let angle_rad = grad.angle_deg.to_radians();
            let cx = rect_scaled.x + rect_scaled.width / 2.0;
            let cy = rect_scaled.y + rect_scaled.height / 2.0;
            let half_diag = ((rect_scaled.width).hypot(rect_scaled.height)) / 2.0;
            let dx = angle_rad.sin() * half_diag;
            let dy = -angle_rad.cos() * half_diag;
            let start = Point::from_xy(cx - dx, cy - dy);
            let end = Point::from_xy(cx + dx, cy + dy);
            let stops: Vec<GradientStop> = grad.stops.iter().map(|(pos, col)| {
                let a = (col.a as f32 * opacity).round().clamp(0.0, 255.0) as u8;
                GradientStop::new(*pos, SkColor::from_rgba8(col.r, col.g, col.b, a))
            }).collect();

            if let Some(shader) = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity()) {
                paint.shader = shader;
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        } else {
            let bg = bg_override.unwrap_or(style.bg);
            let effective_alpha = (bg.a as f32 * opacity).round().clamp(0.0, 255.0) as u8;
            if effective_alpha > 0 {
                paint.set_color_rgba8(bg.r, bg.g, bg.b, effective_alpha);
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }

        // 3. Border
        let border_w = style.border_width * scale;
        if border_w > 0.0 {
            let border_c = border_override.unwrap_or(style.border_color);
            let effective_border_a = (border_c.a as f32 * opacity).round().clamp(0.0, 255.0) as u8;
            if effective_border_a > 0 {
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color_rgba8(border_c.r, border_c.g, border_c.b, effective_border_a);
                stroke_paint.anti_alias = true;
                let stroke = Stroke { width: border_w, ..Stroke::default() };
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
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
        let root_style = self.resolved_style(self.root());
        let default_bg = root_style.as_ref().map(|s| s.bg).unwrap_or(match self.theme {
            Theme::Dark => Color::rgb(32, 32, 32),
            Theme::HighContrast => Color::rgb(0, 0, 0),
            _ => Color::BG,
        });
        pixmap.fill(tiny_skia::Color::from_rgba8(default_bg.r, default_bg.g, default_bg.b, default_bg.a));
        let scale = self.scale_factor;

        let high_contrast = self.theme == Theme::HighContrast;
        let dark_mode = self.theme == Theme::Dark;

        for id in self.paint_order.clone() {
            let Some(node) = self.node(id) else { continue };
            let (kind, logical, logical_clip) = (node.kind, node.rect, node.clip);
            let (cursor, old_scroll, check) = (node.cursor, node.scroll_x, node.check);
            let disabled = node.disabled;
            let image_id = node.image;

            let resolved = self.resolved_style(id).unwrap_or_default();
            let rect = logical.scaled(scale);
            let clip = logical_clip.scaled(scale);
            if clip.width <= 0.0 || clip.height <= 0.0 {
                continue;
            }

            // Interpolated colors if animating transition
            let (draw_bg, draw_fg, draw_border) = if let Some(trans) = &node.transition {
                (trans.current_bg(), trans.current_fg(), trans.current_border())
            } else {
                (resolved.bg, resolved.fg, resolved.border_color)
            };

            // Render background, rounded corners, gradient, drop shadow, and border
            if kind != Kind::Radiobutton && kind != Kind::Checkbutton {
                render_styled_box(
                    &mut pixmap,
                    logical,
                    logical_clip,
                    scale,
                    &resolved,
                    Some(draw_bg),
                    Some(draw_border),
                );
            }

            // High contrast or focused focus ring
            if self.focused == Some(id) && kind.focusable() && !disabled {
                let focus_color = self.style_engine.tokens().palette.focus;
                let ring_w = if high_contrast { 3.0 * scale } else { 2.0 * scale };
                if resolved.radius > 0.0 && !high_contrast {
                    if let Some(path) = build_rounded_rect_path(rect.inset(2.0 * scale, 2.0 * scale), (resolved.radius - 2.0).max(0.0) * scale) {
                        let mut p = Paint::default();
                        p.set_color_rgba8(focus_color.r, focus_color.g, focus_color.b, focus_color.a);
                        p.anti_alias = true;
                        let stroke = Stroke { width: ring_w, ..Stroke::default() };
                        pixmap.stroke_path(&path, &p, &stroke, Transform::identity(), None);
                    }
                } else {
                    outline(&mut pixmap, rect.inset(2.0 * scale, 2.0 * scale), clip, ring_w, (focus_color, focus_color));
                }
            }

            if kind == Kind::Frame {
                continue;
            }

            match kind {
                Kind::Scale => {
                    let track = logical.inset(12.0, 0.0);
                    let y = logical.y + logical.height / 2.0;
                    let track_color = resolved.border_color;
                    fill(
                        &mut pixmap,
                        Rect::new(track.x, y - 2.0, track.width, 4.0).scaled(scale),
                        clip, track_color,
                    );
                    let x = track.x + track.width * self.scale_fraction(id);
                    let thumb_rect = Rect::new(x - 5.0, y - 9.0, 10.0, 18.0);
                    let thumb_color = resolved.fg;
                    fill(&mut pixmap, thumb_rect.scaled(scale), clip, thumb_color);
                }
                Kind::ProgressBar => {
                    let val = node.scale.and_then(|v| v.get(self)).unwrap_or(0).clamp(0, 100);
                    let frac = (val as f32) / 100.0;
                    let inner = logical.inset(2.0, 2.0);
                    let filled_w = inner.width * frac;
                    if filled_w > 0.0 {
                        let fill_rect = Rect::new(inner.x, inner.y, filled_w, inner.height).scaled(scale);
                        fill(&mut pixmap, fill_rect, clip, resolved.fg);
                    }
                }
                Kind::Entry => {
                    let inner = logical.inset(6.0, 5.0);
                    let text = self.widget_text(id).unwrap_or_default();
                    let cursor = cursor_boundary(&text, cursor);
                    let caret = self.measure_with(text.get(..cursor).unwrap_or_default(), &resolved).width;
                    let available = (inner.width - 1.0).max(0.0);
                    let mut scroll = old_scroll.max(0.0);
                    if caret < scroll {
                        scroll = caret;
                    } else if caret > scroll + available {
                        scroll = caret - available;
                    }
                    let total = self.measure_with(&text, &resolved).width;
                    scroll = scroll.min((total - available).max(0.0));
                    if let Some(node) = self.node_mut(id) {
                        node.scroll_x = scroll;
                        node.cursor = cursor;
                    }
                    let origin = (
                        (inner.x - scroll) * scale,
                        (logical.y + (logical.height - resolved.font_size * 1.3) / 2.0) * scale,
                    );
                    let text_clip = clip.intersection(inner.scaled(scale));
                    self.text_engine.draw_styled(
                        &mut pixmap, &text, (resolved.font_size * scale, draw_fg),
                        resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                        origin, text_clip, resolved.opacity,
                    );
                    if self.focused == Some(id) && !disabled {
                        fill(
                            &mut pixmap,
                            Rect::new(inner.x - scroll + caret, inner.y, 1.0, inner.height).scaled(scale),
                            text_clip, draw_fg,
                        );
                    }
                }
                Kind::Spinbox => {
                    let inner = Rect::new(logical.x, logical.y, (logical.width - 20.0).max(0.0), logical.height);
                    let text = format!("{}", node.scale.and_then(|v| v.get(self)).unwrap_or(node.range.0));
                    let origin = (
                        (inner.x + 6.0) * scale,
                        (inner.y + (inner.height - resolved.font_size * 1.3) / 2.0) * scale,
                    );
                    let text_clip = clip.intersection(inner.scaled(scale));
                    self.text_engine.draw_styled(
                        &mut pixmap, &text, (resolved.font_size * scale, draw_fg),
                        resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                        origin, text_clip, resolved.opacity,
                    );
                    // Buttons on the right
                    let btn_rect = Rect::new(logical.x + logical.width - 20.0, logical.y, 20.0, logical.height).scaled(scale);
                    fill(&mut pixmap, btn_rect, clip, if dark_mode { Color::rgb(55, 55, 55) } else { Color::BG });
                    outline(&mut pixmap, btn_rect, clip, scale, (resolved.border_color, resolved.border_color));
                    let half_h = btn_rect.height / 2.0;
                    fill(&mut pixmap, Rect::new(btn_rect.x, btn_rect.y + half_h, btn_rect.width, scale), clip, resolved.border_color);
                    let up_text = "▲";
                    let dn_text = "▼";
                    let up_m = self.measure(up_text, 8.0);
                    let dn_m = self.measure(dn_text, 8.0);
                    self.text_engine.draw_styled(
                        &mut pixmap, up_text, (8.0 * scale, draw_fg),
                        None, 400, false,
                        (btn_rect.x + (btn_rect.width - up_m.width * scale) / 2.0, btn_rect.y + (half_h - up_m.height * scale) / 2.0),
                        clip, resolved.opacity,
                    );
                    self.text_engine.draw_styled(
                        &mut pixmap, dn_text, (8.0 * scale, draw_fg),
                        None, 400, false,
                        (btn_rect.x + (btn_rect.width - dn_m.width * scale) / 2.0, btn_rect.y + half_h + (half_h - dn_m.height * scale) / 2.0),
                        clip, resolved.opacity,
                    );
                }
                Kind::Listbox => {
                    let line_h = (resolved.font_size * 1.3 + 4.0).max(16.0);
                    let start = node.list_scroll;
                    let count = node.list_height;
                    // Clone only the visible rows, not the whole list.
                    let items: Vec<String> = node.list_items.iter().skip(start).take(count).cloned().collect();
                    let sel = node.list_selection;
                    let sel_bg = self.style_engine.tokens().palette.select_bg;
                    let sel_fg = self.style_engine.tokens().palette.select_fg;
                    for i in 0..count {
                        let item_idx = start + i;
                        let Some(item_text) = items.get(i) else { break };
                        let row_rect = Rect::new(
                            logical.x + 2.0,
                            logical.y + 2.0 + (i as f32) * line_h,
                            logical.width - 4.0,
                            line_h,
                        );
                        let is_selected = sel == Some(item_idx);
                        if is_selected {
                            fill(&mut pixmap, row_rect.scaled(scale), clip, sel_bg);
                        }
                        let item_fg = if is_selected { sel_fg } else { draw_fg };
                        let origin = (
                            (row_rect.x + 4.0) * scale,
                            (row_rect.y + (row_rect.height - resolved.font_size * 1.3) / 2.0) * scale,
                        );
                        self.text_engine.draw_styled(
                            &mut pixmap, item_text, (resolved.font_size * scale, item_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, clip.intersection(row_rect.scaled(scale)), resolved.opacity,
                        );
                    }
                }
                Kind::Scrollbar => {
                    let target_info = node.scroll_target.and_then(|tid| self.scroll_metrics(tid));
                    if let Some((total, visible, scroll)) = target_info {
                        {
                            let track_h = logical.height;
                            let thumb_h = (track_h * (visible / total)).clamp(16.0, track_h);
                            let avail = track_h - thumb_h;
                            let max_scroll = total - visible;
                            let thumb_y = if max_scroll > 0.0 { avail * (scroll / max_scroll) } else { 0.0 };
                            let thumb_rect = Rect::new(logical.x + 1.0, logical.y + thumb_y, logical.width - 2.0, thumb_h).scaled(scale);
                            let thumb_color = resolved.fg;
                            fill(&mut pixmap, thumb_rect, clip, thumb_color);
                        }
                    }
                }
                Kind::Radiobutton => {
                    let circle_cx = (logical.x + 12.0) * scale;
                    let circle_cy = (logical.y + logical.height / 2.0) * scale;
                    let circle_r = 6.0 * scale;
                    let inner_bg = if high_contrast { Color::BLACK } else if dark_mode { Color::rgb(45, 45, 45) } else { Color::WHITE };
                    fill_circle(&mut pixmap, circle_cx, circle_cy, circle_r, clip, inner_bg);
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
                        let border_c = if high_contrast { Color::WHITE } else if dark_mode { Color::rgb(180, 180, 180) } else { draw_border };
                        paint.set_color_rgba8(border_c.r, border_c.g, border_c.b, border_c.a);
                        paint.anti_alias = true;
                        let sw = if high_contrast { 2.0 * scale } else { scale };
                        pixmap.stroke_path(&path, &paint, &Stroke { width: sw, ..Stroke::default() }, Transform::identity(), None);
                    }
                    let is_selected = node.radio_var.and_then(|v| v.get(self)) == Some(node.radio_value);
                    if is_selected {
                        let dot_color = self.style_engine.tokens().palette.accent;
                        fill_circle(&mut pixmap, circle_cx, circle_cy, 3.5 * scale, clip, dot_color);
                    }
                    let text = node.text.clone();
                    let measured = self.measure_with(&text, &resolved);
                    let origin = (
                        (logical.x + 24.0) * scale,
                        (logical.y + (logical.height - measured.height) / 2.0) * scale,
                    );
                    self.text_engine.draw_styled(
                        &mut pixmap, &text, (resolved.font_size * scale, draw_fg),
                        resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                        origin, clip, resolved.opacity,
                    );
                }
                Kind::Canvas => {
                    let w = rect.width.round() as u32;
                    let h = rect.height.round() as u32;
                    if w > 0 && h > 0 {
                        if let Some(mut sub) = Pixmap::new(w, h) {
                            sub.fill(tiny_skia::Color::from_rgba8(draw_bg.r, draw_bg.g, draw_bg.b, draw_bg.a));
                            let mut items = node.canvas_items.clone();
                            items.sort_by_key(|slot| slot.seq);
                            let (vx, vy) = node.view;
                            for slot in items {
                                let Some(mut item) = slot.item else { continue };
                                if vx != 0.0 || vy != 0.0 {
                                    item.translate(-vx, -vy);
                                }
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
                                    CanvasItemKind::RoundRect { x0, y0, x1, y1, radius, fill: f, outline } => {
                                        let r = Rect::new(x0.min(x1) * scale, y0.min(y1) * scale, (x1 - x0).abs() * scale, (y1 - y0).abs() * scale);
                                        if let Some(path) = build_rounded_rect_path(r, radius * scale) {
                                            let mut paint = Paint::default();
                                            paint.anti_alias = true;
                                            if let Some(c) = f {
                                                paint.set_color_rgba8(c.r, c.g, c.b, c.a);
                                                sub.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                                            }
                                            if let Some((c, width)) = outline {
                                                paint.set_color_rgba8(c.r, c.g, c.b, c.a);
                                                let stroke = Stroke { width: width * scale, ..Stroke::default() };
                                                sub.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                                            }
                                        }
                                    }
                                    CanvasItemKind::Path { points, color, width, filled } => {
                                        let mut pb = PathBuilder::new();
                                        for (i, (x, y)) in points.iter().enumerate() {
                                            if i == 0 { pb.move_to(x * scale, y * scale) } else { pb.line_to(x * scale, y * scale) }
                                        }
                                        if filled { pb.close(); }
                                        if let Some(path) = pb.finish() {
                                            let mut paint = Paint::default();
                                            paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                                            paint.anti_alias = true;
                                            if filled {
                                                sub.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                                            } else {
                                                let stroke = Stroke { width: width * scale, line_cap: tiny_skia::LineCap::Round, line_join: tiny_skia::LineJoin::Round, ..Stroke::default() };
                                                sub.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                                            }
                                        }
                                    }
                                    CanvasItemKind::Ring { x0, y0, x1, y1, color, width } => {
                                        if let Some(r) = tiny_skia::Rect::from_ltrb(x0.min(x1) * scale, y0.min(y1) * scale, x0.max(x1) * scale, y0.max(y1) * scale) {
                                            if let Some(path) = PathBuilder::from_oval(r) {
                                                let mut paint = Paint::default();
                                                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                                                paint.anti_alias = true;
                                                let stroke = Stroke { width: width * scale, ..Stroke::default() };
                                                sub.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                                            }
                                        }
                                    }
                                    CanvasItemKind::TextEx { x, y, text, color, size, weight } => {
                                        self.text_engine.draw_styled(
                                            &mut sub, &text, (size * scale, color),
                                            resolved.font_family.as_deref(), weight, false,
                                            (x * scale, y * scale),
                                            Rect::new(0.0, 0.0, w as f32, h as f32),
                                            resolved.opacity,
                                        );
                                    }
                                    CanvasItemKind::Text { x, y, text, color } => {
                                        self.text_engine.draw_styled(
                                            &mut sub, &text, (resolved.font_size * scale, color),
                                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                                            (x * scale, y * scale),
                                            Rect::new(0.0, 0.0, w as f32, h as f32),
                                            resolved.opacity,
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
                }
                _ => {
                    let text = self.widget_text(id).unwrap_or_default();
                    let measured = self.measure_with(&text, &resolved);
                    let state = self.widget_state(id);
                    let pressed = state.contains(State::PRESSED);
                    let shift = if pressed { 1.0 } else { 0.0 };
                    let mut text_rect = logical;

                    if kind == Kind::Checkbutton {
                        let square = Rect::new(
                            logical.x + 7.0,
                            logical.y + (logical.height - 14.0) / 2.0,
                            14.0, 14.0,
                        ).within(logical);
                        let check_bg = if high_contrast { Color::BLACK } else if dark_mode { Color::rgb(45, 45, 45) } else { Color::WHITE };
                        let check_border = if high_contrast { Color::WHITE } else if dark_mode { Color::rgb(180, 180, 180) } else { draw_border };
                        fill(&mut pixmap, square.scaled(scale), clip, check_bg);
                        outline(&mut pixmap, square.scaled(scale), clip, if high_contrast { 2.0 * scale } else { scale }, (check_border, check_border));
                        if check.and_then(|var| var.get(self)).unwrap_or(false) {
                            let fill_c = self.style_engine.tokens().palette.accent;
                            fill(
                                &mut pixmap, square.inset(3.0, 3.0).scaled(scale),
                                clip, fill_c,
                            );
                        }
                        text_rect = Rect::new(
                            logical.x + 28.0, logical.y,
                            (logical.width - 28.0).max(0.0), logical.height,
                        ).within(logical);
                    }

                    // Optional Image left of text
                    let mut image_offset = 0.0;
                    if let Some(img_id) = image_id {
                        if let Some(img_pixmap) = self.images.get(&img_id) {
                            let img_w = img_pixmap.width() as f32;
                            let img_h = img_pixmap.height() as f32;
                            let img_x = (text_rect.x + resolved.padding.left + shift) * scale;
                            let img_y = (text_rect.y + (text_rect.height - img_h) / 2.0 + shift) * scale;
                            pixmap.draw_pixmap(
                                img_x.round() as i32,
                                img_y.round() as i32,
                                img_pixmap.as_ref(),
                                &tiny_skia::PixmapPaint::default(),
                                Transform::identity(),
                                None,
                            );
                            image_offset = img_w + 6.0;
                        }
                    }

                    let origin = (
                        (text_rect.x + (text_rect.width - measured.width) / 2.0 + image_offset / 2.0 + shift) * scale,
                        (text_rect.y + (text_rect.height - measured.height) / 2.0 + shift) * scale,
                    );
                    self.text_engine.draw_styled(
                        &mut pixmap,
                        &text,
                        (resolved.font_size * scale, draw_fg),
                        resolved.font_family.as_deref(),
                        resolved.font_weight,
                        resolved.italic,
                        origin,
                        clip.intersection(text_rect.inset(4.0, 2.0).scaled(scale)),
                        resolved.opacity,
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
