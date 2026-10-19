use std::env;
use std::error::Error;

use rtk::{
    Button, Checkbutton, Color, Entry, Frame, Label, Opt, PackOpts, ProgressBar,
    Radiobutton, Scale, Side, StyleSpec, Theme, Tk,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut initial_theme = Theme::Light;
    let mut css_file: Option<String> = None;
    let mut png_out: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--theme" && i + 1 < args.len() {
            initial_theme = match args[i + 1].to_lowercase().as_str() {
                "dark" => Theme::Dark,
                "fluent" => Theme::Fluent,
                "material" => Theme::Material,
                "hc" | "highcontrast" => Theme::HighContrast,
                _ => Theme::Light,
            };
            i += 1;
        } else if args[i] == "--css" && i + 1 < args.len() {
            css_file = Some(args[i + 1].clone());
            i += 1;
        } else if args[i] == "--png" && i + 1 < args.len() {
            png_out = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let mut tk = Tk::new();
    tk.title("rtk Styling & Themes Gallery");
    tk.geometry(960, 600);
    tk.set_theme(initial_theme);

    if let Some(css_path) = css_file {
        if let Err(e) = tk.load_stylesheet_file(&css_path) {
            eprintln!("Warning: failed to load stylesheet '{css_path}': {e}");
        } else {
            tk.watch_stylesheet(&css_path);
        }
    }

    let root = tk.root();

    // Top control bar for theme switching
    let top_bar = Frame::new(&mut tk, root)?;
    tk.pack(top_bar, PackOpts { side: Side::Top, fill: rtk::Fill::X, padx: 8.0, pady: 8.0, ..Default::default() });

    let title_lbl = Label::new(&mut tk, top_bar, "Theme:")?;
    tk.pack(title_lbl, PackOpts { side: Side::Left, padx: 6.0, ..Default::default() });

    let theme_var = tk.int_var(match initial_theme {
        Theme::Light => 0,
        Theme::Dark => 1,
        Theme::Fluent => 2,
        Theme::HighContrast => 4,
        // Modern themes (v0.8) are not in this demo picker.
        _ => 0,
        Theme::HighContrast => 4,
    });

    let themes = [
        ("Light", 0, Theme::Light),
        ("Dark", 1, Theme::Dark),
        ("Fluent", 2, Theme::Fluent),
        ("Material", 3, Theme::Material),
        ("High Contrast", 4, Theme::HighContrast),
    ];
    for (name, val, _) in themes {
        let rb = Radiobutton::new(&mut tk, top_bar, name, theme_var, val)?;
        tk.pack(rb, PackOpts { side: Side::Left, padx: 4.0, ..Default::default() });
    }

    theme_var.trace(&mut tk, move |tk| {
        if let Some(val) = theme_var.get(tk) {
            let t = match val {
                1 => Theme::Dark,
                2 => Theme::Fluent,
                3 => Theme::Material,
                4 => Theme::HighContrast,
                _ => Theme::Light,
            };
            tk.set_theme(t);
        }
    });

    // Content container: 4 side-by-side card frames
    let content = Frame::new(&mut tk, root)?;
    tk.pack(content, PackOpts { side: Side::Top, fill: rtk::Fill::Both, expand: true, padx: 8.0, pady: 4.0, ..Default::default() });

    let card_titles = ["Standard Controls", "Accent & Overrides", "Form Inputs", "State Showcase"];
    for (card_idx, card_title) in card_titles.iter().enumerate() {
        let card = Frame::new(&mut tk, content)?;
        tk.pack(card, PackOpts { side: Side::Left, fill: rtk::Fill::Both, expand: true, padx: 6.0, pady: 6.0, ..Default::default() });

        // Add subtle card borders and rounded corners via style override
        tk.config(card, Opt::StyleOverride(StyleSpec {
            border_width: Some(1.0),
            border_color: Some(Color::rgb(140, 140, 140)),
            radius: Some(8.0),
            padding: Some(rtk::Edges::all(8.0)),
            ..Default::default()
        }));

        let card_lbl = Label::new(&mut tk, card, *card_title)?;
        tk.pack(card_lbl, PackOpts { side: Side::Top, pady: 4.0, ..Default::default() });

        let btn = Button::new(&mut tk, card, "Standard Button")?;
        tk.pack(btn, PackOpts { side: Side::Top, pady: 3.0, fill: rtk::Fill::X, ..Default::default() });

        let accent_btn = Button::new(&mut tk, card, "Accent Action")?;
        tk.config(accent_btn, Opt::Style("Accent.TButton".into()));
        tk.pack(accent_btn, PackOpts { side: Side::Top, pady: 3.0, fill: rtk::Fill::X, ..Default::default() });

        let entry = Entry::new(&mut tk, card)?;
        tk.pack(entry, PackOpts { side: Side::Top, pady: 3.0, fill: rtk::Fill::X, ..Default::default() });
        tk.entry_set(entry, format!("Card {} text", card_idx + 1));

        let chk_var = tk.bool_var(card_idx % 2 == 0);
        let chk = Checkbutton::new(&mut tk, card, "Toggle Switch", chk_var)?;
        tk.pack(chk, PackOpts { side: Side::Top, pady: 2.0, ..Default::default() });

        let r_var = tk.int_var(1);
        let rb = Radiobutton::new(&mut tk, card, "Option Alpha", r_var, 1)?;
        tk.pack(rb, PackOpts { side: Side::Top, pady: 2.0, ..Default::default() });

        let sc_var = tk.int_var(30 + card_idx as i64 * 20);
        let scale = Scale::new(&mut tk, card, 0, 100, sc_var)?;
        tk.pack(scale, PackOpts { side: Side::Top, pady: 3.0, fill: rtk::Fill::X, ..Default::default() });

        let prog = ProgressBar::new(&mut tk, card, 120.0, sc_var)?;
        tk.pack(prog, PackOpts { side: Side::Top, pady: 4.0, fill: rtk::Fill::X, ..Default::default() });

        // Disabled button showcase on card 4
        if card_idx == 3 {
            let dis_btn = Button::new(&mut tk, card, "Disabled Button")?;
            tk.config(dis_btn, Opt::Disabled(true));
            tk.pack(dis_btn, PackOpts { side: Side::Top, pady: 3.0, fill: rtk::Fill::X, ..Default::default() });
        }
    }

    if let Some(path) = png_out {
        tk.save_png(path, 960, 600)?;
        return Ok(());
    }

    tk.mainloop()
}
