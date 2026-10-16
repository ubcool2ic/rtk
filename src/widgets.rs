use std::collections::BTreeMap;

use crate::layout::{GridOpts, PackOpts, Rect, Size};
use crate::{Tk, Var};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u32);

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
    pub const ACTIVE: Self = Self::rgb(236, 236, 236);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

pub type Command = Box<dyn FnMut(&mut Tk)>;

pub enum Opt {
    Text(String),
    Bg(Color),
    Fg(Color),
    Width(u32),
    FontSize(f32),
    TextVariable(Var<String>),
    TextVariableInt(Var<i64>),
    Command(Command),
}

pub struct Frame;
pub struct Label;
pub struct Button;
pub struct Entry;

// Tk-style constructors intentionally allocate in the arena and return IDs.
#[allow(clippy::new_ret_no_self)]
impl Frame {
    pub fn new(tk: &mut Tk, parent: WidgetId) -> WidgetId {
        tk.add(Kind::Frame, parent, String::new())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Label {
    pub fn new(tk: &mut Tk, parent: WidgetId, text: impl Into<String>) -> WidgetId {
        tk.add(Kind::Label, parent, text.into())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Button {
    pub fn new(tk: &mut Tk, parent: WidgetId, text: impl Into<String>) -> WidgetId {
        tk.add(Kind::Button, parent, text.into())
    }
}

#[allow(clippy::new_ret_no_self)]
impl Entry {
    pub fn new(tk: &mut Tk, parent: WidgetId) -> WidgetId {
        tk.add(Kind::Entry, parent, String::new())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Frame,
    Label,
    Button,
    Entry,
}

#[derive(Clone, Copy)]
pub(crate) enum Manager {
    Pack(PackOpts),
    Grid(GridOpts),
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
}

pub(crate) struct WidgetNode {
    pub kind: Kind,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
    pub text: String,
    pub binding: Option<Binding>,
    pub style: Style,
    pub command: Option<Command>,
    pub manager: Option<Manager>,
    pub columns: BTreeMap<usize, u32>,
    pub rows: BTreeMap<usize, u32>,
    pub requested: Size,
    pub rect: Rect,
    pub clip: Rect,
    pub visible: bool,
    pub cursor: usize,
    pub scroll_x: f32,
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
            style: Style {
                bg: if kind == Kind::Entry { Color::WHITE } else { Color::BG },
                fg: Color::FG,
                font_size: 14.0,
                width: None,
            },
            command: None,
            manager: None,
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
            requested: Size::default(),
            rect: Rect::default(),
            clip: Rect::default(),
            visible: false,
            scroll_x: 0.0,
        }
    }
}
