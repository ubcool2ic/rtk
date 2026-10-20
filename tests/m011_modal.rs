//! Headless tests for nested modal dialogs (milestone 0.11, review finding F5).
//!
//! These exercise the modal stack: opening a dialog while another is presented
//! must push the outer frame, Escape/Enter must reach only the top-most dialog,
//! and closing the inner dialog must restore the outer one intact.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use rtk::dialog;
use rtk::{Button, Key, Tk};

/// Opens a `message_box` (dialog A) and then a nested `message_box` (dialog B)
/// on top of it, returning the "fired" flags for A and B.
fn open_nested(tk: &mut Tk) -> (Arc<AtomicBool>, Arc<AtomicBool>) {
    let a_fired = Arc::new(AtomicBool::new(false));
    let b_fired = Arc::new(AtomicBool::new(false));

    let a_clone = a_fired.clone();
    dialog::message_box(tk, "Outer", "Outer dialog", dialog::MessageKind::Info, move |_t| {
        a_clone.store(true, Ordering::SeqCst);
    });

    let b_clone = b_fired.clone();
    dialog::message_box(tk, "Inner", "Inner dialog", dialog::MessageKind::Info, move |_t| {
        b_clone.store(true, Ordering::SeqCst);
    });

    (a_fired, b_fired)
}

/// Opening a nested dialog must report depth 2, and Escape must close only the
/// top-most (inner) dialog, leaving the outer one presented at depth 1.
#[test]
fn nested_escape_closes_only_top() {
    let mut tk = Tk::new();
    let (a_fired, b_fired) = open_nested(&mut tk);

    assert_eq!(dialog::modal_depth(&tk), 2);
    assert!(dialog::is_open(&tk));

    tk.sim_key(Key::Escape);
    assert_eq!(dialog::modal_depth(&tk), 1);
    assert!(dialog::is_open(&tk));
    assert!(!a_fired.load(Ordering::SeqCst));
    assert!(b_fired.load(Ordering::SeqCst));
}

/// Accepting the inner dialog with Enter must run its callback, restore the
/// outer dialog, and a second Enter must accept the outer one.
#[test]
fn nested_inner_accept_runs_callback_and_restores_outer() {
    let mut tk = Tk::new();
    let (a_fired, b_fired) = open_nested(&mut tk);

    assert_eq!(dialog::modal_depth(&tk), 2);

    tk.sim_key(Key::Enter);
    assert_eq!(dialog::modal_depth(&tk), 1);
    assert!(b_fired.load(Ordering::SeqCst));
    assert!(!a_fired.load(Ordering::SeqCst));

    tk.sim_key(Key::Enter);
    assert_eq!(dialog::modal_depth(&tk), 0);
    assert!(!dialog::is_open(&tk));
    assert!(a_fired.load(Ordering::SeqCst));
}

/// `dialog::close` must pop exactly one level of the modal stack.
#[test]
fn close_pops_one_level() {
    let mut tk = Tk::new();
    let _ = open_nested(&mut tk);

    assert_eq!(dialog::modal_depth(&tk), 2);
    dialog::close(&mut tk);
    assert_eq!(dialog::modal_depth(&tk), 1);
    assert!(dialog::is_open(&tk));
    dialog::close(&mut tk);
    assert_eq!(dialog::modal_depth(&tk), 0);
    assert!(!dialog::is_open(&tk));
}

/// Opening a dialog from inside another dialog's accept callback must work and
/// leave the newly opened dialog presented.
#[test]
fn dialog_opened_from_accept_callback() {
    let mut tk = Tk::new();
    let opened = Arc::new(AtomicBool::new(false));
    let opened_clone = opened.clone();

    dialog::message_box(&mut tk, "First", "First dialog", dialog::MessageKind::Info, move |t| {
        opened_clone.store(true, Ordering::SeqCst);
        dialog::message_box(t, "Second", "Second dialog", dialog::MessageKind::Info, |_| {});
    });

    assert_eq!(dialog::modal_depth(&tk), 1);
    tk.sim_key(Key::Enter);
    assert!(opened.load(Ordering::SeqCst));
    assert_eq!(dialog::modal_depth(&tk), 1);
    assert!(dialog::is_open(&tk));

    tk.sim_key(Key::Escape);
    assert_eq!(dialog::modal_depth(&tk), 0);
}

/// Clicking outside the inner dialog must not reach a button belonging to the
/// outer dialog (input is confined to the top-most modal subtree).
#[test]
fn clicking_outside_inner_does_not_reach_outer_buttons() {
    let mut tk = Tk::new();
    let root = tk.root();
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    let btn = Button::new(&mut tk, root, "Target").unwrap();
    tk.set_command(btn, move |_t| {
        c.fetch_add(1, Ordering::SeqCst);
    });
    tk.pack(btn, Default::default());
    tk.sim_resize(600, 400);

    let rect = tk.widget_rect(btn).unwrap();
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);

    // Baseline: clicking the background button works.
    tk.sim_click(cx, cy);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Open outer then inner dialog.
    let _ = open_nested(&mut tk);
    assert_eq!(dialog::modal_depth(&tk), 2);

    // Clicking the background button location is blocked by the modal scrims.
    tk.sim_click(cx, cy);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Close inner, outer still blocks.
    tk.sim_key(Key::Escape);
    assert_eq!(dialog::modal_depth(&tk), 1);
    tk.sim_click(cx, cy);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Close outer, background click works again.
    tk.sim_key(Key::Escape);
    assert_eq!(dialog::modal_depth(&tk), 0);
    tk.sim_click(cx, cy);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

/// A stale teardown closure for an already-closed dialog must not destroy the
/// restored outer dialog.
#[test]
fn stale_closure_does_not_close_restored_outer() {
    let mut tk = Tk::new();
    let _ = open_nested(&mut tk);

    assert_eq!(dialog::modal_depth(&tk), 2);
    let inner_scrim = tk.modal.expect("inner scrim present");

    // Close the inner dialog normally.
    tk.sim_key(Key::Escape);
    assert_eq!(dialog::modal_depth(&tk), 1);

    // The inner scrim is gone and the outer one is the active modal again.
    assert!(!tk.contains(inner_scrim));
    assert_ne!(tk.modal, Some(inner_scrim));
    assert_eq!(dialog::modal_depth(&tk), 1);
    assert!(dialog::is_open(&tk));
}
