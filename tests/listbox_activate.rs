//! Tests for `Tk::listbox_bind_activate`: activation fires on Enter and on a
//! double-click, but not on plain selection changes (single clicks, arrow keys).

use std::cell::RefCell;
use std::rc::Rc;

use rtk::{Listbox, PackOpts, Tk};

/// Builds a listbox with three rows and an activation counter, returning the
/// toolkit, the listbox id and the shared activation log.
fn setup() -> (Tk, rtk::WidgetId, Rc<RefCell<Vec<usize>>>) {
    let mut tk = Tk::new();
    let root = tk.root();
    let lb = Listbox::new(&mut tk, root, 5).unwrap();
    tk.pack(lb, PackOpts::default());
    for (i, name) in ["alpha", "beta", "gamma"].iter().enumerate() {
        tk.listbox_insert(lb, i, *name);
    }
    tk.sim_resize(320, 240);
    let log = Rc::new(RefCell::new(Vec::<usize>::new()));
    let sink = Rc::clone(&log);
    tk.listbox_bind_activate(lb, move |_t, idx| {
        sink.borrow_mut().push(idx);
    });
    (tk, lb, log)
}

/// Arrow-key selection changes must not fire the activate callback.
#[test]
fn arrow_keys_do_not_activate() {
    let (mut tk, lb, log) = setup();
    let rect = tk.widget_rect(lb).unwrap();
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.listbox_select(lb, Some(0));
    tk.sim_key(rtk::Key::Down);
    tk.sim_key(rtk::Key::Down);
    tk.sim_key(rtk::Key::Up);
    assert_eq!(tk.listbox_curselection(lb), Some(1));
    assert!(log.borrow().is_empty(), "arrow keys must not activate, got {:?}", log.borrow());
}

/// Enter on a focused listbox with a selection fires activate with that index.
#[test]
fn enter_activates_selected_row() {
    let (mut tk, lb, log) = setup();
    let rect = tk.widget_rect(lb).unwrap();
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.listbox_select(lb, Some(2));
    tk.sim_key(rtk::Key::Enter);
    assert_eq!(&*log.borrow(), &[2], "Enter must activate the selected row");
}

/// A double-click on a row fires activate for that row.
#[test]
fn double_click_activates_row() {
    let (mut tk, lb, log) = setup();
    let rect = tk.widget_rect(lb).unwrap();
    let line_h = 16.0_f32.max(rect.height / 5.0);
    let x = rect.x + rect.width / 2.0;
    let y = rect.y + line_h * 1.5;
    tk.sim_click(x, y);
    assert!(log.borrow().is_empty(), "single click must not activate");
    tk.sim_click(x, y);
    assert_eq!(&*log.borrow(), &[1], "double click must activate the clicked row");
}
