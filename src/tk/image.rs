//! Image widget APIs: RGBA/PNG/SVG sources, fit modes, and SVG icon loading.

#[allow(unused_imports)]
use super::*;

use crate::image::{ImageData, ImageFit, ImageSource};

impl Tk {
    /// Returns a mutable reference to the image data of `id`, if it is a live image.
    #[allow(dead_code)]
    fn image_data_mut(&mut self, id: WidgetId) -> Option<&mut ImageData> {
        let node = self.node_mut(id)?;
        if node.kind != Kind::Image {
            return None;
        }
        node.image_data.as_deref_mut()
    }

    /// Sets the image of `id` from a straight-alpha RGBA buffer.
    ///
    /// Rejects zero dimensions or a buffer whose length is not `w * h * 4`.
    pub fn image_set_rgba(&mut self, id: WidgetId, w: u32, h: u32, rgba: &[u8]) -> bool {
        if w == 0 || h == 0 {
            return false;
        }
        let expected = (w as usize)
            .checked_mul(h as usize)
            .and_then(|n| n.checked_mul(4));
        if expected != Some(rgba.len()) {
            return false;
        }
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Image {
            return false;
        }
        let fit = node
            .image_data
            .as_deref()
            .map(|d| d.fit)
            .unwrap_or_default();
        node.image_data = Some(Box::new(ImageData {
            source: ImageSource::Rgba { w, h, rgba: rgba.to_vec() },
            fit,
        }));
        self.dirty = true;
        true
    }

    /// Loads a PNG into `id`, either from a file path (UTF-8 string naming a
    /// readable file) or from in-memory PNG bytes. Returns false on decode failure.
    pub fn image_load_png(&mut self, id: WidgetId, path_or_png_bytes: impl AsRef<[u8]>) -> bool {
        let bytes = path_or_png_bytes.as_ref();
        let decoded = if let Ok(s) = std::str::from_utf8(bytes) {
            if let Ok(file_bytes) = fs::read(s) {
                Pixmap::decode_png(&file_bytes).ok()
            } else {
                Pixmap::decode_png(bytes).ok()
            }
        } else {
            Pixmap::decode_png(bytes).ok()
        };
        let Some(pixmap) = decoded else { return false };
        let (w, h) = (pixmap.width(), pixmap.height());
        if w == 0 || h == 0 {
            return false;
        }
        // tiny-skia stores premultiplied RGBA; convert to straight alpha.
        let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
        for px in pixmap.pixels() {
            let c = px.demultiply();
            rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
        }
        self.image_set_rgba(id, w, h, &rgba)
    }

    /// Sets the image of `id` from SVG bytes. Returns false for invalid SVG.
    #[cfg(feature = "svg")]
    pub fn image_set_svg(&mut self, id: WidgetId, svg: &[u8]) -> bool {
        let Some(node) = self.node(id) else { return false };
        if node.kind != Kind::Image {
            return false;
        }
        let Ok(tree) = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()) else {
            return false;
        };
        let fit = node
            .image_data
            .as_deref()
            .map(|d| d.fit)
            .unwrap_or_default();
        let Some(node) = self.node_mut(id) else { return false };
        node.image_data = Some(Box::new(ImageData {
            source: ImageSource::Svg(std::sync::Arc::new(tree)),
            fit,
        }));
        self.dirty = true;
        true
    }

    /// Sets the fit mode of `id`. Returns false if `id` is not a live image.
    pub fn image_set_fit(&mut self, id: WidgetId, fit: ImageFit) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Image {
            return false;
        }
        match node.image_data.as_deref_mut() {
            Some(data) => {
                if data.fit == fit {
                    return true;
                }
                data.fit = fit;
            }
            None => {
                // No source yet: remember the fit for when one is set.
                node.image_data = Some(Box::new(ImageData {
                    source: ImageSource::Rgba { w: 1, h: 1, rgba: vec![0, 0, 0, 0] },
                    fit,
                }));
            }
        }
        self.dirty = true;
        true
    }

    /// Returns the natural (intrinsic) size of the image in `id`, if any.
    pub fn image_natural_size(&self, id: WidgetId) -> Option<(u32, u32)> {
        let node = self.node(id)?;
        if node.kind != Kind::Image {
            return None;
        }
        node.image_data.as_deref().map(|d| d.natural_size())
    }

    /// Clears the image of `id`, leaving an empty widget. Returns false if not a live image.
    pub fn image_clear(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Image {
            return false;
        }
        node.image_data = None;
        self.dirty = true;
        true
    }

    /// Rasterizes SVG bytes to `w`×`h` and registers the result as an image,
    /// returning its [`ImageId`] for use with `Opt::Image` on Buttons/Labels.
    #[cfg(feature = "svg")]
    pub fn load_svg(&mut self, svg: &[u8], w: u32, h: u32) -> Option<ImageId> {
        let rgba = crate::image::svg_to_rgba(svg, w, h)?;
        self.register_image_rgba(w, h, &rgba)
    }
}
