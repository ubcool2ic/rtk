//! A small, software-rendered, tkinter-style GUI.
//!
//! Widgets and variables are Copy handles owned by one [`Tk`]. Handles must
//! only be used with the `Tk` that created them.

mod app;
pub mod layout;
mod render;
mod tk;
mod var;
mod widgets;

pub use layout::{Anchor, Fill, GridOpts, PackOpts, Side, Sticky};
pub use tk::Tk;
pub use var::Var;
pub use widgets::{Button, Color, Command, Entry, Frame, Label, Opt, WidgetId};
