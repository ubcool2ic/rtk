//! Runnable gallery for the milestone 0.11 widgets: menubar with a theme
//! submenu, a notebook with a 10,000-row Table and a 100×10 TreeView, toast
//! buttons, nested dialogs and a context menu bound to the table.
//!
//! Run with `cargo run --example gallery_m011`.

use std::error::Error;

use rtk::{
    Button, Frame, MenuItem, Menubar, Notebook, PackOpts, Side, Table, Theme, Tk, ToastKind,
    TreeView,
};

/// Builds the File menu (Open / Quit) and returns its handle.
fn build_file_menu(tk: &mut Tk) -> rtk::MenuId {
    let file = tk.menu_create();
    let open = tk.menu_add(file, MenuItem::command("Open").with_accel("Ctrl+O"));
    tk.menu_set_command(file, open, |tk| {
        tk.toast_with("Open clicked", ToastKind::Info, 2000);
    });
    tk.menu_add(file, MenuItem::separator());
    let quit = tk.menu_add(file, MenuItem::command("Quit").with_accel("Ctrl+Q"));
    tk.menu_set_command(file, quit, Tk::quit);
    file
}

/// Builds the View menu with a Theme submenu that switches themes.
fn build_view_menu(tk: &mut Tk) -> rtk::MenuId {
    let view = tk.menu_create();
    let themes = tk.menu_create();
    for (label, theme) in [
        ("Light", Theme::Light),
        ("Dark", Theme::Dark),
        ("High Contrast", Theme::HighContrast),
    ] {
        let idx = tk.menu_add(themes, MenuItem::command(label));
        tk.menu_set_command(themes, idx, move |tk| {
            tk.set_theme(theme);
        });
    }
    tk.menu_add(view, MenuItem::submenu("Theme", themes));
    view
}

/// Builds the Table tab with 10,000 generated rows and a context menu.
fn build_table_tab(tk: &mut Tk, parent: rtk::WidgetId) -> rtk::WidgetId {
    let table = Table::new(tk, parent, &["ID", "Name", "Value"], 20).expect("table");
    tk.pack(
        table,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            padx: 6.0,
            pady: 6.0,
            ..PackOpts::default()
        },
    );
    let rows: Vec<Vec<String>> = (0..10_000)
        .map(|i| vec![format!("{i}"), format!("Item {i}"), format!("{}", (i * 37) % 1000)])
        .collect();
    tk.table_set_rows(table, rows);

    let ctx = tk.menu_create();
    let copy = tk.menu_add(ctx, MenuItem::command("Copy"));
    tk.menu_set_command(ctx, copy, |tk| {
        tk.toast_with("Copied", ToastKind::Success, 1500);
    });
    let select_all = tk.menu_add(ctx, MenuItem::command("Select all"));
    tk.menu_set_command(ctx, select_all, |tk| {
        tk.toast_with("Selected all", ToastKind::Info, 1500);
    });
    tk.bind_context_menu(table, ctx);
    table
}

/// Builds the Tree tab with 100 roots each holding 10 children.
fn build_tree_tab(tk: &mut Tk, parent: rtk::WidgetId) -> rtk::WidgetId {
    let tree = TreeView::new(tk, parent, 20).expect("tree");
    tk.pack(
        tree,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            padx: 6.0,
            pady: 6.0,
            ..PackOpts::default()
        },
    );
    for i in 0..100 {
        let root = tk.tree_insert(tree, None, &format!("Group {i}")).expect("root");
        for j in 0..10 {
            tk.tree_insert(tree, Some(root), &format!("Leaf {i}.{j}"));
        }
    }
    tree
}

/// Builds the toolbar with the toast and nested-dialog buttons.
fn build_toolbar(tk: &mut Tk, parent: rtk::WidgetId) {
    let toast_btn = Button::new(tk, parent, "Show toast").expect("toast button");
    tk.pack(
        toast_btn,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    tk.set_command(toast_btn, |tk| {
        tk.toast_with("Hello from the gallery", ToastKind::Info, 3000);
    });

    let dialog_btn = Button::new(tk, parent, "Nested dialogs").expect("dialog button");
    tk.pack(
        dialog_btn,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    tk.set_command(dialog_btn, |tk| {
        rtk::dialog::ask_yes_no(tk, "Confirm", "Open another dialog?", |tk, yes| {
            if yes {
                rtk::dialog::message_box(
                    tk,
                    "Nested",
                    "This dialog was opened from another dialog.",
                    rtk::MessageKind::Info,
                    |_| {},
                );
            }
        });
    });
}

/// Entry point: builds the gallery window and runs the event loop.
fn main() -> Result<(), Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.title("rtk 0.11 gallery");
    tk.geometry(900, 600);
    let root = tk.root();

    let bar = Menubar::new(&mut tk, root)?;
    tk.pack(
        bar,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            ..PackOpts::default()
        },
    );
    let file = build_file_menu(&mut tk);
    let view = build_view_menu(&mut tk);
    tk.menubar_add(bar, "File", file);
    tk.menubar_add(bar, "View", view);

    let toolbar = Frame::new(&mut tk, root)?;
    tk.pack(
        toolbar,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            ..PackOpts::default()
        },
    );
    build_toolbar(&mut tk, toolbar);

    let notebook = Notebook::new(&mut tk, root)?;
    tk.pack(
        notebook,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    let table_page = Frame::new(&mut tk, notebook)?;
    let tree_page = Frame::new(&mut tk, notebook)?;
    tk.notebook_add(notebook, table_page, "Table");
    tk.notebook_add(notebook, tree_page, "Tree");
    build_table_tab(&mut tk, table_page);
    build_tree_tab(&mut tk, tree_page);

    tk.mainloop()
}
