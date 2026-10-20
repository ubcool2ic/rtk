//! Integration tests for the milestone 0.11 Table / DataGrid widget.

use rtk::{Frame, Key, PackOpts, Table, Tk};

/// Builds a table with three columns and `n` numeric rows.
fn build_table(tk: &mut Tk, n: usize) -> rtk::WidgetId {
    let root = tk.root();
    let table = Table::new(tk, root, &["Name", "Score", "City"], 10).expect("create table");
    tk.pack(table, PackOpts::default());
    let rows: Vec<Vec<String>> = (0..n)
        .map(|i| vec![format!("row{i}"), format!("{}", (i * 7) % 100), format!("city{}", i % 5)])
        .collect();
    tk.table_set_rows(table, rows);
    table
}

/// Verifies row/column counts and cell accessors on a freshly built table.
#[test]
fn test_table_basic_rows_and_cells() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 5);
    assert_eq!(tk.table_row_count(table), 5);
    assert_eq!(tk.table_column_count(table), 3);
    assert_eq!(tk.table_cell(table, 0, 0).as_deref(), Some("row0"));
    assert_eq!(tk.table_cell(table, 4, 1).as_deref(), Some("28"));
    assert!(tk.table_cell(table, 99, 0).is_none());
}

/// Verifies numeric and string sorting, descending order and clearing the sort.
#[test]
fn test_table_sort_numeric_and_string() {
    let mut tk = Tk::new();
    let root = tk.root();
    let table = Table::new(&mut tk, root, &["Name", "Score"], 5).expect("table");
    tk.pack(table, PackOpts::default());
    tk.table_set_rows(
        table,
        vec![
            vec!["b".into(), "10".into()],
            vec!["a".into(), "2".into()],
            vec!["c".into(), "100".into()],
        ],
    );

    // Numeric ascending on Score.
    assert!(tk.table_sort_by(table, 1, true));
    assert_eq!(tk.table_sort_state(table), Some((1, true)));
    assert_eq!(tk.table_cell(table, tk.table_display_to_model(table, 0).unwrap(), 1).as_deref(), Some("2"));
    assert_eq!(tk.table_cell(table, tk.table_display_to_model(table, 2).unwrap(), 1).as_deref(), Some("100"));

    // Numeric descending.
    tk.table_sort_by(table, 1, false);
    assert_eq!(tk.table_cell(table, tk.table_display_to_model(table, 0).unwrap(), 1).as_deref(), Some("100"));

    // String ascending on Name.
    tk.table_sort_by(table, 0, true);
    assert_eq!(tk.table_cell(table, tk.table_display_to_model(table, 0).unwrap(), 0).as_deref(), Some("a"));

    // Clear sort restores model order.
    tk.table_clear_sort(table);
    assert_eq!(tk.table_sort_state(table), None);
    assert_eq!(tk.table_display_to_model(table, 0), Some(0));
}

/// Verifies clicking a column header sorts ascending, then toggles descending.
#[test]
fn test_table_header_click_sorts() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 5);
    let rect = tk.widget_rect(table).expect("rect");
    // Click the first column header (well inside, away from the resize edge).
    tk.sim_click(rect.x + 20.0, rect.y + 4.0);
    assert_eq!(tk.table_sort_state(table), Some((0, true)));
    // Second click toggles to descending.
    tk.sim_click(rect.x + 20.0, rect.y + 4.0);
    assert_eq!(tk.table_sort_state(table), Some((0, false)));
}

/// Verifies row selection via mouse click and keyboard navigation keys.
#[test]
fn test_table_selection_click_and_keys() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 10);
    let rect = tk.widget_rect(table).expect("rect");
    let row_h = tk.table_row_height(table);
    let header = row_h;
    // Click the third body row.
    tk.sim_click(rect.x + 20.0, rect.y + header + row_h * 2.0 + 2.0);
    assert_eq!(tk.table_selection(table), Some(2));

    // Keys move selection.
    tk.sim_key(Key::Down);
    assert_eq!(tk.table_selection(table), Some(3));
    tk.sim_key(Key::Up);
    assert_eq!(tk.table_selection(table), Some(2));
    tk.sim_key(Key::End);
    assert_eq!(tk.table_selection(table), Some(9));
    tk.sim_key(Key::Home);
    assert_eq!(tk.table_selection(table), Some(0));
    tk.sim_key(Key::PageDown);
    assert!(tk.table_selection(table).unwrap() > 0);
}

/// Verifies the selection-change callback fires with the new selection.
#[test]
fn test_table_on_select_callback() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 5);
    let var = tk.int_var(-1);
    tk.table_on_select(table, move |tk, sel| {
        var.set(tk, sel.map(|s| s as i64).unwrap_or(-1));
    });
    tk.table_select(table, Some(3));
    assert_eq!(var.get(&tk), Some(3));
    tk.table_select(table, None);
    assert_eq!(var.get(&tk), Some(-1));
}

/// Verifies wheel scrolling moves the view and clamps at both ends.
#[test]
fn test_table_wheel_scroll_bounds() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 100);
    let rect = tk.widget_rect(table).expect("rect");
    tk.sim_mouse_move(rect.x + 20.0, rect.y + 40.0);
    assert_eq!(tk.table_first_visible(table), 0);
    // Scroll down (negative wheel = down).
    tk.sim_wheel(-1.0);
    assert!(tk.table_first_visible(table) > 0);
    // Scroll far up clamps at 0.
    for _ in 0..50 {
        tk.sim_wheel(1.0);
    }
    assert_eq!(tk.table_first_visible(table), 0);
    // Scroll far down clamps at max.
    for _ in 0..500 {
        tk.sim_wheel(-1.0);
    }
    let top = tk.table_first_visible(table);
    assert!(top > 0 && top < 100, "unexpected max scroll position {top}");
    // Already at the bottom: further wheel-down must not move the view.
    tk.sim_wheel(-1.0);
    assert_eq!(tk.table_first_visible(table), top);
}

/// Verifies dragging a column edge resizes it and enforces the minimum width.
#[test]
fn test_table_column_resize_drag() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 5);
    let rect = tk.widget_rect(table).expect("rect");
    let w0 = tk.table_column_width(table, 0).unwrap();
    // Press within 4 px of the first column's right edge.
    let edge_x = rect.x + w0 - 1.0;
    tk.sim_mouse_move(edge_x, rect.y + 4.0);
    tk.sim_mouse_down();
    tk.sim_mouse_move(edge_x + 40.0, rect.y + 4.0);
    tk.sim_mouse_up();
    let w1 = tk.table_column_width(table, 0).unwrap();
    assert!(w1 > w0, "width should grow: {w0} -> {w1}");
    // Minimum width is enforced.
    tk.table_set_column_width(table, 0, 1.0);
    assert_eq!(tk.table_column_width(table, 0), Some(24.0));
}

/// Verifies inserting a row preserves the active sort order.
#[test]
fn test_table_insert_row_keeps_sort() {
    let mut tk = Tk::new();
    let root = tk.root();
    let table = Table::new(&mut tk, root, &["N"], 5).expect("table");
    tk.pack(table, PackOpts::default());
    tk.table_set_rows(table, vec![vec!["5".into()], vec!["1".into()], vec!["9".into()]]);
    tk.table_sort_by(table, 0, true);
    assert!(tk.table_insert_row(table, 1, vec!["3".into()]));
    assert_eq!(tk.table_row_count(table), 4);
    // Sorted order: 1, 3, 5, 9.
    let vals: Vec<String> = (0..4)
        .map(|d| {
            let m = tk.table_display_to_model(table, d).unwrap();
            tk.table_cell(table, m, 0).unwrap()
        })
        .collect();
    assert_eq!(vals, vec!["1", "3", "5", "9"]);
}

/// Verifies clearing removes all rows and resets the selection.
#[test]
fn test_table_clear() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 5);
    tk.table_select(table, Some(2));
    assert_eq!(tk.table_selection(table), Some(2));
    assert!(tk.table_clear(table));
    assert_eq!(tk.table_row_count(table), 0);
    assert_eq!(tk.table_selection(table), None);
}

/// Verifies a sorted, selected table renders to a pixmap without error.
#[test]
fn test_table_renders() {
    let mut tk = Tk::new();
    let table = build_table(&mut tk, 20);
    tk.table_sort_by(table, 1, true);
    tk.table_select(table, Some(3));
    tk.render_to_pixmap(400, 300).expect("render sorted, selected table");
}

/// Verifies 10,000-row sort and navigation complete within a time bound.
#[test]
fn test_table_10k_rows_performance() {
    let mut tk = Tk::new();
    let root = tk.root();
    let table = Table::new(&mut tk, root, &["A", "B"], 20).expect("table");
    tk.pack(table, PackOpts::default());
    let rows: Vec<Vec<String>> = (0..10_000)
        .map(|i| vec![format!("{}", (i * 37) % 10_000), format!("s{}", i)])
        .collect();
    tk.table_set_rows(table, rows);
    assert_eq!(tk.table_row_count(table), 10_000);

    // Sort and select End within a debug-build time bound.
    let start = std::time::Instant::now();
    tk.table_sort_by(table, 0, true);
    let rect = tk.widget_rect(table).expect("rect");
    tk.sim_click(rect.x + 20.0, rect.y + 40.0);
    tk.sim_key(Key::End);
    assert_eq!(tk.table_selection(table), Some(tk.table_display_to_model(table, 9_999).unwrap()));
    let elapsed = start.elapsed();
    assert!(elapsed.as_secs_f64() < 10.0, "10k table ops took {elapsed:?}");
}

/// Verifies a table nested inside a frame renders and reports its rows.
#[test]
fn test_table_in_frame() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).expect("frame");
    tk.pack(frame, PackOpts::default());
    let table = Table::new(&mut tk, frame, &["X"], 3).expect("table");
    tk.pack(table, PackOpts::default());
    tk.table_set_rows(table, vec![vec!["1".into()], vec!["2".into()]]);
    assert_eq!(tk.table_row_count(table), 2);
    tk.render_to_pixmap(300, 200).expect("render table in frame");
}
