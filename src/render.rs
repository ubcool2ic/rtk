//! Software rendering, text shaping, and headless image export.

use std::error::Error;
use std::path::Path;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap};
use tiny_skia::{Color as SkColor, FilterQuality, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, PixmapPaint, PixmapRef, Point, SpreadMode, Stroke, Transform};

use crate::layout::{Rect, Size};
use crate::style::{build_rounded_rect_path, ResolvedStyle, State};
use crate::tk::{cursor_boundary, floor_char_boundary};

mod data;
use crate::widgets::{CanvasItemKind, Kind, Theme};
use crate::{Color, RtkError, Tk, Var};

mod inputs;

/// Shaping-cache key: text, size bits, family, weight, italic.
type TextKey = (String, u32, Option<String>, u16, bool);

/// Upper bound on cached shaped strings; the cache is cleared when exceeded.
const TEXT_CACHE_LIMIT: usize = 2048;
/// Upper bound on cached text measurements.
const MEASURE_CACHE_LIMIT: usize = 65_536;

/// Font system, glyph cache and shaping/measurement caches used for all text drawing.
pub(crate) struct TextEngine {
    fonts: FontSystem,
    cache: SwashCache,
    /// Shaped buffers reused across frames so unchanged text is not reshaped.
    shaped: std::collections::HashMap<TextKey, Buffer>,
    /// Measured sizes; much smaller than buffers, so it can hold many more entries.
    measured: std::collections::HashMap<TextKey, Size>,
    /// Which family became the default sans-serif (bundled Noto Sans normally).
    default_font: crate::fonts::DefaultFont,
}

impl TextEngine {
    /// Loads system fonts plus the bundled Noto Sans and picks the default sans-serif.
    pub fn new() -> Self {
        let mut fonts = FontSystem::new();
        let default_font = crate::fonts::install(fonts.db_mut(), &crate::fonts::BUNDLED_FACES);
        Self {
            default_font,
            fonts,
            cache: SwashCache::new(),
            shaped: std::collections::HashMap::new(),
            measured: std::collections::HashMap::new(),
        }
    }

    /// The family used for the default sans-serif.
    pub fn default_font(&self) -> &crate::fonts::DefaultFont {
        &self.default_font
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

    /// Reports whether any font faces are loaded in the font database.
    fn has_fonts(&self) -> bool {
        self.fonts.db().faces().next().is_some()
    }

    /// Builds a shaped text buffer for the given style, returning None for missing fonts or invalid sizes.
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
            if name.eq_ignore_ascii_case("monospace") {
                attrs = attrs.family(Family::Monospace);
            } else {
                attrs = attrs.family(Family::Name(name));
            }
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

    /// Measures styled text, caching results by text, size, family, weight, and italic.
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

    /// Measures plain text at the given size using default weight and no family.
    pub fn measure(&mut self, text: &str, size: f32) -> Size {
        self.measure_styled(text, size, None, 400, false)
    }

    /// Draws styled text into `pixmap` at `origin` (device px), clipped to `clip`.
    /// Exposed to the child `inputs` module for ring/switch/progress labels.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_styled_pub(
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
        self.draw_styled(pixmap, text, appearance, family, weight, italic, origin, clip, opacity);
    }

    /// Draws styled text into the pixmap at the origin, clipped and with the given opacity.
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
        let origin = (origin.0.round().clamp(-1.0e7, 1.0e7), origin.1.round().clamp(-1.0e7, 1.0e7));
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

/// Alpha multiplier applied to indicator colours of disabled widgets.
const DISABLED_ALPHA: f32 = 0.38;

/// Maximum height of a combobox dropdown popup, in logical px.
pub(crate) const COMBO_POPUP_MAX_H: f32 = 240.0;

/// Returns how many dropdown rows fit inside the popup's maximum height.
///
/// `item_h` is the row height in logical px (see the combobox drawing code);
/// the result is at least 1 so a single row is always addressable.
pub(crate) fn combo_popup_rows_visible(item_h: f32) -> usize {
    if !item_h.is_finite() || item_h <= 0.0 {
        return 1;
    }
    ((COMBO_POPUP_MAX_H / item_h).floor() as usize).max(1)
}

/// Returns the index of the first dropdown row to draw for `rows` total rows.
///
/// The popup scrolls just enough to keep the keyboard-highlighted row visible:
/// when the highlight is at or past the last visible row the window starts at
/// `highlight + 1 - rows_visible`. Without a highlight the list starts at 0.
pub(crate) fn combo_popup_first_row(
    highlight: Option<usize>,
    rows: usize,
    rows_visible: usize,
) -> usize {
    let rows_visible = rows_visible.max(1);
    let Some(highlight) = highlight else { return 0 };
    if highlight < rows_visible {
        return 0;
    }
    let first = highlight + 1 - rows_visible;
    first.min(rows.saturating_sub(1))
}

/// Returns `c` with its alpha channel multiplied by `f` (clamped to `[0, 1]`).
fn with_alpha(c: Color, f: f32) -> Color {
    let a = (c.a as f32 * f.clamp(0.0, 1.0)).round().clamp(0.0, 255.0) as u8;
    Color::rgba(c.r, c.g, c.b, a)
}

/// Returns `(bg, fg)` for a text selection, dimming the highlight when the
/// widget is not focused.
fn selection_colors(palette: &crate::style::Palette, focused: bool) -> (Color, Color) {
    if focused {
        (palette.selection_bg, palette.selection_fg)
    } else {
        let dim = crate::style::blend(palette.selection_bg, palette.field_bg, 0.5);
        (dim, palette.fg)
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

/// Fills the intersection of rect and clip with a solid color.
pub(super) fn fill(pixmap: &mut Pixmap, rect: Rect, clip: Rect, color: Color) {
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

/// Draws a rectangular outline using separate top and bottom colors, clamped to half the rect size.
pub(super) fn outline(pixmap: &mut Pixmap, rect: Rect, clip: Rect, width: f32, colors: (Color, Color)) {
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

/// Fills a circle approximated with cubic Béziers, clipped to the given rectangle.
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
    /// Renders the whole scene at `width`×`height` into a fresh pixmap.
    ///
    /// Always performs a full render; the output is identical to the internal
    /// scene painter with no damage restriction.
    pub fn render_to_pixmap(&mut self, width: u32, height: u32) -> Result<Pixmap, RtkError> {
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 16_777_216 {
            self.dirty = false;
            return Err(RtkError::InvalidImageSize);
        }
        self.sim_resize(width, height);
        let mut pixmap = Pixmap::new(width, height).ok_or(RtkError::InvalidImageSize)?;
        self.paint_scene(&mut pixmap, None);
        self.dirty = false;
        Ok(pixmap)
    }

    /// Paints the scene into `pixmap`.
    ///
    /// When `only` is `Some(r)` (logical px) widgets whose clip does not
    /// intersect `r` are skipped; the background fill is always full and
    /// overlays (combobox dropdown, tooltip) are only drawn in full mode.
    pub(crate) fn paint_scene(&mut self, pixmap: &mut Pixmap, only: Option<Rect>) {
        let pixmap: &mut Pixmap = pixmap;
        let width = pixmap.width();
        let height = pixmap.height();
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
            if let Some(only) = only {
                if logical_clip.intersection(only).width <= 0.0
                    || logical_clip.intersection(only).height <= 0.0
                {
                    continue;
                }
            }
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
                    pixmap,
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
                    outline(pixmap, rect.inset(2.0 * scale, 2.0 * scale), clip, ring_w, (focus_color, focus_color));
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
                        pixmap,
                        Rect::new(track.x, y - 2.0, track.width, 4.0).scaled(scale),
                        clip, track_color,
                    );
                    let x = track.x + track.width * self.scale_fraction(id);
                    let thumb_rect = Rect::new(x - 5.0, y - 9.0, 10.0, 18.0);
                    let thumb_color = resolved.fg;
                    fill(pixmap, thumb_rect.scaled(scale), clip, thumb_color);
                }
                Kind::ProgressBar => {
                    inputs::draw_progress_bar(self, pixmap, id, logical, clip, scale, &resolved);
                }
                Kind::ProgressRing => {
                    inputs::draw_ring(self, pixmap, id, logical, clip, scale, &resolved);
                }
                Kind::Switch => {
                    inputs::draw_switch(self, pixmap, id, logical, clip, scale, &resolved);
                }
                Kind::Slider => {
                    inputs::draw_slider(self, pixmap, id, logical, clip, scale, &resolved);
                }
                Kind::DoubleSpinbox => {
                    inputs::draw_dspin(self, pixmap, id, logical, clip, scale, &resolved);
                }
                Kind::Text => {
                    let inner = logical.inset(4.0, 4.0);
                    let line_h = (resolved.font_size * 1.3 + 4.0).max(16.0);
                    let start_row = node.text_scroll_row;
                    let vis_rows = node.text_dimensions.1;
                    // Own the text so `node`'s borrow ends before drawing (text_engine needs &mut self).
                    let full_text = node.text.clone();
                    let text_cursor = node.cursor;
                    let preedit = self.ime_preedit().and_then(|(pid, s, _)| if pid == id { Some(s.to_owned()) } else { None });
                    let caret = floor_char_boundary(&full_text, text_cursor);
                    let display = if let Some(p) = &preedit {
                        let mut s = String::with_capacity(full_text.len() + p.len());
                        s.push_str(full_text.get(..caret).unwrap_or_default());
                        s.push_str(p);
                        s.push_str(full_text.get(caret..).unwrap_or_default());
                        s
                    } else {
                        full_text.clone()
                    };
                    let lines: Vec<&str> = display.split('\n').collect();
                    let text_clip = clip.intersection(inner.scaled(scale));
                    // Selection offsets are in `full_text` coordinates; ignore while a preedit shifts the layout.
                    let selection = if preedit.is_some() { None } else { self.selection_of(id) };
                    let focused = self.focused == Some(id);
                    let (sel_bg, sel_fg) = selection_colors(&self.style_engine.tokens().palette, focused);

                    // Byte offset of the start of each line.
                    let mut line_starts = Vec::with_capacity(lines.len());
                    let mut acc = 0usize;
                    for line in &lines {
                        line_starts.push(acc);
                        acc += line.len() + 1;
                    }

                    // Selection highlight per visible line.
                    if let Some((sel_start, sel_end)) = selection {
                        for (i, line) in lines.iter().enumerate().skip(start_row).take(vis_rows) {
                            let line_start = line_starts[i];
                            let line_end = line_start + line.len();
                            let seg_start = sel_start.max(line_start);
                            let seg_end = sel_end.min(line_end);
                            if seg_start >= seg_end {
                                continue;
                            }
                            let x0 = self.measure_with(&line[..seg_start - line_start], &resolved).width;
                            let x1 = self.measure_with(&line[..seg_end - line_start], &resolved).width;
                            let row_y = inner.y + (i - start_row) as f32 * line_h;
                            let sel_rect = Rect::new(inner.x + x0, row_y + 2.0, (x1 - x0).max(1.0), line_h - 2.0);
                            fill(pixmap, sel_rect.scaled(scale), text_clip, sel_bg);
                        }
                    }

                    for (i, line) in lines.iter().enumerate().skip(start_row).take(vis_rows) {
                        let row_y = inner.y + (i - start_row) as f32 * line_h;
                        let origin = (inner.x * scale, (row_y + 2.0) * scale);
                        self.text_engine.draw_styled(
                            pixmap, line, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, text_clip, resolved.opacity,
                        );
                    }
                    if let Some(p) = &preedit {
                        let before = full_text.get(..caret).unwrap_or_default();
                        let line_index = before.matches('\n').count();
                        if line_index >= start_row && line_index < start_row + vis_rows {
                            let line_start = before.rfind('\n').map_or(0, |i| i + 1);
                            let col = before.get(line_start..).unwrap_or_default();
                            let col_w = self.measure_with(col, &resolved).width;
                            let preedit_w = self.measure_with(p, &resolved).width;
                            let row_y = inner.y + (line_index - start_row) as f32 * line_h;
                            fill(
                                pixmap,
                                Rect::new(inner.x + col_w, row_y + 2.0 + resolved.font_size * 1.3, preedit_w.max(1.0), 1.0).scaled(scale),
                                text_clip, draw_fg,
                            );
                        }
                    }

                    // Redraw selected glyphs in the selection foreground.
                    if let Some((sel_start, sel_end)) = selection {
                        for (i, line) in lines.iter().enumerate().skip(start_row).take(vis_rows) {
                            let line_start = line_starts[i];
                            let line_end = line_start + line.len();
                            let seg_start = sel_start.max(line_start);
                            let seg_end = sel_end.min(line_end);
                            if seg_start >= seg_end {
                                continue;
                            }
                            let x0 = self.measure_with(&line[..seg_start - line_start], &resolved).width;
                            let row_y = inner.y + (i - start_row) as f32 * line_h;
                            let seg_text = &line[seg_start - line_start..seg_end - line_start];
                            self.text_engine.draw_styled(
                                pixmap, seg_text, (resolved.font_size * scale, sel_fg),
                                resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                                ((inner.x + x0) * scale, (row_y + 2.0) * scale),
                                text_clip, resolved.opacity,
                            );
                        }
                    }

                    if self.focused == Some(id) && !disabled {
                        let cur = caret;
                        let cur_line = full_text[..cur].matches('\n').count();
                        if cur_line >= start_row && cur_line < start_row + vis_rows {
                            let line_start = full_text[..cur].rfind('\n').map_or(0, |p| p + 1);
                            let col_str = &full_text[line_start..cur];
                            let caret_x = self.measure_with(col_str, &resolved).width;
                            let row_y = inner.y + (cur_line - start_row) as f32 * line_h;
                            fill(
                                pixmap,
                                Rect::new(inner.x + caret_x, row_y + 2.0, 1.5, line_h - 2.0).scaled(scale),
                                text_clip, draw_fg,
                            );
                        }
                    }
                }
                Kind::Combobox => {
                    let inner = Rect::new(logical.x, logical.y, (logical.width - 24.0).max(0.0), logical.height);
                    let editable = node.combo_editable;
                    let cur_str = if editable {
                        node.text.clone()
                    } else {
                        self.combobox_get(id).map(|(_, s)| s).unwrap_or_default()
                    };
                    let origin = (
                        (inner.x + 6.0) * scale,
                        (inner.y + (inner.height - resolved.font_size * 1.3) / 2.0) * scale,
                    );
                    let text_clip = clip.intersection(inner.scaled(scale));
                    if editable {
                        let text = cur_str.clone();
                        let cursor = cursor_boundary(&text, node.cursor);
                        let old_scroll_x = node.scroll_x;
                        let caret = self.measure_with(text.get(..cursor).unwrap_or_default(), &resolved).width;
                        // Text is drawn at `inner.x - scroll + 6.0` and clipped to
                        // `inner`, so reserve 6 px on each side plus 1 px for the
                        // caret; otherwise the caret at the end of overflowing text
                        // would land outside the clip and disappear.
                        let available = (inner.width - 13.0).max(0.0);
                        let mut scroll = old_scroll_x.max(0.0);
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
                        let text_origin = (
                            (inner.x - scroll + 6.0) * scale,
                            origin.1,
                        );
                        let selection = self.selection_of(id);
                        if let Some((sel_start, sel_end)) = selection {
                            let focused = self.focused == Some(id);
                            let (sel_bg, _) = selection_colors(&self.style_engine.tokens().palette, focused);
                            let x0 = self.measure_with(text.get(..sel_start).unwrap_or_default(), &resolved).width;
                            let x1 = self.measure_with(text.get(..sel_end).unwrap_or_default(), &resolved).width;
                            let sel_rect = Rect::new(
                                inner.x - scroll + 6.0 + x0,
                                inner.y,
                                (x1 - x0).max(1.0),
                                inner.height,
                            );
                            fill(pixmap, sel_rect.scaled(scale), text_clip, sel_bg);
                        }
                        self.text_engine.draw_styled(
                            pixmap, &text, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            text_origin, text_clip, resolved.opacity,
                        );
                        if let Some((sel_start, sel_end)) = selection {
                            let focused = self.focused == Some(id);
                            let (_, sel_fg) = selection_colors(&self.style_engine.tokens().palette, focused);
                            let x0 = self.measure_with(text.get(..sel_start).unwrap_or_default(), &resolved).width;
                            let sel_text = text.get(sel_start..sel_end).unwrap_or_default();
                            self.text_engine.draw_styled(
                                pixmap, sel_text, (resolved.font_size * scale, sel_fg),
                                resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                                ((inner.x - scroll + 6.0 + x0) * scale, origin.1),
                                text_clip, resolved.opacity,
                            );
                        }
                        if self.focused == Some(id) && !disabled {
                            fill(
                                pixmap,
                                Rect::new(inner.x - scroll + 6.0 + caret, inner.y, 1.0, inner.height).scaled(scale),
                                text_clip, draw_fg,
                            );
                        }
                    } else {
                        self.text_engine.draw_styled(
                            pixmap, &cur_str, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, text_clip, resolved.opacity,
                        );
                    }

                    let arrow_rect = Rect::new(logical.x + logical.width - 22.0, logical.y, 22.0, logical.height).scaled(scale);
                    let arrow_text = "▼";
                    let am = self.measure(arrow_text, 10.0);
                    self.text_engine.draw_styled(
                        pixmap, arrow_text, (10.0 * scale, draw_fg),
                        None, 400, false,
                        (arrow_rect.x + (arrow_rect.width - am.width * scale) / 2.0, arrow_rect.y + (arrow_rect.height - am.height * scale) / 2.0),
                        clip, resolved.opacity,
                    );
                }
                Kind::Menubar => {
                    let titles = node.menubar_titles.clone();
                    let mut cur_x = logical.x;
                    let active = self.menus.active_bar == Some(id);
                    let open_menu = self.menus.open.first().map(|o| o.menu);
                    let sel_bg = self.style_engine.tokens().palette.select_bg;
                    for (title, menu) in titles.iter() {
                        let tw = self.measure_with(title, &resolved).width + 24.0;
                        let tab = Rect::new(cur_x, logical.y, tw, logical.height);
                        if active && open_menu == Some(*menu) {
                            fill(pixmap, tab.scaled(scale), clip, sel_bg);
                        }
                        let origin = (
                            (cur_x + 12.0) * scale,
                            (logical.y + (logical.height - resolved.font_size * 1.3) / 2.0) * scale,
                        );
                        self.text_engine.draw_styled(
                            pixmap, title, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, clip.intersection(tab.scaled(scale)), resolved.opacity,
                        );
                        cur_x += tw;
                    }
                }
                Kind::Table => {
                    data::draw_table(self, pixmap, id, logical, logical_clip, scale, &resolved);
                }
                Kind::TreeView => {
                    data::draw_tree(self, pixmap, id, logical, logical_clip, scale, &resolved);
                }
                Kind::Notebook => {
                    let strip_h = 32.0;
                    let mut cur_x = logical.x;
                    let sel = node.notebook_selected;
                    let accent = self.style_engine.tokens().palette.accent;
                    fill(pixmap, Rect::new(logical.x, logical.y + strip_h - 1.0, logical.width, 1.0).scaled(scale), clip, resolved.border_color);

                    let pages = node.notebook_pages.clone();
                    for (idx, (_, title)) in pages.iter().enumerate() {
                        let tm = self.measure_with(title, &resolved);
                        let tab_w = tm.width + 24.0;
                        let tab_rect = Rect::new(cur_x, logical.y, tab_w, strip_h);
                        let is_selected = idx == sel;

                        if is_selected {
                            fill(pixmap, tab_rect.scaled(scale), clip, crate::style::blend(resolved.bg, accent, 0.15));
                            fill(pixmap, Rect::new(cur_x, logical.y + strip_h - 2.5, tab_w, 2.5).scaled(scale), clip, accent);
                        }

                        let origin = (
                            (cur_x + 12.0) * scale,
                            (logical.y + (strip_h - tm.height) / 2.0) * scale,
                        );
                        self.text_engine.draw_styled(
                            pixmap, title, (resolved.font_size * scale, if is_selected { accent } else { draw_fg }),
                            resolved.font_family.as_deref(), if is_selected { 700 } else { resolved.font_weight }, false,
                            origin, clip.intersection(tab_rect.scaled(scale)), resolved.opacity,
                        );
                        cur_x += tab_w;
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
                    let preedit = self.ime_preedit().and_then(|(pid, s, _)| if pid == id { Some(s.to_owned()) } else { None });
                    if let Some(preedit) = preedit {
                        let before = text.get(..cursor).unwrap_or_default();
                        let after = text.get(cursor..).unwrap_or_default();
                        let before_w = self.measure_with(before, &resolved).width;
                        let preedit_w = self.measure_with(&preedit, &resolved).width;
                        self.text_engine.draw_styled(
                            pixmap, before, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, text_clip, resolved.opacity,
                        );
                        self.text_engine.draw_styled(
                            pixmap, &preedit, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            ((inner.x - scroll + before_w) * scale, origin.1),
                            text_clip, resolved.opacity,
                        );
                        self.text_engine.draw_styled(
                            pixmap, after, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            ((inner.x - scroll + before_w + preedit_w) * scale, origin.1),
                            text_clip, resolved.opacity,
                        );
                        let underline_y = (logical.y + (logical.height + resolved.font_size * 1.3) / 2.0).min(logical.y + logical.height - 1.0);
                        fill(
                            pixmap,
                            Rect::new(inner.x - scroll + before_w, underline_y, preedit_w.max(1.0), 1.0).scaled(scale),
                            text_clip, draw_fg,
                        );
                    } else {
                        // Selection highlight behind the glyphs.
                        let selection = self.selection_of(id);
                        if let Some((sel_start, sel_end)) = selection {
                            let focused = self.focused == Some(id);
                            let (sel_bg, _) = selection_colors(&self.style_engine.tokens().palette, focused);
                            let x0 = self.measure_with(text.get(..sel_start).unwrap_or_default(), &resolved).width;
                            let x1 = self.measure_with(text.get(..sel_end).unwrap_or_default(), &resolved).width;
                            let sel_rect = Rect::new(
                                inner.x - scroll + x0,
                                inner.y,
                                (x1 - x0).max(1.0),
                                inner.height,
                            );
                            fill(pixmap, sel_rect.scaled(scale), text_clip, sel_bg);
                        }
                        self.text_engine.draw_styled(
                            pixmap, &text, (resolved.font_size * scale, draw_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, text_clip, resolved.opacity,
                        );
                        // Redraw the selected glyphs in the selection foreground.
                        if let Some((sel_start, sel_end)) = selection {
                            let focused = self.focused == Some(id);
                            let (_, sel_fg) = selection_colors(&self.style_engine.tokens().palette, focused);
                            let x0 = self.measure_with(text.get(..sel_start).unwrap_or_default(), &resolved).width;
                            let sel_text = text.get(sel_start..sel_end).unwrap_or_default();
                            self.text_engine.draw_styled(
                                pixmap, sel_text, (resolved.font_size * scale, sel_fg),
                                resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                                ((inner.x - scroll + x0) * scale, origin.1),
                                text_clip, resolved.opacity,
                            );
                        }
                    }
                    if self.focused == Some(id) && !disabled {
                        fill(
                            pixmap,
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
                        pixmap, &text, (resolved.font_size * scale, draw_fg),
                        resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                        origin, text_clip, resolved.opacity,
                    );
                    // Buttons on the right
                    let btn_rect = Rect::new(logical.x + logical.width - 20.0, logical.y, 20.0, logical.height).scaled(scale);
                    fill(pixmap, btn_rect, clip, if dark_mode { Color::rgb(55, 55, 55) } else { Color::BG });
                    outline(pixmap, btn_rect, clip, scale, (resolved.border_color, resolved.border_color));
                    let half_h = btn_rect.height / 2.0;
                    fill(pixmap, Rect::new(btn_rect.x, btn_rect.y + half_h, btn_rect.width, scale), clip, resolved.border_color);
                    let up_text = "▲";
                    let dn_text = "▼";
                    let up_m = self.measure(up_text, 8.0);
                    let dn_m = self.measure(dn_text, 8.0);
                    self.text_engine.draw_styled(
                        pixmap, up_text, (8.0 * scale, draw_fg),
                        None, 400, false,
                        (btn_rect.x + (btn_rect.width - up_m.width * scale) / 2.0, btn_rect.y + (half_h - up_m.height * scale) / 2.0),
                        clip, resolved.opacity,
                    );
                    self.text_engine.draw_styled(
                        pixmap, dn_text, (8.0 * scale, draw_fg),
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
                            fill(pixmap, row_rect.scaled(scale), clip, sel_bg);
                        }
                        let item_fg = if is_selected { sel_fg } else { draw_fg };
                        let origin = (
                            (row_rect.x + 4.0) * scale,
                            (row_rect.y + (row_rect.height - resolved.font_size * 1.3) / 2.0) * scale,
                        );
                        self.text_engine.draw_styled(
                            pixmap, item_text, (resolved.font_size * scale, item_fg),
                            resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                            origin, clip.intersection(row_rect.scaled(scale)), resolved.opacity,
                        );
                    }
                }
                Kind::Scrollbar => {
                    let horizontal = node
                        .inputs
                        .as_ref()
                        .is_some_and(|s| s.scrollbar_horizontal);
                    if horizontal {
                        let target_info = node.scroll_target.and_then(|tid| self.scroll_metrics_x(tid));
                        if let Some((total, visible, scroll)) = target_info {
                            let track_w = logical.width.max(0.0);
                            let ratio = if total > 0.0 { visible / total } else { 1.0 };
                            let thumb_w = (track_w * ratio).max(16.0).min(track_w);
                            let avail = track_w - thumb_w;
                            let max_scroll = total - visible;
                            let thumb_x = if max_scroll > 0.0 { avail * (scroll / max_scroll).clamp(0.0, 1.0) } else { 0.0 };
                            let thumb_rect = Rect::new(logical.x + thumb_x, logical.y + 1.0, thumb_w, (logical.height - 2.0).max(0.0)).scaled(scale);
                            let thumb_color = resolved.fg;
                            fill(pixmap, thumb_rect, clip, thumb_color);
                        }
                    } else {
                        let target_info = node.scroll_target.and_then(|tid| self.scroll_metrics(tid));
                        if let Some((total, visible, scroll)) = target_info {
                            let track_h = logical.height.max(0.0);
                            let ratio = if total > 0.0 { visible / total } else { 1.0 };
                            let thumb_h = (track_h * ratio).max(16.0).min(track_h);
                            let avail = track_h - thumb_h;
                            let max_scroll = total - visible;
                            let thumb_y = if max_scroll > 0.0 { avail * (scroll / max_scroll).clamp(0.0, 1.0) } else { 0.0 };
                            let thumb_rect = Rect::new(logical.x + 1.0, logical.y + thumb_y, (logical.width - 2.0).max(0.0), thumb_h).scaled(scale);
                            let thumb_color = resolved.fg;
                            fill(pixmap, thumb_rect, clip, thumb_color);
                        }
                    }
                }
                Kind::Radiobutton => {
                    let circle_cx = (logical.x + 12.0) * scale;
                    let circle_cy = (logical.y + logical.height / 2.0) * scale;
                    let circle_r = 6.0 * scale;
                    let palette = self.style_engine.tokens().palette;
                    let dim = if disabled { DISABLED_ALPHA } else { 1.0 };
                    let inner_bg = with_alpha(palette.field_bg, dim);
                    fill_circle(pixmap, circle_cx, circle_cy, circle_r, clip, inner_bg);
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
                        let border_c = with_alpha(palette.muted, dim);
                        paint.set_color_rgba8(border_c.r, border_c.g, border_c.b, border_c.a);
                        paint.anti_alias = true;
                        let sw = if high_contrast { 2.0 * scale } else { scale };
                        pixmap.stroke_path(&path, &paint, &Stroke { width: sw, ..Stroke::default() }, Transform::identity(), None);
                    }
                    let is_selected = node.radio_var.and_then(|v| v.get(self)) == Some(node.radio_value);
                    if is_selected {
                        let dot_color = with_alpha(palette.accent, dim);
                        fill_circle(pixmap, circle_cx, circle_cy, 3.5 * scale, clip, dot_color);
                    }
                    let text = node.text.clone();
                    let measured = self.measure_with(&text, &resolved);
                    let origin = (
                        (logical.x + 24.0) * scale,
                        (logical.y + (logical.height - measured.height) / 2.0) * scale,
                    );
                    self.text_engine.draw_styled(
                        pixmap, &text, (resolved.font_size * scale, draw_fg),
                        resolved.font_family.as_deref(), resolved.font_weight, resolved.italic,
                        origin, clip, resolved.opacity,
                    );
                }
                Kind::Canvas | Kind::Chart | Kind::Image => {
                    let w = rect.width.round() as u32;
                    let h = rect.height.round() as u32;
                    if w > 0 && h > 0 {
                        if let Some(mut sub) = Pixmap::new(w, h) {
                            let sub_bg = if kind == Kind::Chart {
                                self.style_engine.tokens().palette.surface
                            } else if kind == Kind::Image {
                                self.style_engine.tokens().palette.bg
                            } else {
                                draw_bg
                            };
                            sub.fill(tiny_skia::Color::from_rgba8(sub_bg.r, sub_bg.g, sub_bg.b, sub_bg.a));
                            let mut draw_list: Vec<CanvasItemKind> = if kind == Kind::Chart {
                                match node.chart.as_deref() {
                                    Some(state) => crate::chart::build_items(
                                        state,
                                        logical.width,
                                        logical.height,
                                        &self.style_engine.tokens().palette,
                                        resolved.font_size,
                                    ),
                                    None => Vec::new(),
                                }
                            } else if kind == Kind::Image {
                                match node.image_data.as_deref() {
                                    Some(data) => crate::image::build_items(
                                        data,
                                        logical.width,
                                        logical.height,
                                        scale,
                                    ),
                                    None => Vec::new(),
                                }
                            } else {
                                let mut items = node.canvas_items.clone();
                                items.sort_by_key(|slot| slot.seq);
                                let (vx, vy) = node.view;
                                items.into_iter().filter_map(|slot| {
                                    let mut item = slot.item?;
                                    if vx != 0.0 || vy != 0.0 {
                                        item.translate(-vx, -vy);
                                    }
                                    Some(item)
                                }).collect()
                            };
                            for item in draw_list.drain(..) {
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
                                            let mut paint = Paint { anti_alias: true, ..Paint::default() };
                                            paint.set_color_rgba8(color.r, color.g, color.b, color.a);
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
                                            let mut paint = Paint { anti_alias: true, ..Paint::default() };
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
                                    CanvasItemKind::Image { x, y, w: iw, h: ih, src_w, src_h, rgba, smooth } => {
                                        let mut premul = Vec::with_capacity(rgba.len());
                                        for chunk in rgba.chunks_exact(4) {
                                            let a = u32::from(chunk[3]);
                                            let pr = (u32::from(chunk[0]) * a / 255) as u8;
                                            let pg = (u32::from(chunk[1]) * a / 255) as u8;
                                            let pb = (u32::from(chunk[2]) * a / 255) as u8;
                                            premul.extend_from_slice(&[pr, pg, pb, chunk[3]]);
                                        }
                                        if let Some(src) = PixmapRef::from_bytes(&premul, src_w, src_h) {
                                            let sx = (iw * scale) / src_w as f32;
                                            let sy = (ih * scale) / src_h as f32;
                                            let ts = Transform::from_row(sx, 0.0, 0.0, sy, x * scale, y * scale);
                                            let paint = PixmapPaint {
                                                quality: if smooth { FilterQuality::Bilinear } else { FilterQuality::Nearest },
                                                ..PixmapPaint::default()
                                            };
                                            sub.draw_pixmap(0, 0, src, &paint, ts, None);
                                        }
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
                    // Copy before measuring: measure_with needs &mut self.
                    let compound = node.compound;
                    let is_sidebar_btn = node.named_style.as_deref() == Some("Sidebar.TButton");
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
                        let palette = self.style_engine.tokens().palette;
                        let dim = if disabled { DISABLED_ALPHA } else { 1.0 };
                        let check_bg = with_alpha(palette.field_bg, dim);
                        let check_border = with_alpha(palette.muted, dim);
                        fill(pixmap, square.scaled(scale), clip, check_bg);
                        outline(pixmap, square.scaled(scale), clip, if high_contrast { 2.0 * scale } else { scale }, (check_border, check_border));
                        if self.checkbutton_is_mixed(id) {
                            let dash_c = with_alpha(palette.accent, dim);
                            let dash = Rect::new(
                                square.x + 3.0,
                                square.y + (square.height - 3.0) / 2.0,
                                (square.width - 6.0).max(0.0),
                                3.0,
                            );
                            fill(pixmap, dash.scaled(scale), clip, dash_c);
                        } else if check.and_then(|var| var.get(self)).unwrap_or(false) {
                            let fill_c = with_alpha(palette.accent, dim);
                            fill(
                                pixmap, square.inset(3.0, 3.0).scaled(scale),
                                clip, fill_c,
                            );
                        }
                        text_rect = Rect::new(
                            logical.x + 28.0, logical.y,
                            (logical.width - 28.0).max(0.0), logical.height,
                        ).within(logical);
                    }

                    let img_size = image_id.and_then(|img| self.images.get(&img)).map(|p| (p.width() as f32, p.height() as f32));

                    match (img_size, text.is_empty(), compound) {
                        (Some((iw, ih)), false, crate::widgets::Compound::Left) => {
                            let gap = 8.0;
                            let total_w = iw + gap + measured.width;
                            let start_x = if is_sidebar_btn {
                                text_rect.x + resolved.padding.left + shift
                            } else {
                                text_rect.x + (text_rect.width - total_w) / 2.0 + shift
                            };
                            if let Some(img_pixmap) = image_id.and_then(|i| self.images.get(&i)) {
                                let ix = start_x * scale;
                                let iy = (text_rect.y + (text_rect.height - ih) / 2.0 + shift) * scale;
                                pixmap.draw_pixmap(
                                    ix.round() as i32,
                                    iy.round() as i32,
                                    img_pixmap.as_ref(),
                                    &tiny_skia::PixmapPaint::default(),
                                    Transform::identity(),
                                    None,
                                );
                            }
                            let tx = start_x + iw + gap;
                            let ty = text_rect.y + (text_rect.height - measured.height) / 2.0 + shift;
                            self.text_engine.draw_styled(
                                pixmap,
                                &text,
                                (resolved.font_size * scale, draw_fg),
                                resolved.font_family.as_deref(),
                                resolved.font_weight,
                                resolved.italic,
                                (tx * scale, ty * scale),
                                clip.intersection(text_rect.scaled(scale)),
                                resolved.opacity,
                            );
                        }
                        (Some((iw, ih)), false, crate::widgets::Compound::Top) => {
                            let gap = 6.0;
                            let total_h = ih + gap + measured.height;
                            let start_y = text_rect.y + (text_rect.height - total_h) / 2.0 + shift;
                            if let Some(img_pixmap) = image_id.and_then(|i| self.images.get(&i)) {
                                let ix = (text_rect.x + (text_rect.width - iw) / 2.0 + shift) * scale;
                                let iy = start_y * scale;
                                pixmap.draw_pixmap(
                                    ix.round() as i32,
                                    iy.round() as i32,
                                    img_pixmap.as_ref(),
                                    &tiny_skia::PixmapPaint::default(),
                                    Transform::identity(),
                                    None,
                                );
                            }
                            let tx = text_rect.x + (text_rect.width - measured.width) / 2.0 + shift;
                            let ty = start_y + ih + gap;
                            self.text_engine.draw_styled(
                                pixmap,
                                &text,
                                (resolved.font_size * scale, draw_fg),
                                resolved.font_family.as_deref(),
                                resolved.font_weight,
                                resolved.italic,
                                (tx * scale, ty * scale),
                                clip.intersection(text_rect.scaled(scale)),
                                resolved.opacity,
                            );
                        }
                        (Some((iw, ih)), _, crate::widgets::Compound::ImageOnly) | (Some((iw, ih)), true, _) => {
                            if let Some(img_pixmap) = image_id.and_then(|i| self.images.get(&i)) {
                                let ix = (text_rect.x + (text_rect.width - iw) / 2.0 + shift) * scale;
                                let iy = (text_rect.y + (text_rect.height - ih) / 2.0 + shift) * scale;
                                pixmap.draw_pixmap(
                                    ix.round() as i32,
                                    iy.round() as i32,
                                    img_pixmap.as_ref(),
                                    &tiny_skia::PixmapPaint::default(),
                                    Transform::identity(),
                                    None,
                                );
                            }
                        }
                        _ => {
                            let origin_x = if is_sidebar_btn {
                                text_rect.x + resolved.padding.left + shift
                            } else {
                                text_rect.x + (text_rect.width - measured.width) / 2.0 + shift
                            };
                            let origin = (
                                origin_x * scale,
                                (text_rect.y + (text_rect.height - measured.height) / 2.0 + shift) * scale,
                            );
                            self.text_engine.draw_styled(
                                pixmap,
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
            }
        }

        // ===================================================================
        // Final overlay pass: open Combobox dropdown and active Tooltip
        // ===================================================================

        if only.is_some() {
            // Partial redraws never include overlays; they are refused upstream.
            return;
        }

        if let Some(combo_id) = self.open_combobox {
            let rows = self.combo_rows(combo_id);
            if let Some(cnode) = self.node(combo_id).map(|n| {
                (n.rect, n.combo_values.clone(), n.combo_var, n.combo_editable, n.combo_highlight)
            }).filter(|snap| !(snap.3 && rows.is_empty())) {
                // (An editable combobox with no matching rows draws no popup.)
                // Owned snapshot so drawing can borrow self mutably.
                struct Snap {
                    rect: Rect,
                    combo_values: Vec<String>,
                    combo_var: Option<Var<i64>>,
                    editable: bool,
                    highlight: Option<usize>,
                }
                let cnode = Snap {
                    rect: cnode.0,
                    combo_values: cnode.1,
                    combo_var: cnode.2,
                    editable: cnode.3,
                    highlight: cnode.4,
                };
                let _ = &cnode;
                let resolved = self.resolved_style(combo_id).unwrap_or_default();
                let item_h = (resolved.font_size * 1.3 + 8.0).max(20.0);
                let drop_h = (rows.len() as f32 * item_h).min(240.0).max(item_h);
                let drop_rect = Rect::new(cnode.rect.x, cnode.rect.y + cnode.rect.height, cnode.rect.width, drop_h);
                let drop_clip = Rect::new(0.0, 0.0, width as f32, height as f32);

                // High contrast uses its black field colour so the white row text stays readable.
                let bg_col = if high_contrast {
                    self.style_engine.tokens().palette.field_bg
                } else if dark_mode { Color::rgb(40, 40, 40) } else { Color::WHITE };
                fill(pixmap, drop_rect.scaled(scale), drop_clip, bg_col);
                outline(pixmap, drop_rect.scaled(scale), drop_clip, scale, (resolved.border_color, resolved.border_color));

                let sel_idx = cnode.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
                let (px, py) = self.pointer;

                // Scroll the popup so the keyboard-highlighted row is always
                // visible; the same offset is used by hit testing so clicks pick
                // the drawn row.
                let first = combo_popup_first_row(
                    cnode.highlight,
                    rows.len(),
                    combo_popup_rows_visible(item_h),
                );
                for (pos, &idx) in rows.iter().enumerate().skip(first) {
                    let Some(val) = cnode.combo_values.get(idx) else { continue };
                    // `pos` indexes `rows`; the drawn slot is relative to the
                    // first visible row so a scrolled popup stays inside its box.
                    let slot = (pos - first) as f32;
                    let row_rect = Rect::new(drop_rect.x + 1.0, drop_rect.y + 1.0 + slot * item_h, drop_rect.width - 2.0, item_h);
                    if row_rect.y >= drop_rect.y + drop_rect.height - 1.0 {
                        // Remaining rows would fall outside the popup box.
                        break;
                    }
                    let is_hovered = row_rect.contains(px, py);
                    let is_sel = if cnode.editable {
                        cnode.highlight == Some(pos)
                    } else {
                        idx as i64 == sel_idx
                    };
                    // In editable mode a keyboard highlight wins over hover so
                    // only one row looks active.
                    let hover_active = is_hovered && !(cnode.editable && cnode.highlight.is_some());

                    if hover_active || is_sel {
                        let hl = if hover_active && high_contrast {
                            self.style_engine.tokens().palette.selection_bg
                        } else if hover_active {
                            if dark_mode { Color::rgb(60, 60, 60) } else { Color::rgb(230, 230, 230) }
                        } else {
                            self.style_engine.tokens().palette.select_bg
                        };
                        fill(pixmap, row_rect.scaled(scale), drop_clip, hl);
                    }

                    let fg_col = if (is_sel && !hover_active) || (hover_active && high_contrast) {
                        self.style_engine.tokens().palette.select_fg
                    } else {
                        resolved.fg
                    };

                    let origin = (
                        (row_rect.x + 8.0) * scale,
                        (row_rect.y + (item_h - resolved.font_size * 1.3) / 2.0) * scale,
                    );
                    self.text_engine.draw_styled(
                        pixmap, val, (resolved.font_size * scale, fg_col),
                        resolved.font_family.as_deref(), resolved.font_weight, false,
                        origin, drop_clip.intersection(row_rect.scaled(scale)), 1.0,
                    );
                }
            }
        }

        data::draw_toast_overlay(self, pixmap, scale);
        data::draw_menu_overlay(self, pixmap, scale);

        if self.tooltip_visible {
            if let Some(text) = self.active_tooltip() {
                let m = self.measure(&text, 12.0);
                let tip_w = m.width + 12.0;
                let tip_h = m.height + 8.0;
                let tip_x = (self.pointer.0 + 12.0).clamp(2.0, ((width as f32 / scale) - tip_w - 2.0).max(2.0));
                let tip_y = (self.pointer.1 + 16.0).clamp(2.0, ((height as f32 / scale) - tip_h - 2.0).max(2.0));
                let tip_rect = Rect::new(tip_x, tip_y, tip_w, tip_h);
                let tip_clip = Rect::new(0.0, 0.0, width as f32, height as f32);

                fill(pixmap, tip_rect.scaled(scale), tip_clip, Color::rgb(45, 45, 45));
                outline(pixmap, tip_rect.scaled(scale), tip_clip, scale, (Color::rgb(100, 100, 100), Color::rgb(100, 100, 100)));
                self.text_engine.draw_styled(
                    pixmap, &text, (12.0 * scale, Color::rgb(240, 240, 240)),
                    None, 400, false,
                    ((tip_x + 6.0) * scale, (tip_y + 4.0) * scale),
                    tip_clip, 1.0,
                );
            }
        }

    }

    /// Renders the scene at `width`×`height` and writes it to `path` as a PNG.
    ///
    /// Returns an error when the size is invalid or the file cannot be written.
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
