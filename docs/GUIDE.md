# rtk 1.0 User Guide

`rtk` is a small, tkinter-style GUI crate for Rust. Widgets are cheap `Copy` handles into
an arena owned by `Tk`, callbacks receive `&mut Tk`, and everything is drawn on the CPU with
`tiny-skia` + `cosmic-text` and presented through `softbuffer` on a `winit` window. Redraws
are dirty-rectangle based, and a bundled Noto Sans font means text renders identically on
every platform.

This guide walks through installation, the core API, every widget family, theming, input,
timers, accessibility and headless testing. Every snippet uses only the public API.

---

## 1. Installation

`rtk` is **not published on crates.io**. Add it as a git dependency:

```toml
[dependencies]
rtk = { git = "https://github.com/ubcool2ic/rtk", tag = "v1.0.0" }
```

The minimum supported Rust version is **1.85** (edition 2021).

### Features

| Feature | Default | Notes |
|---|---|---|
| `clipboard` | yes | System clipboard via `arboard`. |
| `svg` | yes | SVG rasterisation via `resvg` (`image_set_svg`, `load_svg`). |
| `tray` | no | System tray icon via `tray-icon`. Requires **Rust 1.90**. |
| `accesskit` | no | Screen-reader bridge via `accesskit` + `accesskit_winit`. Requires **Rust 1.87**. |

Enable optional features explicitly:

```toml
[dependencies]
rtk = { git = "https://github.com/ubcool2ic/rtk", features = ["tray", "accesskit"] }
```

To turn off the defaults (for example to drop the clipboard dependency):

```toml
rtk = { git = "https://github.com/ubcool2ic/rtk", default-features = false }
```

The headless `Tk::accessibility_tree()` snapshot works without the `accesskit` feature, so
accessibility behaviour is testable in CI.

---

## 2. Hello world

```rust,no_run
# use rtk::{Button, Label, Opt, PackOpts, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut tk = Tk::new();
tk.title("Hello");
tk.geometry(320, 160);

let root = tk.root();
let label = Label::new(&mut tk, root, "Hello, rtk!")?;
let button = Button::new(&mut tk, root, "Click me")?;

tk.pack(label, PackOpts::default());
tk.pack(button, PackOpts::default());

tk.set_command(button, |tk| {
    tk.toast("Clicked!");
});

tk.mainloop()
# }
```

Key points:

- `Tk::new()` creates the toolkit; `tk.root()` is the root `Frame`.
- Widget constructors return `Result<WidgetId, RtkError>` — add `?`.
- `tk.pack(widget, opts)` places a widget; `PackOpts::default()` stacks top-to-bottom.
- `tk.set_command(widget, |tk| { .. })` installs a click callback.
- `tk.mainloop()` consumes `Tk` and must run on the main thread.

---

## 3. Geometry managers

`rtk` has the three tkinter managers: **pack**, **grid** and **place**. A parent may mix
managers across different children, but siblings must agree (mixing pack and grid among
siblings is rejected, not panicked).

### pack

```rust
# use rtk::{Button, Fill, PackOpts, Side, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let button = Button::new(&mut tk, root, "Go")?;
tk.pack(button, PackOpts {
    side: Side::Left,
    fill: Fill::X,
    expand: true,
    padx: 6.0,
    pady: 4.0,
    ..PackOpts::default()
});
# Ok(())
# }
```

`Side` is `Top`, `Bottom`, `Left`, `Right`; `Fill` is `None`, `X`, `Y`, `Both`.

### grid

```rust
# use rtk::{Entry, GridOpts, Sticky, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let entry = Entry::new(&mut tk, root)?;
tk.grid(entry, GridOpts {
    row: 0,
    column: 1,
    sticky: Sticky::EW,
    padx: 4.0,
    ..GridOpts::default()
});
tk.grid_columnconfigure(root, 1, 1); // weight
tk.grid_rowconfigure(root, 0, 1);
# Ok(())
# }
```

`Sticky` combines `N`, `S`, `E`, `W` (e.g. `Sticky::NSEW`).

### place

```rust
# use rtk::{Anchor, Label, PlaceOpts, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let widget = Label::new(&mut tk, root, "placed")?;
tk.place(widget, PlaceOpts {
    relx: 0.5,
    rely: 0.5,
    anchor: Anchor::Center,
    ..PlaceOpts::default()
});
# Ok(())
# }
```

`PlaceOpts` supports absolute `x`/`y`, relative `relx`/`rely`, `width`/`height`, and an
`Anchor`. Remove a widget from its manager with `tk.pack_forget(id)` or
`tk.grid_forget(id)`.

---

## 4. Variables and callbacks

Variables are typed handles (`Var<T>`) that widgets bind to. They are owned by the toolkit
and invalidated when destroyed.

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
let count = tk.int_var(0);
let name = tk.string_var("world");
let enabled = tk.bool_var(true);
let ratio = tk.f64_var(0.5);

count.set(&mut tk, 3);
if let Some(v) = count.get(&tk) {
    println!("count = {v}");
}
# let _ = (name, enabled, ratio);
# }
```

Bind a variable to a widget with `Opt`:

```rust
# use rtk::{Checkbutton, Label, Opt, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let label = Label::new(&mut tk, root, "value")?;
# let count = tk.int_var(0);
# let enabled = tk.bool_var(true);
# let check = Checkbutton::new(&mut tk, root, "Enable", enabled)?;
tk.config(label, Opt::TextVariableInt(count));
// Checkbutton/Radiobutton/Switch take their Var at construction.
# let _ = check;
# Ok(())
# }
```

### Commands

```rust
# use rtk::{Button, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let button = Button::new(&mut tk, root, "Increment")?;
# let count = tk.int_var(0);
tk.set_command(button, move |tk| {
    if let Some(v) = count.get(tk) {
        count.set(tk, v + 1);
    }
});
# Ok(())
# }
```

Commands may configure widgets, replace themselves, create or destroy subtrees, or quit.
Reentering a running command is suppressed.

### Entry submit

```rust
# use rtk::{Entry, Label, Opt, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let entry = Entry::new(&mut tk, root)?;
# let echo = Label::new(&mut tk, root, "")?;
tk.entry_bind_submit(entry, move |tk| {
    let text = tk.entry_get(entry).unwrap_or_default();
    tk.config(echo, Opt::Text(format!("Submitted: {text}")));
});
# Ok(())
# }
```

### Traces

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
# let count = tk.int_var(0);
count.trace(&mut tk, |_tk| {
    // runs synchronously after count changes
});
# }
```

---

## 5. Widget tour

### Buttons

```rust
# use rtk::{Button, Opt, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let b = Button::new(&mut tk, root, "Save")?;
tk.config(b, Opt::Style("Accent.TButton".into()));
tk.config(b, Opt::Disabled(true));
tk.set_command(b, |tk| {
    tk.toast("Saved");
});
# Ok(())
# }
```

### Checkbutton, Radiobutton, Switch

```rust
# use rtk::{Checkbutton, Radiobutton, Switch, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let check_var = tk.bool_var(true);
let check = Checkbutton::new(&mut tk, root, "Enable", check_var)?;

let radio_var = tk.int_var(0);
let r1 = Radiobutton::new(&mut tk, root, "Alpha", radio_var, 0)?;
let r2 = Radiobutton::new(&mut tk, root, "Beta", radio_var, 1)?;

let sw_var = tk.bool_var(false);
let sw = Switch::new(&mut tk, root, "Dark mode", sw_var)?;
tk.switch_set(sw, true);
# let _ = (check, r1, r2);
# Ok(())
# }
```

### Entry and Text

```rust
# use rtk::{Entry, Text, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let entry = Entry::new(&mut tk, root)?;
tk.entry_set(entry, "type here");

let text = Text::new(&mut tk, root, 40, 6)?; // 40 cols, 6 rows
tk.text_set(text, "line one\nline two");
tk.text_insert_end(text, "\nappended");
tk.text_set_readonly(text, true);
# Ok(())
# }
```

### Listbox, Combobox, Spinbox

```rust
# use rtk::{Combobox, Listbox, Spinbox, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let listbox = Listbox::new(&mut tk, root, 8)?;
for i in 0..20 {
    tk.listbox_insert(listbox, i, format!("Item {i}"));
}
tk.listbox_select(listbox, Some(0));
tk.listbox_bind_select(listbox, |tk, idx| {
    if let Some(i) = idx { tk.toast(format!("Selected {i}")); }
});

let combo_var = tk.int_var(0);
let combo = Combobox::new(&mut tk, root, vec!["One".into(), "Two".into()], combo_var)?;

let spin_var = tk.int_var(3);
let spin = Spinbox::new(&mut tk, root, 0, 10, spin_var)?;
# let _ = (combo, spin);
# Ok(())
# }
```

### Scale, Slider, DoubleSpinbox

```rust
# use rtk::{DoubleSpinbox, Orient, Scale, Slider, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let scale_var = tk.int_var(50);
let scale = Scale::new(&mut tk, root, 0, 100, scale_var)?;

let slider_var = tk.f64_var(0.35);
let slider = Slider::new(&mut tk, root, 0.0, 1.0, slider_var)?;
tk.slider_set_orient(slider, Orient::Horizontal);
tk.slider_set_step(slider, 0.05);
tk.slider_set_length(slider, 320.0);

let dspin_var = tk.f64_var(3.3);
let dspin = DoubleSpinbox::new(&mut tk, root, 0.0, 12.0, dspin_var)?;
tk.dspin_set_decimals(dspin, 2);
tk.dspin_set_suffix(dspin, " V");
# let _ = scale;
# Ok(())
# }
```

### ProgressBar and ProgressRing

```rust
# use rtk::{ProgressBar, ProgressRing, RingMode, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let progress_var = tk.f64_var(0.35);
let progress_int = tk.int_var(35);
let bar = ProgressBar::new(&mut tk, root, 320.0, progress_int)?;
tk.progress_bind(bar, progress_var);
tk.progress_set_show_percent(bar, true);
tk.progress_set_label(bar, Some("Downloading".into()));

let ring_var = tk.f64_var(0.35);
let ring = ProgressRing::new(&mut tk, root, 96.0, ring_var)?;
tk.ring_set_mode(ring, RingMode::Percent);
tk.ring_set_show_text(ring, true);
# Ok(())
# }
```

### Notebook

```rust
# use rtk::{Frame, Notebook, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let nb = Notebook::new(&mut tk, root)?;
let page1 = Frame::new(&mut tk, nb)?;
let page2 = Frame::new(&mut tk, nb)?;
tk.notebook_add(nb, page1, "General");
tk.notebook_add(nb, page2, "Advanced");
tk.notebook_select(nb, 1);
tk.notebook_bind_changed(nb, |_tk, _idx| { /* .. */ });
# Ok(())
# }
```

### ScrollArea

```rust
# use rtk::{Label, PackOpts, ScrollArea, Scrollbar, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let area = ScrollArea::new(&mut tk, root, 320.0, 200.0)?;
if let Some(content) = tk.scrollarea_content(area) {
    for row in 0..20 {
        let label = Label::new(&mut tk, content, format!("Row {row}"))?;
        tk.pack(label, PackOpts::default());
    }
}
let vbar = Scrollbar::new(&mut tk, root)?;
let hbar = Scrollbar::horizontal(&mut tk, root)?;
tk.scrollarea_attach_scrollbars(area, Some(vbar), Some(hbar));
# Ok(())
# }
```

### Table

```rust
# use rtk::{Table, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let table = Table::new(&mut tk, root, &["ID", "Name", "Value"], 12)?;
let rows: Vec<Vec<String>> = (0..200)
    .map(|i| vec![format!("{i}"), format!("Item {i}"), format!("{}", i * 37)])
    .collect();
tk.table_set_rows(table, rows);
tk.table_sort_by(table, 0, true);
tk.table_on_select(table, |_tk, _row| { /* .. */ });
# Ok(())
# }
```

### TreeView

```rust
# use rtk::{TreeView, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let tree = TreeView::new(&mut tk, root, 10)?;
let group = tk.tree_insert(tree, None, "Group").expect("root");
let child = tk.tree_insert(tree, Some(group), "Child").expect("child");
tk.tree_expand_all(tree);
tk.tree_select(tree, Some(child));
# Ok(())
# }
```

### Menus

```rust
# use rtk::{MenuItem, Menubar, Table, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let bar = Menubar::new(&mut tk, root)?;
let file = tk.menu_create();
let open = tk.menu_add(file, MenuItem::command("Open").with_accel("Ctrl+O"));
tk.menu_set_command(file, open, |tk| {
    tk.toast("Open");
});
tk.menu_add(file, MenuItem::separator());
let quit = tk.menu_add(file, MenuItem::command("Quit"));
tk.menu_set_command(file, quit, Tk::quit);
tk.menubar_add(bar, "File", file);

// Context menu
let table = Table::new(&mut tk, root, &["A", "B"], 8)?;
let ctx = tk.menu_create();
tk.menu_add(ctx, MenuItem::command("Copy"));
tk.bind_context_menu(table, ctx);
# Ok(())
# }
```

### Toasts

```rust
# use rtk::{ToastKind, ToastPosition, Tk};
# fn main() {
# let mut tk = Tk::new();
tk.toast("Saved");
tk.toast_with("Disk full", ToastKind::Error, 3000);
tk.toast_set_position(ToastPosition::BottomRight);
# }
```

### Dialogs

```rust,no_run
# use rtk::{dialog, MessageKind, Tk};
# fn main() {
# let mut tk = Tk::new();
dialog::message_box(&mut tk, "Hello", "This is a message box.", MessageKind::Info, |_tk| {});
dialog::ask_yes_no(&mut tk, "Confirm", "Continue?", |_tk, yes| {
    if yes { /* .. */ }
});
dialog::ask_string(&mut tk, "Name", "Enter a name:", "default", |_tk, res| {
    if let Some(name) = res { /* .. */ }
});
# }
```

File pickers: `dialog::ask_open_filename`, `dialog::ask_save_filename`,
`dialog::ask_directory`. Typed confirmation: `dialog::ask_typed_confirmation`.

### Canvas

```rust
# use rtk::{Canvas, Color, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let canvas = Canvas::new(&mut tk, root, 320.0, 200.0)?;
tk.canvas_create_rect(canvas, 10.0, 10.0, 90.0, 60.0, Color::rgb(79, 140, 255));
tk.canvas_create_oval(canvas, 110.0, 10.0, 190.0, 60.0, Color::rgb(230, 159, 0));
tk.canvas_create_line(canvas, 200.0, 10.0, 300.0, 60.0, Color::rgb(0, 114, 178));
tk.canvas_create_text(canvas, 20.0, 80.0, "canvas", Color::rgb(25, 25, 25));

// Rounded rects, polylines, polygons, rings, big text
tk.canvas_create_round_rect(canvas, 10.0, 100.0, 120.0, 160.0, 14.0, Color::WHITE);
tk.canvas_create_polyline(canvas, &[(0.0, 30.0), (10.0, 12.0), (20.0, 18.0)], Color::BLACK, 2.0);
tk.canvas_create_text_ex(canvas, 10.0, 180.0, "1.12", Color::BLACK, 34.0, 700);
# Ok(())
# }
```

Scrolling a canvas: `tk.canvas_scrollregion(canvas, w, h)` then `tk.link_scrollbar(sb, canvas)`.

### Charts

```rust
# use rtk::{Chart, ChartKind, Sparkline, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let chart = Chart::new(&mut tk, root, 640.0, 200.0, ChartKind::Line)?;
tk.chart_set_time_axis(chart, true);
tk.chart_set_legend(chart, true);
let series = tk.chart_add_series(chart, "temp").unwrap_or(0);
tk.chart_set_data(chart, series, &[(0.0, 20.0), (60.0, 22.0), (120.0, 21.0)]);
tk.chart_append(chart, series, 180.0, 23.0);

let spark = Sparkline::new(&mut tk, root, 640.0, 60.0)?;
tk.chart_set_data(spark, 0, &[(0.0, 0.1), (1.0, 0.4), (2.0, 0.2)]);
# Ok(())
# }
```

### Images and SVG

```rust
# use rtk::{Image, ImageFit, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
let image = Image::new(&mut tk, root, 64.0, 64.0)?;
tk.image_set_svg(image, br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">
    <circle cx="32" cy="32" r="28" fill="#0072B2"/></svg>"##);

let png = Image::new(&mut tk, root, 64.0, 64.0)?;
tk.image_load_png(png, "logo.png");
tk.image_set_fit(png, ImageFit::Contain);
# Ok(())
# }
```

Built-in icons: `rtk::icons::icon(&mut tk, Icon::Bolt, 16, Color::rgb(79, 140, 255))`.

### Drag and drop

```rust
# use rtk::{Label, Opt, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let label = Label::new(&mut tk, root, "Drop files here")?;
tk.bind_file_drop(label, move |tk, paths| {
    let names: Vec<String> = paths.iter()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .collect();
    tk.config(label, Opt::Text(names.join(", ")));
});
# Ok(())
# }
```

### Tray (feature `tray`)

```rust,ignore
use rtk::tray::{Tray, TrayOptions};

let tray = Tray::new(TrayOptions::default(), &[menu_id])?;
tray.set_tooltip("rtk app")?;
```

---

## 6. Theming

### Built-in themes

```rust
# use rtk::{Theme, Tk};
# fn main() {
# let mut tk = Tk::new();
tk.set_theme(Theme::Dark);
# }
```

Available: `Light`, `Dark`, `HighContrast`, `Fluent`, `Material`, `ModernDark`, `ModernLight`.

### Follow the system theme

```rust
# use rtk::{Theme, Tk};
# fn main() {
# let mut tk = Tk::new();
tk.follow_system_theme(Theme::Light, Theme::Dark);
// later
tk.unfollow_system_theme();
# }
```

### Colourblind-safe palette

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
tk.set_colorblind_safe(true);
# }
```

### Accent colour and tokens

```rust
# use rtk::{Color, Tk};
# fn main() {
# let mut tk = Tk::new();
tk.set_accent(Color::rgb(0, 103, 192));

let mut tokens = tk.tokens().clone();
tokens.radius.md = 8.0;
tk.set_tokens(tokens);
# }
```

### Named styles and stylesheets

```rust
# use rtk::{Button, Color, Opt, State, StyleSpec, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let button = Button::new(&mut tk, root, "Styled")?;
tk.style_configure("Accent.TButton", StyleSpec {
    bg: Some(Color::rgb(51, 102, 255)),
    fg: Some(Color::WHITE),
    ..StyleSpec::new()
});
tk.config(button, Opt::Style("Accent.TButton".into()));

tk.load_stylesheet(r#"
    TButton { background: #3366ff; color: white; border-radius: 6px; }
    Accent.TButton:hover { background: #4a7dff; }
"#)?;
# let _ = State::HOVER;
# Ok(())
# }
```

Semantic colours: `tk.semantic_tokens().color(ColorRole::Accent)`.

---

## 7. Keyboard, clipboard, editing, IME

### Shortcuts

```rust
# use rtk::{Shortcut, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
let id = tk.bind_shortcut("Ctrl+S", |tk| {
    tk.toast("Saved");
})?;
tk.unbind_shortcut(id);
# let _ = Shortcut::parse("Ctrl+S")?;
# Ok(())
# }
```

### Clipboard

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
tk.clipboard_append("copied text");
if let Some(text) = tk.clipboard_get() { /* .. */ }
tk.clipboard_clear();
# }
```

### Selection and undo

```rust
# use rtk::{Entry, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let entry = Entry::new(&mut tk, root)?;
tk.entry_select_range(entry, 0, 5);
let sel = tk.entry_selected_text(entry);
tk.edit_undo(entry);
tk.edit_redo(entry);
# let _ = sel;
# Ok(())
# }
```

### IME

IME composition is handled automatically for focused text widgets. In tests:

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
tk.sim_ime_preedit("にほん", None);
tk.sim_ime_commit("日本");
# }
```

---

## 8. Window API

```rust,no_run
# use rtk::Tk;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
tk.title("My App");
tk.geometry(960, 620);
tk.minsize(640, 480);
tk.maxsize(1920, 1080);
tk.resizable(true, false);
# let rgba = vec![0u8; 32 * 32 * 4];
# let png_bytes: &[u8] = &[];
tk.iconphoto_rgba(rgba, 32, 32)?;
tk.iconphoto_png(png_bytes)?;

tk.protocol_close(|tk| {
    tk.quit();
    true // allow close
});
# Ok(())
# }
```

`tk.quit()` requests shutdown; destroying the root also requests shutdown.

---

## 9. Timers and threads

### Timers

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
let id = tk.every(1000, |_tk| { /* every second */ });
tk.after(3000, |_tk| { /* once after 3 s */ });
tk.after_cancel(id);
# }
```

While timers are pending the loop sleeps until the next deadline (no busy-waiting).

### Threads

`TkProxy` is `Send + Clone` and lets background threads post closures back to the UI thread:

```rust
# use rtk::Tk;
# fn main() {
# let mut tk = Tk::new();
# let progress_var = tk.f64_var(0.0);
let proxy = tk.proxy();
std::thread::spawn(move || {
    let p = proxy.clone();
    p.post(move |tk| {
        progress_var.set(tk, 50.0);
    });
});
# }
```

Posted closures run on the next event-loop iteration (or via `tk.process_posted()` in
tests).

---

## 10. Accessibility

With the `accesskit` feature enabled, `rtk` publishes its widget tree to platform screen
readers (Windows UIA, macOS NSAccessibility, Linux AT-SPI). Widgets expose role, label,
value, range, toggled state, disabled/focused flags, bounds and supported actions.

The dependency-free snapshot works without the feature:

```rust
# use rtk::{A11yAction, Button, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# let root = tk.root();
# let button = Button::new(&mut tk, root, "Save")?;
let tree = tk.accessibility_tree();
tk.a11y_set_label(button, "Save document");
tk.a11y_do_action(button, A11yAction::Click, None);
# let _ = tree;
# Ok(())
# }
```

---

## 11. Testing your app headlessly

No display server or event loop is required for layout or rendering.

```rust
# use rtk::{Button, Key, Tk};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
# tk.geometry(400, 300);
# let root = tk.root();
# let button = Button::new(&mut tk, root, "Click")?;
# tk.pack(button, rtk::PackOpts::default());
tk.sim_resize(400, 300);
tk.sim_mouse_move(20.0, 20.0);
tk.sim_click(20.0, 20.0);
tk.sim_key(Key::Tab);
tk.sim_text("hello");

let rect = tk.widget_rect(button).unwrap();
let focused = tk.focused_widget();

let pixmap = tk.render_to_pixmap(400, 300)?;
tk.save_png("out.png", 400, 300)?;
# let _ = (rect, focused, pixmap);
# Ok(())
# }
```

HiDPI: `tk.sim_scale_factor(2.0)` then render at physical dimensions.

### Golden tests

`rtk::testing` provides helpers for pixel-comparison tests:

```rust
# use rtk::testing::{assert_golden, GoldenOpts};
# use rtk::Tk;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut tk = Tk::new();
assert_golden(&mut tk, "my_widget_light_1x.png", "my_widget", 64, 64, GoldenOpts::default());
# Ok(())
# }
```

Set `RTK_BLESS=1` in the environment to (re)write the golden file instead of comparing.

---

## 12. Gallery

The bundled gallery example shows every widget in every theme:

```sh
cargo run --example gallery
```

Render headless screenshots (one PNG per theme and page, plus a 2x HiDPI capture):

```sh
cargo run --example gallery -- --screenshot-dir docs/gallery
```

Other examples: `hello`, `counter`, `demo`, `form`, `modern`, `scrolling`, `styling`,
`timers`, `widgets`, `gallery_charts_media`, `gallery_progress_inputs`, `gallery_m011`.

---

## License

MIT OR Apache-2.0. The bundled Noto Sans font is licensed under the SIL Open Font
License 1.1; see `fonts/OFL.txt`.
