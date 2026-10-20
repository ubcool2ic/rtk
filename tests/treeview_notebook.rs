//! Regression test: a TreeView populated while its Notebook page is not
//! selected must render its rows once that page becomes selected.

use rtk::{Fill, Frame, Notebook, PackOpts, Side, Tk, TreeView};

/// Builds a notebook with two pages, puts a populated TreeView on the second
/// (initially unselected) page, selects it, renders and asserts the tree shows
/// rows and paints non-background pixels inside its area.
#[test]
fn treeview_on_hidden_notebook_page_renders_rows() {
    let mut tk = Tk::new();
    tk.geometry(400, 300);
    let root = tk.root();

    let notebook = Notebook::new(&mut tk, root).expect("notebook");
    tk.pack(
        notebook,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );

    let first = Frame::new(&mut tk, notebook).expect("first page");
    let second = Frame::new(&mut tk, notebook).expect("second page");
    tk.notebook_add(notebook, first, "First");
    tk.notebook_add(notebook, second, "Second");

    // The tree lives on the non-selected page.
    let tree = TreeView::new(&mut tk, second, 10).expect("tree");
    tk.pack(
        tree,
        PackOpts {
            side: Side::Top,
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    for i in 0..3 {
        let group = tk.tree_insert(tree, None, &format!("Group {i}")).expect("root");
        for j in 0..3 {
            tk.tree_insert(tree, Some(group), &format!("Child {i}.{j}")).expect("child");
        }
    }
    tk.tree_expand_all(tree);

    // Select the page holding the tree and lay out.
    tk.notebook_select(notebook, 1);
    tk.sim_resize(400, 300);

    // The tree must report visible rows (3 roots + 9 children).
    let count = tk.tree_visible_count(tree);
    assert_eq!(count, 12, "expected 12 visible rows, got {count}");

    // Render and check that the tree area contains non-background pixels.
    let pixmap = tk.render_to_pixmap(400, 300).expect("render");
    let rect = tk.widget_rect(tree).expect("tree rect");
    assert!(rect.width > 0.0 && rect.height > 0.0, "tree has no area");

    let bg = tk.tokens().palette.bg;
    let mut non_bg = 0usize;
    let x0 = rect.x.max(0.0) as u32;
    let y0 = rect.y.max(0.0) as u32;
    let x1 = ((rect.x + rect.width) as u32).min(pixmap.width());
    let y1 = ((rect.y + rect.height) as u32).min(pixmap.height());
    let data = pixmap.data();
    for y in y0..y1 {
        for x in x0..x1 {
            let idx = ((y * pixmap.width() + x) * 4) as usize;
            let (r, g, b) = (data[idx], data[idx + 1], data[idx + 2]);
            if (r, g, b) != (bg.r, bg.g, bg.b) {
                non_bg += 1;
            }
        }
    }
    assert!(non_bg > 0, "tree area rendered as a blank box");
}
