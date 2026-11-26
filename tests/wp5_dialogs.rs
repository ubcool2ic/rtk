//! WP5 Dialogs and data widgets (B11–B16, B20, L2–L5, P8–P9).

use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rtk::testing::{assert_golden, GoldenOpts};
use rtk::{
    dialog, Button, Combobox, DoubleSpinbox, Frame, Key, Listbox, MenuItem, PackOpts, Slider,
    Spinbox, Table, Tk, TreeView,
};

fn unique_tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rtk_wp5_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// B11/B12: Open dialog filter refresh, tar.gz matching, and listbox activate.
#[test]
fn open_dialog_filter_refresh_targz_and_activate() {
    let tmp = unique_tmp("open");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("archive.tar.gz"), b"tgz").unwrap();
    std::fs::write(tmp.join("plain.gz"), b"gz").unwrap();
    std::fs::write(tmp.join("data.bin"), b"bin").unwrap();

    let mut tk = Tk::new();
    let accepted = Rc::new(RefCell::new(None::<Option<PathBuf>>));
    let sink = accepted.clone();
    dialog::ask_open_filename(
        &mut tk,
        "Open",
        &tmp,
        &[
            ("Binary", &["bin"]),
            ("Archive", &["tar.gz"]),
        ],
        move |_t, res| {
            *sink.borrow_mut() = Some(res);
        },
    );
    assert!(dialog::is_open(&tk));
    tk.sim_resize(640, 480);

    let listbox = dialog::file_listbox(&tk).expect("listbox");
    let items = tk.listbox_get(listbox);
    assert!(items.contains(&"data.bin".to_string()), "initial Binary filter: {items:?}");
    assert!(!items.contains(&"archive.tar.gz".to_string()), "tar.gz must not match Binary: {items:?}");
    assert!(!items.contains(&"plain.gz".to_string()), "plain.gz must not match Binary: {items:?}");

    let combo = dialog::file_filter_combobox(&tk).expect("filter combobox");
    let rect = tk.widget_rect(combo).unwrap();
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    // Wheel on a closed combobox goes to the next value and fires on_select (B11/B15).
    tk.sim_wheel(-1.0);
    let items = tk.listbox_get(listbox);
    assert!(
        items.contains(&"archive.tar.gz".to_string()),
        "Archive filter must list tar.gz: {items:?}"
    );
    assert!(
        !items.contains(&"plain.gz".to_string()),
        "Path::extension gz must not satisfy tar.gz: {items:?}"
    );
    assert!(!items.contains(&"data.bin".to_string()), "bin must drop off Archive filter: {items:?}");

    let idx = items.iter().position(|s| s == "archive.tar.gz").unwrap();
    tk.listbox_select(listbox, Some(idx));
    let lb = tk.widget_rect(listbox).unwrap();
    // Click the selected row twice to activate (same path as Open/Enter).
    let x = lb.x + lb.width / 2.0;
    let y = lb.y + 8.0 + (idx as f32) * 16.0;
    tk.sim_click(x, y);
    tk.sim_click(x, y);

    assert!(!dialog::is_open(&tk), "activate should close the dialog");
    let path = accepted
        .borrow_mut()
        .take()
        .expect("callback")
        .expect("file chosen");
    assert_eq!(path.file_name().unwrap(), "archive.tar.gz");

    let _ = std::fs::remove_dir_all(tmp);
}

/// B13: removing or collapsing nodes clamps the scroll position so the body is not blank.
#[test]
fn tree_remove_and_collapse_clamps_top() {
    let mut tk = Tk::new();
    let root = tk.root();
    let tree = TreeView::new(&mut tk, root, 3).unwrap();
    tk.pack(tree, PackOpts::default());
    tk.sim_resize(240, 160);
    let parent = tk.tree_insert(tree, None, "root").unwrap();
    tk.tree_set_open(tree, parent, true);
    let mut last = 0;
    for i in 0..30 {
        last = tk.tree_insert(tree, Some(parent), &format!("n{i}")).unwrap();
    }
    assert!(tk.tree_see(tree, last));
    let top_before = tk.tree_first_visible(tree);
    assert!(top_before > 0, "should be scrolled down, top={top_before}");

    assert!(tk.tree_collapse_all(tree));
    let top_after_collapse = tk.tree_first_visible(tree);
    let visible = tk.tree_visible_count(tree);
    assert!(
        top_after_collapse < visible.max(1) || top_after_collapse == 0,
        "collapse must clamp top ({top_after_collapse}) into {visible} visible rows"
    );

    tk.tree_set_open(tree, parent, true);
    assert!(tk.tree_see(tree, last));
    // Removing the last rows of a scrolled view must not leave top past the end.
    for key in tk.tree_children(tree, Some(parent)) {
        tk.tree_remove(tree, key);
    }
    let top = tk.tree_first_visible(tree);
    let vis = tk.tree_visible_count(tree);
    assert!(vis == 0 || top < vis.max(1), "top={top} vis={vis}");
    let _ = tk.render_to_pixmap(240, 160);
}

/// B14: listbox_delete must fire the select callback when the selection changes.
#[test]
fn listbox_delete_dispatches_select() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lb = Listbox::new(&mut tk, root, 5).unwrap();
    tk.pack(lb, PackOpts::default());
    for (i, name) in ["a", "b", "c"].iter().enumerate() {
        tk.listbox_insert(lb, i, *name);
    }
    tk.listbox_select(lb, Some(1));
    let log = Rc::new(RefCell::new(Vec::<Option<usize>>::new()));
    let sink = log.clone();
    tk.listbox_bind_select(lb, move |_t, sel| {
        sink.borrow_mut().push(sel);
    });
    assert!(tk.listbox_delete(lb, 1, 1));
    assert_eq!(&*log.borrow(), &[None], "deleting the selected row must dispatch None");
    tk.listbox_select(lb, Some(0));
    log.borrow_mut().clear();
    assert!(tk.listbox_delete(lb, 1, 1));
    assert!(log.borrow().is_empty(), "deleting a non-selected row must not dispatch, got {:?}", log.borrow());
}

/// L3: destroying a widget must drop its file-drop closure immediately.
#[test]
fn destroy_drops_dnd_callbacks() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();
    let dropped = Arc::new(AtomicBool::new(false));
    struct Flag(Arc<AtomicBool>);
    impl Drop for Flag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let flag = Flag(dropped.clone());
    assert!(tk.bind_file_drop(frame, move |_t, _paths| {
        let _keep = &flag;
    }));
    assert!(tk.destroy(frame));
    assert!(dropped.load(Ordering::SeqCst), "drop closure must be pruned in destroy");
}

/// L4: menu_destroy and unbind_context_menu, plus prune on widget destroy.
#[test]
fn menu_destroy_and_unbind_context() {
    let mut tk = Tk::new();
    let root = tk.root();
    let host = Button::new(&mut tk, root, "host").unwrap();
    tk.pack(host, PackOpts::default());
    tk.sim_resize(200, 200);
    let menu = tk.menu_create();
    tk.menu_add(menu, MenuItem::command("One"));
    assert!(tk.bind_context_menu(host, menu));
    assert!(tk.unbind_context_menu(host));
    assert!(!tk.unbind_context_menu(host));
    assert!(tk.bind_context_menu(host, menu));
    assert!(tk.menu_destroy(menu));
    assert!(!tk.menu_destroy(menu));
    assert!(!tk.bind_context_menu(host, menu), "destroyed menu cannot be rebound");

    let menu2 = tk.menu_create();
    tk.menu_add(menu2, MenuItem::command("Two"));
    let host2 = Button::new(&mut tk, root, "host2").unwrap();
    tk.pack(host2, PackOpts::default());
    assert!(tk.bind_context_menu(host2, menu2));
    assert!(tk.destroy(host2));
    // Binding should have been pruned; a new bind on a live widget still works.
    let host3 = Button::new(&mut tk, root, "host3").unwrap();
    assert!(tk.bind_context_menu(host3, menu2));
}

/// P9: bulk table_insert_row keeps model order without quadratic layout/sort on each row.
#[test]
fn table_bulk_insert_keeps_order() {
    let mut tk = Tk::new();
    let root = tk.root();
    let table = Table::new(&mut tk, root, &["A"], 5).unwrap();
    tk.pack(table, PackOpts::default());
    tk.sim_resize(320, 200);
    for i in 0..200 {
        assert!(tk.table_insert_row(table, i, vec![format!("{i}")]));
    }
    assert_eq!(tk.table_row_count(table), 200);
    assert_eq!(tk.table_display_to_model(table, 0), Some(0));
    assert_eq!(tk.table_display_to_model(table, 199), Some(199));
    assert_eq!(tk.table_cell(table, 50, 0).as_deref(), Some("50"));
}

/// L5: dspin edit buffer is capped at 64 bytes (widget stays usable after a long type-in).
#[test]
fn dspin_buffer_is_capped() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let spin = DoubleSpinbox::new(&mut tk, root, 0.0, 1e15, var).unwrap();
    tk.pack(spin, PackOpts::default());
    tk.sim_resize(240, 80);
    let rect = tk.widget_rect(spin).unwrap();
    tk.sim_click(rect.x + 8.0, rect.y + rect.height / 2.0);
    tk.sim_text(&"1".repeat(80));
    tk.sim_key(Key::Enter);
    let value = var.get(&tk).unwrap();
    assert!(value.is_finite(), "capped buffer must still parse, got {value}");
}

/// B16: wheel on an inverted slider moves toward `to`.
#[test]
fn inverted_slider_wheel_moves_toward_to() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(5.0);
    let slider = Slider::new(&mut tk, root, 10.0, 0.0, var).unwrap();
    tk.slider_set_step(slider, 1.0);
    tk.pack(slider, PackOpts::default());
    tk.sim_resize(240, 80);
    let rect = tk.widget_rect(slider).unwrap();
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_wheel(1.0);
    let value = var.get(&tk).unwrap();
    assert!(value < 5.0, "inverted wheel(+1) should move toward to=0, got {value}");
}

/// B15: Spinbox wheel runs the command; Combobox wheel fires on_select.
#[test]
fn spinbox_and_combobox_wheel_fire_callbacks() {
    let mut tk = Tk::new();
    let root = tk.root();

    let spin_var = tk.int_var(5);
    let spin = Spinbox::new(&mut tk, root, 0, 10, spin_var).unwrap();
    tk.pack(spin, PackOpts::default());
    let spin_cmds = Rc::new(RefCell::new(0usize));
    let sc = spin_cmds.clone();
    tk.set_command(spin, move |_t| {
        *sc.borrow_mut() += 1;
    });

    let combo_var = tk.int_var(0);
    let combo = Combobox::new(
        &mut tk,
        root,
        vec!["a".into(), "b".into(), "c".into()],
        combo_var,
    )
    .unwrap();
    tk.pack(combo, PackOpts::default());
    let combo_log = Rc::new(RefCell::new(Vec::<usize>::new()));
    let cl = combo_log.clone();
    tk.combobox_on_select(combo, move |_t, idx, _text| {
        cl.borrow_mut().push(idx);
    });

    tk.sim_resize(240, 200);
    let srect = tk.widget_rect(spin).unwrap();
    tk.sim_mouse_move(srect.x + 8.0, srect.y + srect.height / 2.0);
    tk.sim_wheel(1.0);
    assert_eq!(spin_var.get(&tk), Some(6));
    assert_eq!(*spin_cmds.borrow(), 1, "spinbox wheel must run_command");

    let crect = tk.widget_rect(combo).unwrap();
    tk.sim_mouse_move(crect.x + crect.width / 2.0, crect.y + crect.height / 2.0);
    tk.sim_wheel(-1.0);
    assert_eq!(&*combo_log.borrow(), &[1], "combobox wheel must fire on_select, got {:?}", combo_log.borrow());
    assert_eq!(combo_var.get(&tk), Some(1));
}

/// B20: a missing golden must fail unless RTK_BLESS=1, and must not write the image.
#[test]
fn missing_golden_fails_without_bless() {
    let dir = unique_tmp("golden");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut tk = Tk::new();
    // Ensure we are not blessing in this process.
    std::env::remove_var("RTK_BLESS");
    let result = catch_unwind(AssertUnwindSafe(|| {
        assert_golden(&mut tk, &dir, "does_not_exist_wp5", 32, 24, GoldenOpts::default());
    }));
    assert!(result.is_err(), "missing golden must panic without RTK_BLESS=1");
    assert!(
        !dir.join("does_not_exist_wp5.png").exists(),
        "missing golden must not be created silently"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
