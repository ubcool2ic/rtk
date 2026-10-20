//! Table (`DataGrid`) rendering: header, sort indicator, virtualized body rows.
//!
//! Only the rows in `top..top+visible+1` are touched per frame; the model is
//! never iterated in full while drawing. All state needed for a frame is
//! snapshotted into owned locals up front so the immutable borrow of the widget
//! node ends before `tk.text_engine` is borrowed mutably for text drawing.

/// Snapshot of one menu row for drawing: (label, accelerator, enabled, is_check_item, checked).
type MenuRow = (Option<String>, Option<String>, bool, bool, bool);

use tiny_skia::{Paint, PathBuilder, Pixmap, Transform};

use crate::layout::Rect;
use crate::style::ResolvedStyle;
use crate::widgets::Kind;
use crate::Tk;

/// Header height in logical px for a given body font size.
pub(crate) fn header_height(font_size: f32) -> f32 {
    (font_size * 1.6).round().max(8.0)
}

/// Row height in logical px for a given body font size.
pub(crate) fn row_height(font_size: f32) -> f32 {
    (font_size * 1.6).round().max(8.0)
}

/// Fills a logical rectangle clipped to `clip` (both in device px).
fn fill(pixmap: &mut Pixmap, rect: Rect, clip: Rect, color: crate::Color) {
    let rect = rect.intersection(clip);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    if let Some(r) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = false;
        pixmap.fill_rect(r, &paint, Transform::identity(), None);
    }
}

/// Draws a small filled triangle (sort indicator) with its apex at `(cx, cy)`.
fn triangle(pixmap: &mut Pixmap, cx: f32, cy: f32, half: f32, up: bool, color: crate::Color) {
    let mut pb = PathBuilder::new();
    if up {
        pb.move_to(cx, cy - half);
        pb.line_to(cx + half, cy + half);
        pb.line_to(cx - half, cy + half);
    } else {
        pb.move_to(cx, cy + half);
        pb.line_to(cx + half, cy - half);
        pb.line_to(cx - half, cy - half);
    }
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

/// Owned snapshot of everything `draw_table` needs from the widget node.
///
/// Taking this up front ends the immutable borrow of `tk` so the text engine
/// can be borrowed mutably while drawing. Only the visible window of rows is
/// copied, so large tables stay cheap.
struct TableSnapshot {
    /// Column titles.
    columns: Vec<String>,
    /// Column widths in logical px.
    widths: Vec<f32>,
    /// Active sort: `(column, ascending)`.
    sort: Option<(usize, bool)>,
    /// Selected display row.
    selected: Option<usize>,
    /// Display row shown at the top of the body.
    top: usize,
    /// Total number of model rows.
    rows_len: usize,
    /// Visible rows as `(display_index, cells)` for `top..top+visible+1`.
    visible_rows: Vec<(usize, Vec<String>)>,
}

impl TableSnapshot {
    /// Builds a snapshot from the table state of `id`, or `None` if `id` is not a Table.
    fn capture(tk: &Tk, id: crate::WidgetId, visible: usize) -> Option<Self> {
        let node = tk.node(id)?;
        if node.kind != Kind::Table {
            return None;
        }
        let state = node.table.as_deref()?;
        let top = state.top;
        let rows_len = state.rows.len();
        let end = (top + visible + 1).min(rows_len);
        let mut visible_rows = Vec::with_capacity(end.saturating_sub(top));
        for display in top..end {
            let Some(&model) = state.order.get(display) else { continue };
            let Some(row) = state.rows.get(model) else { continue };
            visible_rows.push((display, row.clone()));
        }
        Some(Self {
            columns: state.columns.clone(),
            widths: state.widths.clone(),
            sort: state.sort,
            selected: state.selected.and_then(|m| state.display_of(m)),
            top,
            rows_len,
            visible_rows,
        })
    }
}

/// Owned snapshot of everything `draw_tree` needs from the widget node.
///
/// Taking this up front ends the immutable borrow of `tk` so the text engine
/// can be borrowed mutably while drawing. Only the visible window of rows is
/// copied, so large trees stay cheap.
struct TreeSnapshot {
    /// Selected node key.
    selected: Option<usize>,
    /// Display row shown at the top of the body.
    top: usize,
    /// Visible rows as `(key, depth, text, has_children, open)` for the window.
    visible_rows: Vec<(usize, usize, String, bool, bool)>,
}

impl TreeSnapshot {
    /// Builds a snapshot from the tree state of `id`, or `None` if `id` is not a TreeView.
    ///
    /// The flattened visible list is rebuilt first when it is stale or empty
    /// while nodes exist, so a tree that was populated while its page was
    /// hidden still renders its rows on the first visible frame.
    fn capture(tk: &mut Tk, id: crate::WidgetId, visible: usize) -> Option<Self> {
        {
            let node = tk.node(id)?;
            if node.kind != Kind::TreeView {
                return None;
            }
            node.tree.as_deref()?;
        }
        let state = tk.tree_state_mut(id)?;
        state.ensure_visible_robust();
        let top = state.top;
        let end = (top + visible + 1).min(state.visible.len());
        let mut visible_rows = Vec::with_capacity(end.saturating_sub(top));
        for display in top..end {
            let Some(&(key, depth)) = state.visible.get(display) else { continue };
            let Some(n) = state.get(key) else { continue };
            visible_rows.push((key, depth, n.text.clone(), !n.children.is_empty(), n.open));
        }
        Some(Self {
            selected: state.selected,
            top,
            visible_rows,
        })
    }
}

/// Draws the toast stack as an overlay (before menus, after the tooltip).
pub(super) fn draw_toast_overlay(tk: &mut Tk, pixmap: &mut Pixmap, scale: f32) {
    if tk.toasts.list.is_empty() {
        return;
    }
    let rects = tk.toast_layout();
    let style = tk.resolved_style(tk.root()).unwrap_or_default();
    let palette = tk.style_engine.tokens().palette;
    let full_clip = Rect::new(0.0, 0.0, pixmap.width() as f32 / scale, pixmap.height() as f32 / scale);
    let font_size = style.font_size;
    let radius = style.radius.max(6.0);

    // Owned snapshot so the immutable borrow of `tk.toasts` ends before drawing.
    let rows: Vec<(Rect, String, crate::tk::ToastKind, f32)> = tk
        .toasts
        .list
        .iter()
        .zip(rects.iter())
        .map(|(t, r)| (*r, t.text.clone(), t.kind, t.alpha()))
        .collect();

    for (rect, text, kind, alpha) in rows {
        if alpha <= 0.0 {
            continue;
        }
        let scaled = rect.scaled(scale);
        let clip = full_clip.intersection(scaled);
        if clip.width <= 0.0 || clip.height <= 0.0 {
            continue;
        }
        let bg = palette.surface2;
        let bg_alpha = (bg.a as f32 * alpha).round().clamp(0.0, 255.0) as u8;
        if let Some(path) = crate::style::build_rounded_rect_path(scaled, radius * scale) {
            let mut paint = Paint::default();
            paint.set_color_rgba8(bg.r, bg.g, bg.b, bg_alpha);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            let border = palette.border;
            let border_alpha = (border.a as f32 * alpha).round().clamp(0.0, 255.0) as u8;
            if border_alpha > 0 {
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color_rgba8(border.r, border.g, border.b, border_alpha);
                stroke_paint.anti_alias = true;
                let stroke = tiny_skia::Stroke { width: scale, ..tiny_skia::Stroke::default() };
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }

        // Leading kind icon.
        let icon_color = crate::tk::toast::kind_color(tk, kind);
        let icon_alpha = (icon_color.a as f32 * alpha).round().clamp(0.0, 255.0) as u8;
        let cx = (rect.x + crate::tk::toast::TOAST_PAD_X + crate::tk::toast::TOAST_ICON_W / 2.0) * scale;
        let cy = (rect.y + rect.height / 2.0) * scale;
        let r = 6.0 * scale;
        let mut paint = Paint::default();
        paint.set_color_rgba8(icon_color.r, icon_color.g, icon_color.b, icon_alpha);
        paint.anti_alias = true;
        match kind {
            crate::tk::ToastKind::Info => {
                if let Some(path) = PathBuilder::from_circle(cx, cy, r) {
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
            crate::tk::ToastKind::Success => {
                let mut pb = PathBuilder::new();
                pb.move_to(cx - r * 0.8, cy);
                pb.line_to(cx - r * 0.2, cy + r * 0.6);
                pb.line_to(cx + r * 0.9, cy - r * 0.7);
                if let Some(path) = pb.finish() {
                    let stroke = tiny_skia::Stroke {
                        width: 2.0 * scale,
                        line_cap: tiny_skia::LineCap::Round,
                        line_join: tiny_skia::LineJoin::Round,
                        ..tiny_skia::Stroke::default()
                    };
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            crate::tk::ToastKind::Warning => {
                let mut pb = PathBuilder::new();
                pb.move_to(cx, cy - r);
                pb.line_to(cx + r, cy + r * 0.8);
                pb.line_to(cx - r, cy + r * 0.8);
                pb.close();
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
            crate::tk::ToastKind::Error => {
                let mut pb = PathBuilder::new();
                pb.move_to(cx - r * 0.7, cy - r * 0.7);
                pb.line_to(cx + r * 0.7, cy + r * 0.7);
                pb.move_to(cx + r * 0.7, cy - r * 0.7);
                pb.line_to(cx - r * 0.7, cy + r * 0.7);
                if let Some(path) = pb.finish() {
                    let stroke = tiny_skia::Stroke {
                        width: 2.0 * scale,
                        line_cap: tiny_skia::LineCap::Round,
                        ..tiny_skia::Stroke::default()
                    };
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
        }

        // Message text.
        let fg = palette.fg;
        let text_x = rect.x + crate::tk::toast::TOAST_PAD_X + crate::tk::toast::TOAST_ICON_W + 8.0;
        let origin = (text_x * scale, (rect.y + (rect.height - font_size * 1.3) / 2.0) * scale);
        let text_clip = clip.intersection(
            Rect::new(text_x, rect.y, (rect.x + rect.width - crate::tk::toast::TOAST_PAD_X - text_x).max(0.0), rect.height)
                .scaled(scale),
        );
        tk.text_engine.draw_styled(
            pixmap,
            &text,
            (font_size * scale, fg),
            style.font_family.as_deref(),
            400,
            style.italic,
            origin,
            text_clip,
            alpha,
        );
    }
}

/// Draws all open menus (menubar dropdowns, popups, submenus) as an overlay.
pub(super) fn draw_menu_overlay(tk: &mut Tk, pixmap: &mut Pixmap, scale: f32) {
    if tk.menus.open.is_empty() {
        return;
    }
    let levels = tk.menus.open.len();
    let (surface, border, sel_bg, sel_fg, muted) = tk.menu_colors();
    let style = tk.resolved_style(tk.root()).unwrap_or_default();
    let font_size = style.font_size;
    let ih = crate::tk::menu::item_height(font_size);
    let full_clip = Rect::new(0.0, 0.0, pixmap.width() as f32 / scale, pixmap.height() as f32 / scale);

    for level in 0..levels {
        let Some(rect) = tk.menu_panel_rect(level) else { continue };
        let Some(open) = tk.menus.open.get(level).copied() else { continue };
        let menu = open.menu;
        let highlight = open.highlight;
        let items: Vec<MenuRow> = tk
            .menus
            .get(menu)
            .map(|m| {
                m.items
                    .iter()
                    .map(|item| match item {
                        crate::tk::menu::MenuItem::Command { label, accel, enabled } => {
                            (Some(label.clone()), accel.clone(), *enabled, false, false)
                        }
                        crate::tk::menu::MenuItem::Check { label, var } => {
                            let checked = var.get(tk).unwrap_or(false);
                            (Some(label.clone()), None, true, true, checked)
                        }
                        crate::tk::menu::MenuItem::Submenu { label, .. } => {
                            (Some(label.clone()), None, true, false, false)
                        }
                        crate::tk::menu::MenuItem::Separator => (None, None, false, false, false),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Panel background.
        fill(pixmap, rect.scaled(scale), full_clip, surface);
        // 1px border.
        fill(pixmap, Rect::new(rect.x, rect.y, rect.width, 1.0).scaled(scale), full_clip, border);
        fill(pixmap, Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0).scaled(scale), full_clip, border);
        fill(pixmap, Rect::new(rect.x, rect.y, 1.0, rect.height).scaled(scale), full_clip, border);
        fill(pixmap, Rect::new(rect.x + rect.width - 1.0, rect.y, 1.0, rect.height).scaled(scale), full_clip, border);

        for (idx, (label, accel, enabled, is_check, checked)) in items.iter().enumerate() {
            let iy = rect.y + 4.0 + idx as f32 * ih;
            let row = Rect::new(rect.x + 1.0, iy, rect.width - 2.0, ih);
            let Some(label) = label else {
                // Separator: 1px line in the middle.
                let sy = iy + ih / 2.0;
                fill(pixmap, Rect::new(rect.x + 6.0, sy, rect.width - 12.0, 1.0).scaled(scale), full_clip, border);
                continue;
            };
            let is_hl = highlight == Some(idx) && *enabled;
            if is_hl {
                fill(pixmap, row.scaled(scale), full_clip, sel_bg);
            }
            let fg = if !*enabled {
                muted
            } else if is_hl {
                sel_fg
            } else {
                style.fg
            };
            // Check mark for checked items.
            if *is_check && *checked {
                let cx = (rect.x + 10.0) * scale;
                let cy = (iy + ih / 2.0) * scale;
                let mut pb = PathBuilder::new();
                pb.move_to(cx - 4.0 * scale, cy);
                pb.line_to(cx - 1.0 * scale, cy + 3.0 * scale);
                pb.line_to(cx + 4.0 * scale, cy - 3.0 * scale);
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(fg.r, fg.g, fg.b, fg.a);
                    paint.anti_alias = true;
                    let stroke = tiny_skia::Stroke { width: 1.5 * scale, ..tiny_skia::Stroke::default() };
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            let origin = ((rect.x + 24.0) * scale, (iy + (ih - font_size * 1.3) / 2.0) * scale);
            tk.text_engine.draw_styled(
                pixmap,
                label,
                (font_size * scale, fg),
                style.font_family.as_deref(),
                400,
                style.italic,
                origin,
                full_clip.intersection(row.scaled(scale)),
                style.opacity,
            );
            if let Some(accel) = accel {
                let aw = tk.measure_with(accel, &style).width;
                let ax = rect.x + rect.width - 12.0 - aw;
                let aorigin = (ax * scale, (iy + (ih - font_size * 1.3) / 2.0) * scale);
                tk.text_engine.draw_styled(
                    pixmap,
                    accel,
                    (font_size * scale, if is_hl { fg } else { muted }),
                    style.font_family.as_deref(),
                    400,
                    style.italic,
                    aorigin,
                    full_clip.intersection(row.scaled(scale)),
                    style.opacity,
                );
            }
            // Submenu arrow.
            let is_sub = tk
                .menus
                .get(menu)
                .and_then(|m| m.items.get(idx))
                .is_some_and(|i| matches!(i, crate::tk::menu::MenuItem::Submenu { .. }));
            if is_sub {
                let cx = (rect.x + rect.width - 10.0) * scale;
                let cy = (iy + ih / 2.0) * scale;
                let mut pb = PathBuilder::new();
                pb.move_to(cx - 2.0 * scale, cy - 4.0 * scale);
                pb.line_to(cx + 3.0 * scale, cy);
                pb.line_to(cx - 2.0 * scale, cy + 4.0 * scale);
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(fg.r, fg.g, fg.b, fg.a);
                    paint.anti_alias = true;
                    let stroke = tiny_skia::Stroke { width: 1.5 * scale, ..tiny_skia::Stroke::default() };
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
        }
    }
}

/// Draws a small disclosure triangle with its apex at `(cx, cy)`.
fn tree_triangle(pixmap: &mut Pixmap, cx: f32, cy: f32, half: f32, open: bool, color: crate::Color) {
    let mut pb = PathBuilder::new();
    if open {
        pb.move_to(cx - half, cy - half * 0.5);
        pb.line_to(cx + half, cy - half * 0.5);
        pb.line_to(cx, cy + half * 0.8);
    } else {
        pb.move_to(cx - half * 0.5, cy - half);
        pb.line_to(cx + half * 0.8, cy);
        pb.line_to(cx - half * 0.5, cy + half);
    }
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

/// Draws the whole tree view: virtualized rows, disclosure triangles, selection.
pub(crate) fn draw_tree(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    logical_clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let font_size = resolved.font_size;
    let rh = row_height(font_size);

    let rect = logical.scaled(scale);
    let clip = logical_clip.scaled(scale);
    if clip.width <= 0.0 || clip.height <= 0.0 {
        return;
    }

    // When the widget has no laid-out height yet (hidden page), fall back to
    // the requested height so the first visible frame still draws rows.
    let height = if logical.height > 0.0 {
        logical.height
    } else {
        tk.node(id).map(|n| n.requested.height).unwrap_or(0.0)
    };
    let visible = (height / rh).floor().max(0.0) as usize;
    let Some(snap) = TreeSnapshot::capture(tk, id, visible) else { return };

    let palette = tk.style_engine.tokens().palette;
    let opacity = resolved.opacity;

    // Body rows (virtualized; only the snapshot window is drawn).
    for (key, depth, text, has_children, open) in &snap.visible_rows {
        let display = snap.top + snap.visible_rows.iter().position(|r| r.0 == *key).unwrap_or(0);
        let row_y = logical.y + (display - snap.top) as f32 * rh;
        let row_rect = Rect::new(logical.x, row_y, logical.width, rh);
        let row_clip = clip.intersection(row_rect.scaled(scale));
        if row_clip.width <= 0.0 || row_clip.height <= 0.0 {
            continue;
        }
        let is_selected = snap.selected == Some(*key);
        let bg = if is_selected { palette.select_bg } else { palette.bg };
        fill(pixmap, row_rect.scaled(scale), clip, bg);
        let fg = if is_selected { palette.select_fg } else { palette.fg };

        let indent = *depth as f32 * 16.0;
        if *has_children {
            let cx = (logical.x + indent + 8.0) * scale;
            let cy = (row_y + rh / 2.0) * scale;
            tree_triangle(pixmap, cx, cy, 4.0 * scale, *open, fg);
        }
        let text_x = logical.x + indent + 16.0;
        let origin = (
            (text_x + 4.0) * scale,
            (row_y + (rh - font_size * 1.3) / 2.0) * scale,
        );
        tk.text_engine.draw_styled(
            pixmap,
            text,
            (font_size * scale, fg),
            resolved.font_family.as_deref(),
            400,
            resolved.italic,
            origin,
            row_clip,
            opacity,
        );
    }

    // Outer border.
    let border = palette.border;
    fill(pixmap, Rect::new(rect.x, rect.y, rect.width, scale).intersection(clip), clip, border);
    fill(
        pixmap,
        Rect::new(rect.x, rect.y + rect.height - scale, rect.width, scale).intersection(clip),
        clip,
        border,
    );
    fill(pixmap, Rect::new(rect.x, rect.y, scale, rect.height).intersection(clip), clip, border);
    fill(
        pixmap,
        Rect::new(rect.x + rect.width - scale, rect.y, scale, rect.height).intersection(clip),
        clip,
        border,
    );
}

/// Draws the whole table: header strip, sort indicator, separators and the
/// virtualized body rows (alternating stripes, selection highlight, cell text).
pub(crate) fn draw_table(
    tk: &mut Tk,
    pixmap: &mut Pixmap,
    id: crate::WidgetId,
    logical: Rect,
    logical_clip: Rect,
    scale: f32,
    resolved: &ResolvedStyle,
) {
    let font_size = resolved.font_size;
    let hh = header_height(font_size);
    let rh = row_height(font_size);

    let rect = logical.scaled(scale);
    let clip = logical_clip.scaled(scale);
    if clip.width <= 0.0 || clip.height <= 0.0 {
        return;
    }

    // Number of fully visible body rows, then snapshot only that window.
    let body_h = (logical.height - hh).max(0.0);
    let visible = (body_h / rh).floor().max(0.0) as usize;
    let Some(snap) = TableSnapshot::capture(tk, id, visible) else { return };
    let _ = snap.rows_len;

    let palette = tk.style_engine.tokens().palette;
    let opacity = resolved.opacity;

    // Header strip.
    let header_rect = Rect::new(logical.x, logical.y, logical.width, hh);
    fill(pixmap, header_rect.scaled(scale), clip, palette.surface2);

    // Column titles + separators + sort indicator.
    let mut cur_x = logical.x;
    for (col, title) in snap.columns.iter().enumerate() {
        let w = snap.widths.get(col).copied().unwrap_or(120.0);
        let cell = Rect::new(cur_x, logical.y, w, hh);
        let cell_clip = clip.intersection(cell.scaled(scale));
        if cell_clip.width > 0.0 {
            let origin = (
                (cur_x + 6.0) * scale,
                (logical.y + (hh - font_size * 1.3) / 2.0) * scale,
            );
            tk.text_engine.draw_styled(
                pixmap,
                title,
                (font_size * scale, palette.fg),
                resolved.font_family.as_deref(),
                700,
                resolved.italic,
                origin,
                cell_clip,
                opacity,
            );
            if let Some((sort_col, asc)) = snap.sort {
                if sort_col == col {
                    let cx = (cur_x + w - 10.0) * scale;
                    let cy = (logical.y + hh / 2.0) * scale;
                    triangle(pixmap, cx, cy, 4.0 * scale, asc, palette.fg);
                }
            }
        }
        // 1px separator at the right edge of the column.
        let sep = Rect::new(cur_x + w - 1.0, logical.y, 1.0, logical.height);
        fill(pixmap, sep.scaled(scale), clip, palette.border);
        cur_x += w;
    }
    // Header bottom border.
    fill(
        pixmap,
        Rect::new(logical.x, logical.y + hh - 1.0, logical.width, 1.0).scaled(scale),
        clip,
        palette.border,
    );

    // Body rows (virtualized; only the snapshot window is drawn).
    let body_top = logical.y + hh;
    for (display, cells) in &snap.visible_rows {
        let display = *display;
        let row_y = body_top + display.saturating_sub(snap.top) as f32 * rh;
        let row_rect = Rect::new(logical.x, row_y, logical.width, rh);
        let row_clip = clip.intersection(row_rect.scaled(scale));
        if row_clip.width <= 0.0 || row_clip.height <= 0.0 {
            continue;
        }
        let is_selected = snap.selected == Some(display);
        let stripe = if display % 2 == 0 { palette.surface } else { palette.bg };
        let bg = if is_selected { palette.select_bg } else { stripe };
        fill(pixmap, row_rect.scaled(scale), clip, bg);
        let fg = if is_selected { palette.select_fg } else { palette.fg };

        let mut cx = logical.x;
        for (col, w) in snap.widths.iter().enumerate() {
            let cell = Rect::new(cx, row_y, *w, rh);
            let cell_clip = clip.intersection(cell.scaled(scale));
            if cell_clip.width > 0.0 {
                if let Some(text) = cells.get(col) {
                    let origin = (
                        (cx + 6.0) * scale,
                        (row_y + (rh - font_size * 1.3) / 2.0) * scale,
                    );
                    tk.text_engine.draw_styled(
                        pixmap,
                        text,
                        (font_size * scale, fg),
                        resolved.font_family.as_deref(),
                        400,
                        resolved.italic,
                        origin,
                        cell_clip,
                        opacity,
                    );
                }
            }
            cx += *w;
        }
    }

    // Outer border.
    let border = palette.border;
    fill(pixmap, Rect::new(rect.x, rect.y, rect.width, scale).intersection(clip), clip, border);
    fill(
        pixmap,
        Rect::new(rect.x, rect.y + rect.height - scale, rect.width, scale).intersection(clip),
        clip,
        border,
    );
    fill(pixmap, Rect::new(rect.x, rect.y, scale, rect.height).intersection(clip), clip, border);
    fill(
        pixmap,
        Rect::new(rect.x + rect.width - scale, rect.y, scale, rect.height).intersection(clip),
        clip,
        border,
    );
}
