//! Core toolkit state: the widget arena, layout driver, headless event
//! simulation (`sim_*`), styling cascade, and per-widget APIs (entry, listbox,
//! canvas, text, combobox, notebook, scrolling). The real window backend lives
//! in `app.rs`; everything here is backend-independent and unit-testable.

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use winit::event_loop::EventLoopProxy;

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

pub(crate) type PostedTask = Box<dyn FnOnce(&mut Tk) + Send + 'static>;
pub(crate) type IdleTask = Box<dyn FnOnce(&mut Tk) + 'static>;

#[derive(Debug)]
pub(crate) struct RtkUserEvent;

pub(crate) struct ProxyShared {
    pub(crate) queue: Mutex<VecDeque<PostedTask>>,
    pub(crate) waker: Mutex<Option<EventLoopProxy<RtkUserEvent>>>,
}

/// A thread-safe handle for posting work to the GUI main thread.
#[derive(Clone)]
pub struct TkProxy {
    shared: Arc<ProxyShared>,
}

impl TkProxy {
    /// Queues a closure to execute on the main thread and wakes the event loop.
    pub fn post(&self, f: impl FnOnce(&mut Tk) + Send + 'static) {
        self.shared.queue.lock().unwrap_or_else(|e| e.into_inner()).push_back(Box::new(f));
        let waker = self.shared.waker.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(proxy) = waker.as_ref() {
            let _ = proxy.send_event(RtkUserEvent);
        }
    }
}

const MAX_ENTRY_BYTES: usize = 64 * 1024;

/// Keys understood by the real backend and the headless event harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// Deletes the character/cluster before the cursor.
    Backspace,
    /// Deletes the character/cluster after the cursor.
    Delete,
    /// Moves the cursor/selection left (or decrements a Scale).
    Left,
    /// Moves the cursor/selection right (or increments a Scale).
    Right,
    /// Moves up one line/row/item.
    Up,
    /// Moves down one line/row/item.
    Down,
    /// Jumps to the start of the line/list/view.
    Home,
    /// Jumps to the end of the line/list/view.
    End,
    /// Moves keyboard focus to the next focusable widget.
    Tab,
    /// Moves keyboard focus to the previous focusable widget.
    ShiftTab,
    /// Activates buttons, inserts a newline in `Text`, accepts a modal dialog.
    Enter,
    /// Activates buttons or inserts a space.
    Space,
    /// Scrolls a scrollable Canvas up one page.
    PageUp,
    /// Scrolls a scrollable Canvas down one page.
    PageDown,
    /// Closes an open Combobox or cancels a modal dialog.
    Escape,
}

/// Scroll step for `yview_scroll`/`xview_scroll`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollUnit {
    /// One row (Listbox/Text) or [`WHEEL_LINE_PX`] logical px (Canvas).
    Units,
    /// One visible page of rows/pixels.
    Pages,
}

/// Logical px a Canvas scrolls per wheel line / scroll unit.
const WHEEL_LINE_PX: f32 = 48.0;

/// The toolkit instance: owns every widget, variable, style and input state.
///
/// Create one with [`Tk::new`], build a widget tree with the constructors in
/// [`crate::widgets`], then either run the real window loop or drive it
/// headlessly with the `sim_*` methods.
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
    pub(crate) pointer: (f32, f32),
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
    pub(crate) hover_timer_ms: u32,
    pub(crate) tooltip_visible: bool,
    pub(crate) open_combobox: Option<WidgetId>,
    pub(crate) proxy_shared: Arc<ProxyShared>,
    pub(crate) idle_queue: VecDeque<IdleTask>,
    /// Root of the current modal overlay subtree, if any.
    pub modal: Option<WidgetId>,
    pub(crate) modal_cancel: Option<Box<dyn FnMut(&mut Tk)>>,
    pub(crate) modal_accept: Option<Box<dyn FnMut(&mut Tk)>>,
    pub(crate) modal_entry_changed: Option<Box<dyn FnMut(&mut Tk, WidgetId, &str)>>,
}

impl Default for Tk {
    fn default() -> Self {
        Self::new()
    }
}

impl Tk {
    /// Creates a toolkit with a root `Frame`, the Light theme and a 640×480 viewport.
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
            hover_timer_ms: 0,
            tooltip_visible: false,
            open_combobox: None,
            proxy_shared: Arc::new(ProxyShared {
                queue: Mutex::new(VecDeque::new()),
                waker: Mutex::new(None),
            }),
            idle_queue: VecDeque::new(),
            modal: None,
            modal_cancel: None,
            modal_accept: None,
            modal_entry_changed: None,
        }
    }

    /// Obtains a thread-safe proxy to post callbacks from background threads.
    pub fn proxy(&self) -> TkProxy {
        TkProxy { shared: self.proxy_shared.clone() }
    }

    /// Queues a closure to run once on the main thread when idle or on next drain.
    pub fn after_idle(&mut self, f: impl FnOnce(&mut Tk) + 'static) {
        self.idle_queue.push_back(Box::new(f));
    }

    /// Drains posted and idle tasks reentrantly, returning the number processed.
    ///
    /// Tasks queued while draining are run in the same call, up to a bounded
    /// number of passes so a self-re-posting task cannot hang the main thread;
    /// anything left over runs on the next call.
    pub fn process_posted(&mut self) -> usize {
        const MAX_PASSES: usize = 256;
        let mut count = 0;
        for _ in 0..MAX_PASSES {
            let mut tasks: VecDeque<PostedTask> = {
                let mut q = self.proxy_shared.queue.lock().unwrap_or_else(|e| e.into_inner());
                std::mem::take(&mut *q)
            };
            let mut idles: VecDeque<IdleTask> = std::mem::take(&mut self.idle_queue);
            if tasks.is_empty() && idles.is_empty() {
                break;
            }
            while let Some(task) = tasks.pop_front() {
                task(self);
                count += 1;
            }
            while let Some(idle) = idles.pop_front() {
                idle(self);
                count += 1;
            }
        }
        if count > 0 {
            self.dirty = true;
        }
        count
    }

    /// Returns true if `id` is `ancestor` or a descendant of `ancestor`.
    pub(crate) fn is_descendant_of(&self, mut id: WidgetId, ancestor: WidgetId) -> bool {
        if id == ancestor {
            return true;
        }
        while let Some(node) = self.node(id) {
            if let Some(pid) = node.parent {
                if pid == ancestor {
                    return true;
                }
                id = pid;
            } else {
                break;
            }
        }
        false
    }

    /// Returns the root (toplevel) widget, which always exists.
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

    /// Returns true if `id` refers to a live widget (tkinter `winfo_exists`).
    pub fn contains(&self, id: WidgetId) -> bool {
        self.node(id).is_some()
    }

    /// Returns the active theme.
    pub fn theme(&self) -> Theme {
        self.theme
    }

    /// Returns the design tokens (palette, fonts, radii) of the active theme.
    pub fn tokens(&self) -> &Tokens {
        self.style_engine.tokens()
    }

    /// Replaces the design tokens without changing `theme()`.
    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.style_engine.set_tokens(tokens);
        self.dirty = true;
    }

    /// Switches the theme, resetting tokens and the legacy per-widget colours
    /// that were not explicitly overridden with `Opt::Bg`/`Opt::Fg`.
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
            Theme::ModernDark => Tokens::modern_dark(),
            Theme::ModernLight => Tokens::modern_light(),
        };
        self.style_engine.set_tokens(tokens);

        let is_dark = matches!(theme, Theme::Dark | Theme::ModernDark);
        let is_hc = theme == Theme::HighContrast;
        for slot in &mut self.nodes {
            if let Some(node) = &mut slot.node {
                let (dark_bg, dark_fg) = dark_default_colors(node.kind);
                if !node.style.bg_overridden {
                    node.style.bg = if is_hc {
                        Color::BLACK
                    } else if is_dark {
                        dark_bg
                    } else {
                        match node.kind {
                            Kind::Entry | Kind::Listbox => Color::WHITE,
                            Kind::Canvas => Color::rgb(240, 240, 240),
                            _ => Color::BG,
                        }
                    };
                }
                if !node.style.fg_overridden {
                    node.style.fg = if is_hc {
                        Color::WHITE
                    } else if is_dark {
                        dark_fg
                    } else {
                        Color::FG
                    };
                }
            }
        }
    }

    /// Sets the brand accent color and updates derived tokens and styles.
    pub fn set_accent(&mut self, accent: Color) {
        let mut tokens = self.style_engine.tokens().clone();
        tokens.palette.accent = accent;
        tokens.palette.focus = accent;
        tokens.palette.select_bg = Color::rgba(accent.r, accent.g, accent.b, 64);
        self.style_engine.set_tokens(tokens);
        self.dirty = true;
    }

    // -----------------------------------------------------------------------
    // Style API
    // -----------------------------------------------------------------------

    /// ttk `style.configure(name, **opts)`: sets the base spec of a named style.
    pub fn style_configure(&mut self, style_name: impl Into<String>, spec: StyleSpec) {
        self.style_engine.configure(style_name, spec);
        self.dirty = true;
    }

    /// ttk `style.map(name, state=[...])`: sets the spec applied while `state` is active.
    pub fn style_map(&mut self, style_name: impl Into<String>, state: State, spec: StyleSpec) {
        self.style_engine.map(style_name, state, spec);
        self.dirty = true;
    }

    /// Parses a CSS-like stylesheet into the style engine, returning the rule count.
    pub fn load_stylesheet(&mut self, css: &str) -> Result<usize, StyleError> {
        let parser = StylesheetParser::new(css);
        let count = parser.parse(&mut self.style_engine)?;
        self.dirty = true;
        Ok(count)
    }

    /// Reads and parses a stylesheet file; I/O errors are reported as `StyleError` at line 0.
    pub fn load_stylesheet_file(&mut self, path: impl AsRef<Path>) -> Result<usize, StyleError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| StyleError {
            line: 0,
            message: format!("cannot read file {}: {e}", path.display()),
        })?;
        self.load_stylesheet(&content)
    }

    /// Loads a stylesheet and remembers its mtime so [`Tk::poll_stylesheet`] can hot-reload it.
    /// Returns false if the file cannot be stat'ed.
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

    /// Reloads the watched stylesheet if its mtime advanced. Returns true when reloaded.
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

    /// Loads a PNG, either from a file path (UTF-8 string that names a readable file)
    /// or from in-memory PNG bytes. Returns `None` if decoding fails.
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

    /// Registers a raw RGBA buffer as an image, premultiplying color channels by alpha.
    pub fn register_image_rgba(&mut self, w: u32, h: u32, rgba: &[u8]) -> Option<ImageId> {
        if w == 0 || h == 0 || rgba.len() != (w as usize) * (h as usize) * 4 {
            return None;
        }
        let mut pixmap = Pixmap::new(w, h)?;
        let data = pixmap.data_mut();
        for (dst, src) in data.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
            let a = u32::from(src[3]);
            dst[0] = (u32::from(src[0]) * a / 255) as u8;
            dst[1] = (u32::from(src[1]) * a / 255) as u8;
            dst[2] = (u32::from(src[2]) * a / 255) as u8;
            dst[3] = src[3];
        }
        let id = ImageId(self.next_image_id);
        self.next_image_id = self.next_image_id.saturating_add(1);
        self.images.insert(id, pixmap);
        Some(id)
    }

    /// Current ttk-style state flags (hover/pressed/focused/checked/...) of a widget.
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
        if node.selected {
            s |= State::SELECTED;
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

    /// Background colour currently shown, including an in-flight transition.
    #[doc(hidden)]
    pub fn displayed_bg(&self, id: WidgetId) -> Option<Color> {
        let node = self.node(id)?;
        match node.transition.as_ref() {
            Some(transition) => Some(transition.current_bg()),
            None => self.resolved_style(id).map(|style| style.bg),
        }
    }

    /// Resolves the concrete style for widget `id` by cascading:
    /// theme default -> "TClass" -> "Named.TClass" -> Frame inheritance -> widget override -> state map -> legacy Opt.
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

        if node.monospace {
            res.font_family = Some("monospace".into());
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
        if self.hovered.is_some() {
            self.hover_timer_ms = self.hover_timer_ms.saturating_add(ms);
            if self.hover_timer_ms >= 500 && !self.tooltip_visible {
                let has_tip = self.hovered
                    .and_then(|id| self.node(id))
                    .and_then(|n| n.tooltip.as_ref())
                    .is_some();
                if has_tip {
                    self.tooltip_visible = true;
                    self.dirty = true;
                }
            }
        }

        for slot in &mut self.nodes {
            if let Some(node) = &mut slot.node {
                if let Some(trans) = &mut node.transition {
                    let done = trans.advance(ms);
                    if done {
                        node.transition = None;
                    }
                    self.dirty = true;
                }
                if node.kind == Kind::ProgressBar && node.progress_indeterminate && node.visible {
                    node.progress_phase = (node.progress_phase + (ms as f32) / 1000.0) % 1.0;
                    self.dirty = true;
                }
            }
        }
    }

    /// True while any style transition or indeterminate progress bar needs frames.
    pub fn is_animating(&self) -> bool {
        self.nodes.iter().filter_map(|s| s.node.as_ref()).any(|n| {
            n.transition.is_some() || (n.kind == Kind::ProgressBar && n.progress_indeterminate && n.visible)
        })
    }

    /// Sets the window title (tkinter `root.title`).
    pub fn title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
        self.dirty = true;
    }

    /// Requests an initial window size in physical px (tkinter `root.geometry("WxH")`).
    pub fn geometry(&mut self, width: u32, height: u32) {
        self.window_size = Some((width, height));
        self.sim_resize(width, height);
    }

    /// Asks the event loop to exit after the current iteration (tkinter `root.quit`).
    pub fn quit(&mut self) {
        self.quit_requested = true;
    }

    /// True once [`Tk::quit`] was called or the root was destroyed.
    pub fn is_quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// True if layout/paint is pending.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn add(
        &mut self,
        kind: Kind,
        parent: WidgetId,
        text: String,
    ) -> Result<WidgetId, RtkError> {
        if !self.node(parent).is_some_and(|node| matches!(node.kind, Kind::Frame | Kind::Notebook)) {
            return Err(RtkError::InvalidParent);
        }
        let mut node = WidgetNode::new(kind, Some(parent), text);
        let is_hc = self.theme == Theme::HighContrast;
        let is_dark = matches!(self.theme, Theme::Dark | Theme::ModernDark);
        if is_hc {
            node.style.bg = Color::BLACK;
            node.style.fg = Color::WHITE;
        } else if is_dark {
            let (bg, fg) = dark_default_colors(kind);
            node.style.bg = bg;
            node.style.fg = fg;
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

    /// Destroys a widget and its whole subtree, releasing their IDs. Destroying
    /// the root requests quit. Returns false if `id` is not live.
    pub fn destroy(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node(id) else { return false };
        let parent = node.parent;
        if let Some(parent) = parent.and_then(|parent| self.node_mut(parent)) {
            parent.children.retain(|child| *child != id);
            if parent.kind == Kind::Notebook {
                if let Some(pos) = parent.notebook_pages.iter().position(|(pid, _)| *pid == id) {
                    parent.notebook_pages.remove(pos);
                    if pos < parent.notebook_selected {
                        parent.notebook_selected -= 1;
                    }
                    let last = parent.notebook_pages.len().saturating_sub(1);
                    parent.notebook_selected = parent.notebook_selected.min(last);
                }
            }
        }
        if self.modal.is_some_and(|m| self.is_descendant_of(m, id)) {
            self.modal = None;
        }
        if self.scrollbar_drag.is_some_and(|(sb, _, _)| self.is_descendant_of(sb, id)) {
            self.scrollbar_drag = None;
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
            if self.open_combobox == Some(current) {
                self.open_combobox = None;
            }
        }
        if id == self.root() {
            self.quit();
        }
        self.dirty = true;
        true
    }

    /// Changes one option of a widget (tkinter `widget.config(...)`). Returns
    /// false if the widget is gone or the option is invalid for its kind.
    pub fn config(&mut self, id: WidgetId, option: Opt) -> bool {
        let Some(kind) = self.node(id).map(|node| node.kind) else { return false };
        match option {
            Opt::Text(text) if kind == Kind::Entry => return self.entry_set(id, text),
            Opt::Text(text) if kind == Kind::Text => return self.text_set(id, text),
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
            Opt::Compound(compound) => {
                if let Some(node) = self.node_mut(id) {
                    node.compound = compound;
                }
            }
            Opt::Monospace(mono) => {
                if let Some(node) = self.node_mut(id) {
                    node.monospace = mono;
                }
            }
        }
        self.dirty = true;
        true
    }

    /// Shorthand for `config(id, Opt::Command(...))`.
    pub fn set_command(&mut self, id: WidgetId, command: impl FnMut(&mut Tk) + 'static) -> bool {
        self.config(id, Opt::Command(Box::new(command)))
    }

    /// Runs `callback` when Enter is pressed in the single-line Entry `id`
    /// (tkinter `entry.bind("<Return>", ...)`), e.g. to submit a search box.
    /// Inside an open dialog, Enter still accepts the dialog first.
    /// Returns false if `id` is not a live Entry. Calling it again replaces the callback.
    pub fn entry_bind_submit(&mut self, id: WidgetId, callback: impl FnMut(&mut Tk) + 'static) -> bool {
        match self.node_mut(id) {
            Some(node) if node.kind == Kind::Entry => {
                node.command = Some(Box::new(callback));
                true
            }
            _ => false,
        }
    }

    /// Current text of an Entry (tkinter `entry.get()`); `None` for other kinds.
    pub fn entry_get(&self, id: WidgetId) -> Option<String> {
        if self.node(id)?.kind != Kind::Entry {
            return None;
        }
        self.widget_text(id)
    }

    /// Replaces an Entry's text (control characters stripped, capped at 64 KiB)
    /// and moves the cursor to the end.
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
            var.set(self, text.clone());
        }
        if let Some(mut cb) = self.modal_entry_changed.take() {
            cb(self, id, &text);
            if self.modal_entry_changed.is_none() {
                self.modal_entry_changed = Some(cb);
            }
        }
    }

    /// Sets or clears the `SELECTED` state bit on a widget (e.g. for sidebar items).
    pub fn set_selected(&mut self, id: WidgetId, selected: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.selected != selected {
            node.selected = selected;
            self.dirty = true;
        }
        true
    }

    /// Displayed text of any widget, honouring a bound `textvariable`.
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
            if node.kind == Kind::Notebook {
                return Vec::new();
            }
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

    /// Headless harness: sets the DPI scale factor (0.1..=16). Returns false if rejected.
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
            if let Some(modal_id) = self.modal {
                if !self.is_descendant_of(*id, modal_id) {
                    return false;
                }
            }
            self.node(*id).is_some_and(|node| {
                let interactive = node.kind.focusable()
                    || matches!(node.kind, Kind::Canvas | Kind::Scrollbar | Kind::Notebook)
                    || node.tooltip.is_some();
                node.visible && interactive && node.clip.contains(x, y)
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

    /// Headless harness: moves the pointer to logical `(x, y)`, updating hover,
    /// drags and tooltips.
    #[doc(hidden)]
    pub fn sim_mouse_move(&mut self, x: f32, y: f32) {
        self.ensure_layout();
        let moved = (self.pointer.0 - x).abs() > 2.0 || (self.pointer.1 - y).abs() > 2.0;
        self.pointer = (x, y);
        let hovered = self.hit_test(x, y);
        if self.hovered != hovered {
            self.hover_timer_ms = 0;
            self.tooltip_visible = false;
            let prev = self.hovered;
            self.hovered = hovered;
            self.dirty = true;
            if let Some(prev_id) = prev {
                // It was hovered until a moment ago.
                self.check_state_transition(prev_id, self.widget_state(prev_id) | State::HOVER);
            }
            if let Some(new_id) = hovered {
                self.check_state_transition(new_id, self.widget_state(new_id).without(State::HOVER));
            }
        } else if moved && !self.tooltip_visible {
            self.hover_timer_ms = 0;
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

    /// Headless harness: presses the primary mouse button at the current pointer.
    #[doc(hidden)]
    pub fn sim_mouse_down(&mut self) {
        self.ensure_layout();
        self.tooltip_visible = false;
        self.hover_timer_ms = 0;
        let (x, y) = self.pointer;
        // Combobox closed by this very click; must not be re-opened below.
        let mut just_closed: Option<WidgetId> = None;

        // Check if click is on open combobox dropdown
        if let Some(combo_id) = self.open_combobox {
            if let Some(cnode) = self.node(combo_id) {
                let item_h = (cnode.style.font_size * 1.3 + 8.0).max(20.0);
                let drop_h = (cnode.combo_values.len() as f32 * item_h).clamp(item_h, 240.0);
                let drop_rect = Rect::new(cnode.rect.x, cnode.rect.y + cnode.rect.height, cnode.rect.width, drop_h);
                if drop_rect.contains(x, y) {
                    let idx = ((y - drop_rect.y) / item_h).floor() as usize;
                    if idx < cnode.combo_values.len() {
                        if let Some(var) = cnode.combo_var {
                            var.set(self, idx as i64);
                        }
                    }
                    if let Some(cn) = self.node_mut(combo_id) {
                        cn.combo_open = false;
                    }
                    self.open_combobox = None;
                    self.dirty = true;
                    return;
                }
            }
            if let Some(cn) = self.node_mut(combo_id) {
                cn.combo_open = false;
            }
            self.open_combobox = None;
            just_closed = Some(combo_id);
            self.dirty = true;
        }

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
                Kind::Text => {
                    let line_h = (node.style.font_size * 1.3 + 4.0).max(16.0);
                    let row = node.text_scroll_row + ((y - node.rect.y - 4.0) / line_h).max(0.0) as usize;
                    let lines: Vec<&str> = node.text.split('\n').collect();
                    let target_line = row.min(lines.len().saturating_sub(1));
                    let byte_offset = lines.iter().take(target_line).map(|l| l.len() + 1).sum::<usize>();
                    if let Some(node) = self.node_mut(id) {
                        node.cursor = byte_offset.min(node.text.len());
                    }
                }
                Kind::Combobox => {
                    let open = !node.combo_open && just_closed != Some(id);
                    if let Some(cn) = self.node_mut(id) {
                        cn.combo_open = open;
                    }
                    self.open_combobox = if open { Some(id) } else { None };
                }
                Kind::Notebook => {
                    if y < node.rect.y + 32.0 {
                        let mut cur_x = node.rect.x;
                        let pages = node.notebook_pages.clone();
                        // Same metrics as `update_requested`, so hit areas match drawn tabs.
                        let tab_style = self.resolved_style(id).unwrap_or_default();
                        for (idx, (_, title)) in pages.iter().enumerate() {
                            let m = self.measure_with(title, &tab_style);
                            let tab_w = m.width + 24.0;
                            if x >= cur_x && x < cur_x + tab_w {
                                self.notebook_select(id, idx);
                                break;
                            }
                            cur_x += tab_w;
                        }
                    }
                }
                _ => {}
            }
        }
        self.dirty = true;
    }

    /// Headless harness: releases the primary mouse button, activating the
    /// pressed widget if the pointer is still over it.
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

    /// Headless harness: move + press + release at logical `(x, y)`.
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
            Kind::Text => {
                let lines = if pixels { dy / 16.0 } else { dy };
                if let Some((_, _, pos)) = self.scroll_metrics(id) {
                    self.set_scroll_pos(id, pos - lines.round());
                }
            }
            Kind::Combobox if !node.combo_open => {
                let old = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
                let max_idx = node.combo_values.len().saturating_sub(1) as i64;
                let step = if dy > 0.0 { -1 } else { 1 };
                let new_val = (old + step).clamp(0, max_idx);
                if let Some(var) = node.combo_var {
                    var.set(self, new_val);
                }
                self.dirty = true;
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
            Kind::Text => (n.text.split('\n').count() as f32, n.text_dimensions.1.max(1) as f32, n.text_scroll_row as f32),
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
            Kind::Text => {
                if let Some((total, visible, _)) = self.scroll_metrics(id) {
                    let v = pos.round().clamp(0.0, total - visible) as usize;
                    if let Some(n) = self.node_mut(id) { n.text_scroll_row = v; }
                    self.dirty = true;
                }
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
        let sane = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        n.scrollregion = Some((sane(width), sane(height)));
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
            (Kind::Text, true) => (n.text.split('\n').count() as f32, n.text_dimensions.1.max(1) as f32, n.text_scroll_row as f32),
            (Kind::Canvas, true) => (n.scrollregion.map_or(n.rect.height, |r| r.1), n.rect.height, n.view.1),
            (Kind::Canvas, false) => (n.scrollregion.map_or(n.rect.width, |r| r.0), n.rect.width, n.view.0),
            _ => return (0.0, 1.0),
        };
        if total <= 0.0 || visible >= total { return (0.0, 1.0); }
        ((pos / total).clamp(0.0, 1.0), ((pos + visible) / total).clamp(0.0, 1.0))
    }

    /// tkinter `yview()`: visible fraction (first, last) of a Listbox, Text or Canvas.
    pub fn yview(&mut self, id: WidgetId) -> (f32, f32) { self.view_fractions(id, true) }
    /// tkinter `xview()` for a Canvas.
    pub fn xview(&mut self, id: WidgetId) -> (f32, f32) { self.view_fractions(id, false) }

    /// tkinter `yview_moveto(fraction)`: scrolls so `fraction` of the content is above the view.
    pub fn yview_moveto(&mut self, id: WidgetId, fraction: f32) {
        self.ensure_layout();
        let Some(n) = self.node(id) else { return };
        match n.kind {
            Kind::Listbox => { let t = n.list_items.len() as f32; self.set_scroll_pos(id, fraction * t); }
            Kind::Text => { let t = n.text.split('\n').count() as f32; self.set_scroll_pos(id, fraction * t); }
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

    /// tkinter `yview_scroll(n, "units"|"pages")`. Units are rows (Listbox, Text) or 48 px (Canvas).
    pub fn yview_scroll(&mut self, id: WidgetId, n: i32, what: ScrollUnit) {
        self.ensure_layout();
        let Some(node) = self.node(id) else { return };
        let (unit, page, pos) = match node.kind {
            Kind::Listbox => (1.0, node.list_height.max(1) as f32, node.list_scroll as f32),
            Kind::Text => (1.0, node.text_dimensions.1.max(1) as f32, node.text_scroll_row as f32),
            Kind::Canvas => (WHEEL_LINE_PX, (node.rect.height - WHEEL_LINE_PX).max(WHEEL_LINE_PX), node.view.1),
            _ => return,
        };
        let step = n as f32 * if what == ScrollUnit::Pages { page } else { unit };
        if node.kind == Kind::Canvas {
            let vx = node.view.0;
            self.canvas_set_view(id, vx, pos + step);
        } else {
            self.set_scroll_pos(id, pos + step);
        }
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
            .filter(|id| {
                if let Some(modal_id) = self.modal {
                    if !self.is_descendant_of(*id, modal_id) {
                        return false;
                    }
                }
                self.is_focusable(*id)
            }).collect();
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

    /// Headless harness: delivers a non-text key to the focused widget (or to the
    /// modal/combobox/focus-traversal handlers that intercept it).
    #[doc(hidden)]
    pub fn sim_key(&mut self, key: Key) {
        self.ensure_layout();
        if key == Key::Escape {
            if let Some(combo_id) = self.open_combobox.take() {
                if let Some(node) = self.node_mut(combo_id) {
                    node.combo_open = false;
                }
                self.dirty = true;
                return;
            }
            if self.modal.is_some() {
                if let Some(mut cancel) = self.modal_cancel.take() {
                    cancel(self);
                    if self.modal.is_some() && self.modal_cancel.is_none() {
                        self.modal_cancel = Some(cancel);
                    }
                    return;
                }
            }
        }
        if key == Key::Enter && self.modal.is_some() {
            let on_button = self.focused.is_some_and(|f| {
                self.node(f).is_some_and(|n| n.kind == Kind::Button)
            });
            if !on_button {
                if let Some(mut accept) = self.modal_accept.take() {
                    accept(self);
                    // If the dialog rejected the input (still open, no new handler
                    // installed), restore the handler so a later Enter can accept.
                    if self.modal.is_some() && self.modal_accept.is_none() {
                        self.modal_accept = Some(accept);
                    }
                    return;
                }
            }
        }
        if matches!(key, Key::Tab | Key::ShiftTab) {
            self.traverse_focus(key == Key::ShiftTab);
            return;
        }
        if let Some(modal_id) = self.modal {
            if let Some(f) = self.focused {
                if !self.is_descendant_of(f, modal_id) {
                    self.traverse_focus(false);
                }
            }
        }
        let Some(id) = self.focused else { return };
        let Some(node) = self.node(id) else { return };
        if node.disabled {
            return;
        }
        if node.kind == Kind::Canvas && node.scrollregion.is_some() {
            match key {
                Key::PageUp => self.yview_scroll(id, -1, ScrollUnit::Pages),
                Key::PageDown | Key::Space => self.yview_scroll(id, 1, ScrollUnit::Pages),
                Key::Up => self.yview_scroll(id, -1, ScrollUnit::Units),
                Key::Down => self.yview_scroll(id, 1, ScrollUnit::Units),
                Key::Left => self.xview_scroll(id, -1, ScrollUnit::Units),
                Key::Right => self.xview_scroll(id, 1, ScrollUnit::Units),
                Key::Home => self.yview_moveto(id, 0.0),
                Key::End => self.yview_moveto(id, 1.0),
                _ => {}
            }
            return;
        }
        match node.kind {
            Kind::Entry => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else if key == Key::Enter {
                    // Submit callback from `entry_bind_submit` (no-op if none).
                    self.run_command(id);
                } else {
                    self.edit_entry(id, Some(key), "");
                }
            }
            Kind::Text => {
                if key == Key::Space {
                    self.sim_text(" ");
                } else {
                    self.edit_text(id, Some(key), "");
                }
            }
            Kind::Combobox if matches!(key, Key::Up | Key::Down) => {
                let old = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
                let max_idx = node.combo_values.len().saturating_sub(1) as i64;
                let new_val = if key == Key::Up { old.saturating_sub(1) } else { old.saturating_add(1) }.clamp(0, max_idx);
                if let Some(var) = node.combo_var {
                    var.set(self, new_val);
                }
                self.dirty = true;
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

    /// Headless harness: types `text` into the focused Entry or Text widget.
    #[doc(hidden)]
    pub fn sim_text(&mut self, text: &str) {
        self.ensure_layout();
        if let Some(id) = self.focused {
            if self.node(id).is_some_and(|node| node.kind == Kind::Entry && !node.disabled) {
                self.edit_entry(id, None, text);
            } else if self.node(id).is_some_and(|node| node.kind == Kind::Text && !node.disabled && !node.text_read_only) {
                self.edit_text(id, None, text);
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

    fn edit_text(&mut self, id: WidgetId, key: Option<Key>, inserted: &str) {
        let Some(node) = self.node(id) else { return };
        if node.kind != Kind::Text { return; }
        let read_only = node.text_read_only;
        let mut text = node.text.clone();
        // The stored cursor may be stale after programmatic text changes; never slice mid-char.
        let mut cursor = floor_char_boundary(&text, node.cursor);
        let vis_rows = node.text_dimensions.1;

        match key {
            Some(Key::Left) => {
                if cursor > 0 {
                    let mut prev = 0;
                    for (i, _) in text.char_indices() {
                        if i >= cursor { break; }
                        prev = i;
                    }
                    cursor = prev;
                }
            }
            Some(Key::Right) => {
                if cursor < text.len() {
                    cursor = text[cursor..].char_indices().nth(1).map(|(i, _)| cursor + i).unwrap_or(text.len());
                }
            }
            Some(Key::Home) => {
                let start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                cursor = start;
            }
            Some(Key::End) => {
                let end = text[cursor..].find('\n').map_or(text.len(), |i| cursor + i);
                cursor = end;
            }
            Some(Key::Up) => {
                let line_start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                let col = text[line_start..cursor].chars().count();
                if line_start > 0 {
                    let prev_start = text[..line_start - 1].rfind('\n').map_or(0, |i| i + 1);
                    let prev_line = &text[prev_start..line_start - 1];
                    let mut new_c = prev_start;
                    for (i, _) in prev_line.chars().enumerate() {
                        if i >= col { break; }
                        new_c += prev_line[new_c - prev_start..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                    }
                    cursor = new_c;
                }
            }
            Some(Key::Down) => {
                let line_start = text[..cursor].rfind('\n').map_or(0, |i| i + 1);
                let col = text[line_start..cursor].chars().count();
                let next_nl = text[cursor..].find('\n').map(|i| cursor + i);
                if let Some(nl) = next_nl {
                    let next_start = nl + 1;
                    let next_end = text[next_start..].find('\n').map_or(text.len(), |i| next_start + i);
                    let next_line = &text[next_start..next_end];
                    let mut new_c = next_start;
                    for (i, _) in next_line.chars().enumerate() {
                        if i >= col { break; }
                        new_c += next_line[new_c - next_start..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                    }
                    cursor = new_c;
                }
            }
            Some(Key::Enter) if !read_only => {
                text.insert(cursor, '\n');
                cursor += 1;
            }
            Some(Key::Backspace) if !read_only && cursor > 0 => {
                let mut prev = 0;
                for (i, _) in text.char_indices() {
                    if i >= cursor { break; }
                    prev = i;
                }
                text.remove(prev);
                cursor = prev;
            }
            Some(Key::Delete) if !read_only && cursor < text.len() => {
                text.remove(cursor);
            }
            None if !read_only && !inserted.is_empty() => {
                text.insert_str(cursor, inserted);
                cursor += inserted.len();
            }
            _ => {}
        }

        let cur_line = text[..cursor].matches('\n').count();
        if let Some(node) = self.node_mut(id) {
            node.text = text;
            node.cursor = cursor;
            if cur_line < node.text_scroll_row {
                node.text_scroll_row = cur_line;
            } else if cur_line >= node.text_scroll_row + vis_rows {
                node.text_scroll_row = cur_line + 1 - vis_rows;
            }
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
    /// Inserts an item at `index` (appends when out of range), shifting the selection.
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

    /// Deletes items `first..=last` (clamped), adjusting selection and scroll.
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

    /// All items of a Listbox (empty for other kinds).
    pub fn listbox_get(&self, id: WidgetId) -> Vec<String> {
        self.node(id)
            .filter(|n| n.kind == Kind::Listbox)
            .map(|n| n.list_items.clone())
            .unwrap_or_default()
    }

    /// Index of the selected row, if any (tkinter `curselection()`).
    pub fn listbox_curselection(&self, id: WidgetId) -> Option<usize> {
        self.node(id)
            .filter(|n| n.kind == Kind::Listbox)
            .and_then(|n| n.list_selection)
    }

    /// Selects row `index` (or clears with `None`), scrolls it into view, fires
    /// the select callback on change and the command on every call.
    pub fn listbox_select(&mut self, id: WidgetId, index: Option<usize>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        let old_sel = node.list_selection;
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
        let new_sel = node.list_selection;
        self.dirty = true;
        if new_sel != old_sel {
            self.dispatch_list_select(id, new_sel);
        }
        self.run_command(id);
        true
    }

    pub fn listbox_bind_select(&mut self, id: WidgetId, cb: impl FnMut(&mut Tk, Option<usize>) + 'static) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Listbox {
            return false;
        }
        node.list_select_cb = Some(Box::new(cb));
        true
    }

    fn dispatch_list_select(&mut self, id: WidgetId, sel: Option<usize>) {
        let Some(node) = self.node_mut(id) else { return };
        if node.list_select_running { return; }
        let Some(mut cb) = node.list_select_cb.take() else { return };
        node.list_select_running = true;
        cb(self, sel);
        if let Some(node) = self.node_mut(id) {
            node.list_select_running = false;
            if node.list_select_cb.is_none() {
                node.list_select_cb = Some(cb);
            }
        }
    }

    // Scrollbar methods
    /// Links a vertical Scrollbar to a Listbox, scrollable Canvas, or Text widget.
    pub fn link_scrollbar(&mut self, scrollbar: WidgetId, listbox: WidgetId) -> bool {
        let is_sb = self.node(scrollbar).is_some_and(|n| n.kind == Kind::Scrollbar);
        let is_lb = self.node(listbox).is_some_and(|n| matches!(n.kind, Kind::Listbox | Kind::Canvas | Kind::Text));
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
            CanvasItemKind::Image { x, y, w, h, .. } => Some([*x, *y, *x + *w, *y + *h]),
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

    /// Creates a raw RGBA image item scaled to `w` × `h` logical pixels.
    #[allow(clippy::too_many_arguments)]
    pub fn canvas_create_image_rgba(&mut self, id: WidgetId, x: f32, y: f32, w: f32, h: f32, src_w: u32, src_h: u32, rgba: &[u8], smooth: bool) -> Option<CanvasItem> {
        if src_w == 0 || src_h == 0 || rgba.len() != (src_w as usize) * (src_h as usize) * 4 {
            return None;
        }
        self.canvas_add_item(id, CanvasItemKind::Image { x, y, w, h, src_w, src_h, rgba: rgba.to_vec(), smooth })
    }

    /// Updates the RGBA pixel data for a canvas image item.
    pub fn canvas_update_image(&mut self, id: WidgetId, item: CanvasItem, src_w: u32, src_h: u32, rgba: &[u8]) -> bool {
        if src_w == 0 || src_h == 0 || rgba.len() != (src_w as usize) * (src_h as usize) * 4 {
            return false;
        }
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Canvas { return false; }
        let Some(slot) = node.canvas_items.get_mut(item.index as usize) else { return false };
        if slot.gen != item.gen { return false; }
        let Some(kind) = slot.item.as_mut() else { return false };
        if let CanvasItemKind::Image { src_w: sw, src_h: sh, rgba: buf, .. } = kind {
            *sw = src_w;
            *sh = src_h;
            *buf = rgba.to_vec();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Text widget API
    // -----------------------------------------------------------------------

    pub fn text_get(&self, id: WidgetId) -> Option<String> {
        self.node(id).filter(|n| n.kind == Kind::Text).map(|n| n.text.clone())
    }

    pub fn text_set(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        let mut s = text.into();
        if let Some(max) = node.text_max_lines {
            trim_lines(&mut s, max);
        }
        node.text = s;
        node.cursor = node.text.len();
        self.dirty = true;
        true
    }

    pub fn text_insert_end(&mut self, id: WidgetId, text: &str) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text.push_str(text);
        if let Some(max) = node.text_max_lines {
            trim_lines(&mut node.text, max);
        }
        if node.text_autoscroll {
            let total = node.text.split('\n').count();
            let vis = node.text_dimensions.1;
            node.text_scroll_row = total.saturating_sub(vis);
        }
        node.cursor = node.text.len();
        self.dirty = true;
        true
    }

    pub fn text_clear(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text.clear();
        node.cursor = 0;
        node.text_scroll_row = 0;
        self.dirty = true;
        true
    }

    pub fn text_set_readonly(&mut self, id: WidgetId, read_only: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text_read_only = read_only;
        true
    }

    pub fn text_line_count(&self, id: WidgetId) -> Option<usize> {
        self.node(id).filter(|n| n.kind == Kind::Text).map(|n| n.text.split('\n').count())
    }

    pub fn text_set_max_lines(&mut self, id: WidgetId, max_lines: usize) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Text { return false; }
        node.text_max_lines = Some(max_lines);
        if max_lines > 0 {
            trim_lines(&mut node.text, max_lines);
            node.cursor = node.cursor.min(node.text.len());
        }
        self.dirty = true;
        true
    }

    // -----------------------------------------------------------------------
    // Combobox API
    // -----------------------------------------------------------------------

    pub fn combobox_set_values(&mut self, id: WidgetId, values: Vec<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::Combobox { return false; }
        node.combo_values = values;
        self.dirty = true;
        true
    }

    pub fn combobox_get(&self, id: WidgetId) -> Option<(i64, String)> {
        let node = self.node(id)?;
        if node.kind != Kind::Combobox { return None; }
        let idx = node.combo_var.and_then(|v| v.get(self)).unwrap_or(0);
        let s = node.combo_values.get(idx as usize).cloned().unwrap_or_default();
        Some((idx, s))
    }

    // -----------------------------------------------------------------------
    // Notebook and Geometry show/hide API
    // -----------------------------------------------------------------------

    pub fn pack_forget(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.manager.is_some() {
            node.manager = None;
            node.visible = false;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn grid_forget(&mut self, id: WidgetId) -> bool {
        self.pack_forget(id)
    }

    pub fn place_forget(&mut self, id: WidgetId) -> bool {
        self.pack_forget(id)
    }

    pub fn is_mapped(&mut self, id: WidgetId) -> bool {
        self.ensure_layout();
        self.node(id).is_some_and(|n| n.visible)
    }

    pub fn notebook_add(&mut self, nb: WidgetId, page: WidgetId, title: impl Into<String>) -> bool {
        let is_nb = self.node(nb).is_some_and(|n| n.kind == Kind::Notebook);
        let is_page = self.node(page).is_some_and(|n| n.kind == Kind::Frame && n.parent == Some(nb));
        if !is_nb || !is_page { return false; }
        let title = title.into();
        if let Some(nb_node) = self.node_mut(nb) {
            nb_node.notebook_pages.push((page, title));
            if nb_node.notebook_pages.len() == 1 {
                nb_node.notebook_selected = 0;
            }
        }
        self.dirty = true;
        true
    }

    pub fn notebook_select(&mut self, nb: WidgetId, index: usize) -> bool {
        let Some(nb_node) = self.node(nb) else { return false };
        if nb_node.kind != Kind::Notebook || index >= nb_node.notebook_pages.len() {
            return false;
        }
        let old = nb_node.notebook_selected;
        if let Some(n) = self.node_mut(nb) {
            n.notebook_selected = index;
        }
        self.dirty = true;
        if old != index {
            self.dispatch_notebook_changed(nb, index);
        }
        true
    }

    pub fn notebook_selected(&self, nb: WidgetId) -> Option<usize> {
        self.node(nb).filter(|n| n.kind == Kind::Notebook).map(|n| n.notebook_selected)
    }

    pub fn notebook_bind_changed(&mut self, nb: WidgetId, cb: impl FnMut(&mut Tk, usize) + 'static) -> bool {
        let Some(node) = self.node_mut(nb) else { return false };
        if node.kind != Kind::Notebook { return false; }
        node.notebook_changed_cb = Some(Box::new(cb));
        true
    }

    fn dispatch_notebook_changed(&mut self, nb: WidgetId, index: usize) {
        let Some(node) = self.node_mut(nb) else { return };
        if node.notebook_changed_running { return; }
        let Some(mut cb) = node.notebook_changed_cb.take() else { return };
        node.notebook_changed_running = true;
        cb(self, index);
        if let Some(node) = self.node_mut(nb) {
            node.notebook_changed_running = false;
            if node.notebook_changed_cb.is_none() {
                node.notebook_changed_cb = Some(cb);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Progressbar var & indeterminate mode
    // -----------------------------------------------------------------------

    pub fn progress_bind(&mut self, id: WidgetId, var: crate::Var<f64>) -> bool {
        if var.get(self).is_none() { return false; }
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressBar { return false; }
        node.progress_f64 = Some(var);
        self.dirty = true;
        true
    }

    pub fn progress_set_indeterminate(&mut self, id: WidgetId, indeterminate: bool) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        if node.kind != Kind::ProgressBar { return false; }
        node.progress_indeterminate = indeterminate;
        self.dirty = true;
        true
    }

    // -----------------------------------------------------------------------
    // Tooltip API
    // -----------------------------------------------------------------------

    pub fn tooltip(&mut self, id: WidgetId, text: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        node.tooltip = Some(text.into());
        true
    }

    pub fn tooltip_remove(&mut self, id: WidgetId) -> bool {
        let Some(node) = self.node_mut(id) else { return false };
        node.tooltip = None;
        if self.hovered == Some(id) {
            self.tooltip_visible = false;
        }
        true
    }

    pub fn active_tooltip(&self) -> Option<String> {
        if self.tooltip_visible {
            self.hovered.and_then(|id| self.node(id)).and_then(|n| n.tooltip.clone())
        } else {
            None
        }
    }
}

fn trim_lines(text: &mut String, max: usize) {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.len() > max {
        let drop_count = lines.len() - max;
        *text = lines[drop_count..].join("\n");
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
        Kind::Text => "TText",
        Kind::Combobox => "TCombobox",
        Kind::Notebook => "TNotebook",
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

/// Legacy (pre-style-engine) dark-theme background/foreground for a widget
/// kind. Shared by `set_theme` and widget creation so both agree (review F9).
fn dark_default_colors(kind: Kind) -> (Color, Color) {
    let bg = match kind {
        Kind::Entry | Kind::Listbox | Kind::Text | Kind::Combobox => Color::rgb(23, 26, 33),
        Kind::Canvas => Color::rgb(20, 22, 28),
        _ => Color::rgb(23, 26, 33),
    };
    (bg, Color::rgb(230, 232, 238))
}

/// Largest char boundary `<= index` in `s` (stable replacement for the
/// unstable `str::floor_char_boundary`). Prevents mid-UTF-8 slicing (review F2).
fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut i = index;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}
