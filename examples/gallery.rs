//! The rtk 1.0 gallery: a single example that shows every widget in every
//! built-in theme.
//!
//! Run interactively with `cargo run --example gallery`, or render headless
//! screenshots with `cargo run --example gallery -- --screenshot-dir DIR`.
//! The headless mode writes one PNG per (theme, notebook page) pair plus a
//! 2x HiDPI capture of the "Basics" page in the Light theme.

use std::error::Error;
use std::path::{Path, PathBuf};

use rtk::icons::{self, Icon};
use rtk::{
    Button, Canvas, Chart, ChartKind, Checkbutton, Color, ColorRole, Combobox, Compound,
    DoubleSpinbox, Entry, Fill, Frame, GridOpts, Image, Label, Listbox, MenuItem, Menubar,
    Notebook, Opt, Orient, PackOpts, ProgressBar, ProgressRing, Radiobutton, RingMode, Scale,
    ScrollArea, Scrollbar, Side, Slider, Sparkline, Spinbox, Switch, Table, Text, Theme, Tk,
    ToastKind, TreeView, WidgetId,
};

/// Handles to the widgets the gallery needs to keep around after building.
struct Pages {
    /// The notebook holding every page.
    notebook: WidgetId,
    /// The theme selector combobox.
    theme_combo: WidgetId,
    /// The integer variable backing the theme selector combobox.
    theme_var: rtk::Var<i64>,
    /// The colourblind-safe toggle.
    colorblind: WidgetId,
    /// The follow-system-theme switch.
    follow_system: WidgetId,
    /// Page slugs in notebook order, used for screenshot filenames.
    slugs: Vec<&'static str>,
}

/// The themes rendered in headless screenshot mode, in order.
const SCREENSHOT_THEMES: [Theme; 7] = [
    Theme::Light,
    Theme::Dark,
    Theme::HighContrast,
    Theme::Fluent,
    Theme::Material,
    Theme::ModernDark,
    Theme::ModernLight,
];

/// Returns the lowercase slug used in screenshot filenames for `theme`.
fn theme_slug(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::HighContrast => "highcontrast",
        Theme::Fluent => "fluent",
        Theme::Material => "material",
        Theme::ModernDark => "moderndark",
        Theme::ModernLight => "modernlight",
        _ => "light",
    }
}

/// Returns the display label for `theme`.
fn theme_label(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "Light",
        Theme::Dark => "Dark",
        Theme::HighContrast => "High Contrast",
        Theme::Fluent => "Fluent",
        Theme::Material => "Material",
        Theme::ModernDark => "Modern Dark",
        Theme::ModernLight => "Modern Light",
        _ => "Light",
    }
}

/// Returns the theme for a combobox index.
fn theme_at(index: i64) -> Theme {
    SCREENSHOT_THEMES
        .get(index.max(0) as usize)
        .copied()
        .unwrap_or(Theme::Light)
}

/// Returns the combobox index for `theme`, or `0` when it is not listed.
fn theme_index(theme: Theme) -> i64 {
    SCREENSHOT_THEMES
        .iter()
        .position(|t| *t == theme)
        .map(|i| i as i64)
        .unwrap_or(0)
}

/// Applies `theme` to the toolkit and keeps the header combobox in sync by
/// driving its backing `IntVar` (which is what the combobox actually reads).
fn apply_theme(tk: &mut Tk, theme_var: rtk::Var<i64>, theme: Theme) {
    tk.set_theme(theme);
    let index = theme_index(theme);
    theme_var.set(tk, index);
}

/// Builds a labelled group frame packed into `parent` on the given side.
fn group(tk: &mut Tk, parent: WidgetId, title: &str, side: Side) -> Result<WidgetId, Box<dyn Error>> {
    let frame = Frame::new(tk, parent)?;
    tk.pack(
        frame,
        PackOpts {
            side,
            fill: Fill::Both,
            expand: true,
            padx: 6.0,
            pady: 6.0,
            ..PackOpts::default()
        },
    );
    let heading = Label::new(tk, frame, title)?;
    tk.config(heading, Opt::Style("Subheading".into()));
    tk.pack(
        heading,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            padx: 4.0,
            pady: 2.0,
            ..PackOpts::default()
        },
    );
    Ok(frame)
}

/// Theme combobox, its backing variable, colourblind toggle and follow-system switch.
type HeaderHandles = (WidgetId, rtk::Var<i64>, WidgetId, WidgetId);

/// Builds the header row: title, theme selector, colourblind toggle and
/// follow-system switch. Returns the theme combobox, its backing variable,
/// the colourblind toggle and the follow-system switch.
fn build_header(
    tk: &mut Tk,
    root: WidgetId,
) -> Result<HeaderHandles, Box<dyn Error>> {
    let header = Frame::new(tk, root)?;
    tk.pack(
        header,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            padx: 8.0,
            pady: 6.0,
            ..PackOpts::default()
        },
    );

    let title = Label::new(tk, header, "rtk gallery")?;
    tk.config(title, Opt::Style("Heading".into()));
    tk.pack(
        title,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            ..PackOpts::default()
        },
    );

    let labels: Vec<String> = SCREENSHOT_THEMES.iter().map(|t| theme_label(*t).into()).collect();
    let theme_var = tk.int_var(0);
    let theme_combo = Combobox::new(tk, header, labels, theme_var)?;
    tk.pack(
        theme_combo,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            ..PackOpts::default()
        },
    );
    tk.set_command(theme_combo, move |tk| {
        let idx = tk.combobox_get(theme_combo).map(|(i, _)| i).unwrap_or(0);
        tk.set_theme(theme_at(idx));
    });

    let cb_var = tk.bool_var(false);
    let colorblind = Checkbutton::new(tk, header, "Colorblind-safe", cb_var)?;
    tk.pack(
        colorblind,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            ..PackOpts::default()
        },
    );
    tk.set_command(colorblind, move |tk| {
        let value = cb_var.get(tk).unwrap_or(false);
        tk.set_colorblind_safe(value);
    });

    let follow_var = tk.bool_var(false);
    let follow_system = Switch::new(tk, header, "Follow system theme", follow_var)?;
    tk.pack(
        follow_system,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            ..PackOpts::default()
        },
    );
    tk.set_command(follow_system, move |tk| {
        let on = tk.switch_get(follow_system).unwrap_or(false);
        if on {
            tk.follow_system_theme(Theme::Light, Theme::Dark);
        } else {
            tk.unfollow_system_theme();
        }
    });

    Ok((theme_combo, theme_var, colorblind, follow_system))
}

/// Builds the "Basics" page: labels, buttons, entry, check/radio, scale,
/// spinbox, combobox, listbox, text, canvas, tooltips and icons.
fn build_basics(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn Error>> {
    let col_a = group(tk, parent, "Buttons & entry", Side::Left)?;
    let col_b = group(tk, parent, "Choices", Side::Left)?;
    let col_c = group(tk, parent, "Text & canvas", Side::Left)?;

    // --- Column A: labels, buttons, entry --------------------------------
    let label = Label::new(tk, col_a, "Hello from rtk")?;
    tk.pack(label, PackOpts::default());

    let row = Frame::new(tk, col_a)?;
    tk.pack(row, PackOpts::default());

    let normal = Button::new(tk, row, "Normal")?;
    tk.pack(normal, PackOpts::default());
    tk.tooltip(normal, "A plain button");
    if let Some(img) = icons::icon(tk, Icon::Bolt, 16, Color::rgb(79, 140, 255)) {
        tk.config(normal, Opt::Image(img));
        tk.config(normal, Opt::Compound(Compound::Left));
    }

    let accent = Button::new(tk, row, "Accent")?;
    tk.config(accent, Opt::Style("Accent.TButton".into()));
    tk.pack(accent, PackOpts::default());

    let disabled = Button::new(tk, row, "Disabled")?;
    tk.config(disabled, Opt::Disabled(true));
    tk.pack(disabled, PackOpts::default());

    let entry = Entry::new(tk, col_a)?;
    tk.pack(entry, PackOpts::default());
    tk.entry_set(entry, "Type here and press Enter");
    let echo = Label::new(tk, col_a, "Type and press Enter")?;
    tk.pack(echo, PackOpts::default());
    tk.entry_bind_submit(entry, move |tk| {
        let text = tk.entry_get(entry).unwrap_or_default();
        tk.config(echo, Opt::Text(format!("Submitted: {text}")));
    });

    // --- Column B: check, radio, scale, spinbox, combobox, listbox -------
    let check_var = tk.bool_var(true);
    let check = Checkbutton::new(tk, col_b, "Check me", check_var)?;
    tk.pack(check, PackOpts::default());

    let radio_var = tk.int_var(0);
    for (i, name) in ["Alpha", "Beta", "Gamma"].iter().enumerate() {
        let radio = Radiobutton::new(tk, col_b, *name, radio_var, i as i64)?;
        tk.pack(radio, PackOpts::default());
    }

    let scale_var = tk.int_var(50);
    let scale = Scale::new(tk, col_b, 0, 100, scale_var)?;
    tk.pack(scale, PackOpts::default());

    let spin_var = tk.int_var(3);
    let spin = Spinbox::new(tk, col_b, 0, 10, spin_var)?;
    tk.pack(spin, PackOpts::default());

    let combo_var = tk.int_var(0);
    let combo = Combobox::new(
        tk,
        col_b,
        vec!["One".into(), "Two".into(), "Three".into()],
        combo_var,
    )?;
    tk.pack(combo, PackOpts::default());

    let list_row = Frame::new(tk, col_b)?;
    tk.pack(list_row, PackOpts::default());
    let listbox = Listbox::new(tk, list_row, 6)?;
    tk.pack(listbox, PackOpts::default());
    for i in 0..20 {
        tk.listbox_insert(listbox, i, format!("Item {i}"));
    }
    let list_bar = Scrollbar::new(tk, list_row)?;
    tk.pack(list_bar, PackOpts::default());
    tk.link_scrollbar(list_bar, listbox);

    // --- Column C: text, canvas ------------------------------------------
    let text = Text::new(tk, col_c, 40, 3)?;
    tk.pack(text, PackOpts::default());
    tk.text_set(text, "line one\nline two\nline three");

    let canvas = Canvas::new(tk, col_c, 320.0, 120.0)?;
    tk.pack(canvas, PackOpts::default());
    let accent_color = tk.tokens().palette.accent;
    tk.canvas_create_rect(canvas, 10.0, 10.0, 90.0, 60.0, accent_color);
    tk.canvas_create_oval(canvas, 110.0, 10.0, 190.0, 60.0, Color::rgb(230, 159, 0));
    tk.canvas_create_line(canvas, 200.0, 10.0, 300.0, 60.0, Color::rgb(0, 114, 178));
    tk.canvas_create_text(canvas, 20.0, 80.0, "canvas", Color::rgb(25, 25, 25));
    Ok(())
}

/// Builds the "Inputs & progress" page: progress bars, rings, switch,
/// sliders, double spinbox and a scroll area with scrollbars.
fn build_inputs(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn Error>> {
    let col_a = group(tk, parent, "Progress", Side::Left)?;
    let col_b = group(tk, parent, "Controls", Side::Left)?;
    let col_c = group(tk, parent, "Scroll area", Side::Left)?;

    // --- Column A: progress bars and rings -------------------------------
    let progress_var = tk.f64_var(0.35);
    let progress_int = tk.int_var(35);
    let progress = ProgressBar::new(tk, col_a, 320.0, progress_int)?;
    tk.pack(progress, PackOpts::default());
    tk.progress_bind(progress, progress_var);
    tk.progress_set_show_percent(progress, true);
    tk.progress_set_label(progress, Some("Downloading".into()));
    tk.progress_set_smooth(progress, true);

    let busy_var = tk.f64_var(0.0);
    let busy_int = tk.int_var(0);
    let busy = ProgressBar::new(tk, col_a, 320.0, busy_int)?;
    tk.pack(busy, PackOpts::default());
    tk.progress_bind(busy, busy_var);
    tk.progress_set_indeterminate(busy, true);

    let ring_var = tk.f64_var(0.35);
    let ring = ProgressRing::new(tk, col_a, 96.0, ring_var)?;
    tk.pack(ring, PackOpts::default());
    tk.ring_set_mode(ring, RingMode::Percent);
    tk.ring_set_show_text(ring, true);

    let spinner_var = tk.f64_var(0.0);
    let spinner = ProgressRing::new(tk, col_a, 64.0, spinner_var)?;
    tk.pack(spinner, PackOpts::default());
    tk.ring_set_mode(spinner, RingMode::Spinner);

    // --- Column B: switch, sliders, double spinbox -----------------------
    let switch_var = tk.bool_var(true);
    let switch = Switch::new(tk, col_b, "Switch", switch_var)?;
    tk.pack(switch, PackOpts::default());

    let slider_var = tk.f64_var(0.35);
    let slider = Slider::new(tk, col_b, 0.0, 1.0, slider_var)?;
    tk.pack(slider, PackOpts::default());
    tk.slider_set_orient(slider, Orient::Horizontal);
    tk.slider_set_step(slider, 0.05);
    tk.slider_set_length(slider, 320.0);

    let vslider_var = tk.f64_var(0.5);
    let vslider = Slider::new(tk, col_b, 0.0, 1.0, vslider_var)?;
    tk.pack(vslider, PackOpts::default());
    tk.slider_set_orient(vslider, Orient::Vertical);
    tk.slider_set_length(vslider, 160.0);

    let dspin_var = tk.f64_var(3.3);
    let dspin = DoubleSpinbox::new(tk, col_b, 0.0, 12.0, dspin_var)?;
    tk.pack(dspin, PackOpts::default());
    tk.dspin_set_decimals(dspin, 2);
    tk.dspin_set_step(dspin, 0.05);
    tk.dspin_set_suffix(dspin, " V");

    // --- Column C: scroll area with attached scrollbars ------------------
    let area = ScrollArea::new(tk, col_c, 320.0, 200.0)?;
    tk.pack(area, PackOpts::default());
    if let Some(content) = tk.scrollarea_content(area) {
        for row in 0..20 {
            let label = Label::new(tk, content, format!("Row {row}"))?;
            tk.pack(label, PackOpts::default());
        }
    }
    let vbar = Scrollbar::new(tk, col_c)?;
    tk.pack(vbar, PackOpts::default());
    let hbar = Scrollbar::horizontal(tk, col_c)?;
    tk.pack(hbar, PackOpts::default());
    tk.scrollarea_attach_scrollbars(area, Some(vbar), Some(hbar));
    Ok(())
}

/// Builds the "Data" page: table, tree, menubar, context menu, toasts and
/// dialogs.
fn build_data(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn Error>> {
    let bar = Menubar::new(tk, parent)?;
    tk.pack(
        bar,
        PackOpts {
            side: Side::Top,
            fill: Fill::X,
            ..PackOpts::default()
        },
    );

    let file = tk.menu_create();
    let open = tk.menu_add(file, MenuItem::command("Open").with_accel("Ctrl+O"));
    tk.menu_set_command(file, open, |tk| {
        tk.toast_with("Open clicked", ToastKind::Info, 2000);
    });
    tk.menu_add(file, MenuItem::separator());
    let quit = tk.menu_add(file, MenuItem::command("Quit").with_accel("Ctrl+Q"));
    tk.menu_set_command(file, quit, Tk::quit);
    tk.menubar_add(bar, "File", file);

    let edit = tk.menu_create();
    let check_var = tk.bool_var(true);
    tk.menu_add(edit, MenuItem::check("Wrap", check_var));
    tk.menubar_add(bar, "Edit", edit);

    let view = tk.menu_create();
    let themes = tk.menu_create();
    for (label, theme) in [
        ("Light", Theme::Light),
        ("Dark", Theme::Dark),
        ("High Contrast", Theme::HighContrast),
    ] {
        let idx = tk.menu_add(themes, MenuItem::command(label));
        tk.menu_set_command(themes, idx, move |tk| tk.set_theme(theme));
    }
    tk.menu_add(view, MenuItem::submenu("Theme", themes));
    tk.menubar_add(bar, "View", view);

    let body = Frame::new(tk, parent)?;
    tk.pack(
        body,
        PackOpts {
            side: Side::Top,
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );

    let col_a = group(tk, body, "Table", Side::Left)?;
    let col_b = group(tk, body, "Tree", Side::Left)?;
    let col_c = group(tk, body, "Text, toasts & dialogs", Side::Left)?;

    // --- Column A: table ------------------------------------------------
    let table = Table::new(tk, col_a, &["ID", "Name", "Value"], 12)?;
    tk.pack(
        table,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            padx: 4.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    let rows: Vec<Vec<String>> = (0..200)
        .map(|i| vec![format!("{i}"), format!("Item {i}"), format!("{}", (i * 37) % 1000)])
        .collect();
    tk.table_set_rows(table, rows);
    tk.table_sort_by(table, 0, true);

    let ctx = tk.menu_create();
    let copy = tk.menu_add(ctx, MenuItem::command("Copy"));
    tk.menu_set_command(ctx, copy, |tk| {
        tk.toast_with("Copied", ToastKind::Success, 1500);
    });
    tk.bind_context_menu(table, ctx);

    // --- Column B: tree -------------------------------------------------
    let tree = TreeView::new(tk, col_b, 10)?;
    tk.pack(
        tree,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            padx: 4.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    for i in 0..3 {
        let root = tk
            .tree_insert(tree, None, &format!("Group {i}"))
            .ok_or("tree_insert failed for a root node")?;
        for j in 0..3 {
            let child = tk
                .tree_insert(tree, Some(root), &format!("Child {i}.{j}"))
                .ok_or("tree_insert failed for a child node")?;
            for k in 0..2 {
                tk.tree_insert(tree, Some(child), &format!("Leaf {i}.{j}.{k}"));
            }
        }
    }
    tk.tree_expand_all(tree);

    // --- Column C: text, toast buttons, dialogs -------------------------
    let text = Text::new(tk, col_c, 40, 4)?;
    tk.pack(text, PackOpts::default());
    tk.text_set(
        text,
        "Sample text\nsecond line\nthird line\nfourth line",
    );

    let toast_row = Frame::new(tk, col_c)?;
    tk.pack(toast_row, PackOpts::default());
    for (label, kind) in [
        ("Info", ToastKind::Info),
        ("Success", ToastKind::Success),
        ("Warning", ToastKind::Warning),
        ("Error", ToastKind::Error),
    ] {
        let btn = Button::new(tk, toast_row, label)?;
        tk.pack(
            btn,
            PackOpts {
                side: Side::Left,
                padx: 2.0,
                ..PackOpts::default()
            },
        );
        tk.set_command(btn, move |tk| {
            tk.toast_with(format!("{label} toast"), kind, 2500);
        });
    }

    let dialog_btn = Button::new(tk, col_c, "Open dialogs")?;
    tk.pack(dialog_btn, PackOpts::default());
    tk.set_command(dialog_btn, |tk| {
        rtk::dialog::message_box(tk, "Hello", "This is a message box.", rtk::MessageKind::Info, |_| {});
        rtk::dialog::ask_yes_no(tk, "Confirm", "Open another dialog?", |tk, yes| {
            if yes {
                rtk::dialog::message_box(tk, "Nested", "Nested dialog.", rtk::MessageKind::Info, |_| {});
            }
        });
    });
    Ok(())
}

/// Builds the "Charts & media" page: line/area/bar charts, sparkline, images
/// and a file drop target.
fn build_charts(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn Error>> {
    let col_a = group(tk, parent, "Charts", Side::Left)?;
    let col_b = group(tk, parent, "Media", Side::Left)?;

    // --- Column A: line, area, bar charts and sparkline ------------------
    let line = Chart::new(tk, col_a, 640.0, 200.0, ChartKind::Line)?;
    tk.pack(line, PackOpts::default());
    tk.chart_set_time_axis(line, true);
    tk.chart_set_legend(line, true);
    tk.chart_set_max_points(line, 120);
    let temp = tk.chart_add_series(line, "temp").ok_or("chart_add_series failed")?;
    let humidity = tk.chart_add_series(line, "humidity").ok_or("chart_add_series failed")?;
    let line_points: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let x = (i * 60) as f64;
            let t = i as f64;
            (x, 20.0 + 5.0 * (t * 0.2).sin())
        })
        .collect();
    tk.chart_set_data(line, temp, &line_points);
    let humidity_points: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let x = (i * 60) as f64;
            let t = i as f64;
            (x, 50.0 + 20.0 * (t * 0.15).cos())
        })
        .collect();
    tk.chart_set_data(line, humidity, &humidity_points);
    let mut tick: u64 = 40;
    tk.every(500, move |tk| {
        tick = tick.wrapping_add(1);
        let x = (tick * 60) as f64;
        let t = tick as f64;
        tk.chart_append(line, temp, x, 20.0 + 5.0 * (t * 0.2).sin());
        tk.chart_append(line, humidity, x, 50.0 + 20.0 * (t * 0.15).cos());
    });

    let area = Chart::new(tk, col_a, 640.0, 140.0, ChartKind::Area)?;
    tk.pack(area, PackOpts::default());
    tk.chart_set_legend(area, true);
    let area_series = tk.chart_add_series(area, "load").ok_or("chart_add_series failed")?;
    let area_points: Vec<(f64, f64)> = (0..40)
        .map(|i| {
            let x = i as f64;
            (x, 50.0 + 30.0 * (x * 0.3).sin())
        })
        .collect();
    tk.chart_set_data(area, area_series, &area_points);

    let bar = Chart::new(tk, col_a, 640.0, 140.0, ChartKind::Bar)?;
    tk.pack(bar, PackOpts::default());
    tk.chart_set_legend(bar, true);
    let bar_series = tk.chart_add_series(bar, "sales").ok_or("chart_add_series failed")?;
    let bar_points: Vec<(f64, f64)> = (0..8)
        .map(|i| (i as f64, 10.0 + (i as f64 * 7.0) % 40.0))
        .collect();
    tk.chart_set_data(bar, bar_series, &bar_points);

    let spark = Sparkline::new(tk, col_a, 640.0, 60.0)?;
    tk.pack(spark, PackOpts::default());
    tk.chart_set_max_points(spark, 120);
    let spark_points: Vec<(f64, f64)> = (0..60)
        .map(|i| (i as f64, (i as f64 * 0.4).sin()))
        .collect();
    tk.chart_set_data(spark, 0, &spark_points);

    // --- Column B: images and drop target -------------------------------
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
        <circle cx="32" cy="32" r="28" fill="#0072B2"/>
        <polygon points="32,12 38,26 53,26 41,35 45,50 32,41 19,50 23,35 11,26 26,26" fill="#E69F00"/>
    </svg>"##;
    let image = Image::new(tk, col_b, 64.0, 64.0)?;
    tk.pack(image, PackOpts::default());
    tk.image_set_svg(image, svg);

    let png_image = Image::new(tk, col_b, 64.0, 64.0)?;
    tk.pack(png_image, PackOpts::default());
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32u32 {
        for x in 0..32u32 {
            rgba.extend_from_slice(&[(x * 8) as u8, (y * 8) as u8, 128, 255]);
        }
    }
    tk.image_set_rgba(png_image, 32, 32, &rgba);

    let drop_label = Label::new(tk, col_b, "Drop files here")?;
    tk.pack(drop_label, PackOpts::default());
    tk.bind_file_drop(drop_label, move |tk, paths| {
        let names: Vec<String> = paths
            .iter()
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| p.display().to_string())
            })
            .collect();
        tk.config(drop_label, Opt::Text(names.join(", ")));
    });
    Ok(())
}

/// Builds the "Theme" page: semantic colour swatches, the Okabe-Ito palette
/// and typography samples.
fn build_theme(tk: &mut Tk, parent: WidgetId) -> Result<(), Box<dyn Error>> {
    let col_a = group(tk, parent, "Semantic colours", Side::Left)?;
    let col_b = group(tk, parent, "Palette & typography", Side::Left)?;

    // --- Column A: semantic colour swatches -----------------------------
    let tokens = tk.semantic_tokens();
    let roles = ColorRole::ALL;
    let canvas = Canvas::new(tk, col_a, 420.0, 600.0)?;
    tk.pack(canvas, PackOpts::default());
    let mut y = 8.0;
    for role in roles {
        let color = tokens.color(role);
        tk.canvas_create_rect(canvas, 8.0, y, 48.0, y + 20.0, color);
        tk.canvas_create_text(canvas, 60.0, y + 4.0, format!("{role:?}"), Color::rgb(25, 25, 25));
        y += 26.0;
    }

    // --- Column B: Okabe-Ito palette and typography ---------------------
    let palette_canvas = Canvas::new(tk, col_b, 420.0, 60.0)?;
    tk.pack(palette_canvas, PackOpts::default());
    for (i, color) in rtk::OKABE_ITO.iter().enumerate() {
        let x = 8.0 + i as f32 * 50.0;
        tk.canvas_create_rect(palette_canvas, x, 8.0, x + 40.0, 48.0, *color);
    }

    for (name, style) in [
        ("Heading", "Heading"),
        ("Subheading", "Subheading"),
        ("Body", "Body"),
    ] {
        let label = Label::new(tk, col_b, name)?;
        tk.config(label, Opt::Style(style.into()));
        tk.pack(label, PackOpts::default());
    }
    Ok(())
}

/// Builds the whole gallery UI and returns the handles needed afterwards.
fn build(tk: &mut Tk) -> Result<Pages, Box<dyn Error>> {
    let root = tk.root();
    let (theme_combo, theme_var, colorblind, follow_system) = build_header(tk, root)?;

    let notebook = Notebook::new(tk, root)?;
    tk.pack(
        notebook,
        PackOpts {
            fill: Fill::Both,
            expand: true,
            ..PackOpts::default()
        },
    );

    let basics = Frame::new(tk, notebook)?;
    let inputs = Frame::new(tk, notebook)?;
    let data = Frame::new(tk, notebook)?;
    let charts = Frame::new(tk, notebook)?;
    let theme = Frame::new(tk, notebook)?;
    tk.notebook_add(notebook, basics, "Basics");
    tk.notebook_add(notebook, inputs, "Inputs & progress");
    tk.notebook_add(notebook, data, "Data");
    tk.notebook_add(notebook, charts, "Charts & media");
    tk.notebook_add(notebook, theme, "Theme");

    build_basics(tk, basics)?;
    build_inputs(tk, inputs)?;
    build_data(tk, data)?;
    build_charts(tk, charts)?;
    build_theme(tk, theme)?;

    Ok(Pages {
        notebook,
        theme_combo,
        theme_var,
        colorblind,
        follow_system,
        slugs: vec!["basics", "inputs", "data", "charts", "theme"],
    })
}

/// Generates a 32x32 RGBA window icon (an Okabe-Ito blue square).
fn make_icon() -> Vec<u8> {
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for _ in 0..(32 * 32) {
        rgba.extend_from_slice(&[0x00, 0x72, 0xB2, 0xFF]);
    }
    rgba
}

/// Renders headless screenshots for every theme and page into `dir`.
fn screenshot(dir: &Path) -> Result<(), Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.title("rtk gallery");
    tk.geometry(1100, 760);
    let pages = build(&mut tk)?;

    for theme in SCREENSHOT_THEMES {
        apply_theme(&mut tk, pages.theme_var, theme);
        for (index, slug) in pages.slugs.iter().enumerate() {
            tk.notebook_select(pages.notebook, index);
            tk.sim_resize(1100, 760);
            let path = dir.join(format!("gallery_{}_{}.png", theme_slug(theme), slug));
            tk.save_png(&path, 1100, 760)?;
            println!("{}", path.display());
        }
    }

    apply_theme(&mut tk, pages.theme_var, Theme::Light);
    tk.notebook_select(pages.notebook, 0);
    tk.sim_scale_factor(2.0);
    tk.sim_resize(2200, 1520);
    let path = dir.join("gallery_light_basics_2x.png");
    tk.save_png(&path, 2200, 1520)?;
    println!("{}", path.display());
    Ok(())
}

/// Entry point: runs the gallery or renders screenshots.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot_dir: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot-dir" {
            let Some(value) = args.get(i + 1) else {
                return Err("--screenshot-dir requires a directory argument".into());
            };
            screenshot_dir = Some(PathBuf::from(value));
            i += 2;
            continue;
        }
        i += 1;
    }

    if let Some(dir) = screenshot_dir {
        std::fs::create_dir_all(&dir)?;
        return screenshot(&dir);
    }

    let mut tk = Tk::new();
    tk.title("rtk gallery");
    tk.geometry(1100, 760);
    tk.minsize(900, 640);
    let _ = tk.iconphoto_rgba(make_icon(), 32, 32);
    tk.protocol_close(|tk| {
        tk.quit();
        true
    });
    let _pages = build(&mut tk)?;
    tk.mainloop()
}

// Silence unused-field warnings in screenshot mode while keeping handles.
#[allow(dead_code)]
/// Returns the header widget handles (used by interactive callbacks).
fn _unused(pages: &Pages) -> (WidgetId, WidgetId, WidgetId) {
    (pages.theme_combo, pages.colorblind, pages.follow_system)
}

// Keep GridOpts import used for potential future grid layouts.
#[allow(dead_code)]
/// Returns default grid options (placeholder to keep imports tidy).
fn _grid_defaults() -> GridOpts {
    GridOpts::default()
}
