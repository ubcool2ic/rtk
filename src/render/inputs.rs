//! Drawing for the milestone 0.10 input widgets: ProgressRing, Switch, and the
//! polished linear ProgressBar (percent/label text and smooth easing).
//!
//! This is a child module of `render`, so it can reuse the private helpers
//! (`fill`, `outline`) via `super::`.

use tiny_skia::{Mask, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::{fill, outline};
use crate::layout::Rect;
use crate::style::ResolvedStyle;
use crate::tk::inputs_state::RingMode;
use crate::widgets::{Color, Kind};
use crate::Tk;

/// Builds a stroked arc path from `start` sweeping `sweep` radians clockwise,
/// approximated with short line segments for smooth anti-aliasing.
fn arc_path(cx: f32, cy: f32, radius: f32, start: f32, sweep: f32) -> Option<tiny_skia::Path> {
    if radius <= 0.0 || sweep.abs() < 1e-4 {
        return None;
    }
    let steps = ((sweep.abs() / 0.08).ceil() as usize).clamp(2, 512);
    let mut pb = PathBuilder::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = start + sweep * t;
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    pb.finish()
}

/// Builds a pixel mask covering `clip` (physical px); `None` if the clip is empty.
fn clip_mask(pixmap: &Pixmap, clip: Rect) -> Option<Mask> {
    let r = tiny_skia::Rect::from_xywh(clip.x, clip.y, clip.width, clip.height)?;
    let mut mask = Mask::new(pixmap.width(), pixmap.height())?;
    mask.fill_path(
        &PathBuilder::from_rect(r),
        tiny_skia::FillRule::Winding,
        false,
        Transform::identity(),
    );
    Some(mask)
}

/// Strokes `path` with a round-capped line of `width` px in `color`, limited to `mask`.
fn stroke_arc(pixmap: &mut Pixmap, path: &tiny_skia::Path, width: f32, color: Color, mask: &Mask) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    paint.anti_alias = true;
    let stroke = Stroke {
        width,
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    };
    pixmap.stroke_path(path, &paint, &stroke, Transform::identity(), Some(mask));
}

/// Draws a ProgressRing (percent arc or rotating spinner) with optional text.
pub(super) fn draw_ring(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let Some(node) = tk.node(id) else { return };
    let diameter = node.progress_length.min(logical.width).min(logical.height);
    let fraction = node.progress_f64.and_then(|v| v.get(tk)).unwrap_or(0.0);
    let fraction = if fraction.is_nan() { 0.0 } else { fraction.clamp(0.0, 1.0) } as f32;
    let (mode, show_text, thickness, phase) = node
        .inputs
        .as_ref()
        .map(|s| (s.ring_mode, s.ring_show_text, s.ring_thickness, s.ring_phase))
        .unwrap_or((RingMode::Percent, false, 0.0, 0.0));
    let accent = tk.style_engine.tokens().palette.accent;
    let track = tk.style_engine.tokens().palette.border;

    let cx = (logical.x + logical.width / 2.0) * scale;
    let cy = (logical.y + logical.height / 2.0) * scale;
    let stroke_w = if thickness > 0.0 {
        thickness * scale
    } else {
        (diameter * 0.12).max(2.0) * scale
    };
    let radius = (diameter * scale - stroke_w) / 2.0;
    if radius <= 0.0 {
        return;
    }
    let Some(mask) = clip_mask(pixmap, clip) else { return };

    // Full track circle.
    if let Some(path) = arc_path(cx, cy, radius, 0.0, std::f32::consts::TAU) {
        stroke_arc(pixmap, &path, stroke_w, track, &mask);
    }

    match mode {
        RingMode::Percent => {
            if fraction > 0.0 {
                // Start at 12 o'clock (-PI/2) and sweep clockwise.
                let sweep = fraction * std::f32::consts::TAU;
                if let Some(path) = arc_path(cx, cy, radius, -std::f32::consts::FRAC_PI_2, sweep) {
                    stroke_arc(pixmap, &path, stroke_w, accent, &mask);
                }
            }
            if show_text {
                let text = format!("{}%", (fraction * 100.0).round() as i32);
                let m = tk.measure_with(&text, resolved);
                let origin = (
                    (logical.x + (logical.width - m.width) / 2.0) * scale,
                    (logical.y + (logical.height - m.height) / 2.0) * scale,
                );
                tk.text_engine.draw_styled_pub(
                    pixmap,
                    &text,
                    (resolved.font_size * scale, resolved.fg),
                    resolved.font_family.as_deref(),
                    400,
                    false,
                    origin,
                    clip,
                    resolved.opacity,
                );
            }
        }
        RingMode::Spinner => {
            let sweep = std::f32::consts::FRAC_PI_2;
            let start = -std::f32::consts::FRAC_PI_2 + phase * std::f32::consts::TAU;
            if let Some(path) = arc_path(cx, cy, radius, start, sweep) {
                stroke_arc(pixmap, &path, stroke_w, accent, &mask);
            }
        }
    }
}

/// Draws a Switch: a pill track with a sliding knob and a label to the right.
pub(super) fn draw_switch(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let Some(node) = tk.node(id) else { return };
    let on = node.check.and_then(|v| v.get(tk)).unwrap_or(false);
    let knob = node
        .inputs
        .as_ref()
        .map(|s| if s.switch_initialised { s.switch_knob } else if on { 1.0 } else { 0.0 })
        .unwrap_or(if on { 1.0 } else { 0.0 });
    let disabled = node.disabled;
    let text = node.text.clone();
    let palette = tk.style_engine.tokens().palette;
    let Some(mask) = clip_mask(pixmap, clip) else { return };

    let track_w = 36.0;
    let track_h = 20.0;
    let track_x = logical.x + resolved.padding.left;
    let track_y = logical.y + (logical.height - track_h) / 2.0;
    let track_rect = Rect::new(track_x, track_y, track_w, track_h).scaled(scale);

    let track_color = if on { palette.accent } else { palette.border };
    let track_color = if disabled {
        Color::rgba(track_color.r, track_color.g, track_color.b, 120)
    } else {
        track_color
    };
    if let Some(path) = crate::style::build_rounded_rect_path(track_rect, track_h / 2.0 * scale) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(track_color.r, track_color.g, track_color.b, track_color.a);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), Some(&mask));
    }

    // Knob: 16 px diameter, travels from 2 px to track_w - 18 px.
    let knob_d = 16.0;
    let travel = track_w - knob_d - 4.0;
    let knob_x = track_x + 2.0 + travel * knob.clamp(0.0, 1.0);
    let knob_y = track_y + (track_h - knob_d) / 2.0;
    let knob_color = if on { palette.accent_fg } else { palette.surface };
    let knob_color = if disabled {
        Color::rgba(knob_color.r, knob_color.g, knob_color.b, 160)
    } else {
        knob_color
    };
    let knob_rect = Rect::new(knob_x, knob_y, knob_d, knob_d).scaled(scale);
    if let Some(path) = crate::style::build_rounded_rect_path(knob_rect, knob_d / 2.0 * scale) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(knob_color.r, knob_color.g, knob_color.b, knob_color.a);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), Some(&mask));
    }

    if !text.is_empty() {
        let m = tk.measure_with(&text, resolved);
        let origin = (
            (track_x + track_w + 8.0) * scale,
            (logical.y + (logical.height - m.height) / 2.0) * scale,
        );
        let fg = if disabled { palette.muted } else { resolved.fg };
        tk.text_engine.draw_styled_pub(
            pixmap,
            &text,
            (resolved.font_size * scale, fg),
            resolved.font_family.as_deref(),
            resolved.font_weight,
            resolved.italic,
            origin,
            clip,
            resolved.opacity,
        );
    }
}

/// Draws a linear ProgressBar, honouring indeterminate mode, smooth easing,
/// percent text and an optional label. The default look is unchanged.
pub(super) fn draw_progress_bar(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let Some(node) = tk.node(id) else { return };
    let inner = logical.inset(2.0, 2.0);
    let accent = tk.style_engine.tokens().palette.accent;
    let (show_percent, label, smooth, drawn) = node
        .inputs
        .as_ref()
        .map(|s| (s.progress_show_percent, s.progress_label.clone(), s.progress_smooth, s.progress_drawn))
        .unwrap_or((false, None, false, 0.0));

    // The filled rectangle in logical px, used both for drawing the fill and
    // for clipping the accent-coloured text overlay.
    let fill_color = if node.progress_indeterminate { accent } else { resolved.fg };
    let filled_rect = if node.progress_indeterminate {
        let block_w = (inner.width * 0.3).max(10.0);
        let travel = inner.width + block_w;
        let x = inner.x - block_w + travel * node.progress_phase;
        let r = Rect::new(x, inner.y, block_w, inner.height).intersection(inner);
        if r.width > 0.0 {
            fill(pixmap, r.scaled(scale), clip, accent);
        }
        r
    } else {
        let frac = if smooth {
            drawn
        } else if let Some(v64) = node.progress_f64 {
            v64.get(tk).unwrap_or(0.0).clamp(0.0, 1.0) as f32
        } else {
            (node.scale.and_then(|v| v.get(tk)).unwrap_or(0).clamp(0, 100) as f32) / 100.0
        };
        let frac = if frac.is_nan() { 0.0 } else { frac.clamp(0.0, 1.0) };
        let filled_w = inner.width * frac;
        let r = Rect::new(inner.x, inner.y, filled_w, inner.height);
        if filled_w > 0.0 {
            fill(pixmap, r.scaled(scale), clip, resolved.fg);
        }
        r
    };

    if show_percent || label.is_some() {
        let text = match (&label, show_percent) {
            (Some(l), true) => format!("{l} {}%", (progress_fraction(tk, id) * 100.0).round() as i32),
            (Some(l), false) => l.clone(),
            (None, true) => format!("{}%", (progress_fraction(tk, id) * 100.0).round() as i32),
            (None, false) => String::new(),
        };
        if !text.is_empty() {
            let m = tk.measure_with(&text, resolved);
            let origin = (
                (logical.x + (logical.width - m.width) / 2.0) * scale,
                (logical.y + (logical.height - m.height) / 2.0) * scale,
            );
            let palette = tk.style_engine.tokens().palette;
            let filled_clip = clip.intersection(filled_rect.scaled(scale));
            let unfilled_clip = clip.intersection(unfilled_rect(logical, filled_rect).scaled(scale));
            // Track (unfilled) pass: the more readable of fg/bg over the track
            // colour (resolved.bg), so HighContrast's white-on-white track is
            // fixed. Clipped to the unfilled part so it never bleeds over the
            // fill.
            let track_text = readable_on(resolved.bg, palette.fg, palette.bg);
            if unfilled_clip.width > 0.0 && unfilled_clip.height > 0.0 {
                tk.text_engine.draw_styled_pub(
                    pixmap,
                    &text,
                    (resolved.font_size * scale, track_text),
                    resolved.font_family.as_deref(),
                    400,
                    false,
                    origin,
                    unfilled_clip,
                    resolved.opacity,
                );
            }
            // Fill pass: the more readable of fg/bg over the fill colour
            // (resolved.fg), clipped to the filled rectangle so the text stays
            // readable where it overlaps the fill.
            if filled_clip.width > 0.0 && filled_clip.height > 0.0 {
                let fill_text = readable_on(fill_color, palette.fg, palette.bg);
                tk.text_engine.draw_styled_pub(
                    pixmap,
                    &text,
                    (resolved.font_size * scale, fill_text),
                    resolved.font_family.as_deref(),
                    400,
                    false,
                    origin,
                    filled_clip,
                    resolved.opacity,
                );
            }
        }
    }
}

/// Returns the unfilled part of a progress bar: `logical` minus the filled
/// rectangle on the horizontal axis (the fill always grows from the left).
fn unfilled_rect(logical: Rect, filled: Rect) -> Rect {
    let start = (filled.x + filled.width).max(logical.x);
    let end = (logical.x + logical.width).max(start);
    Rect::new(start, logical.y, end - start, logical.height)
}

/// Returns whichever of `a` or `b` has the larger WCAG contrast ratio against
/// `bg`, using relative luminance with sRGB linearisation. Ties favour `a`.
fn readable_on(bg: Color, a: Color, b: Color) -> Color {
    /// Linearises one sRGB channel in `0..=255` to `0.0..=1.0`.
    fn linearise(channel: u8) -> f32 {
        let c = channel as f32 / 255.0;
        if c <= 0.040_45 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// WCAG relative luminance of `color` (alpha is ignored).
    fn luminance(color: Color) -> f32 {
        0.2126 * linearise(color.r) + 0.7152 * linearise(color.g) + 0.0722 * linearise(color.b)
    }

    /// WCAG contrast ratio between two relative luminances.
    fn ratio(l1: f32, l2: f32) -> f32 {
        let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
        (hi + 0.05) / (lo + 0.05)
    }

    let bg_lum = luminance(bg);
    let a_ratio = ratio(luminance(a), bg_lum);
    let b_ratio = ratio(luminance(b), bg_lum);
    if a_ratio >= b_ratio {
        a
    } else {
        b
    }
}

/// Returns the current fraction of a progress bar in `0.0..=1.0`.
fn progress_fraction(tk: &Tk, id: crate::WidgetId) -> f64 {
    let Some(node) = tk.node(id).filter(|n| n.kind == Kind::ProgressBar) else { return 0.0 };
    let value = if let Some(v64) = node.progress_f64 {
        v64.get(tk).unwrap_or(0.0)
    } else {
        node.scale.and_then(|v| v.get(tk)).unwrap_or(0) as f64 / 100.0
    };
    if value.is_nan() { 0.0 } else { value.clamp(0.0, 1.0) }
}

/// Keeps `outline` referenced so the import stays used if drawing changes.
#[allow(dead_code)]
fn _keep_outline_import(pixmap: &mut Pixmap, rect: Rect, clip: Rect, scale: f32, color: Color) {
    outline(pixmap, rect, clip, scale, (color, color));
}

/// Draws a Slider: rounded track, accent fill, round thumb, optional ticks.
pub(super) fn draw_slider(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let Some(node) = tk.node(id) else { return };
    let orient = node.inputs.as_ref().map_or(crate::tk::inputs_state::Orient::Horizontal, |s| s.slider_orient);
    let ticks = node.inputs.as_ref().and_then(|s| s.slider_ticks);
    let (from, to) = node.range_f64;
    let disabled = node.disabled;
    let fraction = tk.slider_fraction(id);
    let palette = tk.style_engine.tokens().palette;
    // Track always uses the theme border colour; the filled portion the accent.
    let track_color = palette.border;
    let fill_color = if disabled { palette.muted } else { palette.accent };
    let thumb_border = palette.border;
    // A transparent widget background means "show the surface through", so the
    // thumb uses the theme surface; otherwise it matches the widget background.
    let thumb_fill = if resolved.bg.a == 0 { palette.surface } else { resolved.bg };

    // Clip mask so the path-based drawing below honours `clip`.
    let clip_mask = {
        let mut m = tiny_skia::Mask::new(pixmap.width(), pixmap.height());
        if let Some(mm) = m.as_mut() {
            if let Some(r) = tiny_skia::Rect::from_xywh(clip.x, clip.y, clip.width, clip.height) {
                mm.fill_path(&tiny_skia::PathBuilder::from_rect(r), tiny_skia::FillRule::Winding, false, Transform::identity());
            }
        }
        m
    };

    let track_w = 4.0;
    let thumb_d = 16.0;

    let (track_rect, thumb_center) = match orient {
        crate::tk::inputs_state::Orient::Horizontal => {
            let track = logical.inset(12.0, 0.0);
            let y = logical.y + logical.height / 2.0;
            let track_rect = Rect::new(track.x, y - track_w / 2.0, track.width, track_w);
            let cx = track.x + track.width * fraction;
            (track_rect, (cx, y))
        }
        crate::tk::inputs_state::Orient::Vertical => {
            let track = logical.inset(0.0, 12.0);
            let x = logical.x + logical.width / 2.0;
            let track_rect = Rect::new(x - track_w / 2.0, track.y, track_w, track.height);
            let cy = track.y + track.height * (1.0 - fraction);
            (track_rect, (x, cy))
        }
    };

    // Track background.
    if let Some(path) = crate::style::build_rounded_rect_path(track_rect.scaled(scale), track_w / 2.0 * scale) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(track_color.r, track_color.g, track_color.b, track_color.a);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), clip_mask.as_ref());
    }

    // Filled portion.
    let filled = match orient {
        crate::tk::inputs_state::Orient::Horizontal => {
            Rect::new(track_rect.x, track_rect.y, track_rect.width * fraction, track_rect.height)
        }
        crate::tk::inputs_state::Orient::Vertical => {
            let h = track_rect.height * fraction;
            Rect::new(track_rect.x, track_rect.y + track_rect.height - h, track_rect.width, h)
        }
    };
    if filled.width > 0.0 && filled.height > 0.0 {
        if let Some(path) = crate::style::build_rounded_rect_path(filled.scaled(scale), track_w / 2.0 * scale) {
            let mut paint = Paint::default();
            paint.set_color_rgba8(fill_color.r, fill_color.g, fill_color.b, fill_color.a);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), clip_mask.as_ref());
        }
    }

    // Ticks.
    if let Some(every) = ticks {
        if every > 0.0 && (to - from).abs() > f64::EPSILON {
            let span = (to - from).abs();
            let count = (span / every).floor() as i64;
            // Too dense to be useful (also guards against runaway loops).
            let count = if count > 1000 { -1 } else { count };
            for k in 0..=count {
                let value = from + (k as f64) * every * if to >= from { 1.0 } else { -1.0 };
                let f = ((value - from) / (to - from)).clamp(0.0, 1.0) as f32;
                match orient {
                    crate::tk::inputs_state::Orient::Horizontal => {
                        let x = track_rect.x + track_rect.width * f;
                        fill(
                            pixmap,
                            Rect::new(x, logical.y + 2.0, 1.0, logical.height - 4.0).scaled(scale),
                            clip,
                            palette.muted,
                        );
                    }
                    crate::tk::inputs_state::Orient::Vertical => {
                        let y = track_rect.y + track_rect.height * (1.0 - f);
                        fill(
                            pixmap,
                            Rect::new(logical.x + 2.0, y, logical.width - 4.0, 1.0).scaled(scale),
                            clip,
                            palette.muted,
                        );
                    }
                }
            }
        }
    }

    // Thumb.
    let thumb_rect = Rect::new(
        thumb_center.0 - thumb_d / 2.0,
        thumb_center.1 - thumb_d / 2.0,
        thumb_d,
        thumb_d,
    ).scaled(scale);
    if let Some(path) = crate::style::build_rounded_rect_path(thumb_rect, thumb_d / 2.0 * scale) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(thumb_fill.r, thumb_fill.g, thumb_fill.b, thumb_fill.a);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), clip_mask.as_ref());
        let mut stroke_paint = Paint::default();
        stroke_paint.set_color_rgba8(thumb_border.r, thumb_border.g, thumb_border.b, thumb_border.a);
        stroke_paint.anti_alias = true;
        let stroke = Stroke { width: 1.5 * scale, ..Stroke::default() };
        pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), clip_mask.as_ref());
    }
}

/// Draws a DoubleSpinbox: text field + up/down arrow buttons on the right.
pub(super) fn draw_dspin(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let Some(node) = tk.node(id) else { return };
    let disabled = node.disabled;
    let text = tk.dspin_display(id);
    let palette = tk.style_engine.tokens().palette;
    let fg = if disabled { palette.muted } else { resolved.fg };

    // Arrow column width in logical px; the text field occupies the rest.
    let arrow_w = 20.0_f32.min(logical.width);
    let inner = Rect::new(logical.x, logical.y, (logical.width - arrow_w).max(0.0), logical.height);
    let m = tk.measure_with(&text, resolved);
    let origin = (
        (inner.x + 6.0) * scale,
        (inner.y + (inner.height - m.height) / 2.0) * scale,
    );
    let text_clip = clip.intersection(inner.scaled(scale));
    tk.text_engine.draw_styled_pub(
        pixmap,
        &text,
        (resolved.font_size * scale, fg),
        resolved.font_family.as_deref(),
        resolved.font_weight,
        resolved.italic,
        origin,
        text_clip,
        resolved.opacity,
    );

    // Buttons on the right, matching the integer Spinbox in `render.rs`:
    // resolved.bg background, resolved.border_color outline and separator,
    // resolved.fg arrow glyphs (visible in HighContrast).
    let btn_rect = Rect::new(logical.x + logical.width - arrow_w, logical.y, arrow_w, logical.height).scaled(scale);
    fill(pixmap, btn_rect, clip, resolved.bg);
    outline(pixmap, btn_rect, clip, scale, (resolved.border_color, resolved.border_color));
    let half_h = btn_rect.height / 2.0;
    fill(pixmap, Rect::new(btn_rect.x, btn_rect.y + half_h, btn_rect.width, scale), clip, resolved.border_color);
    let up_text = "▲";
    let dn_text = "▼";
    let up_m = tk.measure(up_text, 8.0);
    let dn_m = tk.measure(dn_text, 8.0);
    tk.text_engine.draw_styled_pub(
        pixmap,
        up_text,
        (8.0 * scale, resolved.fg),
        None,
        400,
        false,
        (btn_rect.x + (btn_rect.width - up_m.width * scale) / 2.0, btn_rect.y + (half_h - up_m.height * scale) / 2.0),
        clip,
        resolved.opacity,
    );
    tk.text_engine.draw_styled_pub(
        pixmap,
        dn_text,
        (8.0 * scale, resolved.fg),
        None,
        400,
        false,
        (btn_rect.x + (btn_rect.width - dn_m.width * scale) / 2.0, btn_rect.y + half_h + (half_h - dn_m.height * scale) / 2.0),
        clip,
        resolved.opacity,
    );
}
