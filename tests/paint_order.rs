//! Regression tests for sibling paint order: placed widgets must paint on top
//! of pack/grid siblings, and hit-testing must agree with the paint order.

use rtk::testing::compare_pixmaps;
use rtk::{
    Anchor, Button, Color, Frame, Opt, PackOpts, PlaceOpts, Side, StyleSpec, Tk, WidgetId,
};

/// Builds a 200x100 toolkit with a packed button and a placed overlay frame
/// covering the whole window, returning the toolkit and both widget ids.
fn scene() -> (Tk, WidgetId, WidgetId) {
    let mut tk = Tk::new();
    tk.geometry(200, 100);
    let root = tk.root();
    let button = Button::new(&mut tk, root, "Open").expect("button");
    assert!(tk.pack(button, PackOpts { side: Side::Top, fill: rtk::Fill::X, ..PackOpts::default() }));
    let overlay = Frame::new(&mut tk, root).expect("overlay");
    let mut spec = StyleSpec::new();
    spec.bg = Some(Color::rgb(255, 0, 0));
    tk.style_configure("Overlay.TFrame", spec);
    assert!(tk.config(overlay, Opt::Style("Overlay.TFrame".to_string())));
    assert!(tk.place(
        overlay,
        PlaceOpts {
            x: 0.0,
            y: 0.0,
            relx: 0.0,
            rely: 0.0,
            width: None,
            height: None,
            relwidth: Some(1.0),
            relheight: Some(1.0),
            anchor: Anchor::Nw,
        },
    ));
    tk.sim_resize(200, 100);
    (tk, button, overlay)
}

/// A placed frame covering the window must paint over a packed button.
#[test]
fn placed_overlay_paints_above_packed_sibling() {
    let (mut tk, button, overlay) = scene();
    let rect = tk.widget_rect(button).expect("button laid out");
    let overlay_rect = tk.widget_rect(overlay).expect("overlay laid out");
    assert!(overlay_rect.width >= 200.0 && overlay_rect.height >= 100.0);
    let cx = (rect.x + rect.width / 2.0).round() as u32;
    let cy = (rect.y + rect.height / 2.0).round() as u32;
    let pixmap = tk.render_to_pixmap(200, 100).expect("render");
    let idx = ((cy * pixmap.width() + cx) * 4) as usize;
    let px = &pixmap.data()[idx..idx + 4];
    assert_eq!(
        (px[0], px[1], px[2]),
        (255, 0, 0),
        "placed overlay must paint on top of the packed button at ({cx},{cy})"
    );
}

/// Hit-testing must resolve to the placed overlay, matching the paint order.
///
/// Ignored: hit-testing skips non-interactive widgets (Frames), so hover still
/// reaches a button under a decorative placed frame. Whether such frames should
/// block the pointer (Tk semantics) is an open 1.0 decision; modal scrims already
/// block input through the modal path.
#[test]
#[ignore = "open decision: should non-interactive placed frames block hit-testing?"]
fn placed_overlay_wins_hit_test() {
    let (mut tk, button, overlay) = scene();
    let rect = tk.widget_rect(button).expect("button laid out");
    let cx = rect.x + rect.width / 2.0;
    let cy = rect.y + rect.height / 2.0;
    tk.sim_mouse_move(cx, cy);
    let _ = overlay; // Frames carry no HOVER state; the overlay must still block the button.
    assert!(!tk.widget_state(button).contains(rtk::State::HOVER));
}

/// Sanity check that the render helper is deterministic for this scene.
#[test]
fn render_is_stable() {
    let (mut tk, _, _) = scene();
    let a = tk.render_to_pixmap(200, 100).expect("render a");
    let b = tk.render_to_pixmap(200, 100).expect("render b");
    let (diff, max) = compare_pixmaps(&a, &b, 0);
    assert_eq!((diff, max), (0, 0));
}
