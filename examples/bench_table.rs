//! Headless performance check for the milestone 0.11 Table and TreeView
//! widgets. No window is created; everything runs through `render_to_pixmap`.
//!
//! Run with `cargo run --release --example bench_table`.

use std::error::Error;
use std::time::Instant;

use rtk::{PackOpts, Table, Tk, TreeView};

/// Viewport width used for every measurement.
const WIDTH: u32 = 1280;
/// Viewport height used for every measurement.
const HEIGHT: u32 = 720;
/// Number of frames rendered per timing loop.
const FRAMES: u32 = 60;
/// Per-frame budget in milliseconds (60 fps).
const BUDGET_MS: f64 = 16.7;

/// Builds a table with 10,000 rows and 4 columns, returning the table id and
/// the wall time spent building and setting the rows.
fn build_table(tk: &mut Tk) -> Result<(rtk::WidgetId, f64), Box<dyn Error>> {
    let root = tk.root();
    let table = Table::new(tk, root, &["ID", "Name", "Value", "Extra"], 20)?;
    tk.pack(
        table,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    let start = Instant::now();
    let rows: Vec<Vec<String>> = (0..10_000)
        .map(|i| {
            vec![
                format!("{i}"),
                format!("Item {i}"),
                format!("{}", (i * 37) % 1000),
                format!("x{}", i % 7),
            ]
        })
        .collect();
    tk.table_set_rows(table, rows);
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok((table, elapsed))
}

/// Builds a tree with 100 roots each holding 100 children (10,000 visible
/// nodes once expanded), returning the tree id and the build time in ms.
fn build_tree(tk: &mut Tk) -> Result<(rtk::WidgetId, f64), Box<dyn Error>> {
    let root = tk.root();
    let tree = TreeView::new(tk, root, 20)?;
    tk.pack(
        tree,
        PackOpts {
            fill: rtk::Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );
    let start = Instant::now();
    for i in 0..100 {
        let group = tk.tree_insert(tree, None, &format!("Group {i}")).expect("root");
        for j in 0..100 {
            tk.tree_insert(tree, Some(group), &format!("Leaf {i}.{j}"));
        }
    }
    tk.tree_expand_all(tree);
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok((tree, elapsed))
}

/// Renders `FRAMES` frames while scrolling `rows_per_frame` rows per frame,
/// returning `(mean_ms, max_ms)`.
fn time_scrolling(
    tk: &mut Tk,
    table: rtk::WidgetId,
    rows_per_frame: f32,
) -> (f64, f64) {
    let rect = tk.widget_rect(table).expect("table rect");
    tk.sim_mouse_move(rect.x + 20.0, rect.y + 40.0);
    let mut total = 0.0_f64;
    let mut max = 0.0_f64;
    for _ in 0..FRAMES {
        tk.sim_wheel(-rows_per_frame);
        let start = Instant::now();
        let _ = tk.render_to_pixmap(WIDTH, HEIGHT);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        if ms > max {
            max = ms;
        }
    }
    (total / f64::from(FRAMES), max)
}

/// Renders `FRAMES` frames of a tree while scrolling, returning `(mean_ms, max_ms)`.
fn time_tree_scrolling(tk: &mut Tk, tree: rtk::WidgetId) -> (f64, f64) {
    let rect = tk.widget_rect(tree).expect("tree rect");
    tk.sim_mouse_move(rect.x + 20.0, rect.y + 40.0);
    let mut total = 0.0_f64;
    let mut max = 0.0_f64;
    for _ in 0..FRAMES {
        tk.sim_wheel(-30.0);
        let start = Instant::now();
        let _ = tk.render_to_pixmap(WIDTH, HEIGHT);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        total += ms;
        if ms > max {
            max = ms;
        }
    }
    (total / f64::from(FRAMES), max)
}

/// Prints a PASS/FAIL line for a mean frame time against the 60 fps budget.
fn report(label: &str, mean: f64, max: f64) {
    let verdict = if mean <= BUDGET_MS { "PASS" } else { "FAIL" };
    println!(
        "{label}: mean {mean:.2} ms, max {max:.2} ms (budget {BUDGET_MS:.1} ms) {verdict}"
    );
}

/// Entry point: runs every measurement and prints one line per metric.
fn main() -> Result<(), Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.sim_resize(WIDTH, HEIGHT);
    let (table, build_ms) = build_table(&mut tk)?;
    println!("table build+set_rows: {build_ms:.2} ms");

    let start = Instant::now();
    let _ = tk.render_to_pixmap(WIDTH, HEIGHT);
    println!(
        "table first layout+render: {:.2} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );

    let (mean, max) = time_scrolling(&mut tk, table, 30.0);
    report("table scroll 30 rows/frame", mean, max);

    let start = Instant::now();
    tk.table_sort_by(table, 2, true);
    println!(
        "table sort numeric: {:.2} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );

    let start = Instant::now();
    tk.table_sort_by(table, 1, true);
    println!(
        "table sort string: {:.2} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );

    let mut tk = Tk::new();
    tk.sim_resize(WIDTH, HEIGHT);
    let (tree, tree_build_ms) = build_tree(&mut tk)?;
    println!("tree build+expand_all: {tree_build_ms:.2} ms");

    let start = Instant::now();
    let _ = tk.render_to_pixmap(WIDTH, HEIGHT);
    println!(
        "tree first layout+render: {:.2} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );

    let (mean, max) = time_tree_scrolling(&mut tk, tree);
    report("tree scroll 30 rows/frame", mean, max);

    Ok(())
}
