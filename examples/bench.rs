//! Offscreen performance benchmark (no window). `cargo run --release --example bench`
use std::error::Error;
use std::time::Instant;

use rtk::{Button, Opt, Theme, 
    Canvas, Checkbutton, Color, Entry, Frame, Label, Listbox, PackOpts, ProgressBar,
    Radiobutton, Scale, Scrollbar, Side, Spinbox, Tk,
};

fn demo_like() -> Result<Tk, Box<dyn Error>> {
    let mut tk = Tk::new();
    let root = tk.root();
    let cols: Vec<_> = (0..3).map(|_| Frame::new(&mut tk, root)).collect::<Result<_, _>>()?;
    for c in &cols {
        tk.pack(*c, PackOpts { side: Side::Left, padx: 8.0, pady: 8.0, ..Default::default() });
    }
    let l = cols[0];
    let w = Label::new(&mut tk, l, "Controls")?;
    tk.pack(w, PackOpts::default());
    let e = Entry::new(&mut tk, l)?;
    tk.pack(e, PackOpts::default());
    let rv = tk.int_var(1);
    for i in 1..=3 {
        let r = Radiobutton::new(&mut tk, l, "Option", rv, i)?;
        tk.pack(r, PackOpts::default());
    }
    let cv = tk.bool_var(true);
    let c = Checkbutton::new(&mut tk, l, "Enable", cv)?;
    tk.pack(c, PackOpts::default());
    let sv = tk.int_var(40);
    let s = Scale::new(&mut tk, l, 0, 100, sv)?;
    tk.pack(s, PackOpts::default());
    let spv = tk.int_var(5);
    let sp = Spinbox::new(&mut tk, l, 0, 10, spv)?;
    tk.pack(sp, PackOpts::default());
    let p = ProgressBar::new(&mut tk, l, 160.0, sv)?;
    tk.pack(p, PackOpts::default());
    let lb = Listbox::new(&mut tk, cols[1], 15)?;
    tk.pack(lb, PackOpts { side: Side::Left, ..Default::default() });
    let sb = Scrollbar::new(&mut tk, cols[1])?;
    tk.pack(sb, PackOpts { side: Side::Left, fill: rtk::Fill::Y, ..Default::default() });
    tk.link_scrollbar(sb, lb);
    for i in 0..20 {
        tk.listbox_insert(lb, usize::MAX, format!("Item {i}"));
    }
    let canvas = Canvas::new(&mut tk, cols[2], 220.0, 200.0)?;
    tk.pack(canvas, PackOpts::default());
    for i in 0..60 {
        let (x, y) = ((i * 13 % 200) as f32, (i * 7 % 180) as f32);
        tk.canvas_create_oval(canvas, x, y, x + 8.0, y + 8.0, Color::rgb(200, 50, 50));
    }
    Ok(tk)
}

fn time_renders(tk: &mut Tk, w: u32, h: u32, n: u32, relayout: bool) -> f64 {
    let _ = tk.render_to_pixmap(w, h); // warm text caches
    let start = Instant::now();
    for i in 0..n {
        if relayout {
            // Alternate a 1px resize to force a full layout pass each frame.
            tk.sim_resize(w - (i % 2), h);
        }
        let _ = tk.render_to_pixmap(w, h);
    }
    start.elapsed().as_secs_f64() * 1000.0 / f64::from(n)
}

fn styled(theme: Theme) -> Result<(Tk, rtk::WidgetId), Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.set_theme(theme);
    let root = tk.root();
    let mut first = root;
    for _ in 0..4 {
        let card = Frame::new(&mut tk, root)?;
        tk.pack(card, PackOpts { side: Side::Left, padx: 6.0, pady: 6.0, ..Default::default() });
        for (i, text) in ["Standard", "Accent"].iter().enumerate() {
            let b = Button::new(&mut tk, card, *text)?;
            if i == 1 {
                tk.config(b, Opt::Style("Accent.TButton".into()));
            }
            tk.pack(b, PackOpts { pady: 4.0, ..Default::default() });
            if first == root {
                first = b;
            }
        }
        let e = Entry::new(&mut tk, card)?;
        tk.pack(e, PackOpts::default());
        let v = tk.int_var(50);
        let s = Scale::new(&mut tk, card, 0, 100, v)?;
        tk.pack(s, PackOpts::default());
        let p = ProgressBar::new(&mut tk, card, 160.0, v)?;
        tk.pack(p, PackOpts::default());
    }
    Ok((tk, first))
}

fn main() -> Result<(), Box<dyn Error>> {
    for theme in [Theme::Light, Theme::Material] {
        let (mut tk, button) = styled(theme)?;
        tk.sim_resize(960, 600);
        let idle = time_renders(&mut tk, 960, 600, 100, false);
        let _ = tk.render_to_pixmap(960, 600);
        let rect = tk.widget_rect(button).ok_or("rect")?;
        let start = Instant::now();
        let mut frames = 0;
        for i in 0..50 {
            // Hover on/off every 10 frames, advancing 16 ms per frame like the event loop.
            if i % 10 == 0 {
                let x = if (i / 10) % 2 == 0 { rect.x + 5.0 } else { -10.0 };
                tk.sim_mouse_move(x, rect.y + 5.0);
            }
            tk.sim_advance_time(16);
            let _ = tk.render_to_pixmap(960, 600);
            frames += 1;
        }
        println!("styled {theme:?} 960x600: static render {idle:.2} ms/frame, animating hover {:.2} ms/frame",
            start.elapsed().as_secs_f64() * 1000.0 / f64::from(frames));
    }

    let t = Instant::now();
    let mut tk = demo_like()?;
    println!("build demo-like tree: {:.1} ms (includes font database load)", t.elapsed().as_secs_f64() * 1000.0);
    for (w, h) in [(800, 600), (1920, 1080)] {
        tk.sim_resize(w, h);
        println!("demo {w}x{h}: render {:.2} ms/frame, layout+render {:.2} ms/frame (avg of 100)",
            time_renders(&mut tk, w, h, 100, false), time_renders(&mut tk, w, h, 100, true));
    }
    for n in [1_000usize, 10_000] {
        let mut tk = Tk::new();
        let root = tk.root();
        let t = Instant::now();
        for i in 0..n {
            let l = Label::new(&mut tk, root, format!("Label {i}"))?;
            tk.pack(l, PackOpts::default());
        }
        let create = t.elapsed().as_secs_f64() * 1000.0;
        tk.sim_resize(800, 600);
        let t = Instant::now();
        let _ = tk.render_to_pixmap(800, 600);
        let first = t.elapsed().as_secs_f64() * 1000.0;
        println!("{n} Labels: create {create:.1} ms, first layout+render {first:.1} ms, layout+render {:.1} ms, render {:.1} ms (avg of 10)",
            time_renders(&mut tk, 800, 600, 10, true), time_renders(&mut tk, 800, 600, 10, false));

        let mut tk = Tk::new();
        let root = tk.root();
        let lb = Listbox::new(&mut tk, root, 20)?;
        tk.pack(lb, PackOpts::default());
        let t = Instant::now();
        for i in 0..n {
            tk.listbox_insert(lb, usize::MAX, format!("Row {i}"));
        }
        let ins = t.elapsed().as_secs_f64() * 1000.0;
        tk.sim_resize(800, 600);
        let t = Instant::now();
        let _ = tk.render_to_pixmap(800, 600);
        let first = t.elapsed().as_secs_f64() * 1000.0;
        println!("Listbox {n} rows: insert {ins:.1} ms, first layout+render {first:.1} ms, layout+render {:.1} ms, render {:.1} ms (avg of 10)",
            time_renders(&mut tk, 800, 600, 10, true), time_renders(&mut tk, 800, 600, 10, false));
    }
    Ok(())
}
