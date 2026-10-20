//! File drag-and-drop types: drop/hover callbacks and the per-toolkit drag state.
//!
//! The real window backend (`app.rs`) forwards winit's `DroppedFile` /
//! `HoveredFile` / `HoveredFileCancelled` events into the headless `sim_*`
//! harness in `tk/dnd.rs`, so drag-and-drop is fully testable without a window.

use std::path::PathBuf;

use crate::tk::Tk;
use crate::WidgetId;

/// Callback invoked when files are dropped on a widget.
pub type FileDropCallback = Box<dyn FnMut(&mut Tk, &[PathBuf])>;

/// Phase of a file drag hovering over a widget.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropPhase {
    /// The drag entered the widget (or moved onto it from another widget).
    Enter,
    /// The drag left the widget (moved away or was cancelled).
    Leave,
    /// The files were dropped on the widget.
    Drop,
}

/// Callback invoked when a file drag enters, leaves or drops on a widget.
pub type FileHoverCallback = Box<dyn FnMut(&mut Tk, DropPhase)>;

/// Per-toolkit drag-and-drop state: bindings, the current hover target and the
/// paths being dragged.
#[derive(Default)]
pub(crate) struct DndState {
    /// Drop bindings, keyed by widget. `None` while the callback is running.
    pub drop: Vec<(WidgetId, Option<FileDropCallback>)>,
    /// Hover bindings, keyed by widget. `None` while the callback is running.
    pub hover: Vec<(WidgetId, Option<FileHoverCallback>)>,
    /// Paths currently being dragged over the window.
    pub hovered: Vec<PathBuf>,
    /// Widget currently under the drag, if any.
    pub target: Option<WidgetId>,
}
