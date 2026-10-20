//! Regression tests for ScrollArea content-width minima and headless rendering.

use rtk::{Label, PackOpts, ScrollArea, Scrollbar, Tk};

/// Vertical-only areas reserve the vertical scrollbar width, while two-axis
/// areas retain the full-width page size used by horizontal scrolling.
#[test]
fn content_width_matches_scrollbar_axes() {
    for horizontal in [false, true] {
        let mut tk = Tk::new();
        let root = tk.root();
        let area = ScrollArea::new(&mut tk, root, 200.0, 120.0).expect("area");
        let vbar = Scrollbar::new(&mut tk, root).expect("vertical scrollbar");
        assert!(tk.pack(area, PackOpts::default()));
        assert!(tk.pack(vbar, PackOpts::default()));

        let hbar = if horizontal {
            let bar = Scrollbar::horizontal(&mut tk, root).expect("horizontal scrollbar");
            assert!(tk.pack(bar, PackOpts::default()));
            Some(bar)
        } else {
            None
        };

        let content = tk.scrollarea_content(area).expect("content frame");
        for row in 0..30 {
            let label = Label::new(&mut tk, content, format!("row {row}")).expect("label");
            assert!(tk.pack(label, PackOpts::default()));
        }

        // Attach before the first layout as well as exercising later layouts.
        assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), hbar));
        tk.sim_resize(320, 200);
        let area_rect = tk.widget_rect(area).expect("area rectangle");
        let bar_rect = tk.widget_rect(vbar).expect("scrollbar rectangle");
        assert!(bar_rect.width > 0.0 && bar_rect.height > 0.0);

        let expected = if horizontal {
            area_rect.width
        } else {
            (area_rect.width - bar_rect.width).max(0.0)
        };
        let (width, _) = tk.scrollarea_content_size(area).expect("content size");
        assert!(
            (width - expected).abs() < 0.5,
            "horizontal={horizontal}: content width {width}, expected {expected}"
        );

        let pixmap = tk.render_to_pixmap(320, 200).expect("headless rendering");
        assert_eq!((pixmap.width(), pixmap.height()), (320, 200));
        let (width, _) = tk.scrollarea_content_size(area).expect("rendered content size");
        assert!(
            (width - expected).abs() < 0.5,
            "horizontal={horizontal}: rendered content width {width}, expected {expected}"
        );
    }
}
