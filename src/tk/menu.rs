//! Menus: menubar strip, popup/context menus, submenus, accelerators and
//! keyboard navigation.
//!
//! All menu state lives in [`MenuState`] on the [`Tk`] instance; the widget
//! arena only carries the `Kind::Menubar` strip and its titles. Drawing lives
//! in `src/render/data.rs` (`draw_menu_overlay`), invoked from the final
//! overlay pass of `render_to_pixmap`.

use std::collections::BTreeMap;

use crate::layout::Rect;
use crate::widgets::{Color, Kind};
use crate::{RtkError, Tk, Var, WidgetId};

/// Generational handle to a menu created with [`Tk::menu_create`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct MenuId(pub(crate) u32);

/// One entry of a menu.
#[non_exhaustive]
pub enum MenuItem {
    /// A plain command with an optional accelerator label (display only).
    Command {
        /// Caption shown in the menu.
        label: String,
        /// Accelerator text drawn right-aligned (e.g. `"Ctrl+S"`).
        accel: Option<String>,
        /// Whether the item can be activated.
        enabled: bool,
    },
    /// A checkable item bound to a boolean variable.
    Check {
        /// Caption shown in the menu.
        label: String,
        /// Variable toggled when the item is activated.
        var: Var<bool>,
    },
    /// A horizontal separator line.
    Separator,
    /// A submenu entry that opens `menu` to the right.
    Submenu {
        /// Caption shown in the menu.
        label: String,
        /// Menu opened when the entry is hovered or activated.
        menu: MenuId,
    },
}

impl MenuItem {
    /// Creates an enabled command item.
    pub fn command(label: impl Into<String>) -> Self {
        Self::Command { label: label.into(), accel: None, enabled: true }
    }

    /// Attaches accelerator text to a command item (display only).
    pub fn with_accel(mut self, accel: impl Into<String>) -> Self {
        if let Self::Command { accel: slot, .. } = &mut self {
            *slot = Some(accel.into());
        }
        self
    }

    /// Creates a checkable item bound to `var`.
    pub fn check(label: impl Into<String>, var: Var<bool>) -> Self {
        Self::Check { label: label.into(), var }
    }

    /// Creates a separator.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Creates a submenu entry pointing at `menu`.
    pub fn submenu(label: impl Into<String>, menu: MenuId) -> Self {
        Self::Submenu { label: label.into(), menu }
    }

    /// Whether the item can be highlighted/activated.
    pub(crate) fn is_enabled(&self) -> bool {
        match self {
            Self::Command { enabled, .. } => *enabled,
            Self::Check { .. } | Self::Submenu { .. } => true,
            Self::Separator => false,
        }
    }
}

/// Boxed callback run when a menu command is activated.
pub(crate) type MenuCommand = Box<dyn FnMut(&mut Tk)>;

/// A menu: its items plus per-item callbacks.
pub(crate) struct MenuData {
    /// Items in display order.
    pub(crate) items: Vec<MenuItem>,
    /// Callbacks keyed by item index (only for `Command` items).
    pub(crate) commands: BTreeMap<usize, MenuCommand>,
}

impl MenuData {
    /// Creates an empty menu.
    fn new() -> Self {
        Self { items: Vec::new(), commands: BTreeMap::new() }
    }
}

/// One level of the open menu stack.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OpenMenu {
    /// The menu shown at this level.
    pub(crate) menu: MenuId,
    /// Anchor point (logical px) of the panel's top-left corner.
    pub(crate) anchor: (f32, f32),
    /// Highlighted item index, if any.
    pub(crate) highlight: Option<usize>,
}

/// All menu state owned by the toolkit.
#[derive(Default)]
pub(crate) struct MenuState {
    /// Menus by id.
    pub(crate) menus: BTreeMap<u32, MenuData>,
    /// Next menu id.
    pub(crate) next_id: u32,
    /// Stack of open menus (root first).
    pub(crate) open: Vec<OpenMenu>,
    /// Menubar whose menu is currently open, if any.
    pub(crate) active_bar: Option<WidgetId>,
    /// Context menus bound to widgets.
    pub(crate) context: BTreeMap<WidgetId, MenuId>,
}

impl MenuState {
    /// Returns the menu data for `id`.
    pub(crate) fn get(&self, id: MenuId) -> Option<&MenuData> {
        self.menus.get(&id.0)
    }

    /// Returns the menu data for `id` mutably.
    pub(crate) fn get_mut(&mut self, id: MenuId) -> Option<&mut MenuData> {
        self.menus.get_mut(&id.0)
    }
}

/// Item height in logical px for a body font size.
pub(crate) fn item_height(font_size: f32) -> f32 {
    (font_size * 1.8).round().max(8.0)
}

/// Width of a menu panel in logical px, measured with `tk`.
pub(crate) fn panel_width(tk: &mut Tk, menu: MenuId) -> f32 {
    let style = tk.resolved_style(tk.root()).unwrap_or_default();
    let items: Vec<(Option<String>, Option<String>, bool)> = tk
        .menus
        .get(menu)
        .map(|m| {
            m.items
                .iter()
                .map(|item| match item {
                    MenuItem::Command { label, accel, .. } => (Some(label.clone()), accel.clone(), true),
                    MenuItem::Check { label, .. } => (Some(label.clone()), None, true),
                    MenuItem::Submenu { label, .. } => (Some(label.clone()), None, true),
                    MenuItem::Separator => (None, None, false),
                })
                .collect()
        })
        .unwrap_or_default();
    let mut max_w = 80.0_f32;
    for (label, accel, _) in items {
        let Some(label) = label else { continue };
        let lw = tk.measure_with(&label, &style).width;
        let aw = accel.map(|a| tk.measure_with(&a, &style).width).unwrap_or(0.0);
        let w = 24.0 + lw + if aw > 0.0 { 24.0 + aw } else { 0.0 } + 16.0;
        max_w = max_w.max(w);
    }
    max_w
}

/// Height of a menu panel in logical px.
pub(crate) fn panel_height(tk: &Tk, menu: MenuId) -> f32 {
    let style = tk.resolved_style(tk.root()).unwrap_or_default();
    let ih = item_height(style.font_size);
    let count = tk.menus.get(menu).map(|m| m.items.len()).unwrap_or(0);
    (count as f32 * ih + 8.0).max(ih)
}

impl Tk {
    /// Creates an empty menu and returns its handle.
    pub fn menu_create(&mut self) -> MenuId {
        let id = self.menus.next_id;
        self.menus.next_id = self.menus.next_id.saturating_add(1);
        self.menus.menus.insert(id, MenuData::new());
        self.dirty = true;
        MenuId(id)
    }

    /// Appends `item` to `menu`, returning its index (or `usize::MAX` if the menu is unknown).
    pub fn menu_add(&mut self, menu: MenuId, item: MenuItem) -> usize {
        let Some(data) = self.menus.get_mut(menu) else { return usize::MAX };
        data.items.push(item);
        let idx = data.items.len() - 1;
        self.dirty = true;
        idx
    }

    /// Installs the callback run when the command item at `index` is activated.
    pub fn menu_set_command(
        &mut self,
        menu: MenuId,
        index: usize,
        command: impl FnMut(&mut Tk) + 'static,
    ) -> bool {
        let Some(data) = self.menus.get_mut(menu) else { return false };
        if index >= data.items.len() {
            return false;
        }
        data.commands.insert(index, Box::new(command));
        true
    }

    /// Enables or disables the command item at `index`.
    pub fn menu_set_enabled(&mut self, menu: MenuId, index: usize, enabled: bool) -> bool {
        let Some(data) = self.menus.get_mut(menu) else { return false };
        let Some(item) = data.items.get_mut(index) else { return false };
        if let MenuItem::Command { enabled: slot, .. } = item {
            *slot = enabled;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Opens `menu` as a popup at logical `(x, y)`, clamped inside the viewport.
    pub fn menu_popup(&mut self, menu: MenuId, x: f32, y: f32) {
        if self.menus.get(menu).is_none() {
            return;
        }
        self.menus.open.clear();
        self.menus.active_bar = None;
        let (ax, ay) = self.clamp_menu_anchor(menu, x, y);
        self.menus.open.push(OpenMenu { menu, anchor: (ax, ay), highlight: None });
        self.dirty = true;
    }

    /// Clamps a menu panel's top-left so it fits inside the viewport.
    pub(crate) fn clamp_menu_anchor(&mut self, menu: MenuId, x: f32, y: f32) -> (f32, f32) {
        let vw = self.viewport.0 as f32 / self.scale_factor;
        let vh = self.viewport.1 as f32 / self.scale_factor;
        let w = panel_width(self, menu);
        let h = panel_height(self, menu);
        let ax = x.clamp(0.0, (vw - w).max(0.0));
        let ay = y.clamp(0.0, (vh - h).max(0.0));
        (ax, ay)
    }

    /// Closes every open menu.
    pub fn menu_close_all(&mut self) {
        if !self.menus.open.is_empty() {
            self.menus.open.clear();
            self.menus.active_bar = None;
            self.dirty = true;
        }
    }

    /// True while any menu is open.
    pub fn menu_is_open(&self) -> bool {
        !self.menus.open.is_empty()
    }

    /// The stack of open menus, root first.
    pub fn menu_open_path(&self) -> Vec<MenuId> {
        self.menus.open.iter().map(|o| o.menu).collect()
    }

    /// Binds `menu` as the context menu of `widget` (right-click opens it).
    pub fn bind_context_menu(&mut self, widget: WidgetId, menu: MenuId) -> bool {
        if self.node(widget).is_none() || self.menus.get(menu).is_none() {
            return false;
        }
        self.menus.context.insert(widget, menu);
        true
    }

    /// Adds a titled menu to a menubar strip.
    pub fn menubar_add(&mut self, bar: WidgetId, title: impl Into<String>, menu: MenuId) -> bool {
        let Some(node) = self.node_mut(bar) else { return false };
        if node.kind != Kind::Menubar {
            return false;
        }
        node.menubar_titles.push((title.into(), menu));
        self.dirty = true;
        true
    }

    /// Opens the menubar menu at `index` below its title.
    pub(crate) fn menubar_open(&mut self, bar: WidgetId, index: usize) {
        let Some(node) = self.node(bar) else { return };
        let Some(&(_, menu)) = node.menubar_titles.get(index) else { return };
        let rect = node.rect;
        let titles: Vec<String> = node.menubar_titles.iter().map(|(t, _)| t.clone()).collect();
        let style = self.resolved_style(bar).unwrap_or_default();
        let mut cur_x = rect.x;
        for (i, title) in titles.iter().enumerate() {
            let tw = self.measure_with(title, &style).width + 24.0;
            if i == index {
                let (ax, ay) = self.clamp_menu_anchor(menu, cur_x, rect.y + rect.height);
                self.menus.open.clear();
                self.menus.open.push(OpenMenu { menu, anchor: (ax, ay), highlight: None });
                self.menus.active_bar = Some(bar);
                self.dirty = true;
                return;
            }
            cur_x += tw;
        }
    }

    /// Returns the menubar title index under logical `(x, y)`, if any.
    pub(crate) fn menubar_hit(&mut self, x: f32, y: f32) -> Option<(WidgetId, usize)> {
        let bars: Vec<WidgetId> = self
            .paint_order
            .iter()
            .copied()
            .filter(|id| self.node(*id).is_some_and(|n| n.kind == Kind::Menubar && n.visible))
            .collect();
        for bar in bars {
            let Some(node) = self.node(bar) else { continue };
            if !node.rect.contains(x, y) {
                continue;
            }
            let rect = node.rect;
            let titles = node.menubar_titles.clone();
            let style = self.resolved_style(bar).unwrap_or_default();
            let mut cur_x = rect.x;
            for (i, (title, _)) in titles.iter().enumerate() {
                let tw = self.measure_with(title, &style).width + 24.0;
                if x >= cur_x && x < cur_x + tw {
                    return Some((bar, i));
                }
                cur_x += tw;
            }
        }
        None
    }

    /// Handles a right-click at logical `(x, y)`: opens a bound context menu.
    pub(crate) fn context_click(&mut self, x: f32, y: f32) {
        self.ensure_layout();
        let hit = self.hit_test(x, y);
        let mut cur = hit;
        while let Some(id) = cur {
            if let Some(&menu) = self.menus.context.get(&id) {
                self.menu_popup(menu, x, y);
                return;
            }
            cur = self.node(id).and_then(|n| n.parent);
        }
        self.menu_close_all();
    }

    /// Headless harness: right-clicks at logical `(x, y)`.
    #[doc(hidden)]
    pub fn sim_right_click(&mut self, x: f32, y: f32) {
        self.pointer = (x, y);
        self.context_click(x, y);
    }

    /// Returns the panel rectangle of the open menu at `level` (logical px).
    #[doc(hidden)]
    pub fn menu_panel_rect(&mut self, level: usize) -> Option<Rect> {
        let open = *self.menus.open.get(level)?;
        let w = panel_width(self, open.menu);
        let h = panel_height(self, open.menu);
        Some(Rect::new(open.anchor.0, open.anchor.1, w, h))
    }

    /// Returns the item index under logical `(x, y)` in the open menu at `level`.
    pub(crate) fn menu_item_at(&mut self, level: usize, x: f32, y: f32) -> Option<usize> {
        let open = *self.menus.open.get(level)?;
        let rect = self.menu_panel_rect(level)?;
        if !rect.contains(x, y) {
            return None;
        }
        let style = self.resolved_style(self.root()).unwrap_or_default();
        let ih = item_height(style.font_size);
        let idx = ((y - rect.y - 4.0) / ih).floor();
        if idx < 0.0 {
            return None;
        }
        let idx = idx as usize;
        let count = self.menus.get(open.menu).map(|m| m.items.len()).unwrap_or(0);
        if idx < count {
            Some(idx)
        } else {
            None
        }
    }

    /// Handles a pointer move while menus are open. Returns true if consumed.
    pub(crate) fn menu_pointer_move(&mut self, x: f32, y: f32) -> bool {
        if self.menus.open.is_empty() {
            return false;
        }
        // Hovering a menubar title switches menus.
        if let Some((bar, index)) = self.menubar_hit(x, y) {
            if self.menus.active_bar == Some(bar) {
                self.menubar_open(bar, index);
                return true;
            }
        }
        // Find the deepest open level containing the pointer.
        let mut level_in: Option<usize> = None;
        for level in 0..self.menus.open.len() {
            if let Some(rect) = self.menu_panel_rect(level) {
                if rect.contains(x, y) {
                    level_in = Some(level);
                }
            }
        }
        let Some(level) = level_in else {
            return true;
        };
        // Close deeper levels than the one hovered.
        if self.menus.open.len() > level + 1 {
            self.menus.open.truncate(level + 1);
            self.dirty = true;
        }
        let Some(idx) = self.menu_item_at(level, x, y) else {
            if let Some(open) = self.menus.open.get_mut(level) {
                if open.highlight.is_some() {
                    open.highlight = None;
                    self.dirty = true;
                }
            }
            return true;
        };
        let menu = self.menus.open[level].menu;
        let enabled = self
            .menus
            .get(menu)
            .and_then(|m| m.items.get(idx))
            .map(MenuItem::is_enabled)
            .unwrap_or(false);
        if let Some(open) = self.menus.open.get_mut(level) {
            if open.highlight != Some(idx) {
                open.highlight = Some(idx);
                self.dirty = true;
            }
        }
        // Hovering a submenu opens it to the right.
        if enabled {
            let submenu = self.menus.get(menu).and_then(|m| match m.items.get(idx) {
                Some(MenuItem::Submenu { menu, .. }) => Some(*menu),
                _ => None,
            });
            if let Some(sub) = submenu {
                let already = self.menus.open.get(level + 1).map(|o| o.menu) == Some(sub);
                if !already {
                    if let Some(rect) = self.menu_panel_rect(level) {
                        let style = self.resolved_style(self.root()).unwrap_or_default();
                        let ih = item_height(style.font_size);
                        let iy = rect.y + 4.0 + idx as f32 * ih;
                        let (ax, ay) = self.clamp_menu_anchor(sub, rect.x + rect.width, iy);
                        self.menus.open.truncate(level + 1);
                        self.menus.open.push(OpenMenu { menu: sub, anchor: (ax, ay), highlight: None });
                        self.dirty = true;
                    }
                }
            }
        }
        true
    }

    /// Handles a left click while menus are open. Returns true if consumed.
    pub(crate) fn menu_click(&mut self, x: f32, y: f32) -> bool {
        if self.menus.open.is_empty() {
            return false;
        }
        // Clicking a menubar title toggles its menu.
        if let Some((bar, index)) = self.menubar_hit(x, y) {
            let same = self.menus.active_bar == Some(bar)
                && self.menus.open.len() == 1
                && self.menus.open.first().map(|o| o.menu)
                    == self.node(bar).and_then(|n| n.menubar_titles.get(index)).map(|(_, m)| *m);
            if same {
                self.menu_close_all();
            } else {
                self.menubar_open(bar, index);
            }
            return true;
        }
        let mut level_in: Option<usize> = None;
        for level in 0..self.menus.open.len() {
            if let Some(rect) = self.menu_panel_rect(level) {
                if rect.contains(x, y) {
                    level_in = Some(level);
                }
            }
        }
        let Some(level) = level_in else {
            self.menu_close_all();
            return true;
        };
        let Some(idx) = self.menu_item_at(level, x, y) else {
            return true;
        };
        self.menu_activate(level, idx);
        true
    }

    /// Activates the item at `index` in the open menu at `level`.
    pub(crate) fn menu_activate(&mut self, level: usize, index: usize) {
        let Some(open) = self.menus.open.get(level).copied() else { return };
        let menu = open.menu;
        // Copy the item kind so no borrow of `self.menus` is held across mutations.
        enum Kind2 {
            Command,
            Check(Var<bool>),
            Submenu(MenuId),
            Separator,
            Disabled,
        }
        let kind = match self.menus.get(menu).and_then(|m| m.items.get(index)) {
            Some(MenuItem::Command { enabled, .. }) if *enabled => Kind2::Command,
            Some(MenuItem::Command { .. }) => Kind2::Disabled,
            Some(MenuItem::Check { var, .. }) => Kind2::Check(*var),
            Some(MenuItem::Submenu { menu, .. }) => Kind2::Submenu(*menu),
            Some(MenuItem::Separator) => Kind2::Separator,
            None => return,
        };
        match kind {
            Kind2::Command => {
                let command = self.menus.get_mut(menu).and_then(|m| m.commands.remove(&index));
                self.menu_close_all();
                if let Some(mut command) = command {
                    command(self);
                    if let Some(data) = self.menus.get_mut(menu) {
                        data.commands.insert(index, command);
                    }
                }
            }
            Kind2::Check(var) => {
                if let Some(value) = var.get(self) {
                    var.set(self, !value);
                }
                self.menu_close_all();
            }
            Kind2::Submenu(sub) => {
                if let Some(rect) = self.menu_panel_rect(level) {
                    let style = self.resolved_style(self.root()).unwrap_or_default();
                    let ih = item_height(style.font_size);
                    let iy = rect.y + 4.0 + index as f32 * ih;
                    let (ax, ay) = self.clamp_menu_anchor(sub, rect.x + rect.width, iy);
                    self.menus.open.truncate(level + 1);
                    self.menus.open.push(OpenMenu { menu: sub, anchor: (ax, ay), highlight: None });
                    self.dirty = true;
                }
            }
            Kind2::Separator | Kind2::Disabled => {}
        }
    }

    /// Handles a key while menus are open. Returns true if consumed.
    pub(crate) fn menu_key(&mut self, key: crate::Key) -> bool {
        if self.menus.open.is_empty() {
            return false;
        }
        let level = self.menus.open.len() - 1;
        let menu = self.menus.open[level].menu;
        let items: Vec<(bool, bool)> = self
            .menus
            .get(menu)
            .map(|m| m.items.iter().map(|i| (i.is_enabled(), matches!(i, MenuItem::Separator))).collect())
            .unwrap_or_default();
        let current = self.menus.open[level].highlight;
        match key {
            crate::Key::Down => {
                let next = next_enabled(&items, current, true);
                self.menus.open[level].highlight = next;
                self.dirty = true;
                true
            }
            crate::Key::Up => {
                let next = next_enabled(&items, current, false);
                self.menus.open[level].highlight = next;
                self.dirty = true;
                true
            }
            crate::Key::Right => {
                let submenu = current.and_then(|idx| {
                    self.menus.get(menu).and_then(|m| match m.items.get(idx) {
                        Some(MenuItem::Submenu { menu, .. }) => Some((idx, *menu)),
                        _ => None,
                    })
                });
                if let Some((idx, sub)) = submenu {
                    if let Some(rect) = self.menu_panel_rect(level) {
                        let style = self.resolved_style(self.root()).unwrap_or_default();
                        let ih = item_height(style.font_size);
                        let iy = rect.y + 4.0 + idx as f32 * ih;
                        let (ax, ay) = self.clamp_menu_anchor(sub, rect.x + rect.width, iy);
                        self.menus.open.push(OpenMenu { menu: sub, anchor: (ax, ay), highlight: None });
                        self.dirty = true;
                    }
                } else if let Some(bar) = self.menus.active_bar {
                    self.menubar_switch(bar, true);
                }
                true
            }
            crate::Key::Left => {
                if self.menus.open.len() > 1 {
                    self.menus.open.pop();
                    self.dirty = true;
                } else if let Some(bar) = self.menus.active_bar {
                    self.menubar_switch(bar, false);
                }
                true
            }
            crate::Key::Enter | crate::Key::Space => {
                if let Some(idx) = current {
                    self.menu_activate(level, idx);
                }
                true
            }
            crate::Key::Escape => {
                if self.menus.open.len() > 1 {
                    self.menus.open.pop();
                } else {
                    self.menu_close_all();
                }
                self.dirty = true;
                true
            }
            _ => true,
        }
    }

    /// Switches the open menubar menu to the previous/next title.
    pub(crate) fn menubar_switch(&mut self, bar: WidgetId, forward: bool) {
        let Some(node) = self.node(bar) else { return };
        let menus: Vec<MenuId> = node.menubar_titles.iter().map(|(_, m)| *m).collect();
        let count = menus.len();
        if count == 0 {
            return;
        }
        let current = self
            .menus
            .open
            .first()
            .and_then(|o| menus.iter().position(|m| *m == o.menu));
        let next = match current {
            Some(i) if forward => (i + 1) % count,
            Some(i) => (i + count - 1) % count,
            None => 0,
        };
        self.menubar_open(bar, next);
    }

    /// Returns the resolved colours used to draw menus.
    pub(crate) fn menu_colors(&self) -> (Color, Color, Color, Color, Color) {
        let p = self.style_engine.tokens().palette;
        (p.surface, p.border, p.select_bg, p.select_fg, p.muted)
    }
}

/// Returns the next enabled, non-separator item index from `current`.
fn next_enabled(items: &[(bool, bool)], current: Option<usize>, forward: bool) -> Option<usize> {
    let n = items.len();
    if n == 0 {
        return None;
    }
    let start = match current {
        Some(i) if forward => (i + 1) % n,
        Some(i) => (i + n - 1) % n,
        None if forward => 0,
        None => n - 1,
    };
    let mut idx = start;
    for _ in 0..n {
        let (enabled, sep) = items[idx];
        if enabled && !sep {
            return Some(idx);
        }
        idx = if forward { (idx + 1) % n } else { (idx + n - 1) % n };
    }
    None
}

/// Constructor namespace for menubar strips.
pub struct Menubar;

#[allow(clippy::new_ret_no_self)]
impl Menubar {
    /// Creates an empty menubar strip under `parent`.
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Menubar, parent, String::new())
    }
}
