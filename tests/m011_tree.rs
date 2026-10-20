//! Integration tests for the milestone 0.11 TreeView widget.

use rtk::{Key, PackOpts, TreeView, Tk};

/// Builds a 3-level tree: root -> child -> grandchild, returning the keys.
fn build_tree(tk: &mut Tk) -> (rtk::WidgetId, usize, usize, usize) {
    let root = tk.root();
    let tree = TreeView::new(tk, root, 10).expect("create tree");
    tk.pack(tree, PackOpts::default());
    let a = tk.tree_insert(tree, None, "A").expect("A");
    let b = tk.tree_insert(tree, Some(a), "B").expect("B");
    let c = tk.tree_insert(tree, Some(b), "C").expect("C");
    (tree, a, b, c)
}

/// Builds a tree with two top-level nodes and returns the keys.
fn build_two_roots(tk: &mut Tk) -> (rtk::WidgetId, usize, usize) {
    let root = tk.root();
    let tree = TreeView::new(tk, root, 10).expect("create tree");
    tk.pack(tree, PackOpts::default());
    let a = tk.tree_insert(tree, None, "A").expect("A");
    let b = tk.tree_insert(tree, None, "B").expect("B");
    (tree, a, b)
}

#[test]
/// Inserting nodes builds the hierarchy and text is readable.
fn test_tree_insert_and_text() {
    let mut tk = Tk::new();
    let (tree, a, b, c) = build_tree(&mut tk);
    assert_eq!(tk.tree_text(tree, a).as_deref(), Some("A"));
    assert_eq!(tk.tree_text(tree, b).as_deref(), Some("B"));
    assert_eq!(tk.tree_text(tree, c).as_deref(), Some("C"));
    assert_eq!(tk.tree_parent(tree, b), Some(a));
    assert_eq!(tk.tree_parent(tree, c), Some(b));
    assert_eq!(tk.tree_parent(tree, a), None);
    assert_eq!(tk.tree_children(tree, None), vec![a]);
    assert_eq!(tk.tree_children(tree, Some(a)), vec![b]);
    assert_eq!(tk.tree_children(tree, Some(b)), vec![c]);
    assert!(tk.tree_insert(tree, Some(9999), "X").is_none());
}

#[test]
/// Open/close changes the visible row count.
fn test_tree_open_close_visible_count() {
    let mut tk = Tk::new();
    let (tree, a, b, _c) = build_tree(&mut tk);
    assert_eq!(tk.tree_visible_count(tree), 1);
    assert!(tk.tree_set_open(tree, a, true));
    assert_eq!(tk.tree_visible_count(tree), 2);
    assert!(tk.tree_set_open(tree, b, true));
    assert_eq!(tk.tree_visible_count(tree), 3);
    assert!(tk.tree_set_open(tree, a, false));
    assert_eq!(tk.tree_visible_count(tree), 1);
    assert!(tk.tree_expand_all(tree));
    assert_eq!(tk.tree_visible_count(tree), 3);
    assert!(tk.tree_collapse_all(tree));
    assert_eq!(tk.tree_visible_count(tree), 1);
}

#[test]
/// Keyboard navigation: Up/Down/Home/End/PageUp/PageDown.
fn test_tree_keyboard_nav() {
    let mut tk = Tk::new();
    let (tree, a, b, c) = build_tree(&mut tk);
    tk.tree_expand_all(tree);
    let rect = tk.widget_rect(tree).expect("rect");
    tk.sim_click(rect.x + 40.0, rect.y + 4.0);
    assert_eq!(tk.tree_selection(tree), Some(a));
    tk.sim_key(Key::Down);
    assert_eq!(tk.tree_selection(tree), Some(b));
    tk.sim_key(Key::Down);
    assert_eq!(tk.tree_selection(tree), Some(c));
    tk.sim_key(Key::Up);
    assert_eq!(tk.tree_selection(tree), Some(b));
    tk.sim_key(Key::Home);
    assert_eq!(tk.tree_selection(tree), Some(a));
    tk.sim_key(Key::End);
    assert_eq!(tk.tree_selection(tree), Some(c));
    tk.sim_key(Key::PageUp);
    assert_eq!(tk.tree_selection(tree), Some(a));
    tk.sim_key(Key::PageDown);
    assert_eq!(tk.tree_selection(tree), Some(c));
}

#[test]
/// Right opens a closed node, then moves to its first child; Left closes or moves to parent.
fn test_tree_left_right_semantics() {
    let mut tk = Tk::new();
    let (tree, a, b, _c) = build_tree(&mut tk);
    let rect = tk.widget_rect(tree).expect("rect");
    tk.sim_click(rect.x + 40.0, rect.y + 4.0);
    assert_eq!(tk.tree_selection(tree), Some(a));
    // Right opens A.
    tk.sim_key(Key::Right);
    assert!(tk.tree_is_open(tree, a));
    assert_eq!(tk.tree_selection(tree), Some(a));
    // Right again moves to first child B.
    tk.sim_key(Key::Right);
    assert_eq!(tk.tree_selection(tree), Some(b));
    // Right opens B.
    tk.sim_key(Key::Right);
    assert!(tk.tree_is_open(tree, b));
    // Left closes B.
    tk.sim_key(Key::Left);
    assert!(!tk.tree_is_open(tree, b));
    // Left moves to parent A.
    tk.sim_key(Key::Left);
    assert_eq!(tk.tree_selection(tree), Some(a));
    // Left closes A.
    tk.sim_key(Key::Left);
    assert!(!tk.tree_is_open(tree, a));
}

#[test]
/// Clicking the triangle toggles open; clicking the row selects.
fn test_tree_click_triangle_vs_row() {
    let mut tk = Tk::new();
    let (tree, a, _b, _c) = build_tree(&mut tk);
    let rect = tk.widget_rect(tree).expect("rect");
    let row_h = tk.tree_row_height(tree);
    // Click the triangle area of the first row.
    tk.sim_click(rect.x + 8.0, rect.y + row_h / 2.0);
    assert!(tk.tree_is_open(tree, a));
    assert_eq!(tk.tree_selection(tree), None);
    // Click the row text area selects.
    tk.sim_click(rect.x + 40.0, rect.y + row_h / 2.0);
    assert_eq!(tk.tree_selection(tree), Some(a));
    // Click the triangle again closes.
    tk.sim_click(rect.x + 8.0, rect.y + row_h / 2.0);
    assert!(!tk.tree_is_open(tree, a));
}

#[test]
/// Removing a node removes its whole subtree.
fn test_tree_remove_subtree() {
    let mut tk = Tk::new();
    let (tree, a, b, c) = build_tree(&mut tk);
    tk.tree_expand_all(tree);
    assert_eq!(tk.tree_visible_count(tree), 3);
    assert!(tk.tree_remove(tree, b));
    assert!(tk.tree_text(tree, b).is_none());
    assert!(tk.tree_text(tree, c).is_none());
    assert_eq!(tk.tree_children(tree, Some(a)), Vec::<usize>::new());
    assert_eq!(tk.tree_visible_count(tree), 1);
    assert!(!tk.tree_remove(tree, b));
    assert!(tk.tree_clear(tree));
    assert_eq!(tk.tree_visible_count(tree), 0);
}

#[test]
/// Selecting a node expands its ancestors and scrolls it into view.
fn test_tree_select_expands_ancestors() {
    let mut tk = Tk::new();
    let (tree, a, b, c) = build_tree(&mut tk);
    assert_eq!(tk.tree_visible_count(tree), 1);
    assert!(tk.tree_select(tree, Some(c)));
    assert!(tk.tree_is_open(tree, a));
    assert!(tk.tree_is_open(tree, b));
    assert_eq!(tk.tree_selection(tree), Some(c));
    assert_eq!(tk.tree_visible_count(tree), 3);
    assert!(!tk.tree_select(tree, Some(9999)));
    assert!(tk.tree_select(tree, None));
    assert_eq!(tk.tree_selection(tree), None);
}

#[test]
/// Selection callback fires on change.
fn test_tree_on_select_callback() {
    let mut tk = Tk::new();
    let (tree, a, b, _c) = build_tree(&mut tk);
    let var = tk.int_var(-1);
    tk.tree_on_select(tree, move |tk, sel| {
        var.set(tk, sel.map(|s| s as i64).unwrap_or(-1));
    });
    tk.tree_select(tree, Some(b));
    assert_eq!(var.get(&tk), Some(b as i64));
    tk.tree_select(tree, Some(a));
    assert_eq!(var.get(&tk), Some(a as i64));
    tk.tree_select(tree, None);
    assert_eq!(var.get(&tk), Some(-1));
}

#[test]
/// Wheel scrolls the tree and clamps at bounds.
fn test_tree_wheel_scroll() {
    let mut tk = Tk::new();
    let root = tk.root();
    let tree = TreeView::new(&mut tk, root, 5).expect("tree");
    tk.pack(tree, PackOpts::default());
    for i in 0..50 {
        tk.tree_insert(tree, None, &format!("n{i}"));
    }
    let rect = tk.widget_rect(tree).expect("rect");
    tk.sim_mouse_move(rect.x + 40.0, rect.y + 20.0);
    assert_eq!(tk.tree_first_visible(tree), 0);
    tk.sim_wheel(-1.0);
    assert!(tk.tree_first_visible(tree) > 0);
    for _ in 0..50 {
        tk.sim_wheel(1.0);
    }
    assert_eq!(tk.tree_first_visible(tree), 0);
}

#[test]
/// Renders without panicking.
fn test_tree_renders() {
    let mut tk = Tk::new();
    let (tree, _a, _b, c) = build_tree(&mut tk);
    tk.tree_expand_all(tree);
    tk.tree_select(tree, Some(c));
    assert!(tk.render_to_pixmap(300, 200).is_ok());
}

#[test]
/// 10k nodes (100 parents x 100 children) expand_all + End within bound.
fn test_tree_10k_nodes_performance() {
    let mut tk = Tk::new();
    let root = tk.root();
    let tree = TreeView::new(&mut tk, root, 20).expect("tree");
    tk.pack(tree, PackOpts::default());
    let start = std::time::Instant::now();
    for p in 0..100 {
        let parent = tk.tree_insert(tree, None, &format!("p{p}")).expect("parent");
        for c in 0..100 {
            tk.tree_insert(tree, Some(parent), &format!("c{c}")).expect("child");
        }
    }
    tk.tree_expand_all(tree);
    assert_eq!(tk.tree_visible_count(tree), 10_100);
    let rect = tk.widget_rect(tree).expect("rect");
    tk.sim_click(rect.x + 40.0, rect.y + 4.0);
    let first = tk.tree_selection(tree);
    assert!(first.is_some());
    tk.sim_key(Key::End);
    let last = tk.tree_selection(tree);
    assert!(last.is_some());
    assert_ne!(first, last, "End must move the selection to the last row");
    let elapsed = start.elapsed();
    assert!(elapsed.as_secs_f64() < 10.0, "10k tree ops took {elapsed:?}");
}

#[test]
/// Two top-level roots are both visible and navigable.
fn test_tree_two_roots() {
    let mut tk = Tk::new();
    let (tree, a, b) = build_two_roots(&mut tk);
    assert_eq!(tk.tree_visible_count(tree), 2);
    assert_eq!(tk.tree_children(tree, None), vec![a, b]);
    assert!(tk.tree_select(tree, Some(b)));
    assert_eq!(tk.tree_selection(tree), Some(b));
    // Keyboard navigation moves between the two top-level nodes.
    let rect = tk.widget_rect(tree).expect("rect");
    let row_h = tk.tree_row_height(tree);
    tk.sim_click(rect.x + 40.0, rect.y + row_h / 2.0);
    assert_eq!(tk.tree_selection(tree), Some(a));
    tk.sim_key(Key::Down);
    assert_eq!(tk.tree_selection(tree), Some(b));
    tk.sim_key(Key::Up);
    assert_eq!(tk.tree_selection(tree), Some(a));
}
