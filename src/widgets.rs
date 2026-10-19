use std::collections::BTreeMap;

use crate::layout::{GridOpts, PackOpts, PlaceOpts, Rect, Size};
use crate::style::{ImageId, StyleSpec, TransitionState};
use crate::{RtkError, Tk, Var};

/// A generational arena handle. Do not transfer widget IDs between toolkits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId {
    pub index: u32,
    pub gen: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BG: Self = Self::rgb(217, 217, 217);
    pub const FG: Self = Self::rgb(25, 25, 25);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const ACTIVE: Self = Self::rgb(236, 236, 236);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    Fluent,
    Material,
    HighContrast,
    ModernDark,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragPhase {
    Press,
    Move,
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasItem {
    pub index: u32,
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

pub type Command = Box<dyn FnMut(&mut Tk)>;
pub type CanvasDragCallback = Box<dyn FnMut(&mut Tk, f32, f32, DragPhase)>;

pub enum Opt {
    Text(String),
    Bg(Color),
    Fg(Color),
    Width(u32),
    FontSize(f32),
    TextVariable(Var<String>),
    TextVariableInt(Var<i64>),
    Command(Command),
    // Advanced styling layer options
    Style(String),
    StyleOverride(StyleSpec),
    Disabled(bool),
    Image(ImageId),
    Compound(Compound),
    Monospace(bool),
}

pub struct Frame;
pub struct Label;
pub struct Button;
pub struct Entry;
pub struct Checkbutton;
pub struct Scale;
pub struct Listbox;
pub struct Scrollbar;
pub struct Radiobutton;
pub struct Canvas;
pub struct Spinbox;
pub struct ProgressBar;
pub struct Text;
pub struct Combobox;
pub struct Notebook;

#[allow(clippy::new_ret_no_self)]
impl Frame {
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Frame, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Label {
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
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Entry, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Checkbutton {
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
    pub fn new(tk: &mut Tk, parent: WidgetId) -> Result<WidgetId, RtkError> {
        tk.add(Kind::Scrollbar, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Radiobutton {
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
    pub list_select_cb: Option<Box<dyn FnMut(&mut Tk, Option<usize>)>>,
    pub list_select_running: bool,
    // Combobox
    pub combo_values: Vec<String>,
    pub combo_var: Option<Var<i64>>,
    pub combo_open: bool,
    // Notebook
    pub notebook_pages: Vec<(WidgetId, String)>,
    pub notebook_selected: usize,
    pub notebook_changed_cb: Option<Box<dyn FnMut(&mut Tk, usize)>>,
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
