use std::sync::Arc;

use crate::layout::{self, GridItem, GridOpts, PackItem, PackOpts, Rect, Size};
use crate::render::TextEngine;
use crate::var::VarSlot;
use crate::widgets::{Binding, Kind, Manager, WidgetNode, WidgetSlot};
use crate::{Opt, RtkError, WidgetId};

const MAX_ENTRY_BYTES: usize = 64 * 1024;

/// Keys understood by the real backend and the headless event harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Tab,
    ShiftTab,
    Enter,
    Space,
}

/// Owns widgets, variables, callbacks, layout, and the software renderer.
///
/// Constructors return `Result`, mutations taking handles return `bool`, and
/// queries return `Option`. False/None means invalid, stale, or incompatible.
/// Widget handles must only be used with their originating toolkit.
pub struct Tk {
    nodes: Vec<WidgetSlot>,
    free: Vec<u32>,
    pub(crate) vars: Vec<VarSlot>,
    pub(crate) var_owner: Arc<()>,
    pub(crate) text_engine: TextEngine,
    pub(crate) dirty: bool,
    pub(crate) window_title: String,
    window_size: Option<(u32, u32)>,
    pub(crate) quit_requested: bool,
    pub(crate) focused: Option<WidgetId>,
    pub(crate) hovered: Option<WidgetId>,
    pub(crate) pressed: Option<WidgetId>,
    pub(crate) paint_order: Vec<WidgetId>,
    viewport: (u32, u32),
    pointer: (f32, f32),
    pub(crate) scale_factor: f32,
}

impl Default for Tk {
    fn default() -> Self {
        Self::new()
    }
}

impl Tk {
    pub fn new() -> Self {
        Self {
            nodes: vec![WidgetSlot {
                gen: 0,
                node: Some(WidgetNode::new(Kind::Frame, None, String::new())),
            }],
            free: Vec::new(),
            vars: Vec::new(),
            var_owner: Arc::new(()),
            text_engine: TextEngine::new(),
            dirty: true,
            window_title: "rtk".into(),
            window_size: None,
            quit_requested: false,
            focused: None,
            hovered: None,
            pressed: None,
            paint_order: Vec::new(),
            viewport: (640, 480),
            pointer: (0.0, 0.0),
            scale_factor: 1.0,
        }
    }

    pub fn root(&self) -> WidgetId {
        WidgetId { index: 0, gen: 0 }
    }

    pub(crate) fn node(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.nodes.get(id.index as usize)
            .filter(|slot| slot.gen == id.gen)?
            .node.as_ref()
    }

    pub(crate) fn node_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode> {
        self.nodes.get_mut(id.index as usize)
            .filter(|slot| slot.gen == id.gen)?
            .node.as_mut()
    }

    pub fn contains(&self, id: WidgetId) -> bool {
        self.node(id).is_some()
    }

    pub fn title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
        self.dirty = true;
    }

    /// Initial window dimensions in logical pixels; zero is permitted.
    pub fn geometry(&mut self, width: u32, height: u32) {
        self.window_size = Some((width, height));
        self.sim_resize(width, height);
    }

    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    pub fn is_quit_requested(&self) -> bool {
        self.quit_requested
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn add(
        &mut self,
        kind: Kind,
        parent: WidgetId,
        text: String,
    ) -> Result<WidgetId, RtkError> {
        if !self.node(parent).is_some_and(|node| node.kind == Kind::Frame) {
            return Err(RtkError::InvalidParent);
        }
        let node = WidgetNode::new(kind, Some(parent), text);
        let id = if let Some(index) = self.free.pop() {
            let Some(slot) = self.nodes.get_mut(index as usize) else {
                return Err(RtkError::ArenaFull);
            };
            let id = WidgetId { index, gen: slot.gen };
            slot.node = Some(node);
            id
        } else {
            let index = u32::try_from(self.nodes.len()).map_err(|_| RtkError::ArenaFull)?;
            self.nodes.push(WidgetSlot { gen: 0, node: Some(node) });
            WidgetId { index, gen: 0 }
        };
        if let Some(parent) = self.node_mut(parent) {
            parent.children.push(id);
        }
        self.dirty = true;
        Ok(id)
    }

    /// Removes a subtree iteratively. Destroying the root also requests quit.
    /// Slots whose generation is exhausted are retired rather than wrapped.
    pub fn destroy(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        let parent = node.parent;
        if let Some(parent) = parent.and_then(|parent| self.node_mut(parent)) {
            parent.children.retain(|child| *child != id);
        }
        let mut pending = vec![id];
        while let Some(current) = pending.pop() {
            let Some(slot) = self.nodes.get_mut(current.index as usize) else { continue };
            if slot.gen != current.gen {
                continue;
            }
            let Some(node) = slot.node.take() else { continue };
            pending.extend(node.children);
            if let Some(next) = slot.gen.checked_add(1) {
                slot.gen = next;
                if current.index != 0 {
                    self.free.push(current.index);
                }
            }
            if self.focused == Some(current) {
                self.focused = None;
            }
            if self.hovered == Some(current) {
                self.hovered = None;
            }
            if self.pressed == Some(current) {
                self.pressed = None;
            }
        }
        if id == self.root() {
            self.quit();
        }
        self.dirty = true;
        true
    }

    pub fn config(&mut self, id: WidgetId, option: Opt) -> bool {
        let Some(kind) = self.node(id).map(|node| node.kind) else { return false };
        match option {
            Opt::Text(text) if kind == Kind::Entry => return self.entry_set(id, text),
            Opt::Text(text) => {
                if let Some(node) = self.node_mut(id) {
                    node.text = text;
                }
            }
            Opt::Bg(color) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.bg = color;
                }
            }
            Opt::Fg(color) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.fg = color;
                }
            }
            Opt::Width(chars) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.width = Some(chars.min(100_000));
                }
            }
            Opt::FontSize(size) => {
                if !size.is_finite() || !(1.0..=512.0).contains(&size) {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    node.style.font_size = size;
                }
            }
            Opt::TextVariable(var) => {
                let Some(text) = var.get(self) else { return false };
                if let Some(node) = self.node_mut(id) {
                    node.binding = Some(Binding::Text(var));
                    node.cursor = clean_entry(&text).len();
                }
            }
            Opt::TextVariableInt(var) => {
                if kind == Kind::Entry || var.get(self).is_none() {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    node.binding = Some(Binding::Int(var));
                }
            }
            Opt::Command(command) => {
                if !kind.has_command() {
                    return false;
                }
                if let Some(node) = self.node_mut(id) {
                    node.command = Some(command);
                }
            }
        }
        self.dirty = true;
        true
    }

    pub fn set_command(&mut self, id: WidgetId, command: impl FnMut(&mut Tk) + 'static) -> bool {
        self.config(id, Opt::Command(Box::new(command)))
    }

    pub fn entry_get(&self, id: WidgetId) -> Option<String> {
        if self.node(id)?.kind != Kind::Entry {
            return None;
        }
        self.widget_text(id)
    }

    pub fn entry_set(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        if !self.node(id).is_some_and(|node| node.kind == Kind::Entry) {
            return false;
        }
        let text = clean_entry(&text.into());
        if let Some(node) = self.node_mut(id) {
            node.cursor = text.len();
        }
        self.write_entry(id, text);
        true
    }

    fn write_entry(&mut self, id: WidgetId, text: String) {
        let binding = self.node(id).and_then(|node| node.binding);
        if let Some(node) = self.node_mut(id) {
            if node.text != text {
                node.text.clone_from(&text);
                self.dirty = true;
            }
        }
        // A trace may destroy this entry. No arena borrow survives notification.
        if let Some(Binding::Text(var)) = binding {
            var.set(self, text);
        }
    }

    /// Returns the current bound/static text, including sanitized Entry text.
    pub fn widget_text(&self, id: WidgetId) -> Option<String> {
        let node = self.node(id)?;
        let text = match node.binding {
            Some(Binding::Text(var)) => var.get(self).unwrap_or_else(|| node.text.clone()),
            Some(Binding::Int(var)) => var.get(self).map(|value| value.to_string())
                .unwrap_or_else(|| node.text.clone()),
            None => node.text.clone(),
        };
        Some(if node.kind == Kind::Entry { clean_entry(&text) } else { text })
    }

    fn set_manager(&mut self, id: WidgetId, manager: Manager) -> bool {
        let Some(parent) = self.node(id).and_then(|node| node.parent) else { return false };
        let pack = matches!(manager, Manager::Pack(_));
        let Some(parent_node) = self.node(parent) else { return false };
        if parent_node.children.iter().copied().filter(|sibling| *sibling != id).any(|sibling| {
            self.node(sibling).and_then(|node| node.manager)
                .is_some_and(|other| matches!(other, Manager::Pack(_)) != pack)
        }) {
            return false;
        }
        if let Some(node) = self.node_mut(id) {
            node.manager = Some(manager);
        }
        self.dirty = true;
        true
    }

    pub fn pack(&mut self, id: WidgetId, options: PackOpts) -> bool {
        self.set_manager(id, Manager::Pack(options))
    }

    pub fn grid(&mut self, id: WidgetId, options: GridOpts) -> bool {
        self.set_manager(id, Manager::Grid(options))
    }

    pub fn grid_columnconfigure(&mut self, parent: WidgetId, column: usize, weight: u32) -> bool {
        self.configure_track(parent, column, weight, true)
    }

    pub fn grid_rowconfigure(&mut self, parent: WidgetId, row: usize, weight: u32) -> bool {
        self.configure_track(parent, row, weight, false)
    }

    fn configure_track(&mut self, parent: WidgetId, index: usize, weight: u32, column: bool) -> bool {
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

    fn managed_children(&self, id: WidgetId) -> Vec<WidgetId> {
        self.node(id).map(|node| {
            node.children.iter().copied()
                .filter(|child| self.node(*child).is_some_and(|node| node.manager.is_some()))
                .collect()
        }).unwrap_or_default()
    }

    fn pack_items(&self, children: &[WidgetId]) -> Vec<PackItem> {
        children.iter().filter_map(|id| {
            let node = self.node(*id)?;
            match node.manager {
                Some(Manager::Pack(opts)) => Some(PackItem { requested: node.requested, opts }),
                _ => None,
            }
        }).collect()
    }

    fn grid_items(&self, children: &[WidgetId]) -> Vec<GridItem> {
        children.iter().filter_map(|id| {
            let node = self.node(*id)?;
            match node.manager {
                Some(Manager::Grid(opts)) => Some(GridItem { requested: node.requested, opts }),
                _ => None,
            }
        }).collect()
    }

    fn uses_pack(&self, children: &[WidgetId]) -> bool {
        children.first().and_then(|id| self.node(*id))
            .is_some_and(|node| matches!(node.manager, Some(Manager::Pack(_))))
    }

    pub(crate) fn measure(&mut self, text: &str, font_size: f32) -> Size {
        let size = self.text_engine.measure(text, font_size * self.scale_factor);
        Size::new(size.width / self.scale_factor, size.height / self.scale_factor)
    }

    fn update_requested(&mut self) {
        let mut order = Vec::new();
        let mut pending = vec![self.root()];
        while let Some(id) = pending.pop() {
            if self.node(id).is_some() {
                order.push(id);
                pending.extend(self.managed_children(id));
            }
        }
        for id in order.into_iter().rev() {
            let Some(node) = self.node(id) else { continue };
            let (kind, style, nested) = (node.kind, node.style, node.parent.is_some());
            let children = self.managed_children(id);
            let char_width = self.measure("0", style.font_size).width.max(style.font_size * 0.5);
            let specified = style.width.map(|chars| chars as f32 * char_width);
            let requested = if kind == Kind::Frame {
                let size = if self.uses_pack(&children) {
                    layout::pack_request(&self.pack_items(&children))
                } else {
                    layout::grid_request(&self.grid_items(&children))
                };
                let border = if nested { 4.0 } else { 0.0 };
                Size::new(
                    (size.width + border).max(specified.unwrap_or(0.0)),
                    size.height + border,
                )
            } else if kind == Kind::Scale {
                Size::new(specified.unwrap_or(160.0), 36.0)
            } else if kind == Kind::Entry {
                Size::new(specified.unwrap_or(20.0 * char_width) + 12.0, style.font_size * 1.3 + 12.0)
            } else {
                let text = self.widget_text(id).unwrap_or_default();
                let measured = self.measure(&text, style.font_size);
                let padding = match kind {
                    Kind::Label => 12.0,
                    Kind::Checkbutton => 40.0,
                    _ => 20.0,
                };
                Size::new(specified.unwrap_or(measured.width) + padding, measured.height + 12.0)
            };
            if let Some(node) = self.node_mut(id) {
                node.requested = requested;
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
            let children = self.managed_children(id);
            let rects = if self.uses_pack(&children) {
                layout::pack(content, &self.pack_items(&children))
            } else {
                let (columns, rows): (Vec<_>, Vec<_>) = self.node(id).map(|node| {
                    (
                        node.columns.iter().map(|(&i, &w)| (i, w)).collect(),
                        node.rows.iter().map(|(&i, &w)| (i, w)).collect(),
                    )
                }).unwrap_or_default();
                layout::grid(content, &self.grid_items(&children), &columns, &rows)
            };
            for (child, child_rect) in children.into_iter().zip(rects).rev() {
                pending.push((child, child_rect, content.within(clip)));
            }
        }
        if !self.focused.is_some_and(|id| self.is_focusable(id)) {
            self.focused = None;
        }
    }

    fn ensure_layout(&mut self) {
        if self.dirty {
            self.layout();
        }
    }

    fn is_focusable(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            node.visible && node.kind.focusable() && node.clip.width > 0.0 && node.clip.height > 0.0
        })
    }

    pub fn widget_rect(&mut self, id: WidgetId) -> Option<Rect> {
        self.ensure_layout();
        self.node(id).filter(|node| node.visible).map(|node| node.rect)
    }

    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.focused.filter(|id| self.node(*id).is_some())
    }

    /// Physical viewport size. At the default scale this equals logical size.
    #[doc(hidden)]
    pub fn sim_resize(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
        self.dirty = true;
        self.layout();
    }

    #[doc(hidden)]
    pub fn sim_scale_factor(&mut self, factor: f64) -> bool {
        if !factor.is_finite() || !(0.1..=16.0).contains(&factor) {
            return false;
        }
        self.scale_factor = factor as f32;
        self.dirty = true;
        self.layout();
        true
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<WidgetId> {
        self.paint_order.iter().rev().copied().find(|id| {
            self.is_focusable(*id)
                && self.node(*id).is_some_and(|node| node.clip.contains(x, y))
        })
    }

    /// Pointer coordinates are logical pixels.
    #[doc(hidden)]
    pub fn sim_mouse_move(&mut self, x: f32, y: f32) {
        self.ensure_layout();
        self.pointer = (x, y);
        let hovered = self.hit_test(x, y);
        if self.hovered != hovered {
            self.hovered = hovered;
            self.dirty = true;
        }
        if let Some(id) = self.pressed {
            if self.node(id).is_some_and(|node| node.kind == Kind::Scale) {
                self.drag_scale(id, x);
            }
        }
    }

    #[doc(hidden)]
    pub fn sim_mouse_down(&mut self) {
        self.ensure_layout();
        let (x, y) = self.pointer;
        let hit = self.hit_test(x, y);
        self.hovered = hit;
        self.pressed = hit;
        self.focused = hit;
        if let Some(id) = hit {
            let Some(node) = self.node(id) else { return };
            match node.kind {
                Kind::Entry => {
                    let target = (x - node.rect.x - 6.0 + node.scroll_x).max(0.0);
                    let size = node.style.font_size;
                    let text = self.widget_text(id).unwrap_or_default();
                    let boundaries = clusters(&text);
                    // Binary search avoids quadratic shaping for a large entry.
                    let (mut low, mut high) = (0, boundaries.len());
                    while low < high {
                        let mid = low + (high - low) / 2;
                        let end = boundaries.get(mid).copied().unwrap_or(text.len());
                        let width = self.measure(text.get(..end).unwrap_or_default(), size).width;
                        if width < target {
                            low = mid + 1;
                        } else {
                            high = mid;
                        }
                    }
                    let right = boundaries.get(low).copied().unwrap_or(text.len());
                    let left = low.checked_sub(1).and_then(|i| boundaries.get(i)).copied().unwrap_or(0);
                    let rw = self.measure(text.get(..right).unwrap_or_default(), size).width;
                    let lw = self.measure(text.get(..left).unwrap_or_default(), size).width;
                    let cursor = if (rw - target).abs() < (lw - target).abs() { right } else { left };
                    if let Some(node) = self.node_mut(id) {
                        node.cursor = cursor;
                    }
                }
                Kind::Scale => self.drag_scale(id, x),
                _ => {}
            }
        }
        self.dirty = true;
    }

    #[doc(hidden)]
    pub fn sim_mouse_up(&mut self) {
        self.ensure_layout();
        let hit = self.hit_test(self.pointer.0, self.pointer.1);
        if let Some(id) = self.pressed.take() {
            if hit == Some(id) {
                self.activate(id);
            }
        }
        self.hovered = hit.filter(|id| self.node(*id).is_some());
        self.dirty = true;
    }

    #[doc(hidden)]
    pub fn sim_click(&mut self, x: f32, y: f32) {
        self.sim_mouse_move(x, y);
        self.sim_mouse_down();
        self.sim_mouse_up();
    }

    fn run_command(&mut self, id: WidgetId) {
        let Some(node) = self.node_mut(id) else { return };
        if node.command_running {
            return;
        }
        let Some(mut command) = node.command.take() else { return };
        node.command_running = true;
        command(self);
        // Revalidate both slot and generation; a newly allocated widget may
        // already occupy this slot. A replacement command always wins.
        if let Some(node) = self.node_mut(id) {
            node.command_running = false;
            if node.command.is_none() {
                node.command = Some(command);
            }
        }
    }

    fn activate(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        let (kind, check) = (node.kind, node.check);
        if kind == Kind::Checkbutton {
            if let Some(var) = check {
                if let Some(value) = var.get(self) {
                    var.set(self, !value);
                }
            }
        }
        if matches!(kind, Kind::Button | Kind::Checkbutton) {
            self.run_command(id);
        }
    }

    pub(crate) fn scale_fraction(&self, id: WidgetId) -> f32 {
        let Some(node) = self.node(id) else { return 0.0 };
        let (from, to) = node.range;
        let value = node.scale.and_then(|var| var.get(self)).unwrap_or(from);
        if from == to {
            0.0
        } else {
            ((value as f64 - from as f64) / (to as f64 - from as f64)).clamp(0.0, 1.0) as f32
        }
    }

    fn set_scale(&mut self, id: WidgetId, value: i64) {
        let Some(node) = self.node(id) else { return };
        let Some(var) = node.scale else { return };
        let (from, to) = node.range;
        let value = value.clamp(from.min(to), from.max(to));
        if var.get(self).is_some_and(|old| old != value) {
            var.set(self, value);
            self.run_command(id);
        }
    }

    fn drag_scale(&mut self, id: WidgetId, x: f32) {
        if !x.is_finite() {
            return;
        }
        let Some(node) = self.node(id) else { return };
        let track = node.rect.inset(12.0, 0.0);
        let fraction = if track.width > 0.0 {
            ((x - track.x) / track.width).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (from, to) = node.range;
        let value = (from as f64 + f64::from(fraction) * (to as f64 - from as f64)).round() as i64;
        self.set_scale(id, value);
    }

    fn traverse_focus(&mut self, backwards: bool) {
        let order: Vec<_> = self.paint_order.iter().copied()
            .filter(|id| self.is_focusable(*id)).collect();
        if order.is_empty() {
            self.focused = None;
            return;
        }
        let current = self.focused.and_then(|id| order.iter().position(|other| *other == id));
        let next = match (current, backwards) {
            (Some(0), true) | (None, true) => order.len() - 1,
            (Some(index), true) => index - 1,
            (Some(index), false) => (index + 1) % order.len(),
            (None, false) => 0,
        };
        self.focused = order.get(next).copied();
        self.dirty = true;
    }

    #[doc(hidden)]
    pub fn sim_key(&mut self, key: Key) {
        self.ensure_layout();
        if matches!(key, Key::Tab | Key::ShiftTab) {
            self.traverse_focus(key == Key::ShiftTab);
            return;
        }
        let Some(id) = self.focused else { return };
        let Some(node) = self.node(id) else { return };
        match node.kind {
            Kind::Entry => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else {
                    self.edit_entry(id, Some(key), "");
                }
            }
            Kind::Button | Kind::Checkbutton if matches!(key, Key::Enter | Key::Space) => {
                self.activate(id);
            }
            Kind::Scale if matches!(key, Key::Left | Key::Right) => {
                let (from, to) = node.range;
                let old = node.scale.and_then(|var| var.get(self)).unwrap_or(from);
                let increasing = (key == Key::Right) == (to >= from);
                self.set_scale(id, if increasing { old.saturating_add(1) } else { old.saturating_sub(1) });
            }
            _ => {}
        }
    }

    #[doc(hidden)]
    pub fn sim_text(&mut self, text: &str) {
        self.ensure_layout();
        if let Some(id) = self.focused {
            if self.node(id).is_some_and(|node| node.kind == Kind::Entry) {
                self.edit_entry(id, None, text);
            }
        }
    }

    fn edit_entry(&mut self, id: WidgetId, key: Option<Key>, inserted: &str) {
        let Some(text) = self.entry_get(id) else { return };
        let Some(node) = self.node(id) else { return };
        let boundaries = clusters(&text);
        let mut cursor = cursor_boundary(&text, node.cursor);
        let previous = boundaries.iter().copied().take_while(|b| *b < cursor).last().unwrap_or(0);
        let next = boundaries.iter().copied().find(|b| *b > cursor).unwrap_or(text.len());
        let mut replacement = None;
        match key {
            Some(Key::Left) => cursor = previous,
            Some(Key::Right) => cursor = next,
            Some(Key::Home) => cursor = 0,
            Some(Key::End) => cursor = text.len(),
            Some(Key::Backspace) if cursor > 0 => {
                replacement = Some(join_text(&text, previous, cursor, ""));
                cursor = previous;
            }
            Some(Key::Delete) if cursor < text.len() => {
                replacement = Some(join_text(&text, cursor, next, ""));
            }
            None => {
                let value = clean_entry_limit(inserted, MAX_ENTRY_BYTES.saturating_sub(text.len()));
                if !value.is_empty() {
                    replacement = Some(join_text(&text, cursor, cursor, &value));
                    cursor += value.len();
                }
            }
            _ => {}
        }
        if let Some(new_text) = replacement {
            // Joining may merge clusters on either side of the insertion.
            cursor = cursor_boundary(&new_text, cursor);
            if let Some(node) = self.node_mut(id) {
                node.cursor = cursor;
            }
            self.write_entry(id, new_text);
        } else if let Some(node) = self.node_mut(id) {
            node.cursor = cursor;
        }
        self.dirty = true;
    }

    pub(crate) fn clear_input(&mut self) {
        self.focused = None;
        self.pressed = None;
        self.hovered = None;
        self.dirty = true;
    }
}

fn join_text(text: &str, start: usize, end: usize, value: &str) -> String {
    let mut result = String::new();
    result.push_str(text.get(..start).unwrap_or_default());
    result.push_str(value);
    result.push_str(text.get(end..).unwrap_or_default());
    result
}

fn clean_entry_limit(text: &str, limit: usize) -> String {
    let mut result = String::new();
    for ch in text.chars().filter(|ch| !ch.is_control()) {
        if result.len() + ch.len_utf8() > limit {
            break;
        }
        result.push(ch);
    }
    result
}

fn clean_entry(text: &str) -> String {
    clean_entry_limit(text, MAX_ENTRY_BYTES)
}

/// A deliberately small grapheme approximation, not Unicode segmentation.
pub(crate) fn clusters(text: &str) -> Vec<usize> {
    let mut result = vec![0];
    let mut previous = '\0';
    for (index, ch) in text.char_indices() {
        let extending = matches!(
            ch,
            '\u{0300}'..='\u{036f}'
                | '\u{fe0e}'..='\u{fe0f}'
                | '\u{1f3fb}'..='\u{1f3ff}'
                | '\u{200d}'
        );
        if index > 0 && !extending && previous != '\u{200d}' {
            result.push(index);
        }
        previous = ch;
    }
    if !text.is_empty() {
        result.push(text.len());
    }
    result
}

pub(crate) fn cursor_boundary(text: &str, cursor: usize) -> usize {
    clusters(text).into_iter().take_while(|boundary| *boundary <= cursor).last().unwrap_or(0)
}
