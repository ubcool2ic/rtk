//! WP4 layout regressions: style margin (B9), Frame padding (B10),
//! and ScrollArea scrollbar lookup (P11). The P5 input-signature skip was
//! dropped (Reviewer1 musts 1–2; Tyler 2026-11-26: do it later).

use rtk::{
    dialog, Button, Edges, Fill, Frame, Label, Notebook, Opt, PackOpts, ScrollArea, Scrollbar,
    Side, StyleSpec, Tk,
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

/// Frame padding insets pack/grid children after the 2px nested border; root has no border inset.
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

/// B9: margin on a Notebook must not let the page escape the widget box.
#[test]
fn b9_notebook_margin_page_inside_box() {
    let mut tk = Tk::new();
    let root = tk.root();
    let nb = Notebook::new(&mut tk, root).unwrap();
    tk.config(
        nb,
        Opt::StyleOverride(StyleSpec {
            margin: Some(Edges::all(20.0)),
            ..StyleSpec::default()
        }),
    );
    let page = Frame::new(&mut tk, nb).unwrap();
    let l = Label::new(&mut tk, page, "page").unwrap();
    tk.pack(l, PackOpts::default());
    assert!(tk.notebook_add(nb, page, "Tab"));
    tk.pack(
        nb,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    tk.sim_resize(400, 300);
    let rn = tk.widget_rect(nb).unwrap();
    let rp = tk.widget_rect(page).unwrap();
    assert!(
        rp.x >= rn.x - 0.5
            && rp.y >= rn.y - 0.5
            && rp.x + rp.width <= rn.x + rn.width + 0.5
            && rp.y + rp.height <= rn.y + rn.height + 0.5,
        "page escapes notebook box: nb={rn:?} page={rp:?}"
    );
}

/// B9: margin on a ScrollArea must not let the content escape the widget box.
#[test]
fn b9_scrollarea_margin_content_inside_box() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 160.0, 100.0).unwrap();
    tk.config(
        area,
        Opt::StyleOverride(StyleSpec {
            margin: Some(Edges::all(20.0)),
            ..StyleSpec::default()
        }),
    );
    let content = tk.scrollarea_content(area).unwrap();
    let l = Label::new(&mut tk, content, "x").unwrap();
    tk.pack(l, PackOpts::default());
    tk.pack(
        area,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    tk.sim_resize(400, 300);
    let ra = tk.widget_rect(area).unwrap();
    let rc = tk.widget_rect(content).unwrap();
    assert!(
        rc.x >= ra.x - 0.5 && rc.y >= ra.y - 0.5,
        "content escapes area box: area={ra:?} content={rc:?}"
    );
}

/// B10: root TFrame padding must not shrink a place-managed modal scrim.
#[test]
fn b10_root_padding_vs_modal_scrim() {
    fn corner(dialog_on: bool) -> [u8; 4] {
        let mut tk = Tk::new();
        tk.style_configure(
            "TFrame",
            StyleSpec {
                padding: Some(Edges::all(12.0)),
                ..StyleSpec::default()
            },
        );
        let root = tk.root();
        let b = Button::new(&mut tk, root, "x").unwrap();
        tk.pack(b, PackOpts::default());
        if dialog_on {
            dialog::message_box(&mut tk, "T", "M", rtk::MessageKind::Info, |_| {});
        }
        let px = tk.render_to_pixmap(400, 300).unwrap();
        let c = px.pixel(3, 3).unwrap();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }
    let plain = corner(false);
    let dimmed = corner(true);
    assert_ne!(
        plain, dimmed,
        "scrim does not cover the window corner when root TFrame has padding"
    );
}

/// P11: a vbar linked after the area's first layout must still reserve width.
#[test]
fn p11_link_after_first_layout_reserves_vbar() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 160.0, 100.0).unwrap();
    let content = tk.scrollarea_content(area).unwrap();
    let l = Label::new(&mut tk, content, "inside").unwrap();
    tk.pack(l, PackOpts::default());
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
    tk.sim_resize(240, 140);
    let _ = tk.widget_rect(area);
    assert!(tk.link_scrollbar(vbar, area));
    tk.sim_resize(240, 140);
    let ra = tk.widget_rect(area).unwrap();
    let rv = tk.widget_rect(vbar).unwrap();
    let rc = tk.widget_rect(content).unwrap();
    assert!(
        (rc.width - (ra.width - rv.width)).abs() < 0.6,
        "content must leave room for the linked vbar: area={ra:?} vbar={rv:?} content={rc:?}"
    );
}

/// P11: attach a vbar, then link an hbar later; min content width is the area width.
#[test]
fn p11_attach_then_link_hbar() {
    let mut tk = Tk::new();
    let root = tk.root();
    let area = ScrollArea::new(&mut tk, root, 160.0, 100.0).unwrap();
    let content = tk.scrollarea_content(area).unwrap();
    let l = Label::new(&mut tk, content, "x").unwrap();
    tk.pack(l, PackOpts::default());
    let vbar = Scrollbar::new(&mut tk, root).unwrap();
    let hbar = Scrollbar::horizontal(&mut tk, root).unwrap();
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
    tk.pack(
        hbar,
        PackOpts {
            side: Side::Bottom,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), None));
    tk.sim_resize(240, 140);
    let _ = tk.widget_rect(area);
    assert!(tk.link_scrollbar(hbar, area));
    tk.sim_resize(240, 140);
    let ra = tk.widget_rect(area).unwrap();
    let rc = tk.widget_rect(content).unwrap();
    assert!(
        rc.width >= ra.width - 0.6,
        "hbar link ignored: area={ra:?} content={rc:?}"
    );
}

/// Attaching both bars still accounts for the vbar, and link_scrollbar after
/// a first layout is covered by the P11 tests above.
#[test]
fn scroll_area_attach_reserves_vbar() {
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
    let hbar = Scrollbar::horizontal(&mut tk, root).unwrap();
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
    tk.pack(
        hbar,
        PackOpts {
            side: Side::Bottom,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );
    assert!(tk.scrollarea_attach_scrollbars(area, Some(vbar), Some(hbar)));
    tk.sim_resize(240, 140);
    let ra = tk.widget_rect(area).expect("area");
    let rv = tk.widget_rect(vbar).expect("vbar");
    let rc = tk.widget_rect(content).expect("content");
    assert!(
        ra.width > 0.0 && ra.height > 0.0,
        "scroll area should be laid out, {ra:?}"
    );
    // With an hbar, min content width is the area width (see layout()).
    assert!(
        rc.width >= ra.width - 0.6,
        "attached hbar should keep content at least area-wide: area={ra:?} vbar={rv:?} content={rc:?}"
    );
}
