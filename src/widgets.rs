//! Widget constructors, configuration options and the internal widget-node arena.
//!
//! Widgets are created with `Frame::new(&mut tk, parent)`-style constructors that
//! return a [`WidgetId`]; all behaviour lives on [`Tk`].

use std::collections::BTreeMap;

use crate::layout::{GridOpts, PackOpts, PlaceOpts, Rect, Size};
use crate::style::{ImageId, StyleSpec, TransitionState};
use crate::{RtkError, Tk, Var};

/// A generational arena handle. Do not transfer widget IDs between toolkits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId {
    /// Slot index in the widget arena.
    pub index: u32,
    /// Generation of the slot when the handle was issued; stale handles are rejected.
    pub gen: u32,
}

impl WidgetId {
    /// Packs the handle into a single `u64` key for external consumers such as
    /// AccessKit node IDs: `(index << 32) | gen`.
    #[allow(dead_code)]
    pub(crate) fn a11y_key(self) -> u64 {
        (u64::from(self.index) << 32) | u64::from(self.gen)
    }

    /// Reconstructs a handle from a key produced by [`WidgetId::a11y_key`].
    #[allow(dead_code)]
    pub(crate) fn from_a11y_key(key: u64) -> Self {
        Self {
            index: (key >> 32) as u32,
            gen: key as u32,
        }
    }
}

/// An 8-bit-per-channel, straight-alpha colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel (255 = opaque).
    pub a: u8,
}

impl Color {
    /// Default widget background (classic Tk light grey).
    pub const BG: Self = Self::rgb(217, 217, 217);
    /// Default text colour.
    pub const FG: Self = Self::rgb(25, 25, 25);
    /// Opaque white.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Opaque black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// Background of hovered/active buttons.
    pub const ACTIVE: Self = Self::rgb(236, 236, 236);

    /// Opaque colour from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Colour from straight-alpha RGBA components.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Built-in visual themes selectable at runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    /// Classic light grey Tk look (default).
    #[default]
    Light,
    /// Classic look with dark backgrounds.
    Dark,
    /// Fluent-inspired flat light theme.
    Fluent,
    /// Material-inspired theme with accent colours.
    Material,
    /// Maximum-contrast theme for accessibility.
    HighContrast,
    /// Modern dark theme with rounded corners and shadows.
    ModernDark,
    /// Modern light theme with rounded corners and shadows.
    ModernLight,
}

/// Placement of an image relative to text inside Buttons and Labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Compound {
    /// Image drawn to the left of text with an 8 px gap (default).
    #[default]
    Left,
    /// Image drawn above text with an 8 px gap.
    Top,
    /// Only the image is rendered.
    ImageOnly,
    /// Only the text is rendered.
    TextOnly,
}

/// Phase of a canvas drag gesture reported to a [`CanvasDragCallback`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragPhase {
    /// Left button pressed inside the canvas.
    Press,
    /// Pointer moved while the button is held.
    Move,
    /// Button released (or the pointer left the window).
    Release,
}

/// Generational handle to an item drawn on a `Canvas`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasItem {
    /// Slot index in the canvas item list.
    pub index: u32,
    /// Generation of the slot when the handle was issued.
    pub gen: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CanvasItemKind {
    Rect { x0: f32, y0: f32, x1: f32, y1: f32, color: Color },
    Oval { x0: f32, y0: f32, x1: f32, y1: f32, color: Color },
    Line { x0: f32, y0: f32, x1: f32, y1: f32, color: Color },
    Text { x: f32, y: f32, text: String, color: Color },
    /// Rounded rectangle; filled when `fill` is set, stroked when `outline` is set.
    RoundRect { x0: f32, y0: f32, x1: f32, y1: f32, radius: f32, fill: Option<Color>, outline: Option<(Color, f32)> },
    /// Open polyline (stroked) or closed polygon (filled).
    Path { points: Vec<(f32, f32)>, color: Color, width: f32, filled: bool },
    /// Stroked ellipse outline.
    Ring { x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32 },
    /// Text with explicit pixel size and weight (100-900).
    TextEx { x: f32, y: f32, text: String, color: Color, size: f32, weight: u16 },
    /// Scaled raw RGBA image item.
    Image { x: f32, y: f32, w: f32, h: f32, src_w: u32, src_h: u32, rgba: Vec<u8>, smooth: bool },
}

#[derive(Clone, Debug)]
pub(crate) struct CanvasItemSlot {
    pub gen: u32,
    pub seq: u64,
    pub item: Option<CanvasItemKind>,
}

/// Boxed widget callback (tkinter `command`). Receives the toolkit so it can mutate the UI.
pub type Command = Box<dyn FnMut(&mut Tk)>;
/// Boxed canvas drag callback: `(tk, x, y, phase)` in canvas-local logical pixels.
pub type CanvasDragCallback = Box<dyn FnMut(&mut Tk, f32, f32, DragPhase)>;
/// Boxed Listbox selection callback: `(tk, selected_index)`.
pub type ListSelectCallback = Box<dyn FnMut(&mut Tk, Option<usize>)>;
/// Boxed Notebook tab-change callback: `(tk, selected_index)`.
pub type NotebookChangedCallback = Box<dyn FnMut(&mut Tk, usize)>;

/// Configuration options accepted by `Tk::config` (tkinter `widget.config(...)`).
pub enum Opt {
    /// Caption text (tkinter `text`).
    Text(String),
    /// Background colour (tkinter `bg`); marks the colour as user-overridden.
    Bg(Color),
    /// Foreground/text colour (tkinter `fg`).
    Fg(Color),
    /// Requested width in characters (tkinter `width`).
    Width(u32),
    /// Font size in logical pixels.
    FontSize(f32),
    /// Bind displayed text to a string variable (tkinter `textvariable`).
    TextVariable(Var<String>),
    /// Bind displayed text to an integer variable.
    TextVariableInt(Var<i64>),
    /// Callback run on activation (tkinter `command`).
    Command(Command),
    /// Named style from the active stylesheet (ttk `style`).
    Style(String),
    /// Per-widget style overrides applied on top of the named style.
    StyleOverride(StyleSpec),
    /// Disables input and renders the disabled state (tkinter `state=DISABLED`).
    Disabled(bool),
    /// Image to display (tkinter `image`).
    Image(ImageId),
    /// Image/text arrangement (tkinter `compound`).
    Compound(Compound),
    /// Use the monospace font family.
    Monospace(bool),
}

/// Constructor namespace for container frames (tkinter `Frame`).
pub struct Frame;
/// Constructor namespace for static text/image labels (tkinter `Label`).
pub struct Label;
/// Constructor namespace for push buttons (tkinter `Button`).
pub struct Button;
/// Constructor namespace for single-line text inputs (tkinter `Entry`).
pub struct Entry;
/// Constructor namespace for boolean check boxes (tkinter `Checkbutton`).
pub struct Checkbutton;
/// Constructor namespace for sliders (tkinter `Scale`).
pub struct Scale;
/// Constructor namespace for single-selection lists (tkinter `Listbox`).
pub struct Listbox;
/// Constructor namespace for scrollbars (tkinter `Scrollbar`).
pub struct Scrollbar;
/// Constructor namespace for mutually exclusive radio buttons (tkinter `Radiobutton`).
pub struct Radiobutton;
/// Constructor namespace for free-form drawing surfaces (tkinter `Canvas`).
pub struct Canvas;
/// Constructor namespace for integer spin boxes (tkinter `Spinbox`).
pub struct Spinbox;
/// Constructor namespace for progress bars (ttk `Progressbar`).
pub struct ProgressBar;
/// Constructor namespace for multi-line text areas (tkinter `Text`).
pub struct Text;
/// Constructor namespace for drop-down selectors (ttk `Combobox`).
pub struct Combobox;
/// Constructor namespace for tabbed containers (ttk `Notebook`).
pub struct Notebook;

#[allow(clippy::new_ret_no_self)]
impl Frame {
    /// Creates an empty container frame under `parent`.
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Frame, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Label {
    /// Creates a label showing `text`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        text: impl Into<String>,
    ) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Label, parent, text.into())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Button {
    /// Creates a button captioned `text`; attach a callback with `Tk::set_command`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        text: impl Into<String>,
    ) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Button, parent, text.into())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Entry {
    /// Creates an empty single-line text input.
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Entry, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Checkbutton {
    /// Creates a check box bound to `variable`. Fails with
    /// [`RtkError::InvalidVariable`] if the variable is not live in `tk`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        text: impl Into<String>,
        variable: Var<bool>,
    ) -> Result<WidgetId, RtkError> {
        if variable.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::Checkbutton, parent, text.into())?;
        if let Some(node) = tk.node_mut(id) {
            node.check = Some(variable);
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Scale {
    /// Creates a slider over `from..=to` bound to `variable`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        from: i64,
        to: i64,
        variable: Var<i64>,
    ) -> Result<WidgetId, RtkError> {
        if variable.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::Scale, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.scale = Some(variable);
            node.range = (from, to);
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Listbox {
    /// Creates a listbox showing `height_rows` rows (at least 1).
    pub fn new(tk: &mut Tk, parent: WidgetId, height_rows: usize) -> Result<WidgetId, RtkError> {
        let id = tk.add(Kind::Listbox, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.list_height = height_rows.max(1);
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Scrollbar {
    /// Creates a scrollbar; link it to a widget with `Tk::scrollbar_attach`-style APIs.
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Scrollbar, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Radiobutton {
    /// Creates a radio button that sets `var` to `value` when selected.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        text: impl Into<String>,
        var: Var<i64>,
        value: i64,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::Radiobutton, parent, text.into())?;
        if let Some(node) = tk.node_mut(id) {
            node.radio_var = Some(var);
            node.radio_value = value;
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Canvas {
    /// Creates a drawing canvas with the requested logical size (negative/NaN → 0).
    pub fn new(tk: &mut Tk, parent: WidgetId, w: f32, h: f32) -> Result<WidgetId, RtkError> {
        let id = tk.add(Kind::Canvas, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.canvas_size = (w.max(0.0), h.max(0.0));
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Spinbox {
    /// Creates an integer spin box over `from..=to` bound to `var`.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        from: i64,
        to: i64,
        var: Var<i64>,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::Spinbox, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.range = (from, to);
            node.scale = Some(var);
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl ProgressBar {
    /// Creates a progress bar of `length` px (min 10) showing `var` in 0..=100.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        length: f32,
        var: Var<i64>,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::ProgressBar, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.progress_length = length.max(10.0);
            node.scale = Some(var);
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Text {
    /// Creates a multi-line text area sized in characters × rows (each at least 1).
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        width_chars: usize,
        height_rows: usize,
    ) -> Result<WidgetId, RtkError> {
        let id = tk.add(Kind::Text, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.text_dimensions = (width_chars.max(1), height_rows.max(1));
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Combobox {
    /// Creates a drop-down over `values`; `var` holds the selected index.
    pub fn new(
        tk: &mut Tk,
        parent: WidgetId,
        values: Vec<String>,
        var: Var<i64>,
    ) -> Result<WidgetId, RtkError> {
        if var.get(tk).is_none() {
            return Err(RtkError::InvalidVariable);
        }
        let id = tk.add(Kind::Combobox, parent, String::new())?;
        if let Some(node) = tk.node_mut(id) {
            node.combo_values = values;
            node.combo_var = Some(var);
        }
        Ok(id)
    }
}

#[allow(clippy::new_ret_no_self)]
impl Notebook {
    /// Creates an empty tabbed container; add pages with the notebook APIs on `Tk`.
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Notebook, parent, String::new())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Frame,
    Label,
    Button,
    Entry,
    Checkbutton,
    Scale,
    Listbox,
    Scrollbar,
    Radiobutton,
    Canvas,
    Spinbox,
    ProgressBar,
    Text,
    Combobox,
    Notebook,
}

impl Kind {
    pub fn focusable(self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::Entry
                | Self::Checkbutton
                | Self::Scale
                | Self::Listbox
                | Self::Radiobutton
                | Self::Spinbox
                | Self::Text
                | Self::Combobox
        )
    }

    pub fn has_command(self) -> bool {
        matches!(
            self,
            Self::Button | Self::Checkbutton | Self::Scale | Self::Listbox | Self::Radiobutton
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Manager {
    Pack(PackOpts),
    Grid(GridOpts),
    Place(PlaceOpts),
}

#[derive(Clone, Copy)]
pub(crate) enum Binding {
    Text(Var<String>),
    Int(Var<i64>),
}

#[derive(Clone, Copy)]
pub(crate) struct Style {
    pub bg: Color,
    pub fg: Color,
    pub font_size: f32,
    pub width: Option<u32>,
    pub bg_overridden: bool,
    pub fg_overridden: bool,
}

pub(crate) struct WidgetNode {
    pub kind: Kind,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
    pub text: String,
    pub binding: Option<Binding>,
    pub check: Option<Var<bool>>,
    pub scale: Option<Var<i64>>,
    pub range: (i64, i64),
    pub style: Style,
    pub command: Option<Command>,
    pub command_running: bool,
    pub manager: Option<Manager>,
    pub columns: BTreeMap<usize, u32>,
    pub rows: BTreeMap<usize, u32>,
    pub requested: Size,
    pub rect: Rect,
    pub clip: Rect,
    pub visible: bool,
    pub cursor: usize,
    pub scroll_x: f32,
    // Listbox
    pub list_items: Vec<String>,
    pub list_height: usize,
    pub list_selection: Option<usize>,
    pub list_scroll: usize,
    // Scrollbar
    pub scroll_target: Option<WidgetId>,
    /// Canvas scroll offset in logical px (x, y).
    pub view: (f32, f32),
    /// Canvas scrollable content size (width, height), like tkinter `scrollregion`.
    pub scrollregion: Option<(f32, f32)>,
    // Radiobutton
    pub radio_var: Option<Var<i64>>,
    pub radio_value: i64,
    // Canvas
    pub canvas_size: (f32, f32),
    pub canvas_items: Vec<CanvasItemSlot>,
    pub canvas_drag: Option<CanvasDragCallback>,
    pub canvas_drag_running: bool,
    // ProgressBar
    pub progress_length: f32,
    pub progress_f64: Option<Var<f64>>,
    pub progress_indeterminate: bool,
    pub progress_phase: f32,
    // Text widget
    pub text_dimensions: (usize, usize),
    pub text_scroll_row: usize,
    pub text_read_only: bool,
    pub text_autoscroll: bool,
    pub text_max_lines: Option<usize>,
    // Listbox callback
    pub list_select_cb: Option<ListSelectCallback>,
    pub list_select_running: bool,
    // Combobox
    pub combo_values: Vec<String>,
    pub combo_var: Option<Var<i64>>,
    pub combo_open: bool,
    // Notebook
    pub notebook_pages: Vec<(WidgetId, String)>,
    pub notebook_selected: usize,
    pub notebook_changed_cb: Option<NotebookChangedCallback>,
    pub notebook_changed_running: bool,
    // Tooltip
    pub tooltip: Option<String>,
    // Monospace
    pub monospace: bool,
    // Advanced styling layer fields
    pub named_style: Option<String>,
    pub style_override: Option<StyleSpec>,
    pub disabled: bool,
    pub selected: bool,
    pub image: Option<ImageId>,
    pub compound: Compound,
    pub transition: Option<TransitionState>,
}

impl WidgetNode {
    pub fn new(kind: Kind, parent: Option<WidgetId>, text: String) -> Self {
        Self {
            kind,
            parent,
            children: Vec::new(),
            cursor: text.len(),
            text,
            binding: None,
            check: None,
            scale: None,
            range: (0, 100),
            style: Style {
                bg: if matches!(kind, Kind::Entry | Kind::Listbox) {
                    Color::WHITE
                } else {
                    Color::BG
                },
                fg: Color::FG,
                font_size: 14.0,
                width: None,
                bg_overridden: false,
                fg_overridden: false,
            },
            command: None,
            command_running: false,
            manager: None,
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
            requested: Size::default(),
            rect: Rect::default(),
            clip: Rect::default(),
            visible: false,
            scroll_x: 0.0,
            list_items: Vec::new(),
            list_height: 5,
            list_selection: None,
            list_scroll: 0,
            scroll_target: None,
            view: (0.0, 0.0),
            scrollregion: None,
            radio_var: None,
            radio_value: 0,
            canvas_size: (100.0, 100.0),
            canvas_items: Vec::new(),
            canvas_drag: None,
            canvas_drag_running: false,
            progress_length: 100.0,
            progress_f64: None,
            progress_indeterminate: false,
            progress_phase: 0.0,
            text_dimensions: (40, 10),
            text_scroll_row: 0,
            text_read_only: false,
            text_autoscroll: true,
            text_max_lines: None,
            list_select_cb: None,
            list_select_running: false,
            combo_values: Vec::new(),
            combo_var: None,
            combo_open: false,
            notebook_pages: Vec::new(),
            notebook_selected: 0,
            notebook_changed_cb: None,
            notebook_changed_running: false,
            tooltip: None,
            monospace: false,
            named_style: None,
            style_override: None,
            disabled: false,
            selected: false,
            image: None,
            compound: Compound::Left,
            transition: None,
        }
    }
}

pub(crate) struct WidgetSlot {
    pub gen: u32,
    pub node: Option<WidgetNode>,
}

impl CanvasItemKind {
    /// Moves the item by (dx, dy) logical px.
    pub(crate) fn translate(&mut self, dx: f32, dy: f32) {
        match self {
            CanvasItemKind::Rect { x0, y0, x1, y1, .. }
            | CanvasItemKind::Oval { x0, y0, x1, y1, .. }
            | CanvasItemKind::Line { x0, y0, x1, y1, .. }
            | CanvasItemKind::RoundRect { x0, y0, x1, y1, .. }
            | CanvasItemKind::Ring { x0, y0, x1, y1, .. } => {
                *x0 += dx;
                *y0 += dy;
                *x1 += dx;
                *y1 += dy;
            }
            CanvasItemKind::Text { x, y, .. } | CanvasItemKind::TextEx { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
            CanvasItemKind::Image { x, y, .. } => {
                *x += dx;
                *y += dy;
            }
            CanvasItemKind::Path { points, .. } => {
                for p in points.iter_mut() {
                    p.0 += dx;
                    p.1 += dy;
                }
            }
        }
    }
}
