//! In-app, non-modal toast notifications.
//!
//! Toasts are owned by [`Tk`] in a small [`ToastState`] and drawn as an overlay
//! (see `src/render/data.rs::draw_toast_overlay`). Timing is driven by the
//! existing animation clock (`sim_advance_time`), never wall time, so headless
//! tests are deterministic. While any non-sticky toast is visible,
//! [`Tk::is_animating`] reports `true` so the real backend keeps ticking.

use crate::layout::Rect;
use crate::widgets::Color;
use crate::Tk;

/// Fade-in duration in milliseconds.
pub(crate) const TOAST_FADE_IN_MS: u32 = 150;
/// Fade-out duration in milliseconds.
pub(crate) const TOAST_FADE_OUT_MS: u32 = 200;
/// Default lifetime of a toast created with [`Tk::toast`].
pub(crate) const TOAST_DEFAULT_MS: u32 = 4000;
/// Default maximum number of simultaneously visible toasts.
pub(crate) const TOAST_DEFAULT_MAX: usize = 4;
/// Vertical gap between stacked toasts in logical px.
pub(crate) const TOAST_GAP: f32 = 8.0;
/// Maximum toast width in logical px.
pub(crate) const TOAST_MAX_WIDTH: f32 = 360.0;
/// Minimum toast width in logical px.
pub(crate) const TOAST_MIN_WIDTH: f32 = 120.0;
/// Toast height in logical px.
pub(crate) const TOAST_HEIGHT: f32 = 36.0;
/// Distance from the viewport edge in logical px.
pub(crate) const TOAST_MARGIN: f32 = 16.0;
/// Horizontal padding inside a toast in logical px.
pub(crate) const TOAST_PAD_X: f32 = 12.0;
/// Width reserved for the leading kind icon in logical px.
pub(crate) const TOAST_ICON_W: f32 = 18.0;

/// The semantic kind of a toast; conveyed by a leading icon shape.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    /// Neutral information (circle icon).
    Info,
    /// Operation succeeded (check icon).
    Success,
    /// Non-fatal problem (triangle icon).
    Warning,
    /// Failure (cross icon).
    Error,
}

/// Generational-free handle to a toast created with [`Tk::toast`] /
/// [`Tk::toast_with`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub(crate) u32);

/// Where the toast stack is anchored inside the viewport.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastPosition {
    /// Stack grows upward from the bottom-right corner (default).
    BottomRight,
    /// Stack grows upward from the bottom-center.
    BottomCenter,
    /// Stack grows downward from the top-right corner.
    TopRight,
    /// Stack grows downward from the top-center.
    TopCenter,
}

impl Default for ToastPosition {
    /// Returns [`ToastPosition::BottomRight`].
    fn default() -> Self {
        Self::BottomRight
    }
}

/// One live toast.
pub(crate) struct Toast {
    /// Handle exposed to the user.
    pub(crate) id: ToastId,
    /// Message text (single line, clipped when too wide).
    pub(crate) text: String,
    /// Semantic kind (drives the icon shape).
    pub(crate) kind: ToastKind,
    /// Total lifetime in ms; `0` means sticky (never auto-dismisses).
    pub(crate) duration_ms: u32,
    /// Milliseconds elapsed since creation.
    pub(crate) elapsed_ms: u32,
    /// Milliseconds elapsed since dismissal was requested (fade-out).
    pub(crate) fade_out_ms: Option<u32>,
}

impl Toast {
    /// Current opacity in `0.0..=1.0` from the fade-in/fade-out phases.
    pub(crate) fn alpha(&self) -> f32 {
        let fade_in = if TOAST_FADE_IN_MS == 0 {
            1.0
        } else {
            (self.elapsed_ms as f32 / TOAST_FADE_IN_MS as f32).clamp(0.0, 1.0)
        };
        match self.fade_out_ms {
            Some(out) => {
                let fade = if TOAST_FADE_OUT_MS == 0 {
                    0.0
                } else {
                    (out as f32 / TOAST_FADE_OUT_MS as f32).clamp(0.0, 1.0)
                };
                fade_in * (1.0 - fade)
            }
            None => fade_in,
        }
    }

    /// True once the fade-out phase has completed and the toast can be removed.
    pub(crate) fn is_finished(&self) -> bool {
        self.fade_out_ms.is_some_and(|out| out >= TOAST_FADE_OUT_MS)
    }
}

/// All toast state owned by the toolkit.
pub(crate) struct ToastState {
    /// Live toasts, oldest first.
    pub(crate) list: Vec<Toast>,
    /// Next toast id.
    pub(crate) next_id: u32,
    /// Stack anchor.
    pub(crate) position: ToastPosition,
    /// Maximum number of visible toasts.
    pub(crate) max: usize,
}

impl Default for ToastState {
    /// Creates an empty stack anchored bottom-right with the default maximum.
    fn default() -> Self {
        Self {
            list: Vec::new(),
            next_id: 1,
            position: ToastPosition::BottomRight,
            max: TOAST_DEFAULT_MAX,
        }
    }
}

impl ToastState {
    /// Returns the index of the toast with `id`, if live.
    pub(crate) fn index_of(&self, id: ToastId) -> Option<usize> {
        self.list.iter().position(|t| t.id == id)
    }

    /// Requests dismissal of `id` (starts the fade-out). Returns false if unknown.
    pub(crate) fn dismiss(&mut self, id: ToastId) -> bool {
        let Some(idx) = self.index_of(id) else { return false };
        let toast = &mut self.list[idx];
        if toast.fade_out_ms.is_some() {
            return false;
        }
        toast.fade_out_ms = Some(0);
        true
    }

    /// Advances every toast by `ms`, removing finished ones. Returns true if anything changed.
    pub(crate) fn advance(&mut self, ms: u32) -> bool {
        if self.list.is_empty() || ms == 0 {
            return false;
        }
        for toast in &mut self.list {
            toast.elapsed_ms = toast.elapsed_ms.saturating_add(ms);
            if let Some(out) = toast.fade_out_ms {
                toast.fade_out_ms = Some(out.saturating_add(ms));
            } else if toast.duration_ms > 0 && toast.elapsed_ms >= toast.duration_ms {
                toast.fade_out_ms = Some(toast.elapsed_ms - toast.duration_ms);
            }
        }
        self.list.retain(|t| !t.is_finished());
        true
    }

    /// True while any toast still needs animation frames.
    pub(crate) fn is_animating(&self) -> bool {
        self.list
            .iter()
            .any(|t| t.duration_ms > 0 || t.fade_out_ms.is_some() || t.elapsed_ms < TOAST_FADE_IN_MS)
    }
}

/// Returns the icon colour for a toast kind from the palette.
pub(crate) fn kind_color(tk: &Tk, kind: ToastKind) -> Color {
    let palette = &tk.style_engine.tokens().palette;
    match kind {
        ToastKind::Info | ToastKind::Success => palette.accent,
        ToastKind::Warning | ToastKind::Error => palette.danger,
    }
}

impl Tk {
    /// Shows an informational toast for 4000 ms and returns its handle.
    pub fn toast(&mut self, text: impl Into<String>) -> ToastId {
        self.toast_with(text, ToastKind::Info, TOAST_DEFAULT_MS)
    }

    /// Shows a toast of `kind` for `duration_ms` (0 = sticky) and returns its handle.
    pub fn toast_with(
        &mut self,
        text: impl Into<String>,
        kind: ToastKind,
        duration_ms: u32,
    ) -> ToastId {
        let id = ToastId(self.toasts.next_id);
        self.toasts.next_id = self.toasts.next_id.saturating_add(1);
        self.toasts.list.push(Toast {
            id,
            text: text.into(),
            kind,
            duration_ms,
            elapsed_ms: 0,
            fade_out_ms: None,
        });
        let max = self.toasts.max.max(1);
        while self.toasts.list.len() > max {
            self.toasts.list.remove(0);
        }
        self.dirty = true;
        id
    }

    /// Starts the fade-out of `id`. Returns false if the toast is unknown or
    /// is already fading out.
    pub fn toast_dismiss(&mut self, id: ToastId) -> bool {
        let ok = self.toasts.dismiss(id);
        if ok {
            self.dirty = true;
        }
        ok
    }

    /// Removes every toast immediately.
    pub fn toast_clear(&mut self) {
        if !self.toasts.list.is_empty() {
            self.toasts.list.clear();
            self.dirty = true;
        }
    }

    /// Returns the visible toasts as `(id, text, kind)`, oldest first.
    pub fn toasts(&self) -> Vec<(ToastId, String, ToastKind)> {
        self.toasts
            .list
            .iter()
            .map(|t| (t.id, t.text.clone(), t.kind))
            .collect()
    }

    /// Sets where the toast stack is anchored.
    pub fn toast_set_position(&mut self, position: ToastPosition) {
        self.toasts.position = position;
        self.dirty = true;
    }

    /// Sets the maximum number of visible toasts (minimum 1); extra oldest are dropped.
    pub fn toast_set_max(&mut self, n: usize) {
        self.toasts.max = n.max(1);
        let max = self.toasts.max;
        while self.toasts.list.len() > max {
            self.toasts.list.remove(0);
        }
        self.dirty = true;
    }

    /// Returns the logical rectangle of the toast with `id`, if it is visible.
    pub fn toast_rect(&mut self, id: ToastId) -> Option<Rect> {
        let idx = self.toasts.index_of(id)?;
        let rects = self.toast_layout();
        rects.get(idx).copied()
    }

    /// Computes the logical rectangles of every visible toast, oldest first.
    pub(crate) fn toast_layout(&mut self) -> Vec<Rect> {
        let count = self.toasts.list.len();
        if count == 0 {
            return Vec::new();
        }
        let vw = self.viewport.0 as f32 / self.scale_factor;
        let vh = self.viewport.1 as f32 / self.scale_factor;
        let style = self.resolved_style(self.root()).unwrap_or_default();
        let mut widths = Vec::with_capacity(count);
        let texts: Vec<String> = self.toasts.list.iter().map(|t| t.text.clone()).collect();
        for text in &texts {
            let text_w = self.measure_with(text, &style).width;
            let w = (text_w + TOAST_PAD_X * 2.0 + TOAST_ICON_W + 8.0)
                .clamp(TOAST_MIN_WIDTH, TOAST_MAX_WIDTH)
                .min((vw - TOAST_MARGIN * 2.0).max(TOAST_MIN_WIDTH));
            widths.push(w);
        }
        let position = self.toasts.position;
        let mut rects = Vec::with_capacity(count);
        for (i, w) in widths.into_iter().enumerate() {
            let x = match position {
                ToastPosition::BottomRight | ToastPosition::TopRight => {
                    vw - TOAST_MARGIN - w
                }
                ToastPosition::BottomCenter | ToastPosition::TopCenter => (vw - w) / 2.0,
            };
            let y = match position {
                ToastPosition::BottomRight | ToastPosition::BottomCenter => {
                    vh - TOAST_MARGIN - TOAST_HEIGHT - i as f32 * (TOAST_HEIGHT + TOAST_GAP)
                }
                ToastPosition::TopRight | ToastPosition::TopCenter => {
                    TOAST_MARGIN + i as f32 * (TOAST_HEIGHT + TOAST_GAP)
                }
            };
            rects.push(Rect::new(x.max(0.0), y.max(0.0), w, TOAST_HEIGHT));
        }
        rects
    }

    /// Handles a click at logical `(x, y)` while toasts are visible.
    ///
    /// Returns true if a toast was hit (the click is consumed and the toast
    /// starts fading out).
    pub(crate) fn toast_click(&mut self, x: f32, y: f32) -> bool {
        if self.toasts.list.is_empty() {
            return false;
        }
        let rects = self.toast_layout();
        for (idx, rect) in rects.iter().enumerate() {
            if rect.contains(x, y) {
                if let Some(toast) = self.toasts.list.get_mut(idx) {
                    if toast.fade_out_ms.is_none() {
                        toast.fade_out_ms = Some(0);
                    }
                }
                self.dirty = true;
                return true;
            }
        }
        false
    }
}
