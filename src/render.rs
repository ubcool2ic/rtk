use std::error::Error;
use std::path::Path;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};
use tiny_skia::{Paint, Pixmap, Transform};

use crate::layout::{Rect, Size};
use crate::tk::cursor_boundary;
use crate::widgets::Kind;
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
        // Buffer construction/shaping must not run against an empty fontdb.
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
                // tiny-skia performs source-over premultiplied blending.
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
    // tiny-skia accepts zero-area rects and may still touch a pixel; skip them.
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

impl Tk {
    /// Renders a physical-pixel image using the current logical scale factor.
    ///
    /// Zero dimensions are rejected without allocating a pixmap. Allocation is
    /// bounded to 16 million pixels (64 MiB of RGBA storage).
    pub fn render_to_pixmap(&mut self, width: u32, height: u32) -> Result<Pixmap, RtkError> {
        self.sim_resize(width, height);
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 16_777_216 {
            self.dirty = false;
            return Err(RtkError::InvalidImageSize);
        }
        let mut pixmap = Pixmap::new(width, height).ok_or(RtkError::InvalidImageSize)?;
        let bg = self.node(self.root()).map(|node| node.style.bg).unwrap_or(Color::BG);
        pixmap.fill(tiny_skia::Color::from_rgba8(bg.r, bg.g, bg.b, bg.a));
        let scale = self.scale_factor;
        // Tree order, not slot order: a child can reuse a slot before its parent.
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
                Color::ACTIVE
            } else {
                style.bg
            };
            fill(&mut pixmap, rect, clip, bg);
            let light = Color::rgb(250, 250, 250);
            let dark = Color::rgb(110, 110, 110);
            if matches!(kind, Kind::Button | Kind::Entry) || (kind == Kind::Frame && id != self.root()) {
                let raised = kind == Kind::Button && !pressed;
                outline(
                    &mut pixmap, rect, clip, scale,
                    if raised { (light, dark) } else { (dark, light) },
                );
            }
            if self.focused == Some(id) && kind.focusable() {
                let color = Color::rgb(40, 95, 190);
                outline(&mut pixmap, rect.inset(2.0 * scale, 2.0 * scale), clip, scale, (color, color));
            }
            if kind == Kind::Frame {
                continue;
            }
            if kind == Kind::Scale {
                let track = logical.inset(12.0, 0.0);
                let y = logical.y + logical.height / 2.0;
                fill(
                    &mut pixmap,
                    Rect::new(track.x, y - 2.0, track.width, 4.0).scaled(scale),
                    clip, dark,
                );
                let x = track.x + track.width * self.scale_fraction(id);
                fill(
                    &mut pixmap,
                    Rect::new(x - 5.0, y - 9.0, 10.0, 18.0).scaled(scale),
                    clip, Color::rgb(65, 105, 175),
                );
                continue;
            }
            let text = self.widget_text(id).unwrap_or_default();
            if kind == Kind::Entry {
                let inner = logical.inset(6.0, 5.0);
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
            } else {
                let measured = self.measure(&text, style.font_size);
                let shift = if pressed { 1.0 } else { 0.0 };
                let mut text_rect = logical;
                if kind == Kind::Checkbutton {
                    let square = Rect::new(
                        logical.x + 7.0,
                        logical.y + (logical.height - 14.0) / 2.0,
                        14.0, 14.0,
                    ).within(logical);
                    fill(&mut pixmap, square.scaled(scale), clip, Color::WHITE);
                    outline(&mut pixmap, square.scaled(scale), clip, scale, (dark, light));
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
