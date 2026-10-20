//! A small, software-rendered, tkinter-style GUI.

#![deny(missing_docs)]

mod app;
pub mod a11y;
#[cfg(feature = "accesskit")]
mod a11y_accesskit;
pub mod chart;
pub mod colormap;
pub mod dialog;
pub mod dnd;
pub mod file_dialog;
pub mod fonts;
pub mod icons;
pub mod image;
pub mod layout;
mod render;
pub mod shortcut;
pub mod style;
pub mod testing;
pub mod theme;
mod timer;
#[cfg(feature = "tray")]
pub mod tray;
mod tk;
mod var;
mod widgets;

pub use a11y::{A11yAction, A11yNode, A11yRole};
pub use chart::ChartKind;
pub use colormap::Colormap;
pub use dialog::MessageKind;
pub use dnd::{DropPhase, FileDropCallback, FileHoverCallback};
pub use file_dialog::{FileDialog, FileDialogBackend, FileDialogDone, FileDialogKind, FileDialogResult, FileFilter, MockFileDialog};
pub use fonts::DefaultFont;
pub use icons::Icon;
pub use image::ImageFit;
pub use layout::{Anchor, Fill, GridOpts, PackOpts, PlaceOpts, Rect, Side, Sticky};
pub use shortcut::{Modifiers, Shortcut, ShortcutError, ShortcutId};
pub use style::{Edges, ImageId, LinearGradient, Palette, Radius, ResolvedStyle, Shadow, State, StyleError, StyleSpec, Tokens, TypeScale};
pub use theme::{ColorRole, Radii, SemanticColors, SemanticTokens, Spacing, SystemTheme, Typography, OKABE_ITO};
pub use timer::TimerId;
pub use tk::{FrameDamage, Key, ScrollUnit, Tk, TkProxy, ToastId, ToastKind, ToastPosition};
pub use tk::menu::{MenuId, MenuItem, Menubar};
pub use tk::inputs_state::{Orient, RingMode};
pub use tk::ring::ProgressRing;
pub use tk::switch::Switch;
pub use tk::slider::Slider;
pub use tk::double_spinbox::DoubleSpinbox;
pub use tk::scroll_area::ScrollArea;
pub use var::Var;
pub use widgets::{
    Button, Canvas, CanvasDragCallback, CanvasItem, Chart, Checkbutton, Color, Combobox, Command,
    Compound, DragPhase, Entry, Frame, Image, Label, Listbox, Notebook, Opt, ProgressBar,
    Radiobutton, Scale, Scrollbar, Sparkline, Spinbox, Table, Text, Theme, TreeView, WidgetId,
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
    /// Formats the error as a human-readable message describing the failure cause.
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

/// Compiles the code samples in docs/GUIDE.md as doctests.
#[cfg(doctest)]
#[doc = include_str!("../docs/GUIDE.md")]
pub struct GuideDoctests;
