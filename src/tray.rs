//! Optional system tray icon support, gated behind the `tray` cargo feature.
//!
//! This module wraps the [`tray_icon`] crate to expose a small, tkinter-style
//! API for creating a system tray icon with an optional context menu. The
//! feature is **off by default**; enable it with `features = ["tray"]`.
//!
//! # Threading and lifecycle
//!
//! On macOS the tray icon must be created **after** the event loop has started
//! and on the UI thread. The recommended pattern is therefore to create the
//! [`Tray`] from inside a `tk.after_idle(...)` callback (or any other callback
//! that runs on the UI thread once `mainloop` is active).
//!
//! Tray events (clicks and menu selections) are delivered on platform-specific
//! background threads. This module forwards them through
//! [`TkProxy::post`](crate::TkProxy::post), which wakes the event loop only
//! when an event actually arrives. As a result the idle CPU cost stays at
//! zero: no polling, no timers, no busy loops.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "tray")] {
//! use rtk::{Tk, tray::{Tray, TrayEvent, TrayOptions}};
//!
//! let mut tk = Tk::new();
//! let proxy = tk.proxy();
//! tk.after_idle(move |tk| {
//!     let opts = TrayOptions {
//!         tooltip: "rtk demo".into(),
//!         icon_rgba: vec![0xff; 16 * 16 * 4],
//!         icon_width: 16,
//!         icon_height: 16,
//!         menu_items: vec!["Show".into(), "Quit".into()],
//!     };
//!     let _tray = Tray::new(tk, opts, |_tk, ev| {
//!         eprintln!("tray event: {ev:?}");
//!     });
//! });
//! # }
//! ```

use std::cell::RefCell;
use std::fmt;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::Tk;

/// Maximum icon edge length accepted by [`validate_icon`].
const MAX_ICON_EDGE: u32 = 256;

/// An event originating from the system tray icon.
///
/// `Menu(index)` carries the zero-based position of the clicked item in the
/// `menu_items` list that was supplied when the [`Tray`] was created.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrayEvent {
    /// The primary (left) mouse button was released over the icon.
    Click,
    /// The icon was double-clicked.
    DoubleClick,
    /// A menu item at the given index was activated.
    Menu(usize),
}

/// Errors returned by tray construction and mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TrayError {
    /// The supplied RGBA buffer is empty, has the wrong length, or the
    /// dimensions are zero or exceed the supported maximum (256x256).
    InvalidIcon,
    /// The underlying platform reported an error.
    Platform(String),
}

impl fmt::Display for TrayError {
    /// Formats the error for human consumption.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIcon => f.write_str(
                "invalid tray icon: expected non-empty RGBA buffer of w*h*4 bytes with 1..=256 px edges",
            ),
            Self::Platform(msg) => write!(f, "tray platform error: {msg}"),
        }
    }
}

impl std::error::Error for TrayError {}

/// Options used to construct a [`Tray`].
#[derive(Clone, Debug)]
pub struct TrayOptions {
    /// Tooltip text shown when hovering the icon.
    pub tooltip: String,
    /// Raw RGBA8 pixel data, `icon_width * icon_height * 4` bytes.
    pub icon_rgba: Vec<u8>,
    /// Icon width in pixels (1..=256).
    pub icon_width: u32,
    /// Icon height in pixels (1..=256).
    pub icon_height: u32,
    /// Menu item labels. When empty, no context menu is attached.
    pub menu_items: Vec<String>,
}

/// Validates an RGBA icon buffer.
///
/// The buffer must be non-empty, exactly `w * h * 4` bytes long, and both
/// dimensions must be in `1..=256`.
pub fn validate_icon(rgba: &[u8], w: u32, h: u32) -> Result<(), TrayError> {
    if w == 0 || h == 0 || w > MAX_ICON_EDGE || h > MAX_ICON_EDGE {
        return Err(TrayError::InvalidIcon);
    }
    let expected = (w as usize)
        .checked_mul(h as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or(TrayError::InvalidIcon)?;
    if rgba.len() != expected {
        return Err(TrayError::InvalidIcon);
    }
    Ok(())
}

/// Maps a [`MenuId`] to its index within `ids`.
///
/// Returns `None` when the id is not present.
pub fn map_menu_index(ids: &[MenuId], id: &MenuId) -> Option<usize> {
    ids.iter().position(|candidate| candidate == id)
}

/// Boxed tray event callback stored on the UI thread.
type TrayHandler = Box<dyn FnMut(&mut Tk, TrayEvent)>;

thread_local! {
    /// The active tray event callback, stored on the UI thread.
    static HANDLER: RefCell<Option<TrayHandler>> = const { RefCell::new(None) };
}

/// Dispatches a tray event to the currently installed handler.
///
/// The handler is temporarily taken out of the thread-local slot so that it
/// may replace itself (via [`set_handler_for_test`] or by installing a new
/// [`Tray`]) without re-entrancy panics. After the call returns, the handler
/// is restored unless it was replaced during dispatch.
#[doc(hidden)]
pub fn dispatch(tk: &mut Tk, event: TrayEvent) {
    let taken = HANDLER.with(|slot| slot.borrow_mut().take());
    let Some(mut cb) = taken else {
        return;
    };
    cb(tk, event);
    HANDLER.with(|slot| {
        let mut guard = slot.borrow_mut();
        if guard.is_none() {
            *guard = Some(cb);
        }
    });
}

/// Installs a tray event handler for testing purposes.
///
/// This bypasses the platform tray entirely and only sets the thread-local
/// callback used by [`dispatch`].
#[doc(hidden)]
pub fn set_handler_for_test(cb: impl FnMut(&mut Tk, TrayEvent) + 'static) {
    HANDLER.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(cb));
    });
}

/// Clears the thread-local tray event handler.
fn clear_handler() {
    HANDLER.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

/// A live system tray icon with an optional context menu.
///
/// Dropping the `Tray` removes the icon and uninstalls the global event
/// handlers.
pub struct Tray {
    /// The underlying platform tray icon.
    icon: TrayIcon,
    /// Menu ids in the same order as the labels passed at creation.
    menu_ids: Vec<MenuId>,
}

impl fmt::Debug for Tray {
    /// Formats the tray for debugging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tray")
            .field("menu_ids", &self.menu_ids)
            .finish_non_exhaustive()
    }
}

impl Tray {
    /// Creates a new tray icon attached to `tk`.
    ///
    /// `on_event` is invoked on the UI thread whenever a tray event arrives.
    /// On macOS this must be called after `mainloop` has started (for example
    /// from `tk.after_idle`).
    pub fn new(
        tk: &mut Tk,
        opts: TrayOptions,
        on_event: impl FnMut(&mut Tk, TrayEvent) + 'static,
    ) -> Result<Tray, TrayError> {
        validate_icon(&opts.icon_rgba, opts.icon_width, opts.icon_height)?;
        let icon = Icon::from_rgba(opts.icon_rgba, opts.icon_width, opts.icon_height)
            .map_err(|e| TrayError::Platform(e.to_string()))?;

        let mut menu_ids: Vec<MenuId> = Vec::with_capacity(opts.menu_items.len());
        let menu = if opts.menu_items.is_empty() {
            None
        } else {
            let menu = Menu::new();
            for label in &opts.menu_items {
                let item = MenuItem::new(label.as_str(), true, None);
                menu_ids.push(item.id().clone());
                menu.append(&item)
                    .map_err(|e| TrayError::Platform(e.to_string()))?;
            }
            Some(menu)
        };

        let mut builder = TrayIconBuilder::new()
            .with_tooltip(opts.tooltip)
            .with_icon(icon);
        if let Some(menu) = menu {
            builder = builder.with_menu(Box::new(menu));
        }
        let icon = builder
            .build()
            .map_err(|e| TrayError::Platform(e.to_string()))?;

        // Install the callback on the UI thread.
        HANDLER.with(|slot| {
            *slot.borrow_mut() = Some(Box::new(on_event));
        });

        // Forward tray icon events through the proxy.
        let proxy = tk.proxy();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            let mapped = match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => Some(TrayEvent::Click),
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => Some(TrayEvent::DoubleClick),
                _ => None,
            };
            if let Some(ev) = mapped {
                proxy.post(move |tk| dispatch(tk, ev));
            }
        }));

        // Forward menu events through the proxy.
        let ids_for_menu = menu_ids.clone();
        let proxy = tk.proxy();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(index) = map_menu_index(&ids_for_menu, &event.id) {
                proxy.post(move |tk| dispatch(tk, TrayEvent::Menu(index)));
            }
        }));

        Ok(Tray { icon, menu_ids })
    }

    /// Updates the tooltip text.
    pub fn set_tooltip(&self, text: &str) -> Result<(), TrayError> {
        self.icon
            .set_tooltip(Some(text))
            .map_err(|e| TrayError::Platform(e.to_string()))
    }

    /// Replaces the tray icon image.
    pub fn set_icon(&self, rgba: &[u8], w: u32, h: u32) -> Result<(), TrayError> {
        validate_icon(rgba, w, h)?;
        let icon = Icon::from_rgba(rgba.to_vec(), w, h)
            .map_err(|e| TrayError::Platform(e.to_string()))?;
        self.icon
            .set_icon(Some(icon))
            .map_err(|e| TrayError::Platform(e.to_string()))
    }

    /// Shows or hides the tray icon.
    pub fn set_visible(&self, visible: bool) -> Result<(), TrayError> {
        self.icon
            .set_visible(visible)
            .map_err(|e| TrayError::Platform(e.to_string()))
    }

    /// Returns the menu ids in creation order.
    ///
    /// Useful for tests and for mapping menu events back to labels.
    pub fn menu_ids(&self) -> &[MenuId] {
        &self.menu_ids
    }
}

impl Drop for Tray {
    /// Uninstalls the global event handlers and clears the callback slot.
    fn drop(&mut self) {
        TrayIconEvent::set_event_handler(None::<fn(TrayIconEvent)>);
        MenuEvent::set_event_handler(None::<fn(MenuEvent)>);
        clear_handler();
    }
}
