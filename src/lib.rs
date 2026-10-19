//! A small, software-rendered, tkinter-style GUI.

mod app;
pub mod colormap;
pub mod layout;
mod render;
pub mod style;
mod timer;
mod tk;
mod var;
mod widgets;

pub use colormap::Colormap;
pub use layout::{Anchor, Fill, GridOpts, PackOpts, PlaceOpts, Rect, Side, Sticky};
pub use style::{Edges, ImageId, LinearGradient, Palette, Radius, ResolvedStyle, Shadow, State, StyleError, StyleSpec, Tokens, TypeScale};
pub use timer::TimerId;
pub use tk::{Key, ScrollUnit, Tk};
pub use var::Var;
pub use widgets::{
    Button, Canvas, CanvasDragCallback, CanvasItem, Checkbutton, Color, Combobox, Command,
    DragPhase, Entry, Frame, Label, Listbox, Notebook, Opt, ProgressBar, Radiobutton, Scale,
    Scrollbar, Spinbox, Text, Theme, WidgetId,
};

/// Errors from widget construction and headless image allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtkError {
    InvalidParent,
    InvalidVariable,
    ArenaFull,
    InvalidImageSize,
}

impl std::fmt::Display for RtkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidParent => "parent is not a live frame",
            Self::InvalidVariable => "variable is invalid or belongs to another toolkit",
            Self::ArenaFull => "widget arena is full",
            Self::InvalidImageSize => "image dimensions are zero or excessive",
        })
    }
}

impl std::error::Error for RtkError {}
