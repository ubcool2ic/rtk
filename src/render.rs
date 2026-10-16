use std::error::Error;
use std::path::Path;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};
use tiny_skia::{Paint, Pixmap, Transform};

use crate::layout::{Rect, Size};
use crate::tk::cursor_boundary;
use crate::widgets::Kind;
use crate::{Color, Tk, WidgetId};

pub(crate) struct TextEngine {
    fonts: FontSystem,
    cache: SwashCache,
}

impl TextEngine {
    pub fn new() -> Self {
        Self {
            // cosmic-text/fontdb discovers the host's system fonts here.
            fonts: FontSystem::new(),
            cache: SwashCache::new(),
        }
    }

    fn buffer(&mut self, text: &str, size: f32) -> Buffer {
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
        buffer
    }

    pub fn measure(&mut self, text: &str, size: f32) -> Size {
        let buffer = self.buffer(text, size);
        let width = buffer.layout_runs().map(|run| run.line_w).fold(0.0_f32, f32::max);
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
        let buffer = self.buffer(text, size);
        let origin = (origin.0.round(), origin.1.round());
        buffer.draw(
            &mut self.fonts,
            &mut self.cache,
            cosmic_text::Color::rgba(color.r, color.g, color.b, color.a),
            |x, y, width, height, color| {
                blend_rect(
                    pixmap,
                    Rect::new(
                        origin.0 + x as f32,
                        origin.1 + y as f32,
                        width as f32,
                        height as f32,
                    ).intersection(clip),
                    Color::rgba(color.r(), color.g(), color.b(), color.a()),
                );
            },
        );
    }
}

// cosmic-text supplies unassociated RGBA; tiny-skia stores premultiplied RGBA.
fn blend_rect(pixmap: &mut Pixmap, rect: Rect, color: Color) {
    if color.a == 0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let rect = rect.intersection(Rect::new(
        0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32,
    ));
    let (x0, y0) = (rect.x.ceil() as usize, rect.y.ceil() as usize);
    let (x1, y1) = (
        (rect.x + rect.width).ceil() as usize,
        (rect.y + rect.height).ceil() as usize,
    );
    let stride = pixmap.width() as usize * 4;
    let data = pixmap.data_mut();
    let alpha = u32::from(color.a);
    let inverse = 255 - alpha;
    for y in y0..y1 {
        for x in x0..x1 {
            let pixel = &mut data[y * stride + x * 4..y * stride + x * 4 + 4];
            for (destination, source) in pixel[..3].iter_mut().zip([color.r, color.g, color.b]) {
                *destination = ((u32::from(source) * alpha
                    + u32::from(*destination) * inverse + 127) / 255) as u8;
            }
            pixel[3] = (alpha + (u32::from(pixel[3]) * inverse + 127) / 255) as u8;
        }
    }
}

fn fill(pixmap: &mut Pixmap, rect: Rect, clip: Rect, color: Color) {
    let rect = rect.intersection(clip);
    if let Some(rect) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = false;
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn border(pixmap: &mut Pixmap, rect: Rect, clip: Rect, raised: bool) {
    let light = Color::rgb(250, 250, 250);
    let dark = Color::rgb(110, 110, 110);
    let (top, bottom) = if raised { (light, dark) } else { (dark, light) };
    fill(pixmap, Rect::new(rect.x, rect.y, rect.width, 1.0), clip, top);
    fill(pixmap, Rect::new(rect.x, rect.y, 1.0, rect.height), clip, top);
    fill(
        pixmap,
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        clip, bottom,
    );
    fill(
        pixmap,
        Rect::new(rect.x + rect.width - 1.0, rect.y, 1.0, rect.height),
        clip, bottom,
    );
}

impl Tk {
    /// Lays out and renders without creating a window or event loop.
    ///
    /// Dimensions must be nonzero and fit in a tiny-skia pixmap.
    pub fn render_to_pixmap(&mut self, width: u32, height: u32) -> Pixmap {
        let mut pixmap = Pixmap::new(width, height).expect("invalid or excessive pixmap size");
        self.layout(width, height);
        let bg = self.nodes[0].style.bg;
        pixmap.fill(tiny_skia::Color::from_rgba8(bg.r, bg.g, bg.b, bg.a));
        // Parents are always allocated before children.
        let visible: Vec<_> = self.nodes.iter().enumerate()
            .filter(|(_, node)| node.visible)
            .map(|(index, _)| WidgetId(index as u32))
            .collect();
        for id in visible {
            let index = id.0 as usize;
            let node = &self.nodes[index];
            let (kind, rect, clip, style) = (node.kind, node.rect, node.clip, node.style);
            if clip.width <= 0.0 || clip.height <= 0.0 {
                continue;
            }
            let pressed = self.pressed == Some(id) && self.hovered == Some(id);
            let bg = if kind == Kind::Button && self.hovered == Some(id) {
                Color::ACTIVE
            } else {
                style.bg
            };
            fill(&mut pixmap, rect, clip, bg);
            match kind {
                Kind::Frame => {
                    if id != self.root() {
                        border(&mut pixmap, rect, clip, false);
                    }
                    continue;
                }
                Kind::Button => border(&mut pixmap, rect, clip, !pressed),
                Kind::Entry => border(&mut pixmap, rect, clip, false),
                Kind::Label => {}
            }
            let text = self.display_text(id);
            if kind == Kind::Entry {
                let inner = rect.inset(6.0, 5.0);
                let cursor = cursor_boundary(&text, self.nodes[index].cursor);
                let caret = self.text_engine.measure(&text[..cursor], style.font_size).width;
                let available = (inner.width - 1.0).max(0.0);
                let mut scroll = self.nodes[index].scroll_x;
                if caret < scroll {
                    scroll = caret;
                } else if caret > scroll + available {
                    scroll = caret - available;
                }
                self.nodes[index].scroll_x = scroll;
                self.nodes[index].cursor = cursor;
                let origin = (
                    inner.x - scroll,
                    rect.y + (rect.height - style.font_size * 1.3) / 2.0,
                );
                let text_clip = clip.intersection(inner);
                self.text_engine.draw(
                    &mut pixmap, &text, (style.font_size, style.fg), origin, text_clip,
                );
                if self.focused == Some(id) {
                    fill(
                        &mut pixmap,
                        Rect::new(origin.0 + caret, inner.y, 1.0, inner.height),
                        text_clip, style.fg,
                    );
                }
            } else {
                let measured = self.text_engine.measure(&text, style.font_size);
                let shift = if pressed { 1.0 } else { 0.0 };
                self.text_engine.draw(
                    &mut pixmap,
                    &text,
                    (style.font_size, style.fg),
                    (
                        rect.x + (rect.width - measured.width) / 2.0 + shift,
                        rect.y + (rect.height - measured.height) / 2.0 + shift,
                    ),
                    clip.intersection(rect.inset(4.0, 2.0)),
                );
            }
        }
        self.dirty = false;
        pixmap
    }

    /// Saves a headless software rendering. No display server is required.
    pub fn save_png(
        &mut self,
        path: impl AsRef<Path>,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn Error>> {
        self.render_to_pixmap(width, height).save_png(path)?;
        Ok(())
    }
}
