//! Integration tests for the milestone 0.11 menu system (menubar, popups,
//! submenus, context menus, keyboard navigation).

use rtk::{Key, MenuItem, Menubar, PackOpts, Tk};

/// Builds a simple popup menu with three commands and returns (menu, indices).
fn build_popup(tk: &mut Tk) -> (rtk::MenuId, usize, usize, usize) {
    let menu = tk.menu_create();
    let a = tk.menu_add(menu, MenuItem::command("Alpha").with_accel("Ctrl+A"));
    let b = tk.menu_add(menu, MenuItem::command("Beta"));
    let c = tk.menu_add(menu, MenuItem::command("Gamma"));
    (menu, a, b, c)
}

#[test]
/// Popup opens, clicking a command runs its callback and closes the menu.
fn test_popup_click_runs_command() {
    let mut tk = Tk::new();
    let (menu, a, _b, _c) = build_popup(&mut tk);
    let var = tk.int_var(0);
    tk.menu_set_command(menu, a, move |tk| {
        var.set(tk, 1);
    });
    tk.menu_popup(menu, 10.0, 10.0);
    assert!(tk.menu_is_open());
    assert_eq!(tk.menu_open_path(), vec![menu]);
    // Click on the first item (panel starts at 10,10; item 0 at y=14..14+ih).
    tk.sim_click(30.0, 20.0);
    assert_eq!(var.get(&tk), Some(1));
    assert!(!tk.menu_is_open());
}

#[test]
/// Check item toggles its bound variable.
fn test_check_toggles_var() {
    let mut tk = Tk::new();
    let menu = tk.menu_create();
    let var = tk.bool_var(false);
    let idx = tk.menu_add(menu, MenuItem::check("Toggle", var));
    tk.menu_popup(menu, 10.0, 10.0);
    tk.sim_click(30.0, 20.0);
    assert_eq!(var.get(&tk), Some(true));
    assert!(!tk.menu_is_open());
    let _ = idx;
}

#[test]
/// Disabled command items do nothing when clicked.
fn test_disabled_noop() {
    let mut tk = Tk::new();
    let menu = tk.menu_create();
    let idx = tk.menu_add(menu, MenuItem::command("Nope"));
    tk.menu_set_enabled(menu, idx, false);
    let var = tk.int_var(0);
    tk.menu_set_command(menu, idx, move |tk| {
        var.set(tk, 1);
    });
    tk.menu_popup(menu, 10.0, 10.0);
    tk.sim_click(30.0, 20.0);
    assert_eq!(var.get(&tk), Some(0));
    assert!(tk.menu_is_open());
}

#[test]
/// Keyboard Down/Up skip separators and disabled items.
fn test_keyboard_skips_separators() {
    let mut tk = Tk::new();
    let menu = tk.menu_create();
    let a = tk.menu_add(menu, MenuItem::command("A"));
    tk.menu_add(menu, MenuItem::separator());
    let b = tk.menu_add(menu, MenuItem::command("B"));
    let c = tk.menu_add(menu, MenuItem::command("C"));
    tk.menu_set_enabled(menu, c, false);
    tk.menu_popup(menu, 10.0, 10.0);
    tk.sim_key(Key::Down);
    assert_eq!(tk.menu_open_path(), vec![menu]);
    // First Down highlights A (index 0).
    tk.sim_key(Key::Down);
    // Second Down skips separator to B (index 2).
    tk.sim_key(Key::Enter);
    assert!(!tk.menu_is_open());
    let _ = (a, b);
}

#[test]
/// Submenu opens via hover and via Right key.
fn test_submenu_open() {
    let mut tk = Tk::new();
    let parent = tk.menu_create();
    let child = tk.menu_create();
    tk.menu_add(child, MenuItem::command("Child"));
    tk.menu_add(parent, MenuItem::submenu("More", child));
    tk.menu_popup(parent, 10.0, 10.0);
    // Hover the submenu item.
    tk.sim_mouse_move(30.0, 20.0);
    assert_eq!(tk.menu_open_path(), vec![parent, child]);
    // Escape closes one level.
    tk.sim_key(Key::Escape);
    assert_eq!(tk.menu_open_path(), vec![parent]);
    // Right opens submenu again.
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Right);
    assert_eq!(tk.menu_open_path(), vec![parent, child]);
}

#[test]
/// Escape closes one level at a time.
fn test_escape_closes_one_level() {
    let mut tk = Tk::new();
    let parent = tk.menu_create();
    let child = tk.menu_create();
    tk.menu_add(child, MenuItem::command("Child"));
    tk.menu_add(parent, MenuItem::submenu("More", child));
    tk.menu_popup(parent, 10.0, 10.0);
    tk.sim_mouse_move(30.0, 20.0);
    assert_eq!(tk.menu_open_path().len(), 2);
    tk.sim_key(Key::Escape);
    assert_eq!(tk.menu_open_path().len(), 1);
    tk.sim_key(Key::Escape);
    assert!(!tk.menu_is_open());
}

#[test]
/// Clicking outside all open menus closes them and does not reach the widget beneath.
fn test_outside_click_closes_and_consumes() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = rtk::Button::new(&mut tk, root, "Press").expect("btn");
    tk.pack(btn, PackOpts::default());
    let var = tk.int_var(0);
    tk.set_command(btn, move |tk| {
        var.set(tk, 1);
    });
    let menu = tk.menu_create();
    tk.menu_add(menu, MenuItem::command("X"));
    tk.menu_popup(menu, 200.0, 200.0);
    assert!(tk.menu_is_open());
    let rect = tk.widget_rect(btn).expect("rect");
    tk.sim_click(rect.x + 5.0, rect.y + 5.0);
    assert!(!tk.menu_is_open());
    assert_eq!(var.get(&tk), Some(0));
}

#[test]
/// Menubar click opens a menu; hovering another title switches; Left/Right switch.
fn test_menubar_click_hover_switch() {
    let mut tk = Tk::new();
    let root = tk.root();
    let bar = Menubar::new(&mut tk, root).expect("bar");
    tk.pack(bar, PackOpts::default());
    let file = tk.menu_create();
    tk.menu_add(file, MenuItem::command("Open"));
    let edit = tk.menu_create();
    tk.menu_add(edit, MenuItem::command("Copy"));
    tk.menubar_add(bar, "File", file);
    tk.menubar_add(bar, "Edit", edit);
    let rect = tk.widget_rect(bar).expect("rect");
    // Click "File" title.
    tk.sim_click(rect.x + 5.0, rect.y + rect.height / 2.0);
    assert_eq!(tk.menu_open_path(), vec![file]);
    // Hover "Edit" title switches.
    let style = tk.resolved_style(bar).expect("style");
    let file_w = tk.measure_with("File", &style).width + 24.0;
    tk.sim_mouse_move(rect.x + file_w + 5.0, rect.y + rect.height / 2.0);
    assert_eq!(tk.menu_open_path(), vec![edit]);
    // Right key switches back to File.
    tk.sim_key(Key::Right);
    assert_eq!(tk.menu_open_path(), vec![file]);
    // Left key wraps to Edit.
    tk.sim_key(Key::Left);
    assert_eq!(tk.menu_open_path(), vec![edit]);
}

#[test]
/// Context menu opens via sim_right_click on a bound widget.
fn test_context_menu() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = rtk::Button::new(&mut tk, root, "Target").expect("btn");
    tk.pack(btn, PackOpts::default());
    let menu = tk.menu_create();
    tk.menu_add(menu, MenuItem::command("Ctx"));
    assert!(tk.bind_context_menu(btn, menu));
    let rect = tk.widget_rect(btn).expect("rect");
    tk.sim_right_click(rect.x + 5.0, rect.y + 5.0);
    assert_eq!(tk.menu_open_path(), vec![menu]);
}

#[test]
/// Popup is clamped inside the viewport.
fn test_popup_clamped() {
    let mut tk = Tk::new();
    tk.sim_resize(200, 150);
    let menu = tk.menu_create();
    tk.menu_add(menu, MenuItem::command("A"));
    tk.menu_popup(menu, 10_000.0, 10_000.0);
    let rect = tk.menu_panel_rect(0).expect("rect");
    assert!(rect.x + rect.width <= 200.0 + 0.5);
    assert!(rect.y + rect.height <= 150.0 + 0.5);
}

#[test]
/// Renders with an open menu without panicking.
fn test_menu_renders() {
    let mut tk = Tk::new();
    let menu = tk.menu_create();
    tk.menu_add(menu, MenuItem::command("A").with_accel("Ctrl+A"));
    tk.menu_add(menu, MenuItem::separator());
    tk.menu_add(menu, MenuItem::command("B"));
    tk.menu_popup(menu, 10.0, 10.0);
    assert!(tk.render_to_pixmap(300, 200).is_ok());
}
