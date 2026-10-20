//! Golden-image (screenshot) tests for the milestone 0.11 widgets: Table,
//! TreeView, menus/menubar, toasts and nested modal dialogs.
//!
//! Each scene is rendered across the same theme × scale matrix used by
//! `tests/golden_widgets.rs` (Light, Dark, HighContrast at 1.0 and 2.0) and
//! compared against a checked-in PNG under `tests/golden`. Set `RTK_BLESS=1`
//! to (re)write the golden images.

use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
use rtk::{
    dialog, Button, Frame, Key, Label, MenuItem, Menubar, PackOpts, Table, Theme, Tk, ToastKind,
    TreeView,
};

/// The themes exercised by every golden scene.
const THEMES: [Theme; 3] = [Theme::Light, Theme::Dark, Theme::HighContrast];

/// The scale factors exercised by every golden scene.
const SCALES: [f64; 2] = [1.0, 2.0];

/// Returns the golden name for `prefix` at `theme` and `scale`, matching the
/// `widgets_<theme>_<n>x` convention of `tests/golden_widgets.rs`.
fn golden_name(prefix: &str, theme: Theme, scale: f64) -> String {
    format!("{prefix}_{theme:?}_{scale}x").to_lowercase()
}

/// Builds the Table scene: Name/Qty/Price columns, 8 rows, sorted by Qty
/// descending, with row 2 selected.
fn build_table_scene(tk: &mut Tk) {
    let root = tk.root();
    let table = Table::new(tk, root, &["Name", "Qty", "Price"], 8).expect("table");
    tk.pack(
        table,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );
    let rows: Vec<Vec<String>> = vec![
        vec!["Widget".into(), "12".into(), "3.50".into()],
        vec!["Gadget".into(), "48".into(), "1.25".into()],
        vec!["Sprocket".into(), "7".into(), "9.99".into()],
        vec!["Cog".into(), "31".into(), "2.10".into()],
        vec!["Lever".into(), "19".into(), "4.75".into()],
        vec!["Spring".into(), "56".into(), "0.80".into()],
        vec!["Bolt".into(), "23".into(), "1.05".into()],
        vec!["Nut".into(), "40".into(), "0.45".into()],
    ];
    tk.table_set_rows(table, rows);
    tk.table_sort_by(table, 1, false);
    tk.table_select(table, Some(2));
}

/// Builds the TreeView scene: two roots, the first open with three children
/// (one child open with a grandchild), with a child selected.
fn build_tree_scene(tk: &mut Tk) {
    let root = tk.root();
    let tree = TreeView::new(tk, root, 8).expect("tree");
    tk.pack(
        tree,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );
    let a = tk.tree_insert(tree, None, "Alpha").expect("root a");
    let b = tk.tree_insert(tree, None, "Beta").expect("root b");
    let a1 = tk.tree_insert(tree, Some(a), "Alpha one").expect("child a1");
    let a2 = tk.tree_insert(tree, Some(a), "Alpha two").expect("child a2");
    let _a3 = tk.tree_insert(tree, Some(a), "Alpha three").expect("child a3");
    let _a2x = tk.tree_insert(tree, Some(a2), "Alpha two.x").expect("grandchild");
    let _b1 = tk.tree_insert(tree, Some(b), "Beta one").expect("child b1");
    tk.tree_set_open(tree, a, true);
    tk.tree_set_open(tree, a2, true);
    tk.tree_select(tree, Some(a1));
}

/// Builds the menu scene: a menubar with File/Edit plus an open popup that
/// contains an accelerated command, a separator, a checked item, a disabled
/// item and a submenu, with the first item highlighted.
fn build_menu_scene(tk: &mut Tk) {
    let root = tk.root();
    let bar = Menubar::new(tk, root).expect("menubar");
    tk.pack(
        bar,
        PackOpts {
            side: rtk::Side::Top,
            fill: rtk::Fill::X,
            ..PackOpts::default()
        },
    );
    let file = tk.menu_create();
    tk.menu_add(file, MenuItem::command("Open"));
    tk.menu_add(file, MenuItem::command("Quit"));
    let edit = tk.menu_create();
    tk.menu_add(edit, MenuItem::command("Copy"));
    tk.menubar_add(bar, "File", file);
    tk.menubar_add(bar, "Edit", edit);

    let popup = tk.menu_create();
    tk.menu_add(popup, MenuItem::command("Save").with_accel("Ctrl+S"));
    tk.menu_add(popup, MenuItem::separator());
    let check_var = tk.bool_var(true);
    tk.menu_add(popup, MenuItem::check("Word wrap", check_var));
    let disabled = tk.menu_add(popup, MenuItem::command("Disabled"));
    tk.menu_set_enabled(popup, disabled, false);
    let sub = tk.menu_create();
    tk.menu_add(sub, MenuItem::command("Nested"));
    tk.menu_add(popup, MenuItem::submenu("More", sub));

    tk.menu_popup(popup, 40.0, 40.0);
    // Highlight the first item.
    tk.sim_mouse_move(60.0, 50.0);
}

/// Builds the toast scene: three sticky toasts (Info, Success, Error) after
/// enough simulated time to be fully faded in.
fn build_toast_scene(tk: &mut Tk) {
    let root = tk.root();
    let label = Label::new(tk, root, "Toasts").expect("label");
    tk.pack(label, PackOpts::default());
    tk.toast_with("Saved successfully", ToastKind::Success, 0);
    tk.toast_with("Something went wrong", ToastKind::Error, 0);
    tk.toast_with("Heads up", ToastKind::Info, 0);
    tk.sim_advance_time(400);
}

/// Builds the nested modal scene: a message box with a yes/no dialog on top.
fn build_modal_scene(tk: &mut Tk) {
    let root = tk.root();
    let label = Label::new(tk, root, "Background").expect("label");
    tk.pack(label, PackOpts::default());
    let button = Button::new(tk, root, "Open").expect("button");
    tk.pack(button, PackOpts::default());
    dialog::message_box(tk, "Notice", "This is a message.", rtk::MessageKind::Info, |_| {});
    dialog::ask_yes_no(tk, "Confirm", "Are you sure?", |_, _| {});
}

/// Renders the Table scene across themes and scales and compares goldens.
#[test]
fn golden_m011_table() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_table_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (360.0 * scale) as u32;
            let h = (200.0 * scale) as u32;
            let name = golden_name("m011_table", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Renders the TreeView scene across themes and scales and compares goldens.
#[test]
fn golden_m011_tree() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_tree_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (280.0 * scale) as u32;
            let h = (200.0 * scale) as u32;
            let name = golden_name("m011_tree", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Renders the menu scene across themes and scales and compares goldens.
#[test]
fn golden_m011_menu() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_menu_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (360.0 * scale) as u32;
            let h = (240.0 * scale) as u32;
            let name = golden_name("m011_menu", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Renders the toast scene across themes and scales and compares goldens.
#[test]
fn golden_m011_toast() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_toast_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (360.0 * scale) as u32;
            let h = (240.0 * scale) as u32;
            let name = golden_name("m011_toast", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Renders the nested modal scene across themes and scales and compares goldens.
#[test]
fn golden_m011_modal() {
    for theme in THEMES {
        for scale in SCALES {
            let mut tk = Tk::new();
            tk.set_theme(theme);
            build_modal_scene(&mut tk);
            assert!(tk.sim_scale_factor(scale));
            let w = (480.0 * scale) as u32;
            let h = (320.0 * scale) as u32;
            let name = golden_name("m011_modal", theme, scale);
            assert_golden(&mut tk, golden_dir(), &name, w, h, GoldenOpts::default());
        }
    }
}

/// Verifies that the menu scene responds to keyboard navigation without
/// panicking (a smoke check that the golden scene is interactive).
#[test]
fn golden_m011_menu_keyboard_smoke() {
    let mut tk = Tk::new();
    build_menu_scene(&mut tk);
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Up);
    assert!(tk.menu_is_open());
    assert!(tk.render_to_pixmap(360, 240).is_ok());
}

/// Verifies that the nested modal scene reports a depth of two.
#[test]
fn golden_m011_modal_depth() {
    let mut tk = Tk::new();
    build_modal_scene(&mut tk);
    assert_eq!(dialog::modal_depth(&tk), 2);
    assert!(tk.render_to_pixmap(360, 240).is_ok());
}

/// Ensures the scene builders do not leave the toolkit in a broken state by
/// rendering each scene once at 1x.
#[test]
fn golden_m011_scenes_render() {
    let mut tk = Tk::new();
    build_table_scene(&mut tk);
    assert!(tk.render_to_pixmap(360, 200).is_ok());

    let mut tk = Tk::new();
    build_tree_scene(&mut tk);
    assert!(tk.render_to_pixmap(280, 200).is_ok());

    let mut tk = Tk::new();
    build_toast_scene(&mut tk);
    assert!(tk.render_to_pixmap(360, 240).is_ok());

    let mut tk = Tk::new();
    build_menu_scene(&mut tk);
    assert!(tk.render_to_pixmap(360, 240).is_ok());

    let mut tk = Tk::new();
    build_modal_scene(&mut tk);
    assert!(tk.render_to_pixmap(480, 320).is_ok());

    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, PackOpts::default());
    assert!(tk.render_to_pixmap(360, 240).is_ok());
}
