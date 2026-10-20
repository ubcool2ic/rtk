//! Integration tests for the window API: icon, size constraints, resizable
//! flags, and the close protocol callback.

use rtk::{RtkError, Tk};

/// Builds a tiny 2×2 straight-RGBA buffer.
fn sample_rgba() -> Vec<u8> {
    vec![
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        0, 0, 255, 255, // blue
        255, 255, 255, 255, // white
    ]
}

/// Verifies iconphoto_rgba rejects zero-width or zero-height images.
#[test]
fn iconphoto_rgba_rejects_zero_size() {
    let mut tk = Tk::new();
    assert_eq!(tk.iconphoto_rgba(Vec::new(), 0, 0), Err(RtkError::InvalidImageSize));
    assert_eq!(tk.iconphoto_rgba(Vec::new(), 0, 4), Err(RtkError::InvalidImageSize));
    assert_eq!(tk.iconphoto_rgba(Vec::new(), 4, 0), Err(RtkError::InvalidImageSize));
    assert!(tk.window_icon().is_none());
}

/// Ensures iconphoto_rgba rejects buffers whose length does not match the dimensions.
#[test]
fn iconphoto_rgba_rejects_wrong_length() {
    let mut tk = Tk::new();
    // 2x2 needs 16 bytes; give 15.
    let mut bad = sample_rgba();
    bad.pop();
    assert_eq!(tk.iconphoto_rgba(bad, 2, 2), Err(RtkError::InvalidImageSize));
    assert!(tk.window_icon().is_none());
}

/// Checks that iconphoto_rgba accepts a valid buffer and records the icon size.
#[test]
fn iconphoto_rgba_accepts_valid_buffer() {
    let mut tk = Tk::new();
    tk.iconphoto_rgba(sample_rgba(), 2, 2).unwrap();
    assert_eq!(tk.window_icon(), Some((2, 2)));
}

/// Tests that a PNG encoded in memory round-trips through iconphoto_png with the correct size.
#[test]
fn iconphoto_png_roundtrip_size() {
    // Build a 3x2 PNG via tiny-skia and feed it back through iconphoto_png.
    let mut pixmap = tiny_skia::Pixmap::new(3, 2).unwrap();
    for (i, px) in pixmap.pixels_mut().iter_mut().enumerate() {
        let v = (i as u8).wrapping_mul(20);
        *px = tiny_skia::PremultipliedColorU8::from_rgba(v, v, v, 255).unwrap();
    }
    let png = pixmap.encode_png().unwrap();
    let mut tk = Tk::new();
    tk.iconphoto_png(&png).unwrap();
    assert_eq!(tk.window_icon(), Some((3, 2)));
}

/// Verifies iconphoto_png rejects non-PNG data with an invalid image size error.
#[test]
fn iconphoto_png_rejects_garbage() {
    let mut tk = Tk::new();
    assert_eq!(tk.iconphoto_png(b"not a png"), Err(RtkError::InvalidImageSize));
    assert!(tk.window_icon().is_none());
}

/// Tests that minsize and maxsize setters are reflected by their getters.
#[test]
fn minsize_maxsize_getters() {
    let mut tk = Tk::new();
    assert_eq!(tk.window_minsize(), None);
    assert_eq!(tk.window_maxsize(), None);
    tk.minsize(200, 150);
    tk.maxsize(800, 600);
    assert_eq!(tk.window_minsize(), Some((200, 150)));
    assert_eq!(tk.window_maxsize(), Some((800, 600)));
}

/// Verifies the resizable getter reflects each combination of width and height flags.
#[test]
fn resizable_getter() {
    let mut tk = Tk::new();
    assert_eq!(tk.is_resizable(), (true, true));
    tk.resizable(false, false);
    assert_eq!(tk.is_resizable(), (false, false));
    tk.resizable(true, false);
    assert_eq!(tk.is_resizable(), (true, false));
    tk.resizable(false, true);
    assert_eq!(tk.is_resizable(), (false, true));
}

/// Checks that a close protocol callback returning false vetoes quitting.
#[test]
fn protocol_close_veto_keeps_running() {
    let mut tk = Tk::new();
    tk.protocol_close(|_tk| false);
    let quit = tk.sim_close_request();
    assert!(!quit);
    assert!(!tk.is_quit_requested());
}

/// Verifies a close protocol callback returning true allows the window to quit.
#[test]
fn protocol_close_allow_quits() {
    let mut tk = Tk::new();
    tk.protocol_close(|_tk| true);
    let quit = tk.sim_close_request();
    assert!(quit);
    assert!(tk.is_quit_requested());
}

/// Ensures the default close behavior requests quit when no protocol is set.
#[test]
fn protocol_close_default_quits() {
    let mut tk = Tk::new();
    let quit = tk.sim_close_request();
    assert!(quit);
    assert!(tk.is_quit_requested());
}

/// Tests that a close protocol callback can re-enter the toolkit to change window state.
#[test]
fn protocol_close_callback_can_reenter() {
    let mut tk = Tk::new();
    tk.protocol_close(|tk| {
        tk.title("closing");
        false
    });
    assert!(!tk.sim_close_request());
    assert_eq!(tk.window_title(), "closing");
    assert!(!tk.is_quit_requested());
}

/// Verifies the window title getter returns the value set by title.
#[test]
fn window_title_getter() {
    let mut tk = Tk::new();
    tk.title("hello");
    assert_eq!(tk.window_title(), "hello");
}
