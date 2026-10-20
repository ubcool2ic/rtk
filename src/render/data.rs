//! Table (`DataGrid`) rendering: header, sort indicator, virtualized body rows.
//!
//! Only the rows in `top..top+visible+1` are touched per frame; the model is
//! never iterated in full while drawing. All state needed for a frame is
//! snapshotted into owned locals up front so the immutable borrow of the widget
//! node ends before `tk.text_engine` is borrowed mutably for text drawing.

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
            selected: state.selected,
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
    fn capture(tk: &mut Tk, id: crate::WidgetId, visible: usize) -> Option<Self> {
        let node = tk.node(id)?;
        if node.kind != Kind::TreeView {
            return None;
        }
        let state = node.tree.as_deref()?;
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

    let visible = (logical.height / rh).floor().max(0.0) as usize;
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
