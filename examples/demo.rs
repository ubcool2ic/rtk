use std::env;
use std::error::Error;

use rtk::{
    Button, Canvas, Checkbutton, Color, DragPhase, Entry, Frame, Label, Listbox, PackOpts,
    ProgressBar, Radiobutton, Scale, Scrollbar, Side, Spinbox, Theme, Tk,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut dark = false;
    let mut png_out: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--dark" {
            dark = true;
        } else if args[i] == "--png" && i + 1 < args.len() {
            png_out = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let mut tk = Tk::new();
    tk.title("rtk demo");
    tk.geometry(640, 480);
    if dark {
        tk.set_theme(Theme::Dark);
    }

    let root = tk.root();

    // Main layout frames
    let left_col = Frame::new(&mut tk, root)?;
    let mid_col = Frame::new(&mut tk, root)?;
    let right_col = Frame::new(&mut tk, root)?;

    tk.pack(left_col, PackOpts { side: Side::Left, padx: 8.0, pady: 8.0, ..Default::default() });
    tk.pack(mid_col, PackOpts { side: Side::Left, padx: 8.0, pady: 8.0, ..Default::default() });
    tk.pack(right_col, PackOpts { side: Side::Left, padx: 8.0, pady: 8.0, ..Default::default() });

    // --- Left column ---
    let title = Label::new(&mut tk, left_col, "Controls")?;
    tk.pack(title, PackOpts::default());

    let entry = Entry::new(&mut tk, left_col)?;
    tk.pack(entry, PackOpts::default());
    tk.entry_set(entry, "Type here");

    let radio_var = tk.int_var(1);
    let r1 = Radiobutton::new(&mut tk, left_col, "Option A", radio_var, 1)?;
    let r2 = Radiobutton::new(&mut tk, left_col, "Option B", radio_var, 2)?;
    let r3 = Radiobutton::new(&mut tk, left_col, "Option C", radio_var, 3)?;
    tk.pack(r1, PackOpts::default());
    tk.pack(r2, PackOpts::default());
    tk.pack(r3, PackOpts::default());

    let check_var = tk.bool_var(true);
    let cb = Checkbutton::new(&mut tk, left_col, "Enable Feature", check_var)?;
    tk.pack(cb, PackOpts::default());

    let scale_var = tk.int_var(40);
    let scale = Scale::new(&mut tk, left_col, 0, 100, scale_var)?;
    tk.pack(scale, PackOpts::default());

    let spin_var = tk.int_var(5);
    let spinbox = Spinbox::new(&mut tk, left_col, 0, 10, spin_var)?;
    tk.pack(spinbox, PackOpts::default());

    let progress = ProgressBar::new(&mut tk, left_col, 160.0, scale_var)?;
    tk.pack(progress, PackOpts::default());

    // Theme toggle checkbutton
    let dark_var = tk.bool_var(dark);
    let theme_cb = Checkbutton::new(&mut tk, left_col, "Dark theme", dark_var)?;
    tk.pack(theme_cb, PackOpts::default());
    dark_var.trace(&mut tk, move |tk| {
        if let Some(is_dark) = dark_var.get(tk) {
            tk.set_theme(if is_dark { Theme::Dark } else { Theme::Light });
        }
    });

    // --- Middle column: Listbox + Scrollbar ---
    let lb_frame = Frame::new(&mut tk, mid_col)?;
    tk.pack(lb_frame, PackOpts::default());

    let listbox = Listbox::new(&mut tk, lb_frame, 15)?;
    tk.pack(listbox, PackOpts { side: Side::Left, ..Default::default() });

    let scrollbar = Scrollbar::new(&mut tk, lb_frame)?;
    tk.pack(scrollbar, PackOpts { side: Side::Left, fill: rtk::Fill::Y, ..Default::default() });
    tk.link_scrollbar(scrollbar, listbox);

    let fruits = [
        "Apple", "Apricot", "Avocado", "Banana", "Blackberry",
        "Blueberry", "Cherry", "Cranberry", "Date", "Dragonfruit",
        "Elderberry", "Fig", "Grape", "Grapefruit", "Guava",
        "Honeydew", "Kiwi", "Lemon", "Lime", "Mango",
    ];
    for fruit in fruits {
        tk.listbox_insert(listbox, usize::MAX, fruit);
    }

    // --- Right column: Canvas 220x200 + Clear Button ---
    let canvas_title = Label::new(&mut tk, right_col, "Canvas (Drag to draw)")?;
    tk.pack(canvas_title, PackOpts::default());

    let canvas = Canvas::new(&mut tk, right_col, 220.0, 200.0)?;
    tk.pack(canvas, PackOpts::default());

    tk.canvas_bind_drag(canvas, move |tk, cx, cy, phase| {
        if matches!(phase, DragPhase::Press | DragPhase::Move) {
            let color = if tk.theme() == Theme::Dark {
                Color::rgb(100, 200, 255)
            } else {
                Color::rgb(200, 50, 50)
            };
            tk.canvas_create_oval(canvas, cx - 4.0, cy - 4.0, cx + 4.0, cy + 4.0, color);
        }
    });

    let clear_btn = Button::new(&mut tk, right_col, "Clear Canvas")?;
    tk.pack(clear_btn, PackOpts::default());
    tk.set_command(clear_btn, move |tk| {
        tk.canvas_clear(canvas);
    });

    if let Some(path) = png_out {
        tk.save_png(path, 640, 480)?;
        return Ok(());
    }

    tk.mainloop()
}
