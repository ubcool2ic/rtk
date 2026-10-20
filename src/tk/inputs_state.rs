//! Per-widget state for the milestone 0.10 input widgets (ProgressRing, Switch,
//! Slider, DoubleSpinbox, and the polished linear ProgressBar).
//!
//! All new widget state lives in this single struct so that `WidgetNode` only
//! needs one additive field (`inputs`). Other milestone branches extend this
//! struct with additional `Default`-able fields instead of touching
//! `WidgetNode` again.

/// Visual mode of a [`crate::ProgressRing`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RingMode {
    /// A determinate arc showing a fraction of a full circle.
    #[default]
    Percent,
    /// An indeterminate arc that rotates continuously.
    Spinner,
}

/// Orientation of a [`crate::Slider`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Orient {
    /// Horizontal slider (default).
    #[default]
    Horizontal,
    /// Vertical slider (top = max, bottom = min).
    Vertical,
}

/// State shared by the milestone 0.10 input widgets.
#[derive(Clone, Debug, Default)]
pub(crate) struct InputsState {
    // ---- ProgressRing ----
    /// Whether the ring is in percent or spinner mode.
    pub(crate) ring_mode: RingMode,
    /// Whether the percent ring draws its centered "NN%" text.
    pub(crate) ring_show_text: bool,
    /// Stroke thickness in logical px (0 = derive from diameter).
    pub(crate) ring_thickness: f32,
    /// Spinner rotation phase in `0.0..1.0` (one revolution per second).
    pub(crate) ring_phase: f32,

    // ---- Switch ----
    /// Knob position in `0.0..=1.0` (0 = off, 1 = on); animated toward the var.
    pub(crate) switch_knob: f32,
    /// True once the knob has been initialised from the bound variable.
    pub(crate) switch_initialised: bool,

    // ---- Linear ProgressBar polish ----
    /// Whether the bar draws a centered "NN%" label.
    pub(crate) progress_show_percent: bool,
    /// Optional label drawn over the bar.
    pub(crate) progress_label: Option<String>,
    /// Whether the drawn fraction eases toward the bound value.
    pub(crate) progress_smooth: bool,
    /// The eased fraction actually drawn (0..=1).
    pub(crate) progress_drawn: f32,
    /// True once `progress_drawn` has been initialised from the bound value.
    pub(crate) progress_initialised: bool,

    // ---- Slider ----
    /// Slider orientation.
    pub(crate) slider_orient: Orient,
    /// Snap step (0 = continuous).
    pub(crate) slider_step: f64,
    /// Tick spacing along the track (`None` = no ticks).
    pub(crate) slider_ticks: Option<f64>,
    /// Slider length in logical px.
    pub(crate) slider_length: f32,

    // ---- DoubleSpinbox ----
    /// Step used by arrows/keys/wheel.
    pub(crate) dspin_step: f64,
    /// Number of decimals used for display.
    pub(crate) dspin_decimals: u8,
    /// Suffix appended to the displayed value.
    pub(crate) dspin_suffix: String,
    /// In-progress text edit buffer (`None` = show the formatted value).
    pub(crate) dspin_buffer: Option<String>,

    // ---- Scrollbar orientation ----
    /// True when the scrollbar runs horizontally (thumb along x).
    pub(crate) scrollbar_horizontal: bool,

    // ---- ScrollArea ----
    /// The inner content frame of a ScrollArea.
    pub(crate) scroll_area_content: Option<crate::WidgetId>,
    /// Current scroll offset `(x, y)` of a ScrollArea in logical px.
    pub(crate) scroll_area_offset: (f32, f32),
}
