//! Headless integration tests for milestone 0.11 toast notifications.

use rtk::{Rect, ToastKind, ToastPosition, Tk};

/// Builds a toolkit with a fixed 640x480 viewport.
fn make_tk() -> Tk {
    let mut tk = Tk::new();
    let _ = tk.render_to_pixmap(640, 480);
    tk
}

/// A toast auto-dismisses after its duration once the animation clock advances.
#[test]
fn toast_auto_dismisses_after_duration() {
    let mut tk = make_tk();
    let id = tk.toast_with("saved", ToastKind::Success, 1000);
    assert_eq!(tk.toasts().len(), 1);
    assert!(tk.toast_rect(id).is_some());

    tk.sim_advance_time(500);
    assert_eq!(tk.toasts().len(), 1);

    // Past the duration: fade-out starts, still visible.
    tk.sim_advance_time(600);
    assert_eq!(tk.toasts().len(), 1);

    // Fade-out completes and the toast is removed.
    tk.sim_advance_time(300);
    assert!(tk.toasts().is_empty());
    assert!(tk.toast_rect(id).is_none());
}

/// A sticky toast (duration 0) never auto-dismisses.
#[test]
fn sticky_toast_stays() {
    let mut tk = make_tk();
    let id = tk.toast_with("sticky", ToastKind::Info, 0);
    tk.sim_advance_time(60_000);
    assert_eq!(tk.toasts().len(), 1);
    assert!(tk.toast_rect(id).is_some());
    assert!(tk.toast_dismiss(id));
    tk.sim_advance_time(400);
    assert!(tk.toasts().is_empty());
}

/// Exceeding the maximum drops the oldest toasts.
#[test]
fn max_drops_oldest() {
    let mut tk = make_tk();
    tk.toast_set_max(2);
    let a = tk.toast("a");
    let b = tk.toast("b");
    let c = tk.toast("c");
    let ids: Vec<_> = tk.toasts().into_iter().map(|(id, _, _)| id).collect();
    assert_eq!(ids, vec![b, c]);
    assert!(tk.toast_rect(a).is_none());
    assert!(tk.toast_rect(b).is_some());
}

/// Dismissal by id and by click both remove the toast after the fade-out.
#[test]
fn dismiss_by_id_and_click() {
    let mut tk = make_tk();
    let id = tk.toast_with("hello", ToastKind::Warning, 0);
    assert!(tk.toast_dismiss(id));
    assert!(!tk.toast_dismiss(id));
    tk.sim_advance_time(300);
    assert!(tk.toasts().is_empty());

    let id2 = tk.toast_with("click me", ToastKind::Error, 0);
    let rect = tk.toast_rect(id2).expect("toast rect");
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_advance_time(300);
    assert!(tk.toasts().is_empty());
}

/// `is_animating` is true while a toast is visible and false once gone.
#[test]
fn is_animating_tracks_toasts() {
    let mut tk = make_tk();
    assert!(!tk.is_animating());
    let _ = tk.toast_with("x", ToastKind::Info, 200);
    assert!(tk.is_animating());
    tk.sim_advance_time(1000);
    assert!(!tk.is_animating());
}

/// Each position anchors the toast stack in the expected corner.
#[test]
fn position_variants_place_toasts() {
    let mut tk = make_tk();
    let id = tk.toast_with("pos", ToastKind::Info, 0);

    tk.toast_set_position(ToastPosition::BottomRight);
    let br = tk.toast_rect(id).expect("br");
    assert!(br.x + br.width <= 640.0);
    assert!(br.y + br.height <= 480.0);
    assert!(br.x > 320.0);
    assert!(br.y > 240.0);

    tk.toast_set_position(ToastPosition::BottomCenter);
    let bc = tk.toast_rect(id).expect("bc");
    assert!(((bc.x + bc.width / 2.0) - 320.0).abs() < 1.0);
    assert!(bc.y > 240.0);

    tk.toast_set_position(ToastPosition::TopRight);
    let tr = tk.toast_rect(id).expect("tr");
    assert!(tr.y < 240.0);
    assert!(tr.x > 320.0);

    tk.toast_set_position(ToastPosition::TopCenter);
    let tc = tk.toast_rect(id).expect("tc");
    assert!(((tc.x + tc.width / 2.0) - 320.0).abs() < 1.0);
    assert!(tc.y < 240.0);
}

/// Toasts render without panicking and produce a non-empty pixmap.
#[test]
fn toasts_render() {
    let mut tk = make_tk();
    let baseline = tk.render_to_pixmap(640, 480).expect("baseline render");
    let baseline_data = baseline.data().to_vec();
    let _ = tk.toast_with("info", ToastKind::Info, 0);
    let _ = tk.toast_with("ok", ToastKind::Success, 0);
    let _ = tk.toast_with("warn", ToastKind::Warning, 0);
    let _ = tk.toast_with("err", ToastKind::Error, 0);
    // Let the fade-in finish so the toasts are fully opaque.
    tk.sim_advance_time(500);
    let pixmap = tk.render_to_pixmap(640, 480).expect("render");
    assert_eq!(pixmap.width(), 640);
    assert_ne!(
        pixmap.data(),
        &baseline_data[..],
        "rendering toasts must change the pixels"
    );
    let rect: Rect = tk.toast_rect(tk.toasts()[0].0).expect("rect");
    assert!(rect.width > 0.0);
}
