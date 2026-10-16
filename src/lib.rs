//! A small, software-rendered, tkinter-style GUI.
//!
//! Fallible constructors return `Result`; handle-based mutations return `bool`
//! (`false` means rejected), and queries return `Option`. Invalid or stale
//! handles never index the arena unchecked. Widget handles belong to their
//! originating [`Tk`]; variable handles additionally validate toolkit ownership.

mod app;
pub mod layout;
mod render;
mod tk;
mod var;
mod widgets;

pub use layout::{Anchor, Fill, GridOpts, PackOpts, Rect, Side, Sticky};
pub use tk::{Key, Tk};
pub use var::Var;
pub use widgets::{
    Button, Checkbutton, Color, Command, Entry, Frame, Label, Opt, Scale, WidgetId,
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
