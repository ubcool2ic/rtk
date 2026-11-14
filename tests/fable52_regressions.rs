//! Regression tests for the 1.0.2 behaviour fixes that are observable through
//! the headless API.

use rtk::{FrameDamage, PackOpts, ProgressBar, Tk, TreeView, WidgetId};

/// Builds a tree A -> B -> C plus a second root D, returning the keys.
fn build_tree(tk: &mut Tk) -> (WidgetId, usize, usize, usize, usize) {
    let root = tk.root();
    let tree = TreeView::new(tk, root, 10).expect("create tree");
    tk.pack(tree, PackOpts::default());
    let a = tk.tree_insert(tree, None, "A").expect("A");
    let b = tk.tree_insert(tree, Some(a), "B").expect("B");
    let c = tk.tree_insert(tree, Some(b), "C").expect("C");
    let d = tk.tree_insert(tree, None, "D").expect("D");
    (tree, a, b, c, d)
}

/// Registers an on_select callback recording the last selection (-1 for None,
/// -2 for "never fired") and the number of calls.
fn track_selection(tk: &mut Tk, tree: WidgetId) -> (rtk::Var<i64>, rtk::Var<i64>) {
    let last = tk.int_var(-2);
    let count = tk.int_var(0);
    assert!(tk.tree_on_select(tree, move |tk, sel| {
        last.set(tk, sel.map(|s| s as i64).unwrap_or(-1));
        let n = count.get(&*tk).unwrap_or(0);
        count.set(tk, n + 1);
    }));
    (last, count)
}

/// Removing the selected node fires on_select with `None`.
#[test]
fn tree_remove_selected_fires_on_select_none() {
    let mut tk = Tk::new();
    let (tree, _a, _b, _c, d) = build_tree(&mut tk);
    let (last, count) = track_selection(&mut tk, tree);
    assert!(tk.tree_select(tree, Some(d)));
    assert_eq!(last.get(&tk), Some(d as i64));
    assert_eq!(count.get(&tk), Some(1));
    assert!(tk.tree_remove(tree, d));
    assert_eq!(tk.tree_selection(tree), None);
    assert_eq!(last.get(&tk), Some(-1));
    assert_eq!(count.get(&tk), Some(2));
}

/// Removing an ancestor of the selected node fires on_select with `None`.
#[test]
fn tree_remove_ancestor_of_selected_fires_on_select_none() {
    let mut tk = Tk::new();
    let (tree, _a, b, c, _d) = build_tree(&mut tk);
    let (last, count) = track_selection(&mut tk, tree);
    assert!(tk.tree_select(tree, Some(c)));
    assert_eq!(last.get(&tk), Some(c as i64));
    assert!(tk.tree_remove(tree, b));
    assert_eq!(tk.tree_selection(tree), None);
    assert_eq!(last.get(&tk), Some(-1));
    assert_eq!(count.get(&tk), Some(2));
}

/// Removing a node unrelated to the selection does not fire the callback.
#[test]
fn tree_remove_unrelated_does_not_fire_on_select() {
    let mut tk = Tk::new();
    let (tree, _a, b, _c, d) = build_tree(&mut tk);
    let (last, count) = track_selection(&mut tk, tree);
    assert!(tk.tree_select(tree, Some(d)));
    assert!(tk.tree_remove(tree, b));
    assert_eq!(tk.tree_selection(tree), Some(d));
    assert_eq!(last.get(&tk), Some(d as i64));
    assert_eq!(count.get(&tk), Some(1));
}

/// Clearing a tree with a selection fires on_select with `None`; clearing
/// again (no selection) does not fire.
#[test]
fn tree_clear_fires_on_select_none_once() {
    let mut tk = Tk::new();
    let (tree, a, _b, _c, _d) = build_tree(&mut tk);
    let (last, count) = track_selection(&mut tk, tree);
    assert!(tk.tree_select(tree, Some(a)));
    assert_eq!(count.get(&tk), Some(1));
    assert!(tk.tree_clear(tree));
    assert_eq!(tk.tree_selection(tree), None);
    assert_eq!(last.get(&tk), Some(-1));
    assert_eq!(count.get(&tk), Some(2));
    assert!(tk.tree_clear(tree));
    assert_eq!(count.get(&tk), Some(2));
}

/// Clearing a tree that never had a selection does not fire the callback.
#[test]
fn tree_clear_without_selection_does_not_fire() {
    let mut tk = Tk::new();
    let (tree, _a, _b, _c, _d) = build_tree(&mut tk);
    let (last, count) = track_selection(&mut tk, tree);
    assert!(tk.tree_clear(tree));
    assert_eq!(last.get(&tk), Some(-2));
    assert_eq!(count.get(&tk), Some(0));
}

/// Calling `progress_set_smooth(true)` on an already-smooth bar must not snap
/// an in-flight easing animation to its target.
#[test]
fn progress_set_smooth_twice_does_not_snap() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.f64_var(0.0);
    let int_var = tk.int_var(0);
    let bar = ProgressBar::new(&mut tk, root, 200.0, int_var).expect("bar");
    assert!(tk.progress_bind(bar, var));
    tk.pack(bar, PackOpts::default());
    tk.sim_resize(320, 200);
    tk.sim_advance_time(16);
    assert!(tk.progress_set_smooth(bar, true));
    var.set(&mut tk, 1.0);
    assert!(tk.is_animating());
    // Re-enabling must keep the animation running.
    assert!(tk.progress_set_smooth(bar, true));
    assert!(tk.is_animating(), "second progress_set_smooth(true) snapped the animation");
    tk.sim_advance_time(300);
    assert!(!tk.is_animating());
    assert_eq!(tk.progress_get(bar), Some(1.0));
}

/// A wheel event that cannot scroll the tree (already at the top) does not
/// request a redraw; one that scrolls does.
#[test]
fn tree_wheel_at_top_requests_no_redraw() {
    let mut tk = Tk::new();
    let root = tk.root();
    let tree = TreeView::new(&mut tk, root, 5).expect("tree");
    tk.pack(tree, PackOpts::default());
    for i in 0..50 {
        tk.tree_insert(tree, None, &format!("n{i}"));
    }
    tk.sim_resize(320, 200);
    let rect = tk.widget_rect(tree).expect("rect");
    tk.sim_mouse_move(rect.x + 40.0, rect.y + 20.0);
    tk.sim_advance_time(1000);
    for _ in 0..3 {
        tk.render_frame(320, 200).expect("frame");
    }
    assert!(!tk.needs_redraw(), "scene should be settled before the wheel");
    assert_eq!(tk.tree_first_visible(tree), 0);
    tk.sim_wheel(1.0);
    assert_eq!(tk.tree_first_visible(tree), 0);
    assert!(!tk.needs_redraw(), "no-op wheel must not request a redraw");
    assert_eq!(tk.render_frame(320, 200).expect("frame"), FrameDamage::None);
    tk.sim_wheel(-1.0);
    assert!(tk.tree_first_visible(tree) > 0);
    assert!(tk.needs_redraw());
}

/// Smoke test: an indeterminate bar with label and percent text renders at
/// several animation phases (exercises the extra left-of-block text pass).
#[test]
fn indeterminate_progress_with_text_renders() {
    let mut tk = Tk::new();
    let root = tk.root();
    let var = tk.int_var(40);
    let bar = ProgressBar::new(&mut tk, root, 200.0, var).expect("bar");
    assert!(tk.progress_set_show_percent(bar, true));
    assert!(tk.progress_set_label(bar, Some("Loading".into())));
    assert!(tk.progress_set_indeterminate(bar, true));
    tk.pack(bar, PackOpts::default());
    for _ in 0..12 {
        tk.sim_advance_time(120);
        assert!(tk.render_to_pixmap(320, 200).is_ok());
    }
}
