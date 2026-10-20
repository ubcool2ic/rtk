//! Image widget data model: RGBA and SVG sources, fit modes, and draw-item
//! generation. The rendering itself happens in `render.rs`; this module only
//! turns an [`ImageData`] into positioned [`CanvasItemKind::Image`] items.

use crate::widgets::CanvasItemKind;

/// How an image is placed inside its widget rectangle.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageFit {
    /// Scale preserving aspect ratio to fit inside the widget, centred (default).
    #[default]
    Contain,
    /// Stretch to fill the whole widget rectangle.
    Fill,
    /// Draw at the image's natural size, centred (no scaling).
    Center,
}

/// The pixel source backing an image widget.
pub(crate) enum ImageSource {
    /// Straight-alpha RGBA8 pixels.
    Rgba {
        /// Width in pixels.
        w: u32,
        /// Height in pixels.
        h: u32,
        /// Straight (un-premultiplied) RGBA bytes, length `w * h * 4`.
        rgba: Vec<u8>,
    },
    /// A parsed SVG tree, rasterized on demand at device resolution.
    #[cfg(feature = "svg")]
    Svg(std::sync::Arc<resvg::usvg::Tree>),
}

/// The full state of an image widget: its source and fit mode.
pub(crate) struct ImageData {
    /// Pixel source.
    pub source: ImageSource,
    /// Placement mode inside the widget rectangle.
    pub fit: ImageFit,
}

impl ImageData {
    /// Natural (intrinsic) size of the image in pixels, at least 1×1.
    pub fn natural_size(&self) -> (u32, u32) {
        match &self.source {
            ImageSource::Rgba { w, h, .. } => ((*w).max(1), (*h).max(1)),
            #[cfg(feature = "svg")]
            ImageSource::Svg(tree) => {
                let size = tree.size();
                (
                    (size.width().ceil() as u32).max(1),
                    (size.height().ceil() as u32).max(1),
                )
            }
        }
    }
}

/// Rasterizes a parsed SVG tree to fit `w`×`h` (aspect preserved, centred),
/// returning straight-alpha RGBA of length `w * h * 4`.
#[cfg(feature = "svg")]
fn render_svg(tree: &resvg::usvg::Tree, w: u32, h: u32) -> Option<Vec<u8>> {
    if w == 0 || h == 0 {
        return None;
    }
    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
    let size = tree.size();
    let (sw, sh) = (size.width(), size.height());
    if sw <= 0.0 || sh <= 0.0 {
        return None;
    }
    let scale = (w as f32 / sw).min(h as f32 / sh);
    let draw_w = sw * scale;
    let draw_h = sh * scale;
    let tx = (w as f32 - draw_w) / 2.0;
    let ty = (h as f32 - draw_h) / 2.0;
    let transform = tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);
    resvg::render(tree, transform, &mut pixmap.as_mut());
    // tiny-skia stores premultiplied RGBA; convert to straight alpha.
    let mut out = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for px in pixmap.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Some(out)
}

/// Parses SVG bytes and rasterizes them to fit `w`×`h`, returning straight-alpha
/// RGBA of length `w * h * 4`. Returns `None` for invalid SVG or zero size.
#[cfg(feature = "svg")]
pub fn svg_to_rgba(svg: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    if w == 0 || h == 0 {
        return None;
    }
    let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()).ok()?;
    render_svg(&tree, w, h)
}

/// Returns `true` when the target size `dw`×`dh` is an integer multiple (at
/// least 2×) of the source size `sw`×`sh` in both directions, within a small
/// tolerance for floating-point rounding.
fn is_integer_upscale(dw: f32, dh: f32, sw: u32, sh: u32) -> bool {
    if sw == 0 || sh == 0 || dw < 1.0 || dh < 1.0 {
        return false;
    }
    let factor_x = dw / sw as f32;
    let factor_y = dh / sh as f32;
    let integer = |f: f32| f >= 2.0 && (f - f.round()).abs() <= 1e-3;
    integer(factor_x) && integer(factor_y)
}

/// Builds the draw items for an image widget of logical size `w`×`h` at `scale`.
///
/// Returns a single [`CanvasItemKind::Image`] positioned per the fit mode, or an
/// empty list if the widget is degenerate. SVG sources are rasterized at device
/// resolution (logical size × `scale`) so high-DPI output stays crisp.
pub(crate) fn build_items(data: &ImageData, w: f32, h: f32, scale: f32) -> Vec<CanvasItemKind> {
    if w < 1.0 || h < 1.0 {
        return Vec::new();
    }
    let (nat_w, nat_h) = data.natural_size();
    let (nat_w_f, nat_h_f) = (nat_w as f32, nat_h as f32);

    // Target logical rectangle for the image.
    let (dx, dy, dw, dh) = match data.fit {
        ImageFit::Fill => (0.0, 0.0, w, h),
        ImageFit::Center => {
            let dw = nat_w_f.min(w);
            let dh = nat_h_f.min(h);
            ((w - dw) / 2.0, (h - dh) / 2.0, dw, dh)
        }
        ImageFit::Contain => {
            let s = (w / nat_w_f).min(h / nat_h_f);
            let dw = nat_w_f * s;
            let dh = nat_h_f * s;
            ((w - dw) / 2.0, (h - dh) / 2.0, dw, dh)
        }
    };

    match &data.source {
        ImageSource::Rgba { w: sw, h: sh, rgba } => {
            if *sw == 0 || *sh == 0 {
                return Vec::new();
            }
            // Pixel art should stay crisp when enlarged by an integer factor;
            // any other scaling keeps bilinear smoothing.
            let smooth = !is_integer_upscale(dw, dh, *sw, *sh);
            vec![CanvasItemKind::Image {
                x: dx,
                y: dy,
                w: dw,
                h: dh,
                src_w: *sw,
                src_h: *sh,
                rgba: rgba.clone(),
                smooth,
            }]
        }
        #[cfg(feature = "svg")]
        ImageSource::Svg(tree) => {
            let rw = ((dw * scale).round() as u32).max(1);
            let rh = ((dh * scale).round() as u32).max(1);
            let Some(rgba) = render_svg(tree, rw, rh) else {
                return Vec::new();
            };
            vec![CanvasItemKind::Image {
                x: dx,
                y: dy,
                w: dw,
                h: dh,
                src_w: rw,
                src_h: rh,
                rgba,
                smooth: true,
            }]
        }
    }
}
