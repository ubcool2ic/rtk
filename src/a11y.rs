//! Dependency-free accessibility tree types.
//!
//! These types describe the widget tree in a form suitable for screen readers
//! and are always available. The optional `accesskit` feature maps them onto
//! AccessKit's `TreeUpdate` in `src/a11y_accesskit.rs`.

use crate::layout::Rect;
use crate::WidgetId;

/// The semantic role of an accessibility node.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum A11yRole {
    /// A top-level window.
    Window,
    /// A grouping container (frame).
    Group,
    /// Static text.
    Label,
    /// A push button.
    Button,
    /// A check box.
    CheckBox,
    /// A radio button.
    RadioButton,
    /// A single-line text input.
    TextInput,
    /// A multi-line text input.
    MultilineTextInput,
    /// A slider.
    Slider,
    /// A spin button.
    SpinButton,
    /// A progress indicator.
    ProgressIndicator,
    /// A list box.
    ListBox,
    /// A combo box.
    ComboBox,
    /// A tab list.
    TabList,
    /// A scroll bar.
    ScrollBar,
    /// A drawing canvas.
    Canvas,
    /// An unrecognised widget.
    Unknown,
}

/// An action that assistive technology can request on a node.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum A11yAction {
    /// Activate a button, check box or radio button.
    Click,
    /// Move keyboard focus to the widget.
    Focus,
    /// Replace the widget's text value.
    SetValue,
    /// Increase a range widget's value.
    Increment,
    /// Decrease a range widget's value.
    Decrement,
    /// Toggle a check box or switch between on and off.
    Toggle,
}

/// A snapshot of one widget in the accessibility tree.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct A11yNode {
    /// The widget this node describes.
    pub id: WidgetId,
    /// The semantic role.
    pub role: A11yRole,
    /// Human-readable label (text or override).
    pub label: Option<String>,
    /// Current text/selection value.
    pub value: Option<String>,
    /// `(value, min, max)` for range widgets.
    pub numeric: Option<(f64, f64, f64)>,
    /// Toggled state for check boxes and radio buttons.
    pub toggled: Option<bool>,
    /// Whether the widget is disabled.
    pub disabled: bool,
    /// Whether the widget currently holds keyboard focus.
    pub focused: bool,
    /// Bounds in logical pixels.
    pub bounds: Rect,
    /// Child widget IDs in document order.
    pub children: Vec<WidgetId>,
    /// Actions supported by the widget.
    pub actions: Vec<A11yAction>,
}
