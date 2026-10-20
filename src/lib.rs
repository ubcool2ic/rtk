//! A small, software-rendered, tkinter-style GUI.

mod app;
pub mod a11y;
#[cfg(feature = "accesskit")]
mod a11y_accesskit;
pub mod colormap;
pub mod dialog;
pub mod fonts;
pub mod icons;
pub mod layout;
mod render;
pub mod style;
pub mod testing;
pub mod theme;
mod timer;
mod tk;
mod var;
mod widgets;

pub use a11y::{A11yAction, A11yNode, A11yRole};
pub use colormap::Colormap;
pub use dialog::MessageKind;
pub use fonts::DefaultFont;
pub use icons::Icon;
pub use layout::{Anchor, Fill, GridOpts, PackOpts, PlaceOpts, Rect, Side, Sticky};
pub use style::{Edges, ImageId, LinearGradient, Palette, Radius, ResolvedStyle, Shadow, State, StyleError, StyleSpec, Tokens, TypeScale};
pub use theme::{ColorRole, Radii, SemanticColors, SemanticTokens, Spacing, SystemTheme, Typography, OKABE_ITO};
pub use timer::TimerId;
pub use tk::{FrameDamage, Key, ScrollUnit, Tk, TkProxy};
pub use var::Var;
pub use widgets::{
    Button, Canvas, CanvasDragCallback, CanvasItem, Checkbutton, Color, Combobox, Command,
    Compound, DragPhase, Entry, Frame, Label, Listbox, Notebook, Opt, ProgressBar, Radiobutton,
    Scale, Scrollbar, Spinbox, Text, Theme, WidgetId,
};

/// Errors from widget construction and headless image allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtkError {
    /// The given parent is not a live `Frame` (or root) of this toolkit.
    InvalidParent,
    /// The variable handle is destroyed or belongs to another toolkit.
    InvalidVariable,
    /// The widget arena cannot hold another widget.
    ArenaFull,
    /// Image width/height is zero or exceeds the supported maximum.
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
