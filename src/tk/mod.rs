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


mod style;
mod window;
mod tree;
mod geometry;
mod events;
mod scroll;
mod canvas;
mod entry;
mod text;
mod listbox;
mod combobox;
mod notebook;
mod progress;
mod tooltip;
pub(crate) mod inputs_state;
pub(crate) mod ring;
pub(crate) mod switch;
/// Decimal slider construction and interaction.
pub(crate) mod slider;
/// Decimal spinbox construction, editing, and interaction.
pub(crate) mod double_spinbox;
mod shortcuts;
mod clipboard;
mod edit;
mod theme;

pub(crate) use entry::{clean_entry, clusters, cursor_boundary};
pub(crate) use events::floor_char_boundary;
#[allow(unused_imports)]
pub(crate) use inputs_state::RingMode;
pub(crate) use style::{dark_default_colors, NodeSnapshot};

pub(crate) type PostedTask = Box<dyn FnOnce(&mut Tk) + Send + 'static>;
pub(crate) type IdleTask = Box<dyn FnOnce(&mut Tk) + 'static>;
/// Callback run when a modal dialog is cancelled (Escape).
pub(crate) type ModalCancel = Box<dyn FnMut(&mut Tk)>;
/// Callback run when a modal dialog is accepted (Enter / default button).
pub(crate) type ModalAccept = Box<dyn FnMut(&mut Tk)>;
/// Callback run when the text of an Entry inside a modal dialog changes.
pub(crate) type ModalEntryChanged = Box<dyn FnMut(&mut Tk, WidgetId, &str)>;
/// Callback invoked on a window close request; returns true to allow quit.
pub(crate) type CloseCallback = Box<dyn FnMut(&mut Tk) -> bool>;
/// A bound keyboard shortcut: its id, the parsed chord, and its callback.
pub(crate) type ShortcutBinding = (
    crate::shortcut::ShortcutId,
    crate::shortcut::Shortcut,
    Box<dyn FnMut(&mut Tk)>,
);

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
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// A character key: a lowercase letter/digit/punctuation pressed with a
    /// modifier, or a bare key when no text input applies.
    Char(char),
    /// A function key, F1..=F24.
    F(u8),
    /// The Insert key.
    Insert,
}

/// Scroll step for `yview_scroll`/`xview_scroll`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollUnit {
    /// One row (Listbox/Text) or a fixed wheel step of 48 logical px (Canvas).
    Units,
    /// One visible page of rows/pixels.
    Pages,
}

/// Logical px a Canvas scrolls per wheel line / scroll unit.
const WHEEL_LINE_PX: f32 = 48.0;

/// The toolkit instance: owns every widget, variable, style and input state.
///
/// Create one with [`Tk::new`], build a widget tree with the widget
/// constructors, then either run the real window loop or drive it headlessly
/// with the `sim_*` methods.
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
    pub(crate) modal_cancel: Option<ModalCancel>,
    pub(crate) modal_accept: Option<ModalAccept>,
    pub(crate) modal_entry_changed: Option<ModalEntryChanged>,
    /// Window icon as (width, height, straight RGBA bytes).
    pub(crate) window_icon: Option<(u32, u32, Vec<u8>)>,
    /// Minimum window size in logical px.
    pub(crate) window_minsize: Option<(u32, u32)>,
    /// Maximum window size in logical px.
    pub(crate) window_maxsize: Option<(u32, u32)>,
    /// Whether the window is resizable horizontally and vertically.
    pub(crate) window_resizable: (bool, bool),
    /// Callback invoked on the window close button; returns true to allow quit.
    pub(crate) close_callback: Option<CloseCallback>,
    /// True when window properties (icon, size constraints) need re-applying.
    pub(crate) window_props_dirty: bool,
    /// Modifiers of the most recent key press.
    pub(crate) mods: crate::shortcut::Modifiers,
    /// Bound keyboard shortcuts, in binding order.
    pub(crate) shortcuts: Vec<ShortcutBinding>,
    /// Next shortcut id to hand out.
    pub(crate) next_shortcut_id: u64,
    /// In-memory clipboard contents (always available, headless-safe).
    pub(crate) clipboard_text: String,
    /// Lazily created OS clipboard handle (only when the real loop runs).
    #[cfg(feature = "clipboard")]
    pub(crate) os_clipboard: Option<arboard::Clipboard>,
}

impl Default for Tk {
    fn default() -> Self {
        Self::new()
    }
}

impl Tk {
    /// Creates a toolkit with a root `Frame`, the Light theme and a 640×480 viewport.
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
            window_icon: None,
            window_minsize: None,
            window_maxsize: None,
            window_resizable: (true, true),
            close_callback: None,
            window_props_dirty: false,
            mods: crate::shortcut::Modifiers::NONE,
            shortcuts: Vec::new(),
            next_shortcut_id: 1,
            clipboard_text: String::new(),
            #[cfg(feature = "clipboard")]
            os_clipboard: None,
        }
    }

    /// Obtains a thread-safe proxy to post callbacks from background threads.
    /// Obtains a thread-safe proxy to post callbacks from background threads.
    pub fn proxy(&self) -> TkProxy {
        TkProxy { shared: self.proxy_shared.clone() }
    }

    /// Queues a closure to run once on the main thread when idle or on next drain.
    /// Queues a closure to run once on the main thread when idle or on next drain.
    pub fn after_idle(&mut self, f: impl FnOnce(&mut Tk) + 'static) {
        self.idle_queue.push_back(Box::new(f));
    }

    /// Drains posted and idle tasks reentrantly, returning the number processed.
    ///
    /// Tasks queued while draining are run in the same call, up to a bounded
    /// number of passes so a self-re-posting task cannot hang the main thread;
    /// anything left over runs on the next call.
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
    /// Returns true if `id` refers to a live widget (tkinter `winfo_exists`).
    pub fn contains(&self, id: WidgetId) -> bool {
        self.node(id).is_some()
    }

    /// Returns the active theme.
    /// Returns the active theme.
    pub fn theme(&self) -> Theme {
        self.theme
    }

    /// Which font family became the default sans-serif: the bundled Noto Sans,
    /// or Arial / a platform sans if the bundled font could not be loaded.
    /// Which font family became the default sans-serif: the bundled Noto Sans,
    /// or Arial / a platform sans if the bundled font could not be loaded.
    pub fn default_font(&self) -> &crate::fonts::DefaultFont {
        self.text_engine.default_font()
    }

    /// Returns the design tokens (palette, fonts, radii) of the active theme.
    /// Returns the design tokens (palette, fonts, radii) of the active theme.
    pub fn tokens(&self) -> &Tokens {
        self.style_engine.tokens()
    }

    /// Replaces the design tokens without changing `theme()`.
    /// Replaces the design tokens without changing `theme()`.
    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.style_engine.set_tokens(tokens);
        self.dirty = true;
    }

    /// Switches the theme, resetting tokens and the legacy per-widget colours
    /// that were not explicitly overridden with `Opt::Bg`/`Opt::Fg`.
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
    /// Sets the brand accent color and updates derived tokens and styles.
    pub fn set_accent(&mut self, accent: Color) {
        let mut tokens = self.style_engine.tokens().clone();
        tokens.palette.accent = accent;
        tokens.palette.focus = accent;
        tokens.palette.select_bg = Color::rgba(accent.r, accent.g, accent.b, 64);
        self.style_engine.set_tokens(tokens);
        self.dirty = true;
    }


}
