//! Built-in vector icons rendered dynamically via tiny-skia.

use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::style::ImageId;
use crate::widgets::Color;
use crate::Tk;

/// Built-in monochrome icon set drawn as crisp vector paths at any size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    /// A house outline.
    Home,
    /// A microchip with pins.
    Chip,
    /// A USB trident.
    Usb,
    /// Stacked books/library shelves.
    Library,
    /// Vertical sliders with handles.
    Sliders,
    /// A wrench.
    Wrench,
    /// A filled lightning bolt.
    Bolt,
    /// A gear.
    Settings,
    /// An open book.
    Book,
    /// A shield outline.
    Shield,
    /// A downward arrow onto a baseline.
    Download,
    /// An upward arrow from a baseline.
    Upload,
    /// A check mark.
    Check,
    /// A warning triangle with an exclamation mark.
    Warning,
    /// A circle with an X.
    Error,
    /// A circle with an "i".
    Info,
    /// A circular refresh arrow.
    Refresh,
    /// A folder outline.
    Folder,
    /// A floppy-disk save symbol.
    Save,
    /// A filled play triangle.
    Play,
    /// A filled stop square.
    Stop,
    /// A sun with rays.
    Sun,
    /// A filled crescent moon.
    Moon,
    /// A magnifying glass.
    Search,
    /// A memory module.
    Memory,
    /// A partitioned square.
    Partition,
    /// A battery outline with a terminal.
    Battery,
    /// A clock face.
    Clock,
}

/// Renders `icon` at logical size `size_px` with the given `color` into an RGBA byte buffer.
pub fn render_rgba(icon: Icon, size_px: u32, color: Color) -> Vec<u8> {
    let size = size_px.max(8);
    let mut pixmap = Pixmap::new(size, size).unwrap_or_else(|| Pixmap::new(8, 8).unwrap());
    let s = size as f32;
    let k = s / 24.0;
    let sw = (1.8 * k).max(1.0);

    let mut stroke_paint = Paint::default();
    stroke_paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    stroke_paint.anti_alias = true;
    let stroke = Stroke {
        width: sw,
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    };

    let mut fill_paint = Paint::default();
    fill_paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    fill_paint.anti_alias = true;

    match icon {
        Icon::Home => {
            let mut pb = PathBuilder::new();
            pb.move_to(3.0 * k, 11.0 * k);
            pb.line_to(12.0 * k, 3.0 * k);
            pb.line_to(21.0 * k, 11.0 * k);
            pb.line_to(19.0 * k, 11.0 * k);
            pb.line_to(19.0 * k, 21.0 * k);
            pb.line_to(14.0 * k, 21.0 * k);
            pb.line_to(14.0 * k, 14.0 * k);
            pb.line_to(10.0 * k, 14.0 * k);
            pb.line_to(10.0 * k, 21.0 * k);
            pb.line_to(5.0 * k, 21.0 * k);
            pb.line_to(5.0 * k, 11.0 * k);
            pb.close();
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Chip => {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_skia::Rect::from_xywh(6.0 * k, 6.0 * k, 12.0 * k, 12.0 * k).unwrap());
            // Top/bottom pins
            for &x in &[9.0, 12.0, 15.0] {
                pb.move_to(x * k, 2.0 * k); pb.line_to(x * k, 6.0 * k);
                pb.move_to(x * k, 18.0 * k); pb.line_to(x * k, 22.0 * k);
            }
            // Left/right pins
            for &y in &[9.0, 12.0, 15.0] {
                pb.move_to(2.0 * k, y * k); pb.line_to(6.0 * k, y * k);
                pb.move_to(18.0 * k, y * k); pb.line_to(22.0 * k, y * k);
            }
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Usb => {
            let mut pb = PathBuilder::new();
            pb.move_to(12.0 * k, 19.0 * k); pb.line_to(12.0 * k, 5.0 * k);
            pb.move_to(12.0 * k, 14.0 * k); pb.line_to(7.0 * k, 10.0 * k); pb.line_to(7.0 * k, 8.0 * k);
            pb.move_to(12.0 * k, 12.0 * k); pb.line_to(17.0 * k, 9.0 * k); pb.line_to(17.0 * k, 7.0 * k);
            // Center arrowhead
            pb.move_to(10.0 * k, 7.0 * k); pb.line_to(12.0 * k, 4.0 * k); pb.line_to(14.0 * k, 7.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Library => {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_skia::Rect::from_xywh(4.0 * k, 4.0 * k, 3.5 * k, 16.0 * k).unwrap());
            pb.push_rect(tiny_skia::Rect::from_xywh(9.0 * k, 4.0 * k, 3.5 * k, 16.0 * k).unwrap());
            pb.push_rect(tiny_skia::Rect::from_xywh(14.0 * k, 4.0 * k, 3.5 * k, 16.0 * k).unwrap());
            pb.move_to(19.0 * k, 20.0 * k); pb.line_to(22.0 * k, 6.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Sliders => {
            let mut pb = PathBuilder::new();
            for &x in &[6.0, 12.0, 18.0] {
                pb.move_to(x * k, 3.0 * k); pb.line_to(x * k, 21.0 * k);
            }
            pb.push_rect(tiny_skia::Rect::from_xywh(4.0 * k, 7.0 * k, 4.0 * k, 3.5 * k).unwrap());
            pb.push_rect(tiny_skia::Rect::from_xywh(10.0 * k, 14.0 * k, 4.0 * k, 3.5 * k).unwrap());
            pb.push_rect(tiny_skia::Rect::from_xywh(16.0 * k, 9.0 * k, 4.0 * k, 3.5 * k).unwrap());
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Wrench => {
            let mut pb = PathBuilder::new();
            pb.move_to(5.0 * k, 19.0 * k); pb.line_to(13.0 * k, 11.0 * k);
            pb.move_to(13.0 * k, 11.0 * k); pb.line_to(15.0 * k, 7.0 * k); pb.line_to(19.0 * k, 7.0 * k);
            pb.line_to(19.0 * k, 9.0 * k); pb.line_to(17.0 * k, 11.0 * k); pb.line_to(18.0 * k, 14.0 * k);
            pb.line_to(15.0 * k, 15.0 * k); pb.line_to(13.0 * k, 13.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Bolt => {
            let mut pb = PathBuilder::new();
            pb.move_to(13.0 * k, 2.0 * k);
            pb.line_to(6.0 * k, 13.0 * k);
            pb.line_to(12.0 * k, 13.0 * k);
            pb.line_to(10.0 * k, 22.0 * k);
            pb.line_to(18.0 * k, 10.0 * k);
            pb.line_to(12.0 * k, 10.0 * k);
            pb.close();
            if let Some(p) = pb.finish() {
                pixmap.fill_path(&p, &fill_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
        Icon::Settings => {
            let mut pb = PathBuilder::new();
            pb.push_circle(12.0 * k, 12.0 * k, 4.0 * k);
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::PI / 4.0;
                let c = angle.cos();
                let s = angle.sin();
                pb.move_to((12.0 + 6.0 * c) * k, (12.0 + 6.0 * s) * k);
                pb.line_to((12.0 + 9.5 * c) * k, (12.0 + 9.5 * s) * k);
            }
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Book => {
            let mut pb = PathBuilder::new();
            pb.move_to(12.0 * k, 6.0 * k); pb.line_to(4.0 * k, 4.0 * k); pb.line_to(4.0 * k, 18.0 * k);
            pb.line_to(12.0 * k, 20.0 * k); pb.line_to(20.0 * k, 18.0 * k); pb.line_to(20.0 * k, 4.0 * k);
            pb.close();
            pb.move_to(12.0 * k, 6.0 * k); pb.line_to(12.0 * k, 20.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Shield => {
            let mut pb = PathBuilder::new();
            pb.move_to(12.0 * k, 3.0 * k);
            pb.line_to(19.0 * k, 6.0 * k);
            pb.line_to(19.0 * k, 13.0 * k);
            pb.cubic_to(19.0 * k, 18.0 * k, 15.0 * k, 21.0 * k, 12.0 * k, 22.0 * k);
            pb.cubic_to(9.0 * k, 21.0 * k, 5.0 * k, 18.0 * k, 5.0 * k, 13.0 * k);
            pb.line_to(5.0 * k, 6.0 * k);
            pb.close();
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Download => {
            let mut pb = PathBuilder::new();
            pb.move_to(12.0 * k, 4.0 * k); pb.line_to(12.0 * k, 16.0 * k);
            pb.move_to(7.0 * k, 11.0 * k); pb.line_to(12.0 * k, 16.0 * k); pb.line_to(17.0 * k, 11.0 * k);
            pb.move_to(4.0 * k, 20.0 * k); pb.line_to(20.0 * k, 20.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Upload => {
            let mut pb = PathBuilder::new();
            pb.move_to(12.0 * k, 16.0 * k); pb.line_to(12.0 * k, 4.0 * k);
            pb.move_to(7.0 * k, 9.0 * k); pb.line_to(12.0 * k, 4.0 * k); pb.line_to(17.0 * k, 9.0 * k);
            pb.move_to(4.0 * k, 20.0 * k); pb.line_to(20.0 * k, 20.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Check => {
            let mut pb = PathBuilder::new();
            pb.move_to(4.0 * k, 12.0 * k); pb.line_to(9.0 * k, 17.0 * k); pb.line_to(20.0 * k, 6.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Warning => {
            let mut pb = PathBuilder::new();
            pb.move_to(12.0 * k, 3.0 * k);
            pb.line_to(22.0 * k, 20.0 * k);
            pb.line_to(2.0 * k, 20.0 * k);
            pb.close();
            pb.move_to(12.0 * k, 8.0 * k); pb.line_to(12.0 * k, 14.0 * k);
            pb.move_to(12.0 * k, 17.0 * k); pb.line_to(12.0 * k, 17.5 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Error => {
            let mut pb = PathBuilder::new();
            pb.push_circle(12.0 * k, 12.0 * k, 9.0 * k);
            pb.move_to(8.0 * k, 8.0 * k); pb.line_to(16.0 * k, 16.0 * k);
            pb.move_to(16.0 * k, 8.0 * k); pb.line_to(8.0 * k, 16.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Info => {
            let mut pb = PathBuilder::new();
            pb.push_circle(12.0 * k, 12.0 * k, 9.0 * k);
            pb.move_to(12.0 * k, 7.5 * k); pb.line_to(12.0 * k, 8.5 * k);
            pb.move_to(12.0 * k, 11.0 * k); pb.line_to(12.0 * k, 16.5 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Refresh => {
            let mut pb = PathBuilder::new();
            pb.move_to(20.0 * k, 12.0 * k);
            pb.cubic_to(20.0 * k, 16.4 * k, 16.4 * k, 20.0 * k, 12.0 * k, 20.0 * k);
            pb.cubic_to(7.6 * k, 20.0 * k, 4.0 * k, 16.4 * k, 4.0 * k, 12.0 * k);
            pb.cubic_to(4.0 * k, 7.6 * k, 7.6 * k, 4.0 * k, 12.0 * k, 4.0 * k);
            pb.line_to(17.0 * k, 4.0 * k);
            pb.move_to(14.0 * k, 1.0 * k); pb.line_to(17.0 * k, 4.0 * k); pb.line_to(14.0 * k, 7.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Folder => {
            let mut pb = PathBuilder::new();
            pb.move_to(3.0 * k, 6.0 * k); pb.line_to(9.0 * k, 6.0 * k); pb.line_to(11.0 * k, 8.0 * k);
            pb.line_to(21.0 * k, 8.0 * k); pb.line_to(21.0 * k, 19.0 * k); pb.line_to(3.0 * k, 19.0 * k);
            pb.close();
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Save => {
            let mut pb = PathBuilder::new();
            pb.move_to(4.0 * k, 4.0 * k); pb.line_to(17.0 * k, 4.0 * k); pb.line_to(20.0 * k, 7.0 * k);
            pb.line_to(20.0 * k, 20.0 * k); pb.line_to(4.0 * k, 20.0 * k); pb.close();
            pb.push_rect(tiny_skia::Rect::from_xywh(8.0 * k, 4.0 * k, 8.0 * k, 5.0 * k).unwrap());
            pb.push_rect(tiny_skia::Rect::from_xywh(6.0 * k, 13.0 * k, 12.0 * k, 7.0 * k).unwrap());
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Play => {
            let mut pb = PathBuilder::new();
            pb.move_to(6.0 * k, 4.0 * k); pb.line_to(19.0 * k, 12.0 * k); pb.line_to(6.0 * k, 20.0 * k);
            pb.close();
            if let Some(p) = pb.finish() {
                pixmap.fill_path(&p, &fill_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
        Icon::Stop => {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_skia::Rect::from_xywh(5.0 * k, 5.0 * k, 14.0 * k, 14.0 * k).unwrap());
            if let Some(p) = pb.finish() {
                pixmap.fill_path(&p, &fill_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
        Icon::Sun => {
            let mut pb = PathBuilder::new();
            pb.push_circle(12.0 * k, 12.0 * k, 4.5 * k);
            for i in 0..8 {
                let a = (i as f32) * std::f32::consts::PI / 4.0;
                let c = a.cos();
                let s = a.sin();
                pb.move_to((12.0 + 6.5 * c) * k, (12.0 + 6.5 * s) * k);
                pb.line_to((12.0 + 9.5 * c) * k, (12.0 + 9.5 * s) * k);
            }
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Moon => {
            let mut pb = PathBuilder::new();
            pb.move_to(14.0 * k, 3.0 * k);
            pb.cubic_to(19.0 * k, 5.0 * k, 21.0 * k, 11.0 * k, 18.0 * k, 16.0 * k);
            pb.cubic_to(15.0 * k, 20.0 * k, 9.0 * k, 21.0 * k, 5.0 * k, 18.0 * k);
            pb.cubic_to(11.0 * k, 18.0 * k, 15.0 * k, 13.0 * k, 14.0 * k, 3.0 * k);
            pb.close();
            if let Some(p) = pb.finish() {
                pixmap.fill_path(&p, &fill_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
        Icon::Search => {
            let mut pb = PathBuilder::new();
            pb.push_circle(10.5 * k, 10.5 * k, 6.0 * k);
            pb.move_to(15.0 * k, 15.0 * k); pb.line_to(21.0 * k, 21.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Memory => {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_skia::Rect::from_xywh(3.0 * k, 8.0 * k, 18.0 * k, 8.0 * k).unwrap());
            for &x in &[5.0, 7.5, 10.0, 14.0, 16.5, 19.0] {
                pb.move_to(x * k, 16.0 * k); pb.line_to(x * k, 18.5 * k);
            }
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Partition => {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_skia::Rect::from_xywh(4.0 * k, 4.0 * k, 16.0 * k, 16.0 * k).unwrap());
            pb.move_to(12.0 * k, 4.0 * k); pb.line_to(12.0 * k, 20.0 * k);
            pb.move_to(4.0 * k, 12.0 * k); pb.line_to(12.0 * k, 12.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Battery => {
            let mut pb = PathBuilder::new();
            pb.push_rect(tiny_skia::Rect::from_xywh(3.0 * k, 7.0 * k, 15.0 * k, 10.0 * k).unwrap());
            pb.move_to(18.0 * k, 10.0 * k); pb.line_to(20.5 * k, 10.0 * k); pb.line_to(20.5 * k, 14.0 * k); pb.line_to(18.0 * k, 14.0 * k);
            pb.push_rect(tiny_skia::Rect::from_xywh(5.5 * k, 9.5 * k, 5.0 * k, 5.0 * k).unwrap());
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Icon::Clock => {
            let mut pb = PathBuilder::new();
            pb.push_circle(12.0 * k, 12.0 * k, 9.0 * k);
            pb.move_to(12.0 * k, 12.0 * k); pb.line_to(12.0 * k, 7.0 * k);
            pb.move_to(12.0 * k, 12.0 * k); pb.line_to(16.0 * k, 12.0 * k);
            if let Some(p) = pb.finish() {
                pixmap.stroke_path(&p, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    // Unpremultiply tiny-skia internal pixels so `register_image_rgba` can premultiply consistently.
    let mut straight = Vec::with_capacity(pixmap.data().len());
    for chunk in pixmap.data().chunks_exact(4) {
        let a = u32::from(chunk[3]);
        if a == 0 {
            straight.extend_from_slice(&[0, 0, 0, 0]);
        } else if a == 255 {
            straight.extend_from_slice(chunk);
        } else {
            let r = ((u32::from(chunk[0]) * 255 + a / 2) / a).min(255) as u8;
            let g = ((u32::from(chunk[1]) * 255 + a / 2) / a).min(255) as u8;
            let b = ((u32::from(chunk[2]) * 255 + a / 2) / a).min(255) as u8;
            straight.extend_from_slice(&[r, g, b, chunk[3]]);
        }
    }
    straight
}

/// Helper that renders `icon` and registers it directly into `tk`, returning its `ImageId`.
pub fn icon(tk: &mut Tk, icon: Icon, size_px: u32, color: Color) -> Option<ImageId> {
    let rgba = render_rgba(icon, size_px, color);
    tk.register_image_rgba(size_px, size_px, &rgba)
}
