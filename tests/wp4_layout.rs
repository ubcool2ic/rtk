//! WP4 layout regressions: style margin (B9), Frame padding (B10),
//! input-event relayout skipping (P5), and ScrollArea scrollbar lookup (P11).

use rtk::{
    Button, Edges, Fill, Frame, Label, Opt, PackOpts, ScrollArea, Scrollbar, Side, StyleSpec, Tk,
};

fn almost(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.51
}

/// Style margin must sit outside `widget_rect`, not inflate the widget's own box.
#[test]
fn style_margin_is_outside_the_widget_box() {
    let mut tk = Tk::new();
    let root = tk.root();
    tk.sim_resize(400, 300);
    let first = Label::new(&mut tk, root, "Hello").unwrap();
    let second = Label::new(&mut tk, root, "Hello").unwrap();
    tk.pack(
        first,
        PackOpts {
            side: Side::Top,
            ..PackOpts::default()
        },
    );
    tk.style_configure(
        "Margin.TLabel",
        StyleSpec {
            margin: Some(Edges::all(10.0)),
            ..StyleSpec::default()
        },
    );
    tk.config(second, Opt::Style("Margin.TLabel".into()));
    tk.pack(
        second,
        PackOpts {
            side: Side::Top,
            ..PackOpts::default()
        },
    );

    let a = tk.widget_rect(first).expect("first rect");
    let b = tk.widget_rect(second).expect("second rect");
    assert!(
        almost(b.y, a.y + a.height + 10.0),
        "expected a 10px top-margin gap, first={a:?} second={b:?}"
    );
    assert!(
        almost(a.width, b.width) && almost(a.height, b.height),
        "margin must not stretch the widget box, first={a:?} second={b:?}"
    );

    let third = Label::new(&mut tk, root, "Hello").unwrap();
    tk.style_configure(
        "Asym.TLabel",
        StyleSpec {
            margin: Some(Edges::new(2.0, 8.0, 4.0, 6.0)),
            ..StyleSpec::default()
        },
    );
    tk.config(third, Opt::Style("Asym.TLabel".into()));
    tk.pack(
        third,
        PackOpts {
            side: Side::Top,
            ..PackOpts::default()
        },
    );
    let c = tk.widget_rect(third).expect("third rect");
    assert!(
        almost(c.y, b.y + b.height + 10.0 + 2.0),
        "asymmetric top margin after previous bottom margin, second={b:?} third={c:?}"
    );
    assert!(
        almost(c.height, a.height),
        "asymmetric margin must not stretch height, first={a:?} third={c:?}"
    );
}

/// Frame padding insets children after the 2px nested border; root has no border inset.
#[test]
fn frame_padding_insets_children_after_border() {
    let mut tk = Tk::new();
    let root = tk.root();
    tk.sim_resize(400, 300);

    let default_frame = Frame::new(&mut tk, root).unwrap();
    tk.pack(
        default_frame,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    let default_child = Label::new(&mut tk, default_frame, "plain").unwrap();
    tk.pack(
        default_child,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    let fr = tk.widget_rect(default_frame).expect("default frame");
    let ch = tk.widget_rect(default_child).expect("default child");
    assert!(
        almost(ch.x, fr.x + 2.0) && almost(ch.y, fr.y + 2.0),
        "default nested Frame must still inset children by only 2px, frame={fr:?} child={ch:?}"
    );

    let padded = Frame::new(&mut tk, root).unwrap();
    tk.style_configure(
        "Pad.TFrame",
        StyleSpec {
            padding: Some(Edges::all(12.0)),
            ..StyleSpec::default()
        },
    );
    tk.config(padded, Opt::Style("Pad.TFrame".into()));
    tk.pack(
        padded,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    let inner = Label::new(&mut tk, padded, "padded").unwrap();
    tk.pack(
        inner,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    let pf = tk.widget_rect(padded).expect("padded frame");
    let pi = tk.widget_rect(inner).expect("padded child");
    assert!(
        almost(pi.x, pf.x + 2.0 + 12.0) && almost(pi.y, pf.y + 2.0 + 12.0),
        "nested Frame content must be inset by 2px border plus padding, frame={pf:?} child={pi:?}"
    );

    let mut root_tk = Tk::new();
    let root = root_tk.root();
    root_tk.sim_resize(400, 300);
    root_tk.style_configure(
        "TFrame",
        StyleSpec {
            padding: Some(Edges::all(12.0)),
            ..StyleSpec::default()
        },
    );
    let root_child = Label::new(&mut root_tk, root, "root").unwrap();
    root_tk.pack(
        root_child,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    let rr = root_tk.widget_rect(root).expect("root");
    let rc = root_tk.widget_rect(root_child).expect("root child");
    assert!(
        almost(rc.x, rr.x + 12.0) && almost(rc.y, rr.y + 12.0),
        "root Frame has no 2px border; padding only, root={rr:?} child={rc:?}"
    );
}

/// Mouse-move events must not re-run layout once inputs are unchanged.
#[test]
fn mouse_moves_do_not_rerun_layout() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Hover").unwrap();
    tk.pack(btn, PackOpts::default());
    tk.sim_resize(400, 200);
    tk.render_frame(400, 200).unwrap();
    let after_render = tk.layout_pass_count();
    assert!(after_render > 0, "initial layout should have run");

    let rect = tk.widget_rect(btn).expect("button");
    for i in 0..40 {
        tk.sim_mouse_move(rect.x + 2.0 + i as f32 * 0.3, rect.y + 2.0);
        let _ = tk.widget_rect(btn);
    }
    assert_eq!(
        tk.layout_pass_count(),
        after_render,
        "hover/mouse-move must not re-run layout while inputs are unchanged"
    );

    let extra = Label::new(&mut tk, root, "extra").unwrap();
    tk.pack(
        extra,
        PackOpts {
            side: Side::Top,
            ..PackOpts::default()
        },
    );
    let _ = tk.widget_rect(extra);
    assert!(
        tk.layout_pass_count() > after_render,
        "packing a new widget must run layout again"
    );
}

/// Attaching scrollbars should not scan the arena on later layouts, and content
/// geometry should still account for a vertical bar.
#[test]
fn scroll_area_uses_stored_bar_ids() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 160.0, 100.0).unwrap();
    let content = tk.scrollarea_content(area).expect("content frame");
    let tall = Label::new(&mut tk, content, "inside").unwrap();
    tk.pack(
        tall,
        PackOpts {
            side: Side::Top,
            ..PackOpts::default()
        },
    );
    let vbar = Scrollbar::new(&mut tk, root).unwrap();
    tk.pack(
        area,
        PackOpts {
            side: Side::Left,
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    tk.pack(
        vbar,
        PackOpts {
            side: Side::Right,
            fill: Fill::Y,
            ..PackOpts::default()
        },
    );
    assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), None));
    tk.sim_resize(240, 140);

    let before = tk.scrollarea_bar_scan_count();
    let ra = tk.widget_rect(area).expect("area");
    let _rv = tk.widget_rect(vbar).expect("vbar");
    let _rc = tk.widget_rect(content).expect("content");
    let after_first = tk.scrollarea_bar_scan_count();
    assert_eq!(
        after_first, before,
        "attach stored the bar ids, so the first layout should not scan the arena"
    );

    // A second pass (still dirty until render) must not scan either.
    let _ = tk.widget_rect(area);
    assert_eq!(tk.scrollarea_bar_scan_count(), after_first);
    assert!(
        ra.width > 0.0 && ra.height > 0.0,
        "scroll area should be laid out, {ra:?}"
    );

    // link_scrollbar path: a new area without attach may scan once, then reuse the cache.
    let mut tk2 = Tk::new();
    let root2 = tk2.root();
    let area2 = ScrollArea::new(&mut tk2, root2, 160.0, 100.0).unwrap();
    let sb2 = Scrollbar::new(&mut tk2, root2).unwrap();
    tk2.pack(
        area2,
        PackOpts {
            side: Side::Left,
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    tk2.pack(
        sb2,
        PackOpts {
            side: Side::Right,
            fill: Fill::Y,
            ..PackOpts::default()
        },
    );
    assert!(tk2.link_scrollbar(sb2, area2));
    let scans0 = tk2.scrollarea_bar_scan_count();
    assert_eq!(scans0, 0, "no scan before the first layout");
    tk2.sim_resize(240, 140);
    let scans1 = tk2.scrollarea_bar_scan_count();
    assert!(
        scans1 > scans0,
        "link_scrollbar should fall back to one arena scan, before={scans0} after={scans1}"
    );
    let _ = tk2.widget_rect(area2);
    assert_eq!(
        tk2.scrollarea_bar_scan_count(),
        scans1,
        "subsequent layouts should reuse the cached bar ids"
    );
}
