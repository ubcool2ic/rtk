use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use tiny_skia::Pixmap;

use crate::layout::{self, GridItem, GridOpts, PackItem, PackOpts, PlaceOpts, Rect, Size};
use crate::render::TextEngine;
use crate::style::{
    ImageId, ResolvedStyle, State, StyleEngine, StyleError, StyleSpec,
    StylesheetParser, Tokens, TransitionState,
};
use crate::var::VarSlot;
use crate::widgets::{
    Binding, CanvasItem, CanvasItemKind, CanvasItemSlot, Color, DragPhase,
    Kind, Manager, Theme, WidgetNode, WidgetSlot,
};
use crate::{Opt, RtkError, WidgetId};

const MAX_ENTRY_BYTES: usize = 64 * 1024;

/// Keys understood by the real backend and the headless event harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Tab,
    ShiftTab,
    Enter,
    Space,
    PageUp,
    PageDown,
}

/// Scroll step for `yview_scroll`/`xview_scroll`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollUnit {
    Units,
    Pages,
}

/// Logical px a Canvas scrolls per wheel line / scroll unit.
const WHEEL_LINE_PX: f32 = 48.0;

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
    pub(crate) theme: Theme,
    drag_active: Option<WidgetId>,
    drag_start: (f32, f32),
    scrollbar_drag: Option<(WidgetId, f32, usize)>,
    scrollbar_drag_start: f32,
    // Advanced styling layer
    pub(crate) style_engine: StyleEngine,
    pub(crate) images: BTreeMap<ImageId, Pixmap>,
    pub(crate) next_image_id: u32,
    pub(crate) watched_stylesheet: Option<(PathBuf, SystemTime)>,
    pub(crate) timers: crate::timer::Timers,
}

impl Default for Tk {
    fn default() -> Self {
        Self::new()
    }
}

impl Tk {
    pub fn new() -> Self {
        let _ = crate::app::START.get_or_init(std::time::Instant::now);
        let style_engine = StyleEngine::new(Tokens::light());
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
            theme: Theme::Light,
            drag_active: None,
            drag_start: (0.0, 0.0),
            scrollbar_drag: None,
            scrollbar_drag_start: 0.0,
            style_engine,
            images: BTreeMap::new(),
            next_image_id: 1,
            watched_stylesheet: None,
            timers: crate::timer::Timers::default(),
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

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn tokens(&self) -> &Tokens {
        self.style_engine.tokens()
    }

    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.style_engine.set_tokens(tokens);
        self.dirty = true;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        if self.theme == theme {
            return;
        }
        self.theme = theme;
        self.dirty = true;
        let tokens = match theme {
            Theme::Light => Tokens::light(),
            Theme::Dark => Tokens::dark(),
            Theme::Fluent => Tokens::fluent(),
            Theme::Material => Tokens::material(),
            Theme::HighContrast => Tokens::high_contrast(),
        };
        self.style_engine.set_tokens(tokens);

        let is_dark = theme == Theme::Dark;
        let is_hc = theme == Theme::HighContrast;
        for slot in &mut self.nodes {
            if let Some(node) = &mut slot.node {
                if !node.style.bg_overridden {
                    node.style.bg = match node.kind {
                        Kind::Entry | Kind::Listbox => {
                            if is_hc { Color::BLACK } else if is_dark { Color::rgb(35, 35, 35) } else { Color::WHITE }
                        }
                        Kind::Canvas => {
                            if is_hc { Color::BLACK } else if is_dark { Color::rgb(25, 25, 25) } else { Color::rgb(240, 240, 240) }
                        }
                        _ => {
                            if is_hc { Color::BLACK } else if is_dark { Color::rgb(45, 45, 45) } else { Color::BG }
                        }
                    };
                }
                if !node.style.fg_overridden {
                    node.style.fg = if is_hc {
                        Color::WHITE
                    } else if is_dark {
                        Color::rgb(230, 230, 230)
                    } else {
                        Color::FG
                    };
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Style API
    // -----------------------------------------------------------------------

    pub fn style_configure(&mut self, style_name: impl Into<String>, spec: StyleSpec) {
        self.style_engine.configure(style_name, spec);
        self.dirty = true;
    }

    pub fn style_map(&mut self, style_name: impl Into<String>, state: State, spec: StyleSpec) {
        self.style_engine.map(style_name, state, spec);
        self.dirty = true;
    }

    pub fn load_stylesheet(&mut self, css: &str) -> Result<usize, StyleError> {
        let parser = StylesheetParser::new(css);
        let count = parser.parse(&mut self.style_engine)?;
        self.dirty = true;
        Ok(count)
    }

    pub fn load_stylesheet_file(&mut self, path: impl AsRef<Path>) -> Result<usize, StyleError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| StyleError {
            line: 0,
            message: format!("cannot read file {}: {e}", path.display()),
        })?;
        self.load_stylesheet(&content)
    }

    pub fn watch_stylesheet(&mut self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref().to_path_buf();
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                let _ = self.load_stylesheet_file(&path);
                self.watched_stylesheet = Some((path, mtime));
                return true;
            }
        }
        false
    }

    pub fn poll_stylesheet(&mut self) -> bool {
        let Some((path, last_mtime)) = self.watched_stylesheet.clone() else { return false };
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if mtime > last_mtime {
                    self.watched_stylesheet = Some((path.clone(), mtime));
                    let _ = self.load_stylesheet_file(&path);
                    return true;
                }
            }
        }
        false
    }

    pub fn load_image(&mut self, path_or_png_bytes: impl AsRef<[u8]>) -> Option<ImageId> {
        let bytes = path_or_png_bytes.as_ref();
        let pixmap = if let Ok(s) = std::str::from_utf8(bytes) {
            if let Ok(file_bytes) = fs::read(s) {
                Pixmap::decode_png(&file_bytes).ok()
            } else {
                Pixmap::decode_png(bytes).ok()
            }
        } else {
            Pixmap::decode_png(bytes).ok()
        }?;
        let id = ImageId(self.next_image_id);
        self.next_image_id = self.next_image_id.saturating_add(1);
        self.images.insert(id, pixmap);
        Some(id)
    }

    pub fn widget_state(&self, id: WidgetId) -> State {
        let Some(node) = self.node(id) else { return State::empty() };
        let mut s = State::empty();
        if node.disabled {
            s |= State::DISABLED;
        }
        if self.hovered == Some(id) {
            s |= State::HOVER;
        }
        if self.pressed == Some(id) {
            s |= State::PRESSED;
        }
        if self.focused == Some(id) {
            s |= State::FOCUSED;
        }
        if node.kind == Kind::Checkbutton {
            if node.check.and_then(|v| v.get(self)).unwrap_or(false) {
                s |= State::CHECKED;
            }
        } else if node.kind == Kind::Radiobutton {
            if node.radio_var.and_then(|v| v.get(self)) == Some(node.radio_value) {
                s |= State::CHECKED;
            }
        } else if node.kind == Kind::Listbox && node.list_selection.is_some() {
            s |= State::SELECTED;
        }
        s
    }

    /// Resolves the concrete style for widget `id` by cascading:
    /// theme default -> "TClass" -> "Named.TClass" -> Frame inheritance -> widget override -> state map -> legacy Opt.
    /// Background colour currently shown, including an in-flight transition.
    #[doc(hidden)]
    pub fn displayed_bg(&self, id: WidgetId) -> Option<Color> {
        let node = self.node(id)?;
        match node.transition.as_ref() {
            Some(transition) => Some(transition.current_bg()),
            None => self.resolved_style(id).map(|style| style.bg),
        }
    }

    pub fn resolved_style(&self, id: WidgetId) -> Option<ResolvedStyle> {
        self.resolved_style_in(id, self.widget_state(id))
    }

    /// Resolves the cascade as if the widget were in `state`.
    fn resolved_style_in(&self, id: WidgetId, state: State) -> Option<ResolvedStyle> {
        let node = self.node(id)?;
        let kind = node.kind;
        let class_name = kind_class_name(kind);
        let mut res = ResolvedStyle::default();

        // 1. Theme default for class
        if let Some(spec) = self.style_engine.lookup(class_name) {
            spec.merge_into(&mut res);
        }

        // 2. Named style if specified on node
        if let Some(named) = &node.named_style {
            if named != class_name {
                if let Some(spec) = self.style_engine.lookup(named) {
                    spec.merge_into(&mut res);
                }
            }
        }

        // 3. Inherited properties from ancestor Frames (fg, font_family, font_size, font_weight, italic)
        let mut ancestor = node.parent;
        let mut inherited_fg = None;
        let mut inherited_font_family = None;
        let mut inherited_font_size = None;
        let mut inherited_font_weight = None;
        let mut inherited_italic = None;

        while let Some(pid) = ancestor {
            if let Some(pnode) = self.node(pid) {
                if let Some(named) = &pnode.named_style {
                    if let Some(spec) = self.style_engine.lookup(named) {
                        if inherited_fg.is_none() { inherited_fg = spec.fg; }
                        if inherited_font_family.is_none() { inherited_font_family = spec.font_family.clone(); }
                        if inherited_font_size.is_none() { inherited_font_size = spec.font_size; }
                        if inherited_font_weight.is_none() { inherited_font_weight = spec.font_weight; }
                        if inherited_italic.is_none() { inherited_italic = spec.italic; }
                    }
                }
                if let Some(ovr) = &pnode.style_override {
                    if inherited_fg.is_none() { inherited_fg = ovr.fg; }
                    if inherited_font_family.is_none() { inherited_font_family = ovr.font_family.clone(); }
                    if inherited_font_size.is_none() { inherited_font_size = ovr.font_size; }
                    if inherited_font_weight.is_none() { inherited_font_weight = ovr.font_weight; }
                    if inherited_italic.is_none() { inherited_italic = ovr.italic; }
                }
                ancestor = pnode.parent;
            } else {
                break;
            }
        }
        if let Some(c) = inherited_fg { res.fg = c; }
        if let Some(ff) = inherited_font_family { res.font_family = ff; }
        if let Some(fs) = inherited_font_size { res.font_size = fs; }
        if let Some(fw) = inherited_font_weight { res.font_weight = fw; }
        if let Some(it) = inherited_italic { res.italic = it; }

        // 4. Per-widget override
        if let Some(ovr) = &node.style_override {
            ovr.merge_into(&mut res);
        }

        // 5. State maps for the requested states
        let style_key = node.named_style.as_deref().unwrap_or(class_name);
        for check_state in [
            State::HOVER, State::FOCUSED, State::PRESSED,
            State::CHECKED, State::SELECTED, State::DISABLED,
        ] {
            if state.contains(check_state) {
                // Check specific named style then fallback to class
                if let Some(spec) = self.style_engine.lookup_state(style_key, check_state) {
                    spec.merge_into(&mut res);
                } else if style_key != class_name {
                    if let Some(spec) = self.style_engine.lookup_state(class_name, check_state) {
                        spec.merge_into(&mut res);
                    }
                }
            }
        }

        // 6. Legacy Opt::Bg/Fg/FontSize wins if explicitly set
        if node.style.bg_overridden {
            res.bg = node.style.bg;
        }
        if node.style.fg_overridden {
            res.fg = node.style.fg;
        }
        if node.style.font_size != 14.0 {
            res.font_size = node.style.font_size;
        }

        Some(res)
    }

    /// Advances simulated animation time by `ms` milliseconds for transitions.
    pub fn sim_advance_time(&mut self, ms: u32) {
        for slot in &mut self.nodes {
            if let Some(node) = &mut slot.node {
                if let Some(trans) = &mut node.transition {
                    let done = trans.advance(ms);
                    if done {
                        node.transition = None;
                    }
                    self.dirty = true;
                }
            }
        }
    }

    pub fn is_animating(&self) -> bool {
        self.nodes.iter().filter_map(|s| s.node.as_ref()).any(|n| n.transition.is_some())
    }

    pub fn title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
        self.dirty = true;
    }

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
        let mut node = WidgetNode::new(kind, Some(parent), text);
        let is_hc = self.theme == Theme::HighContrast;
        let is_dark = self.theme == Theme::Dark;
        if is_hc {
            node.style.bg = Color::BLACK;
            node.style.fg = Color::WHITE;
        } else if is_dark {
            node.style.bg = match kind {
                Kind::Entry | Kind::Listbox => Color::rgb(35, 35, 35),
                Kind::Canvas => Color::rgb(25, 25, 25),
                _ => Color::rgb(45, 45, 45),
            };
            node.style.fg = Color::rgb(230, 230, 230);
        }
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
            if self.drag_active == Some(current) {
                self.drag_active = None;
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
                    node.style.bg_overridden = true;
                }
            }
            Opt::Fg(color) => {
                if let Some(node) = self.node_mut(id) {
                    node.style.fg = color;
                    node.style.fg_overridden = true;
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
            Opt::Style(style_name) => {
                if let Some(node) = self.node_mut(id) {
                    node.named_style = Some(style_name);
                }
            }
            Opt::StyleOverride(spec) => {
                if let Some(node) = self.node_mut(id) {
                    node.style_override = Some(spec);
                }
            }
            Opt::Disabled(disabled) => {
                if let Some(node) = self.node_mut(id) {
                    node.disabled = disabled;
                }
                if disabled && self.focused == Some(id) {
                    self.traverse_focus(false);
                }
            }
            Opt::Image(img_id) => {
                if let Some(node) = self.node_mut(id) {
                    node.image = Some(img_id);
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
        if let Some(Binding::Text(var)) = binding {
            var.set(self, text);
        }
    }

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

    pub fn pack(&mut self, id: WidgetId, options: PackOpts) -> bool {
        self.set_manager(id, Manager::Pack(options))
    }

    pub fn grid(&mut self, id: WidgetId, options: GridOpts) -> bool {
        self.set_manager(id, Manager::Grid(options))
    }

    pub fn place(&mut self, id: WidgetId, options: PlaceOpts) -> bool {
        self.set_manager(id, Manager::Place(options))
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
                .filter(|child| self.node(*child).is_some_and(|n| n.manager.is_some()))
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
        children.iter().filter_map(|id| self.node(*id)).any(|n| matches!(n.manager, Some(Manager::Pack(_))))
    }

    /// Logical-pixel text size using the resolved font family, weight and slant.
    pub(crate) fn measure_with(&mut self, text: &str, style: &ResolvedStyle) -> Size {
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
            let resolved = self.resolved_style(id).unwrap_or_default();
            let node_snap = NodeSnapshot {
                list_items: node.list_items.clone(),
                text: node.text.clone(),
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
                self.images.get(&img_id).map(|p| p.width() as f32 + 6.0).unwrap_or(0.0)
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
                    let max_w = node_snap.list_items.iter().take(256).map(|item| self.measure_with(item, &resolved).width).fold(0.0_f32, f32::max);
                    Size::new(specified.unwrap_or(max_w + 24.0).max(80.0) + pad_w, (node_snap.list_height as f32 * line_h).max(30.0) + pad_h)
                }
                Kind::Scrollbar => Size::new(16.0 + pad_w, 100.0 + pad_h),
                Kind::Radiobutton => {
                    let measured = self.measure_with(&node_snap.text, &resolved);
                    Size::new(specified.unwrap_or(measured.width) + 32.0 + pad_w + image_extra_w, measured.height.max(16.0) + 8.0 + pad_h)
                }
                Kind::Canvas => Size::new(node_snap.canvas_size.0 + pad_w, node_snap.canvas_size.1 + pad_h),
                Kind::Spinbox => {
                    let text = format!("{}", node_snap.scale.and_then(|v| v.get(self)).unwrap_or(node_snap.range.0));
                    let m = self.measure_with(&text, &resolved);
                    Size::new(specified.unwrap_or(m.width + 50.0).max(70.0) + pad_w, resolved.font_size * 1.3 + 12.0 + pad_h)
                }
                Kind::ProgressBar => Size::new(node_snap.progress_length + pad_w, 20.0 + pad_h),
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
            let children = self.managed_children(id);

            let pack_children: Vec<_> = children.iter().copied()
                .filter(|c| self.node(*c).is_some_and(|n| matches!(n.manager, Some(Manager::Pack(_))))).collect();
            let grid_children: Vec<_> = children.iter().copied()
                .filter(|c| self.node(*c).is_some_and(|n| matches!(n.manager, Some(Manager::Grid(_))))).collect();
            let place_children: Vec<_> = children.iter().copied()
                .filter(|c| self.node(*c).is_some_and(|n| matches!(n.manager, Some(Manager::Place(_))))).collect();

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

    fn ensure_layout(&mut self) {
        if self.dirty {
            self.layout();
        }
    }

    fn is_focusable(&self, id: WidgetId) -> bool {
        self.node(id).is_some_and(|node| {
            !node.disabled && node.visible && (node.kind.focusable() || (node.kind == Kind::Canvas && node.scrollregion.is_some())) && node.clip.width > 0.0 && node.clip.height > 0.0
        })
    }

    pub fn widget_rect(&mut self, id: WidgetId) -> Option<Rect> {
        self.ensure_layout();
        self.node(id).filter(|node| node.visible).map(|node| node.rect)
    }

    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.focused.filter(|id| self.node(*id).is_some())
    }

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
            self.node(*id).is_some_and(|node| {
                node.visible && (node.kind.focusable() || node.kind == Kind::Canvas || node.kind == Kind::Scrollbar) && node.clip.contains(x, y)
            })
        })
    }

    fn check_state_transition(&mut self, id: WidgetId, old_state: State) {
        let new_state = self.widget_state(id);
        if old_state == new_state {
            return;
        }
        let Some(resolved) = self.resolved_style_in(id, new_state) else { return };
        let Some(previous) = self.resolved_style_in(id, old_state) else { return };
        if resolved.transition_ms > 0 {
            if let Some(node) = self.node_mut(id) {
                // Start from what is on screen: an in-flight transition, else the old state's style.
                let from_bg = node.transition.as_ref().map(|t| t.current_bg()).unwrap_or(previous.bg);
                let from_fg = node.transition.as_ref().map(|t| t.current_fg()).unwrap_or(previous.fg);
                let from_border = node.transition.as_ref().map(|t| t.current_border()).unwrap_or(previous.border_color);
                node.transition = Some(TransitionState::new(
                    from_bg, resolved.bg,
                    from_fg, resolved.fg,
                    from_border, resolved.border_color,
                    resolved.transition_ms,
                ));
            }
        }
    }

    #[doc(hidden)]
    pub fn sim_mouse_move(&mut self, x: f32, y: f32) {
        self.ensure_layout();
        self.pointer = (x, y);
        let hovered = self.hit_test(x, y);
        if self.hovered != hovered {
            let prev = self.hovered;
            self.hovered = hovered;
            self.dirty = true;
            if let Some(prev_id) = prev {
                let mut prev_st = self.widget_state(prev_id);
                prev_st.0 |= State::HOVER.bits(); // was hover
                self.check_state_transition(prev_id, prev_st);
            }
            if let Some(new_id) = hovered {
                let mut old_st = self.widget_state(new_id);
                old_st.0 &= !State::HOVER.bits(); // was not hover
                self.check_state_transition(new_id, old_st);
            }
        }

        if let Some(canvas_id) = self.drag_active {
            if let Some(node) = self.node(canvas_id) {
                let cx = x - node.rect.x + node.view.0;
                let cy = y - node.rect.y + node.view.1;
                self.dispatch_canvas_drag(canvas_id, cx, cy, DragPhase::Move);
            }
        }

        if let Some((scrollbar_id, start_my, _)) = self.scrollbar_drag {
            let bar_h = self.node(scrollbar_id).map(|n| n.rect.height).unwrap_or(0.0);
            if let Some(target_id) = self.node(scrollbar_id).and_then(|n| n.scroll_target) {
                if let Some((total, visible, _)) = self.scroll_metrics(target_id) {
                    let thumb_h = (bar_h * (visible / total)).clamp(16.0, bar_h);
                    let avail = bar_h - thumb_h;
                    if avail > 0.0 {
                        let start = self.scrollbar_drag_start;
                        let pos = start + (y - start_my) / avail * (total - visible);
                        self.set_scroll_pos(target_id, pos);
                    }
                }
            }
        }

        if let Some(id) = self.pressed {
            if self.node(id).is_some_and(|node| node.kind == Kind::Scale && !node.disabled) {
                self.drag_scale(id, x);
            }
        }
    }

    #[doc(hidden)]
    pub fn sim_mouse_down(&mut self) {
        self.ensure_layout();
        let (x, y) = self.pointer;
        let hit = self.hit_test(x, y);
        let is_disabled = hit.and_then(|id| self.node(id)).map(|n| n.disabled).unwrap_or(false);

        if is_disabled {
            self.hovered = hit;
            return;
        }

        self.hovered = hit;
        let prev_pressed = self.pressed;
        self.pressed = hit;
        if let Some(prev) = prev_pressed {
            self.check_state_transition(prev, self.widget_state(prev) | State::PRESSED);
        }
        if let Some(cur) = hit {
            self.check_state_transition(cur, self.widget_state(cur).without(State::PRESSED));
        }

        if hit.is_some_and(|id| self.is_focusable(id)) {
            let prev_focus = self.focused;
            self.focused = hit;
            if prev_focus != hit {
                if let Some(pf) = prev_focus {
                    self.check_state_transition(pf, self.widget_state(pf) | State::FOCUSED);
                }
                if let Some(nf) = hit {
                    self.check_state_transition(nf, self.widget_state(nf).without(State::FOCUSED));
                }
            }
        }
        if let Some(id) = hit {
            let Some(node) = self.node(id) else { return };
            match node.kind {
                Kind::Entry => {
                    let target = (x - node.rect.x - 6.0 + node.scroll_x).max(0.0);
                    let text = self.widget_text(id).unwrap_or_default();
                    let font = self.resolved_style(id).unwrap_or_default();
                    let boundaries = clusters(&text);
                    let (mut low, mut high) = (0, boundaries.len());
                    while low < high {
                        let mid = low + (high - low) / 2;
                        let end = boundaries.get(mid).copied().unwrap_or(text.len());
                        let width = self.measure_with(text.get(..end).unwrap_or_default(), &font).width;
                        if width < target {
                            low = mid + 1;
                        } else {
                            high = mid;
                        }
                    }
                    let right = boundaries.get(low).copied().unwrap_or(text.len());
                    let left = low.checked_sub(1).and_then(|i| boundaries.get(i)).copied().unwrap_or(0);
                    let rw = self.measure_with(text.get(..right).unwrap_or_default(), &font).width;
                    let lw = self.measure_with(text.get(..left).unwrap_or_default(), &font).width;
                    let cursor = if (rw - target).abs() < (lw - target).abs() { right } else { left };
                    if let Some(node) = self.node_mut(id) {
                        node.cursor = cursor;
                    }
                }
                Kind::Scale => self.drag_scale(id, x),
                Kind::Listbox => {
                    let line_h = (node.style.font_size * 1.3 + 4.0).max(16.0);
                    let local_y = y - node.rect.y;
                    let row_idx = node.list_scroll + (local_y / line_h).floor() as usize;
                    if row_idx < node.list_items.len() {
                        self.listbox_select(id, Some(row_idx));
                    }
                }
                Kind::Spinbox => {
                    let rx = x - node.rect.x;
                    let rw = node.rect.width;
                    let ry = y - node.rect.y;
                    let rh = node.rect.height;
                    if rx >= rw - 20.0 {
                        let (from, to) = node.range;
                        let step = if to >= from { 1 } else { -1 };
                        let old = node.scale.and_then(|v| v.get(self)).unwrap_or(from);
                        let val = if ry < rh / 2.0 {
                            old.saturating_add(step)
                        } else {
                            old.saturating_sub(step)
                        };
                        let val = val.clamp(from.min(to), from.max(to));
                        if let Some(var) = node.scale {
                            var.set(self, val);
                        }
                    }
                }
                Kind::Scrollbar => {
                    let ry = y - node.rect.y;
                    let bar_h = node.rect.height;
                    if let Some(target_id) = node.scroll_target {
                        if let Some((total, visible, pos)) = self.scroll_metrics(target_id) {
                            let thumb_h = (bar_h * (visible / total)).clamp(16.0, bar_h);
                            let avail = bar_h - thumb_h;
                            let max = total - visible;
                            let thumb_y = if max > 0.0 { avail * (pos / max) } else { 0.0 };
                            if ry < thumb_y {
                                self.set_scroll_pos(target_id, pos - visible);
                            } else if ry > thumb_y + thumb_h {
                                self.set_scroll_pos(target_id, pos + visible);
                            } else {
                                self.scrollbar_drag = Some((id, y, 0));
                                self.scrollbar_drag_start = pos;
                            }
                        }
                    }
                }
                Kind::Canvas => {
                    let (local_x, local_y) = (x - node.rect.x + node.view.0, y - node.rect.y + node.view.1);
                    if node.scrollregion.is_some() {
                        self.focused = Some(id);
                    }
                    self.drag_active = Some(id);
                    self.drag_start = (local_x, local_y);
                    self.dispatch_canvas_drag(id, local_x, local_y, DragPhase::Press);
                }
                _ => {}
            }
        }
        self.dirty = true;
    }

    #[doc(hidden)]
    pub fn sim_mouse_up(&mut self) {
        self.ensure_layout();
        let hit = self.hit_test(self.pointer.0, self.pointer.1);
        if let Some(canvas_id) = self.drag_active.take() {
            if let Some(node) = self.node(canvas_id) {
                let cx = self.pointer.0 - node.rect.x + node.view.0;
                let cy = self.pointer.1 - node.rect.y + node.view.1;
                self.dispatch_canvas_drag(canvas_id, cx, cy, DragPhase::Release);
            }
        }
        self.scrollbar_drag = None;
        if let Some(id) = self.pressed.take() {
            let is_disabled = self.node(id).map(|n| n.disabled).unwrap_or(false);
            if hit == Some(id) && !is_disabled {
                self.activate(id);
            }
            // It was pressed until `take()` above.
            self.check_state_transition(id, self.widget_state(id) | State::PRESSED);
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

    /// Mouse wheel in lines (positive = up/away). Listboxes scroll by rows, canvases by 48 px per line.
    #[doc(hidden)]
    pub fn sim_wheel(&mut self, dy_lines: f32) {
        self.sim_wheel_xy(0.0, dy_lines, false);
    }

    /// Wheel/touchpad scroll. `pixels` = true for precise touchpad deltas (logical px), else lines.
    /// Positive values scroll up/left (content moves down/right), as winit reports them.
    #[doc(hidden)]
    pub fn sim_wheel_xy(&mut self, dx: f32, dy: f32, pixels: bool) {
        self.ensure_layout();
        let (x, y) = self.pointer;
        let Some(mut id) = self.hit_test(x, y) else { return };
        // A Scrollbar forwards wheel events to its target.
        if let Some(t) = self.node(id).filter(|n| n.kind == Kind::Scrollbar).and_then(|n| n.scroll_target) {
            id = t;
        }
        let Some(node) = self.node(id) else { return };
        match node.kind {
            Kind::Listbox => {
                let rows = if pixels { dy / 20.0 } else { dy };
                if let Some((_, _, pos)) = self.scroll_metrics(id) {
                    self.set_scroll_pos(id, pos - rows.round());
                }
            }
            Kind::Canvas if node.scrollregion.is_some() => {
                let k = if pixels { 1.0 } else { WHEEL_LINE_PX };
                let (vx, vy) = node.view;
                self.canvas_set_view(id, vx - dx * k, vy - dy * k);
            }
            _ => {}
        }
    }

    /// Scroll state of a Listbox (rows) or scrollable Canvas (px, vertical): (total, visible, position).
    /// None when nothing overflows.
    pub(crate) fn scroll_metrics(&self, id: WidgetId) -> Option<(f32, f32, f32)> {
        let n = self.node(id)?;
        let (total, visible, pos) = match n.kind {
            Kind::Listbox => (n.list_items.len() as f32, n.list_height.max(1) as f32, n.list_scroll as f32),
            Kind::Canvas => (n.scrollregion?.1, n.rect.height, n.view.1),
            _ => return None,
        };
        (total > visible && visible > 0.0).then_some((total, visible, pos))
    }

    pub(crate) fn set_scroll_pos(&mut self, id: WidgetId, pos: f32) {
        let Some(kind) = self.node(id).map(|n| n.kind) else { return };
        match kind {
            Kind::Listbox => {
                if let Some((total, visible, _)) = self.scroll_metrics(id) {
                    let v = pos.round().clamp(0.0, total - visible) as usize;
                    if let Some(n) = self.node_mut(id) { n.list_scroll = v; }
                    self.dirty = true;
                }
            }
            Kind::Canvas => {
                let vx = self.node(id).map(|n| n.view.0).unwrap_or(0.0);
                self.canvas_set_view(id, vx, pos);
            }
            _ => {}
        }
    }

    fn canvas_set_view(&mut self, id: WidgetId, x: f32, y: f32) {
        let Some(n) = self.node(id) else { return };
        let (rw, rh) = n.scrollregion.unwrap_or((n.rect.width, n.rect.height));
        let nx = x.clamp(0.0, (rw - n.rect.width).max(0.0));
        let ny = y.clamp(0.0, (rh - n.rect.height).max(0.0));
        if (nx, ny) != n.view {
            if let Some(n) = self.node_mut(id) { n.view = (nx, ny); }
            self.dirty = true;
        }
    }

    /// Sets the scrollable content size of a Canvas (tkinter `scrollregion=(0, 0, w, h)`).
    /// The current view is clamped to the new region.
    pub fn canvas_scrollregion(&mut self, id: WidgetId, width: f32, height: f32) -> bool {
        self.ensure_layout();
        let Some(n) = self.node_mut(id) else { return false };
        if n.kind != Kind::Canvas { return false; }
        n.scrollregion = Some((width.max(0.0), height.max(0.0)));
        let v = n.view;
        self.canvas_set_view(id, v.0, v.1);
        self.dirty = true;
        true
    }

    /// Current scroll offset of a Canvas in logical px (x, y).
    pub fn canvas_view(&self, id: WidgetId) -> Option<(f32, f32)> {
        self.node(id).filter(|n| n.kind == Kind::Canvas).map(|n| n.view)
    }

    fn view_fractions(&mut self, id: WidgetId, vertical: bool) -> (f32, f32) {
        self.ensure_layout();
        let Some(n) = self.node(id) else { return (0.0, 1.0) };
        let (total, visible, pos) = match (n.kind, vertical) {
            (Kind::Listbox, true) => (n.list_items.len() as f32, n.list_height.max(1) as f32, n.list_scroll as f32),
            (Kind::Canvas, true) => (n.scrollregion.map_or(n.rect.height, |r| r.1), n.rect.height, n.view.1),
            (Kind::Canvas, false) => (n.scrollregion.map_or(n.rect.width, |r| r.0), n.rect.width, n.view.0),
            _ => return (0.0, 1.0),
        };
        if total <= 0.0 || visible >= total { return (0.0, 1.0); }
        ((pos / total).clamp(0.0, 1.0), ((pos + visible) / total).clamp(0.0, 1.0))
    }

    /// tkinter `yview()`: visible fraction (first, last) of a Listbox or Canvas.
    pub fn yview(&mut self, id: WidgetId) -> (f32, f32) { self.view_fractions(id, true) }
    /// tkinter `xview()` for a Canvas.
    pub fn xview(&mut self, id: WidgetId) -> (f32, f32) { self.view_fractions(id, false) }

    /// tkinter `yview_moveto(fraction)`: scrolls so `fraction` of the content is above the view.
    pub fn yview_moveto(&mut self, id: WidgetId, fraction: f32) {
        self.ensure_layout();
        let Some(n) = self.node(id) else { return };
        match n.kind {
            Kind::Listbox => { let t = n.list_items.len() as f32; self.set_scroll_pos(id, fraction * t); }
            Kind::Canvas => { let (vx, t) = (n.view.0, n.scrollregion.map_or(0.0, |r| r.1)); self.canvas_set_view(id, vx, fraction * t); }
            _ => {}
        }
    }

    /// tkinter `xview_moveto(fraction)` for a Canvas.
    pub fn xview_moveto(&mut self, id: WidgetId, fraction: f32) {
        self.ensure_layout();
        let Some(n) = self.node(id).filter(|n| n.kind == Kind::Canvas) else { return };
        let (t, vy) = (n.scrollregion.map_or(0.0, |r| r.0), n.view.1);
        self.canvas_set_view(id, fraction * t, vy);
    }

    /// tkinter `yview_scroll(n, "units"|"pages")`. Units are rows (Listbox) or 48 px (Canvas).
    pub fn yview_scroll(&mut self, id: WidgetId, n: i32, what: ScrollUnit) {
        self.ensure_layout();
        let Some(node) = self.node(id) else { return };
        let (unit, page) = match node.kind {
            Kind::Listbox => (1.0, node.list_height.max(1) as f32),
            Kind::Canvas => (WHEEL_LINE_PX, (node.rect.height - WHEEL_LINE_PX).max(WHEEL_LINE_PX)),
            _ => return,
        };
        let step = n as f32 * if what == ScrollUnit::Pages { page } else { unit };
        let pos = match node.kind { Kind::Listbox => node.list_scroll as f32, _ => node.view.1 };
        if node.kind == Kind::Listbox { self.set_scroll_pos(id, pos + step) } else { let vx = node.view.0; self.canvas_set_view(id, vx, pos + step) }
    }

    /// tkinter `xview_scroll(n, "units"|"pages")` for a Canvas.
    pub fn xview_scroll(&mut self, id: WidgetId, n: i32, what: ScrollUnit) {
        self.ensure_layout();
        let Some(node) = self.node(id).filter(|n| n.kind == Kind::Canvas) else { return };
        let step = n as f32 * if what == ScrollUnit::Pages { (node.rect.width - WHEEL_LINE_PX).max(WHEEL_LINE_PX) } else { WHEEL_LINE_PX };
        let (vx, vy) = node.view;
        self.canvas_set_view(id, vx + step, vy);
    }

    fn dispatch_canvas_drag(&mut self, id: WidgetId, cx: f32, cy: f32, phase: DragPhase) {
        let Some(node) = self.node_mut(id) else { return };
        if node.canvas_drag_running {
            return;
        }
        let Some(mut cb) = node.canvas_drag.take() else { return };
        node.canvas_drag_running = true;
        cb(self, cx, cy, phase);
        if let Some(node) = self.node_mut(id) {
            node.canvas_drag_running = false;
            if node.canvas_drag.is_none() {
                node.canvas_drag = Some(cb);
            }
        }
    }

    fn run_command(&mut self, id: WidgetId) {
        let Some(node) = self.node_mut(id) else { return };
        if node.disabled || node.command_running {
            return;
        }
        let Some(mut command) = node.command.take() else { return };
        node.command_running = true;
        command(self);
        if let Some(node) = self.node_mut(id) {
            node.command_running = false;
            if node.command.is_none() {
                node.command = Some(command);
            }
        }
    }

    fn activate(&mut self, id: WidgetId) {
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        let (kind, check) = (node.kind, node.check);
        if kind == Kind::Checkbutton {
            if let Some(var) = check {
                if let Some(value) = var.get(self) {
                    var.set(self, !value);
                }
            }
        } else if kind == Kind::Radiobutton {
            if let Some(var) = node.radio_var {
                var.set(self, node.radio_value);
            }
        }
        if matches!(kind, Kind::Button | Kind::Checkbutton | Kind::Radiobutton) {
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
        if node.disabled {
            return;
        }
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
        if node.disabled {
            return;
        }
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
        if node.disabled {
            return;
        }
        if node.kind == Kind::Canvas && node.scrollregion.is_some() {
            let total = node.scrollregion.map_or(0.0, |r| r.1);
            match key {
                Key::PageUp => self.yview_scroll(id, -1, ScrollUnit::Pages),
                Key::PageDown | Key::Space => self.yview_scroll(id, 1, ScrollUnit::Pages),
                Key::Up => self.yview_scroll(id, -1, ScrollUnit::Units),
                Key::Down => self.yview_scroll(id, 1, ScrollUnit::Units),
                Key::Left => self.xview_scroll(id, -1, ScrollUnit::Units),
                Key::Right => self.xview_scroll(id, 1, ScrollUnit::Units),
                Key::Home => self.yview_moveto(id, 0.0),
                Key::End => { let _ = total; self.yview_moveto(id, 1.0) }
                _ => {}
            }
            return;
        }
        match node.kind {
            Kind::Entry => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else {
                    self.edit_entry(id, Some(key), "");
                }
            }
            Kind::Button | Kind::Checkbutton | Kind::Radiobutton if matches!(key, Key::Enter | Key::Space) => {
                self.activate(id);
            }
            Kind::Scale if matches!(key, Key::Left | Key::Right) => {
                let (from, to) = node.range;
                let old = node.scale.and_then(|var| var.get(self)).unwrap_or(from);
                let increasing = (key == Key::Right) == (to >= from);
                self.set_scale(id, if increasing { old.saturating_add(1) } else { old.saturating_sub(1) });
            }
            Kind::Listbox => {
                let total = node.list_items.len();
                if total > 0 {
                    let sel = node.list_selection;
                    let new_sel = match key {
                        Key::Up => Some(sel.unwrap_or(0).saturating_sub(1)),
                        Key::Down => Some((sel.map_or(0, |s| s + 1)).min(total - 1)),
                        Key::Home => Some(0),
                        Key::End => Some(total - 1),
                        _ => sel,
                    };
                    if new_sel != sel {
                        self.listbox_select(id, new_sel);
                    }
                }
            }
            Kind::Spinbox if matches!(key, Key::Up | Key::Down) => {
                let (from, to) = node.range;
                let step = if to >= from { 1 } else { -1 };
                let old = node.scale.and_then(|v| v.get(self)).unwrap_or(from);
                let val = if key == Key::Up {
                    old.saturating_add(step)
                } else {
                    old.saturating_sub(step)
                };
                let val = val.clamp(from.min(to), from.max(to));
                if let Some(var) = node.scale {
                    var.set(self, val);
                }
            }
            _ => {}
        }
    }

    #[doc(hidden)]
    pub fn sim_text(&mut self, text: &str) {
        self.ensure_layout();
        if let Some(id) = self.focused {
            if self.node(id).is_some_and(|node| node.kind == Kind::Entry && !node.disabled) {
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
        self.drag_active = None;
        self.scrollbar_drag = None;
        self.dirty = true;
    }

    // Listbox methods
    pub fn listbox_insert(&mut self, id: WidgetId, index: usize, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        let item = text.into();
        if index >= node.list_items.len() {
            node.list_items.push(item);
        } else {
            node.list_items.insert(index, item);
            if let Some(sel) = node.list_selection {
                if index <= sel {
                    node.list_selection = Some(sel + 1);
                }
            }
        }
        self.dirty = true;
        true
    }

    pub fn listbox_delete(&mut self, id: WidgetId, first: usize, last: usize) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox || first >= node.list_items.len() {
            return false;
        }
        let end = (last.saturating_add(1)).min(node.list_items.len());
        if first < end {
            node.list_items.drain(first..end);
            if let Some(sel) = node.list_selection {
                if sel >= first && sel < end {
                    node.list_selection = None;
                } else if sel >= end {
                    node.list_selection = Some(sel - (end - first));
                }
            }
            let total = node.list_items.len();
            let vis = node.list_height.max(1);
            let max_scroll = total.saturating_sub(vis);
            node.list_scroll = node.list_scroll.min(max_scroll);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn listbox_get(&self, id: WidgetId) -> Vec<String> {
        self.node(id)
            .filter(|n| n.kind == Kind::Listbox)
            .map(|n| n.list_items.clone())
            .unwrap_or_default()
    }

    pub fn listbox_curselection(&self, id: WidgetId) -> Option<usize> {
        self.node(id)
            .filter(|n| n.kind == Kind::Listbox)
            .and_then(|n| n.list_selection)
    }

    pub fn listbox_select(&mut self, id: WidgetId, index: Option<usize>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        if let Some(idx) = index {
            if idx >= node.list_items.len() {
                return false;
            }
            node.list_selection = Some(idx);
            let vis = node.list_height.max(1);
            if idx < node.list_scroll {
                node.list_scroll = idx;
            } else if idx >= node.list_scroll + vis {
                node.list_scroll = idx + 1 - vis;
            }
        } else {
            node.list_selection = None;
        }
        self.dirty = true;
        self.run_command(id);
        true
    }

    // Scrollbar methods
    /// Links a vertical Scrollbar to a Listbox or a scrollable Canvas (tkinter `yscrollcommand`/`command`).
    pub fn link_scrollbar(&mut self, scrollbar: WidgetId, listbox: WidgetId) -> bool {
        let is_sb = self.node(scrollbar).is_some_and(|n| n.kind == Kind::Scrollbar);
        let is_lb = self.node(listbox).is_some_and(|n| matches!(n.kind, Kind::Listbox | Kind::Canvas));
        if !is_sb || !is_lb {
            return false;
        }
        if let Some(node) = self.node_mut(scrollbar) {
            node.scroll_target = Some(listbox);
        }
        self.dirty = true;
        true
    }

    // Canvas methods
    pub fn canvas_create_rect(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Rect { x0, y0, x1, y1, color })
    }

    pub fn canvas_create_oval(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Oval { x0, y0, x1, y1, color })
    }

    pub fn canvas_create_line(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Line { x0, y0, x1, y1, color })
    }

    pub fn canvas_create_text(&mut self, id: WidgetId, x: f32, y: f32, text: impl Into<String>, color: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Text { x, y, text: text.into(), color })
    }

    fn canvas_add_item(&mut self, id: WidgetId, kind: CanvasItemKind) -> Option<CanvasItem> {
        const MAX_CANVAS_ITEMS: usize = 1 << 20;
        let node = self.node_mut(id)?;
        if node.kind != Kind::Canvas {
            return None;
        }
        let items = &mut node.canvas_items;
        let seq = items.iter().map(|slot| slot.seq).max().map_or(0, |max| max.saturating_add(1));
        let free = items.iter().position(|slot| slot.item.is_none() && slot.gen < u32::MAX);
        let item = if let Some(index) = free {
            let slot = items.get_mut(index)?;
            slot.item = Some(kind);
            slot.seq = seq;
            CanvasItem { index: u32::try_from(index).ok()?, gen: slot.gen }
        } else {
            if items.len() >= MAX_CANVAS_ITEMS {
                return None;
            }
            let index = u32::try_from(items.len()).ok()?;
            items.push(CanvasItemSlot { gen: 0, seq, item: Some(kind) });
            CanvasItem { index, gen: 0 }
        };
        self.dirty = true;
        Some(item)
    }

    pub fn canvas_clear(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        for slot in node.canvas_items.iter_mut().filter(|slot| slot.item.is_some()) {
            slot.item = None;
            slot.gen = slot.gen.saturating_add(1);
        }
        self.dirty = true;
        true
    }

    pub fn canvas_delete(&mut self, id: WidgetId, item: CanvasItem) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        let Some(slot) = node.canvas_items.get_mut(item.index as usize) else { return false };
        if slot.gen != item.gen || slot.item.is_none() {
            return false;
        }
        slot.item = None;
        slot.gen = slot.gen.saturating_add(1);
        self.dirty = true;
        true
    }

    pub fn canvas_move(&mut self, id: WidgetId, item: CanvasItem, dx: f32, dy: f32) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        let Some(slot) = node.canvas_items.get_mut(item.index as usize) else { return false };
        if slot.gen != item.gen {
            return false;
        }
        let Some(kind) = slot.item.as_mut() else { return false };
        kind.translate(dx, dy);
        self.dirty = true;
        true
    }

    pub fn canvas_coords(&self, id: WidgetId, item: CanvasItem) -> Option<[f32; 4]> {
        let node = self.node(id)?;
        if node.kind != Kind::Canvas {
            return None;
        }
        let slot = node.canvas_items.get(item.index as usize)?;
        if slot.gen != item.gen {
            return None;
        }
        match slot.item.as_ref()? {
            CanvasItemKind::Rect { x0, y0, x1, y1, .. }
            | CanvasItemKind::Oval { x0, y0, x1, y1, .. }
            | CanvasItemKind::Line { x0, y0, x1, y1, .. } => Some([*x0, *y0, *x1, *y1]),
            CanvasItemKind::Text { x, y, .. } | CanvasItemKind::TextEx { x, y, .. } => Some([*x, *y, *x, *y]),
            CanvasItemKind::RoundRect { x0, y0, x1, y1, .. } | CanvasItemKind::Ring { x0, y0, x1, y1, .. } => Some([*x0, *y0, *x1, *y1]),
            CanvasItemKind::Path { points, .. } => {
                let (mut a, mut b, mut c, mut d) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for (x, y) in points { a = a.min(*x); b = b.min(*y); c = c.max(*x); d = d.max(*y); }
                if points.is_empty() { None } else { Some([a, b, c, d]) }
            }
        }
    }

    /// Filled rounded rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_round_rect(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, radius: f32, fill: Color) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::RoundRect { x0, y0, x1, y1, radius, fill: Some(fill), outline: None })
    }

    /// Rounded rectangle with optional fill and an outline of `width` logical pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_round_rect_outline(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, radius: f32, fill: Option<Color>, outline: Color, width: f32) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::RoundRect { x0, y0, x1, y1, radius, fill, outline: Some((outline, width)) })
    }

    /// Stroked polyline through `points` (e.g. sparklines, icon strokes). Needs at least two points.
    pub fn canvas_create_polyline(&mut self, id: WidgetId, points: &[(f32, f32)], color: Color, width: f32) -> Option<CanvasItem> {
        if points.len() < 2 { return None; }
        self.canvas_add_item(id, CanvasItemKind::Path { points: points.to_vec(), color, width, filled: false })
    }

    /// Filled closed polygon. Needs at least three points.
    pub fn canvas_create_polygon(&mut self, id: WidgetId, points: &[(f32, f32)], color: Color) -> Option<CanvasItem> {
        if points.len() < 3 { return None; }
        self.canvas_add_item(id, CanvasItemKind::Path { points: points.to_vec(), color, width: 0.0, filled: true })
    }

    /// Ellipse outline inside the box, stroked `width` logical pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_ring(&mut self, id: WidgetId, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::Ring { x0, y0, x1, y1, color, width })
    }

    /// Text with its own pixel size and weight; `(x, y)` is the top-left.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_text_ex(&mut self, id: WidgetId, x: f32, y: f32, text: impl Into<String>, color: Color, size: f32, weight: u16) -> Option<CanvasItem> {
        self.canvas_add_item(id, CanvasItemKind::TextEx { x, y, text: text.into(), color, size: size.max(1.0), weight: weight.clamp(100, 900) })
    }

    /// Measures text in logical pixels (width, height) for layout/centering on a Canvas.
    pub fn text_size(&mut self, text: &str, size: f32, weight: u16) -> (f32, f32) {
        let family = self.style_engine.tokens().font_family.clone();
        let s = self.text_engine.measure_styled(text, size, family.as_deref(), weight.clamp(100, 900), false);
        (s.width, s.height)
    }

    pub fn canvas_items(&self, id: WidgetId) -> usize {
        self.node(id)
            .filter(|n| n.kind == Kind::Canvas)
            .map(|n| n.canvas_items.iter().filter(|s| s.item.is_some()).count())
            .unwrap_or(0)
    }

    pub fn canvas_bind_drag(&mut self, id: WidgetId, cb: impl FnMut(&mut Tk, f32, f32, DragPhase) + 'static) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas {
            return false;
        }
        node.canvas_drag = Some(Box::new(cb));
        true
    }
}

pub(crate) fn kind_class_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Frame => "TFrame",
        Kind::Label => "TLabel",
        Kind::Button => "TButton",
        Kind::Entry => "TEntry",
        Kind::Checkbutton => "TCheckbutton",
        Kind::Scale => "TScale",
        Kind::Listbox => "TListbox",
        Kind::Scrollbar => "TScrollbar",
        Kind::Radiobutton => "TRadiobutton",
        Kind::Canvas => "TCanvas",
        Kind::Spinbox => "TSpinbox",
        Kind::ProgressBar => "TProgressbar",
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

struct NodeSnapshot {
    list_items: Vec<String>,
    text: String,
    scale: Option<crate::Var<i64>>,
    range: (i64, i64),
    canvas_size: (f32, f32),
    progress_length: f32,
    list_height: usize,
    parent: Option<WidgetId>,
    kind: Kind,
    style: crate::widgets::Style,
    image: Option<ImageId>,
}
