//! Geometry managers, measurement and layout driver.

#[allow(unused_imports)]
use super::*;

impl Tk {
    pub(crate) fn set_manager(&mut self, id: WidgetId, manager: Manager) -> bool {
        let Some(parent) = self.node(id).and_then(|node| node.parent) else { return false };
        let Some(parent_node) = self.node(parent) else { return false };
        match manager {
            Manager::Pack(_) => {
                if parent_node.children.iter().copied().filter(|sibling| *sibling != id).any(|sibling| {
                    self.node(sibling).and_then(|n| n.manager).is_some_and(|m| matches!(m, Manager::Grid(_)))
                }) {
                    return false;
                }
            }
            Manager::Grid(_) => {
                if parent_node.children.iter().copied().filter(|sibling| *sibling != id).any(|sibling| {
                    self.node(sibling).and_then(|n| n.manager).is_some_and(|m| matches!(m, Manager::Pack(_)))
                }) {
                    return false;
                }
            }
            Manager::Place(_) => {}
        }
        if let Some(node) = self.node_mut(id) {
            node.manager = Some(manager);
        }
        self.dirty = true;
        true
    }

    /// Manages `id` with the pack geometry manager. Fails if a sibling uses grid.
    pub fn pack(&mut self, id: WidgetId, options: PackOpts) -> bool {
        self.set_manager(id, Manager::Pack(options))
    }

    /// Manages `id` with the grid geometry manager. Fails if a sibling uses pack.
    pub fn grid(&mut self, id: WidgetId, options: GridOpts) -> bool {
        self.set_manager(id, Manager::Grid(options))
    }

    /// Manages `id` with the place geometry manager (absolute/relative coordinates).
    pub fn place(&mut self, id: WidgetId, options: PlaceOpts) -> bool {
        self.set_manager(id, Manager::Place(options))
    }

    /// Sets the stretch weight of grid column `column` in Frame `parent`.
    pub fn grid_columnconfigure(&mut self, parent: WidgetId, column: usize, weight: u32) -> bool {
        self.configure_track(parent, column, weight, true)
    }

    /// Sets the stretch weight of grid row `row` in Frame `parent`.
    pub fn grid_rowconfigure(&mut self, parent: WidgetId, row: usize, weight: u32) -> bool {
        self.configure_track(parent, row, weight, false)
    }

    pub(crate) fn configure_track(&mut self, parent: WidgetId, index: usize, weight: u32, column: bool) -> bool {
        let Some(node) = self.node_mut(parent) else { return false };
        if node.kind != Kind::Frame || index >= layout::MAX_TRACKS {
            return false;
        }
        if column {
            node.columns.insert(index, weight);
        } else {
            node.rows.insert(index, weight);
        }
        self.dirty = true;
        true
    }

    pub(crate) fn managed_children(&self, id: WidgetId) -> Vec<WidgetId> {
        self.node(id).map(|node| {
            if node.kind == Kind::Notebook {
                return Vec::new();
            }
            node.children.iter().copied()
                .filter(|child| self.node(*child).is_some_and(|n| n.manager.is_some()))
                .collect()
        }).unwrap_or_default()
    }

    pub(crate) fn pack_items(&self, children: &[WidgetId]) -> Vec<PackItem> {
        children.iter().filter_map(|id| {
            let node = self.node(*id)?;
            match node.manager {
                Some(Manager::Pack(opts)) => Some(PackItem { requested: node.requested, opts }),
                _ => None,
            }
        }).collect()
    }

    pub(crate) fn grid_items(&self, children: &[WidgetId]) -> Vec<GridItem> {
        children.iter().filter_map(|id| {
            let node = self.node(*id)?;
            match node.manager {
                Some(Manager::Grid(opts)) => Some(GridItem { requested: node.requested, opts }),
                _ => None,
            }
        }).collect()
    }

    pub(crate) fn uses_pack(&self, children: &[WidgetId]) -> bool {
        children.iter().filter_map(|id| self.node(*id)).any(|n| matches!(n.manager, Some(Manager::Pack(_))))
    }

    /// Logical-pixel text size using the resolved font family, weight and slant.
    #[doc(hidden)]
    pub fn measure_with(&mut self, text: &str, style: &ResolvedStyle) -> Size {
        let scale = self.scale_factor;
        let size = self.text_engine.measure_styled(
            text,
            style.font_size * scale,
            style.font_family.as_deref(),
            style.font_weight,
            style.italic,
        );
        Size::new(size.width / scale, size.height / scale)
    }

    pub(crate) fn measure(&mut self, text: &str, font_size: f32) -> Size {
        let size = self.text_engine.measure(text, font_size * self.scale_factor);
        Size::new(size.width / self.scale_factor, size.height / self.scale_factor)
    }

    pub(crate) fn update_requested(&mut self) {
        let mut order = Vec::new();
        let mut pending = vec![self.root()];
        while let Some(id) = pending.pop() {
            if let Some(node) = self.node(id) {
                order.push(id);
                if node.kind == Kind::Notebook {
                    // Pages are not geometry-managed; they must still be measured
                    // (before the notebook itself, i.e. later in `order`).
                    pending.extend(node.notebook_pages.iter().map(|(pid, _)| *pid));
                } else {
                    pending.extend(self.managed_children(id));
                }
            }
        }
        for id in order.into_iter().rev() {
            let Some(node) = self.node(id) else { continue };
            let resolved = self.resolved_style(id).unwrap_or_default();
            let node_snap = NodeSnapshot {
                scale: node.scale,
                range: node.range,
                canvas_size: node.canvas_size,
                progress_length: node.progress_length,
                list_height: node.list_height,
                parent: node.parent,
                kind: node.kind,
                style: node.style,
                image: node.image,
            };
            let (kind, nested) = (node_snap.kind, node_snap.parent.is_some());
            let children = self.managed_children(id);
            let char_width = self.measure_with("0", &resolved).width.max(resolved.font_size * 0.5);
            let specified = node_snap.style.width.map(|chars| chars as f32 * char_width);

            let pad_w = resolved.padding.left + resolved.padding.right;
            let pad_h = resolved.padding.top + resolved.padding.bottom;
            let mar_w = resolved.margin.left + resolved.margin.right;
            let mar_h = resolved.margin.top + resolved.margin.bottom;

            let image_extra_w = if let Some(img_id) = node_snap.image {
                self.images.get(&img_id).map(|p| p.width() as f32 + 8.0).unwrap_or(0.0)
            } else {
                0.0
            };

            let requested = match kind {
                Kind::Frame => {
                    let size = if self.uses_pack(&children) {
                        layout::pack_request(&self.pack_items(&children))
                    } else {
                        layout::grid_request(&self.grid_items(&children))
                    };
                    let border = if nested { 4.0 } else { 0.0 };
                    Size::new(
                        (size.width + border + pad_w).max(specified.unwrap_or(0.0)),
                        size.height + border + pad_h,
                    )
                }
                Kind::Scale => Size::new(specified.unwrap_or(160.0) + pad_w, 36.0 + pad_h),
                Kind::Entry => Size::new(specified.unwrap_or(20.0 * char_width) + 12.0 + pad_w, resolved.font_size * 1.3 + 12.0 + pad_h),
                Kind::Listbox => {
                    let line_h = (resolved.font_size * 1.3 + 4.0).max(16.0);
                    // Like tkinter (width in characters), do not measure every row of huge lists.
                    // Temporarily take the items out instead of cloning the whole list every layout.
                    let items = self.node_mut(id).map(|n| std::mem::take(&mut n.list_items)).unwrap_or_default();
                    let max_w = items.iter().take(256).map(|item| self.measure_with(item, &resolved).width).fold(0.0_f32, f32::max);
                    if let Some(n) = self.node_mut(id) {
                        n.list_items = items;
                    }
                    Size::new(specified.unwrap_or(max_w + 24.0).max(80.0) + pad_w, (node_snap.list_height as f32 * line_h).max(30.0) + pad_h)
                }
                Kind::Scrollbar => Size::new(16.0 + pad_w, 100.0 + pad_h),
                Kind::Radiobutton => {
                    let text = self.node(id).map(|n| n.text.clone()).unwrap_or_default();
                    let measured = self.measure_with(&text, &resolved);
                    Size::new(specified.unwrap_or(measured.width) + 32.0 + pad_w + image_extra_w, measured.height.max(16.0) + 8.0 + pad_h)
                }
                Kind::Canvas => Size::new(node_snap.canvas_size.0 + pad_w, node_snap.canvas_size.1 + pad_h),
                Kind::Spinbox => {
                    let text = format!("{}", node_snap.scale.and_then(|v| v.get(self)).unwrap_or(node_snap.range.0));
                    let m = self.measure_with(&text, &resolved);
                    Size::new(specified.unwrap_or(m.width + 50.0).max(70.0) + pad_w, resolved.font_size * 1.3 + 12.0 + pad_h)
                }
                Kind::ProgressBar => Size::new(node_snap.progress_length + pad_w, 20.0 + pad_h),
                Kind::Text => {
                    let (cols, rows) = self.node(id).map(|n| n.text_dimensions).unwrap_or((40, 10));
                    let line_h = (resolved.font_size * 1.3 + 4.0).max(16.0);
                    Size::new(
                        specified.unwrap_or(cols as f32 * char_width + 16.0) + pad_w,
                        (rows as f32 * line_h + 12.0) + pad_h,
                    )
                }
                Kind::Combobox => {
                    let cur_str = self.combobox_get(id).map(|(_, s)| s).unwrap_or_default();
                    let m = self.measure_with(&cur_str, &resolved);
                    Size::new(specified.unwrap_or(m.width + 48.0).max(80.0) + pad_w, resolved.font_size * 1.3 + 12.0 + pad_h)
                }
                Kind::Table => {
                    let row_h = (resolved.font_size * 1.6).round().max(8.0);
                    let (width_sum, height_rows) = self
                        .node(id)
                        .and_then(|n| n.table.as_deref().map(|t| (t.widths.iter().sum::<f32>(), n.table_height_rows)))
                        .unwrap_or((0.0, 5));
                    Size::new(
                        specified.unwrap_or(width_sum.max(120.0)) + pad_w,
                        row_h + height_rows as f32 * row_h + 2.0 + pad_h,
                    )
                }
                Kind::TreeView => {
                    let row_h = (resolved.font_size * 1.6).round().max(8.0);
                    let height_rows = self.node(id).map(|n| n.tree_height_rows).unwrap_or(5);
                    Size::new(
                        specified.unwrap_or(200.0) + pad_w,
                        height_rows as f32 * row_h + 2.0 + pad_h,
                    )
                }
                Kind::Menubar => {
                    let titles = self.node(id).map(|n| n.menubar_titles.clone()).unwrap_or_default();
                    let mut w = 0.0_f32;
                    for (title, _) in &titles {
                        w += self.measure_with(title, &resolved).width + 24.0;
                    }
                    Size::new(w.max(specified.unwrap_or(0.0)) + pad_w, resolved.font_size * 1.8 + 8.0 + pad_h)
                }
                Kind::Notebook => {
                    let mut tab_w_sum = 0.0_f32;
                    let mut max_w = 0.0_f32;
                    let mut max_h = 0.0_f32;
                    let pages = self.node(id).map(|nb| nb.notebook_pages.clone()).unwrap_or_default();
                    {
                        for (pid, title) in &pages {
                            let tm = self.measure_with(title, &resolved);
                            tab_w_sum += tm.width + 24.0;
                            if let Some(pn) = self.node(*pid) {
                                max_w = max_w.max(pn.requested.width);
                                max_h = max_h.max(pn.requested.height);
                            }
                        }
                    }
                    Size::new(tab_w_sum.max(max_w) + pad_w, (32.0 + max_h) + pad_h)
                }
                _ => {
                    let text = self.widget_text(id).unwrap_or_default();
                    let measured = self.measure_with(&text, &resolved);
                    let padding = match kind {
                        Kind::Label => 12.0,
                        Kind::Checkbutton => 40.0,
                        _ => 20.0,
                    };
                    Size::new(
                        specified.unwrap_or(measured.width) + padding + pad_w + image_extra_w,
                        measured.height + 12.0 + pad_h,
                    )
                }
            };
            if let Some(node) = self.node_mut(id) {
                node.requested = Size::new(requested.width + mar_w, requested.height + mar_h);
            }
        }
    }

    pub(crate) fn initial_size(&mut self) -> (u32, u32) {
        if let Some(size) = self.window_size {
            return size;
        }
        self.update_requested();
        let size = self.node(self.root()).map(|node| node.requested).unwrap_or_default();
        (size.width.ceil().max(200.0) as u32, size.height.ceil().max(100.0) as u32)
    }

    pub(crate) fn layout(&mut self) {
        self.update_requested();
        for slot in &mut self.nodes {
            if let Some(node) = &mut slot.node {
                node.visible = false;
            }
        }
        self.paint_order.clear();
        let rect = Rect::new(
            0.0, 0.0,
            self.viewport.0 as f32 / self.scale_factor,
            self.viewport.1 as f32 / self.scale_factor,
        );
        let mut pending = vec![(self.root(), rect, rect)];
        while let Some((id, rect, parent_clip)) = pending.pop() {
            let Some(node) = self.node_mut(id) else { continue };
            let clip = rect.within(parent_clip);
            node.rect = rect;
            node.clip = clip;
            node.visible = true;
            let content = if node.parent.is_some() { rect.inset(2.0, 2.0) } else { rect };
            self.paint_order.push(id);
            let Some(node) = self.node(id) else { continue };

            if node.kind == Kind::Notebook {
                let sel = node.notebook_selected;
                if let Some(&(page_id, _)) = node.notebook_pages.get(sel) {
                    let page_area = Rect::new(rect.x, rect.y + 32.0, rect.width, (rect.height - 32.0).max(0.0));
                    pending.push((page_id, page_area, page_area.within(clip)));
                }
                continue;
            }

            let children = self.managed_children(id);
            let (mut pack_children, mut grid_children, mut place_children) = (Vec::new(), Vec::new(), Vec::new());
            for child in children {
                match self.node(child).and_then(|n| n.manager) {
                    Some(Manager::Pack(_)) => pack_children.push(child),
                    Some(Manager::Grid(_)) => grid_children.push(child),
                    Some(Manager::Place(_)) => place_children.push(child),
                    None => {}
                }
            }

            if !pack_children.is_empty() {
                let rects = layout::pack(content, &self.pack_items(&pack_children));
                for (child, child_rect) in pack_children.into_iter().zip(rects).rev() {
                    pending.push((child, child_rect, content.within(clip)));
                }
            } else if !grid_children.is_empty() {
                let (columns, rows): (Vec<_>, Vec<_>) = self.node(id).map(|node| {
                    (
                        node.columns.iter().map(|(&i, &w)| (i, w)).collect(),
                        node.rows.iter().map(|(&i, &w)| (i, w)).collect(),
                    )
                }).unwrap_or_default();
                let rects = layout::grid(content, &self.grid_items(&grid_children), &columns, &rows);
                for (child, child_rect) in grid_children.into_iter().zip(rects).rev() {
                    pending.push((child, child_rect, content.within(clip)));
                }
            }
            for child in place_children.into_iter().rev() {
                let Some(cnode) = self.node(child) else { continue };
                if let Some(Manager::Place(opts)) = cnode.manager {
                    let prect = layout::place_rect(content, cnode.requested, opts);
                    pending.push((child, prect, content.within(clip)));
                }
            }
        }
        if !self.focused.is_some_and(|id| self.is_focusable(id)) {
            self.focused = None;
        }
    }

    pub(crate) fn ensure_layout(&mut self) {
        if self.dirty {
            self.layout();
        }
    }

    pub(crate) fn is_focusable(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            !node.disabled && node.visible && (node.kind.focusable() || (node.kind == Kind::Canvas && node.scrollregion.is_some())) && node.clip.width > 0.0 && node.clip.height > 0.0
        })
    }

    /// Laid-out rectangle in logical px of a mapped widget (runs layout if dirty).
    pub fn widget_rect(&mut self, id: WidgetId) -> Option<Rect> {
        self.ensure_layout();
        self.node(id).filter(|node| node.visible).map(|node| node.rect)
    }

    /// Widget holding keyboard focus (tkinter `focus_get`).
    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.focused.filter(|id| self.node(*id).is_some())
    }

    /// Headless harness: sets the viewport in physical px and re-runs layout.
    #[doc(hidden)]
    pub fn sim_resize(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
        self.dirty = true;
        self.layout();
    }


    pub fn place_forget(&mut self, id: WidgetId) -> bool {
        self.pack_forget(id)
    }

    pub fn is_mapped(&mut self, id: WidgetId) -> bool {
        self.ensure_layout();
        self.node(id).is_some_and(|n| n.visible)
    }

}
