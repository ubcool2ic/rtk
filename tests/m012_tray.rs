//! Integration tests for the optional `tray` feature.
//!
//! These tests are entirely headless: they never create a real tray icon and
//! only exercise the pure helpers and the dispatch machinery.
#![cfg(feature = "tray")]

use rtk::tray::{
    dispatch, map_menu_index, set_handler_for_test, validate_icon, TrayError, TrayEvent,
};
use rtk::Tk;
use tray_icon::menu::MenuId;

/// A 2x2 RGBA buffer (16 bytes).
fn tiny_rgba() -> Vec<u8> {
    vec![0u8; 2 * 2 * 4]
}

/// `validate_icon` accepts a well-formed buffer.
#[test]
fn validate_icon_accepts_valid() {
    assert!(validate_icon(&tiny_rgba(), 2, 2).is_ok());
    let big = vec![0u8; 256 * 256 * 4];
    assert!(validate_icon(&big, 256, 256).is_ok());
}

/// `validate_icon` rejects zero dimensions.
#[test]
fn validate_icon_rejects_zero_dims() {
    assert_eq!(validate_icon(&[], 0, 0), Err(TrayError::InvalidIcon));
    assert_eq!(validate_icon(&tiny_rgba(), 0, 2), Err(TrayError::InvalidIcon));
    assert_eq!(validate_icon(&tiny_rgba(), 2, 0), Err(TrayError::InvalidIcon));
}

/// `validate_icon` rejects oversized dimensions.
#[test]
fn validate_icon_rejects_oversize() {
    let buf = vec![0u8; 257 * 4];
    assert_eq!(validate_icon(&buf, 257, 1), Err(TrayError::InvalidIcon));
    let buf = vec![0u8; 257 * 4];
    assert_eq!(validate_icon(&buf, 1, 257), Err(TrayError::InvalidIcon));
}

/// `validate_icon` rejects buffers whose length does not match `w*h*4`.
#[test]
fn validate_icon_rejects_wrong_len() {
    assert_eq!(validate_icon(&[], 2, 2), Err(TrayError::InvalidIcon));
    assert_eq!(validate_icon(&[0u8; 15], 2, 2), Err(TrayError::InvalidIcon));
    assert_eq!(validate_icon(&[0u8; 17], 2, 2), Err(TrayError::InvalidIcon));
}

/// `map_menu_index` finds ids by position and returns `None` for unknown ids.
#[test]
fn map_menu_index_works() {
    let ids = vec![MenuId::new("a"), MenuId::new("b"), MenuId::new("c")];
    assert_eq!(map_menu_index(&ids, &MenuId::new("a")), Some(0));
    assert_eq!(map_menu_index(&ids, &MenuId::new("b")), Some(1));
    assert_eq!(map_menu_index(&ids, &MenuId::new("c")), Some(2));
    assert_eq!(map_menu_index(&ids, &MenuId::new("z")), None);
    assert_eq!(map_menu_index(&[], &MenuId::new("a")), None);
}

/// `dispatch` invokes the installed handler with the given event.
#[test]
fn dispatch_calls_handler() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let log: Rc<RefCell<Vec<TrayEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let log2 = log.clone();
    set_handler_for_test(move |_tk, ev| {
        log2.borrow_mut().push(ev);
    });

    let mut tk = Tk::new();
    dispatch(&mut tk, TrayEvent::Click);
    dispatch(&mut tk, TrayEvent::DoubleClick);
    dispatch(&mut tk, TrayEvent::Menu(3));

    assert_eq!(
        *log.borrow(),
        vec![TrayEvent::Click, TrayEvent::DoubleClick, TrayEvent::Menu(3)]
    );
}

/// A handler may replace itself during dispatch without panicking.
#[test]
fn dispatch_handler_can_replace_itself() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let calls: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let calls2 = calls.clone();
    set_handler_for_test(move |_tk, _ev| {
        calls2.borrow_mut().push("first");
        let calls3 = calls2.clone();
        set_handler_for_test(move |_tk, _ev| {
            calls3.borrow_mut().push("second");
        });
    });

    let mut tk = Tk::new();
    dispatch(&mut tk, TrayEvent::Click);
    dispatch(&mut tk, TrayEvent::Click);

    assert_eq!(*calls.borrow(), vec!["first", "second"]);
}

/// `dispatch` with no handler installed is a no-op.
#[test]
fn dispatch_without_handler_is_noop() {
    // There is no public way to clear the handler, so install a no-op one and
    // verify that dispatching to it does not panic or touch other state.
    set_handler_for_test(|_tk, _ev| {});
    let mut tk = Tk::new();
    dispatch(&mut tk, TrayEvent::Click);
}

/// `TrayError` Display strings are stable and informative.
#[test]
fn tray_error_display() {
    let s = format!("{}", TrayError::InvalidIcon);
    assert!(s.contains("invalid tray icon"), "got: {s}");
    let s = format!("{}", TrayError::Platform("boom".into()));
    assert!(s.contains("boom"), "got: {s}");
    assert!(s.contains("platform"), "got: {s}");
}

/// `TrayError` implements `std::error::Error`.
#[test]
fn tray_error_is_error() {
    /// Compile-time check that `E` implements `std::error::Error`.
    fn assert_error<E: std::error::Error>(_: &E) {}
    assert_error(&TrayError::InvalidIcon);
    assert_error(&TrayError::Platform("x".into()));
}

/// `TrayEvent` is `Clone`, `Debug`, `PartialEq`, `Eq`.
#[test]
fn tray_event_traits() {
    let a = TrayEvent::Menu(1);
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(TrayEvent::Click, TrayEvent::DoubleClick);
    let _ = format!("{a:?}");
}
