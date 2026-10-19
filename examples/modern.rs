//! Modern themed GUI showcase demonstrating cards, chips, sidebar, icons,
//! threading with TkProxy, dialogs, and software rendering.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rtk::dialog;
use rtk::icons::{self, Icon};
use rtk::{
    Button, Color, Combobox, Compound, Fill, Frame, GridOpts, Label, Notebook, Opt, PackOpts,
    ProgressBar, Side, Text, Theme, Tk,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut screenshot_path: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            screenshot_path = Some(PathBuf::from(&args[i + 1]));
            i += 2;
            continue;
        }
        i += 1;
    }

    let mut tk = Tk::new();
    tk.title("rtk modern dashboard");
    tk.geometry(960, 620);
    tk.set_theme(Theme::ModernDark);

    let root = tk.root();

    // -----------------------------------------------------------------------
    // Layout: Left Sidebar + Right Content Area
    // -----------------------------------------------------------------------
    let sidebar = Frame::new(&mut tk, root)?;
    tk.config(sidebar, Opt::Style("Sidebar".into()));
    let mut side_pack = PackOpts::default();
    side_pack.side = Side::Left;
    side_pack.fill = Fill::Y;
    side_pack.padx = 8.0;
    side_pack.pady = 8.0;
    tk.pack(sidebar, side_pack);

    let title_icon = icons::icon(&mut tk, Icon::Bolt, 20, Color::rgb(79, 140, 255));
    let brand = Label::new(&mut tk, sidebar, " RTK Core")?;
    tk.config(brand, Opt::Style("Heading".into()));
    if let Some(img) = title_icon {
        tk.config(brand, Opt::Image(img));
        tk.config(brand, Opt::Compound(Compound::Left));
    }
    let mut p = PackOpts::default();
    p.side = Side::Top;
    p.pady = 12.0;
    tk.pack(brand, p);

    let sidebar_items = [
        (Icon::Home, "Dashboard"),
        (Icon::Chip, "Hardware"),
        (Icon::Memory, "Telemetry"),
        (Icon::Settings, "Settings"),
    ];

    let mut sidebar_btns = Vec::new();
    for (idx, (ic, label)) in sidebar_items.iter().enumerate() {
        let btn = Button::new(&mut tk, sidebar, *label)?;
        tk.config(btn, Opt::Style("Sidebar.TButton".into()));
        let img = icons::icon(&mut tk, *ic, 18, Color::rgb(230, 232, 238));
        if let Some(i) = img {
            tk.config(btn, Opt::Image(i));
            tk.config(btn, Opt::Compound(Compound::Left));
        }
        let mut bp = PackOpts::default();
        bp.side = Side::Top;
        bp.fill = Fill::X;
        bp.pady = 3.0;
        tk.pack(btn, bp);
        if idx == 0 {
            tk.set_selected(btn, true);
        }
        sidebar_btns.push(btn);
    }

    let main_area = Frame::new(&mut tk, root)?;
    let mut main_pack = PackOpts::default();
    main_pack.side = Side::Right;
    main_pack.fill = Fill::Both;
    main_pack.expand = true;
    main_pack.padx = 12.0;
    main_pack.pady = 12.0;
    tk.pack(main_area, main_pack);

    // Header with theme toggle
    let header = Frame::new(&mut tk, main_area)?;
    let mut hp = PackOpts::default();
    hp.side = Side::Top;
    hp.fill = Fill::X;
    hp.pady = 4.0;
    tk.pack(header, hp);

    let head_lbl = Label::new(&mut tk, header, "Overview")?;
    tk.config(head_lbl, Opt::Style("Heading".into()));
    let mut lp = PackOpts::default();
    lp.side = Side::Left;
    tk.pack(head_lbl, lp);

    let theme_btn = Button::new(&mut tk, header, " Toggle Theme")?;
    tk.config(theme_btn, Opt::Style("Ghost.TButton".into()));
    let sun_icon = icons::icon(&mut tk, Icon::Sun, 16, Color::rgb(241, 196, 15));
    if let Some(img) = sun_icon {
        tk.config(theme_btn, Opt::Image(img));
        tk.config(theme_btn, Opt::Compound(Compound::Left));
    }
    let mut tp = PackOpts::default();
    tp.side = Side::Right;
    tk.pack(theme_btn, tp);

    tk.set_command(theme_btn, |t| {
        if t.theme() == Theme::ModernDark {
            t.set_theme(Theme::ModernLight);
        } else {
            t.set_theme(Theme::ModernDark);
        }
    });

    // Top Cards grid: Status Card & Chips Card
    let cards_row = Frame::new(&mut tk, main_area)?;
    let mut crp = PackOpts::default();
    crp.side = Side::Top;
    crp.fill = Fill::X;
    crp.pady = 8.0;
    tk.pack(cards_row, crp);

    let card1 = Frame::new(&mut tk, cards_row)?;
    tk.config(card1, Opt::Style("Card".into()));
    let mut c1p = GridOpts::default();
    c1p.column = 0;
    c1p.row = 0;
    c1p.sticky = rtk::Sticky::NSEW;
    c1p.padx = 6.0;
    tk.grid(card1, c1p);

    let c1_title = Label::new(&mut tk, card1, "System Health")?;
    tk.config(c1_title, Opt::Style("Subheading".into()));
    tk.pack(c1_title, PackOpts::default());

    let chips_box = Frame::new(&mut tk, card1)?;
    let mut chip_pack = PackOpts::default();
    chip_pack.side = Side::Top;
    chip_pack.pady = 6.0;
    tk.pack(chips_box, chip_pack);

    let chip1 = Label::new(&mut tk, chips_box, "ONLINE")?;
    tk.config(chip1, Opt::Style("Chip.Success".into()));
    let mut cp = PackOpts::default();
    cp.side = Side::Left;
    cp.padx = 4.0;
    tk.pack(chip1, cp);

    let chip2 = Label::new(&mut tk, chips_box, "SYNCED")?;
    tk.config(chip2, Opt::Style("Chip.Info".into()));
    tk.pack(chip2, cp);

    let chip3 = Label::new(&mut tk, chips_box, "42°C")?;
    tk.config(chip3, Opt::Style("Chip.Neutral".into()));
    tk.pack(chip3, cp);

    let card2 = Frame::new(&mut tk, cards_row)?;
    tk.config(card2, Opt::Style("Card".into()));
    let mut c2p = GridOpts::default();
    c2p.column = 1;
    c2p.row = 0;
    c2p.sticky = rtk::Sticky::NSEW;
    c2p.padx = 6.0;
    tk.grid(card2, c2p);

    let c2_title = Label::new(&mut tk, card2, "Actions")?;
    tk.config(c2_title, Opt::Style("Subheading".into()));
    tk.pack(c2_title, PackOpts::default());

    let prompt_btn = Button::new(&mut tk, card2, "Prompt User")?;
    tk.config(prompt_btn, Opt::Style("Accent.TButton".into()));
    let mut pbp = PackOpts::default();
    pbp.side = Side::Top;
    pbp.pady = 4.0;
    tk.pack(prompt_btn, pbp);

    tk.set_command(prompt_btn, |t| {
        dialog::ask_string(t, "Configure Device", "Set node alias:", "Worker-1", |t, res| {
            if let Some(alias) = res {
                dialog::message_box(t, "Saved", &format!("Device updated: {alias}"), dialog::MessageKind::Info, |_| {});
            }
        });
    });

    tk.grid_columnconfigure(cards_row, 0, 1);
    tk.grid_columnconfigure(cards_row, 1, 1);

    // Notebook for Tabs
    let notebook = Notebook::new(&mut tk, main_area)?;
    let mut np = PackOpts::default();
    np.side = Side::Top;
    np.fill = Fill::Both;
    np.expand = true;
    np.pady = 8.0;
    tk.pack(notebook, np);

    let page_log = Frame::new(&mut tk, notebook)?;
    tk.notebook_add(notebook, page_log, "Log Output");

    let log_text = Text::new(&mut tk, page_log, 60, 8)?;
    tk.config(log_text, Opt::Monospace(true));
    let mut text_pack = PackOpts::default();
    text_pack.side = Side::Top;
    text_pack.fill = Fill::Both;
    text_pack.expand = true;
    text_pack.pady = 4.0;
    tk.pack(log_text, text_pack);
    tk.text_insert_end(log_text, "[system] initialization completed\n[network] connected to gateway\n");

    let page_controls = Frame::new(&mut tk, notebook)?;
    tk.notebook_add(notebook, page_controls, "Parameters");

    let combo_var = tk.int_var(0);
    let combo = Combobox::new(
        &mut tk,
        page_controls,
        vec!["Balanced Profile".into(), "Performance Mode".into(), "Power Saver".into()],
        combo_var,
    )?;
    let mut comb_pack = PackOpts::default();
    comb_pack.side = Side::Top;
    comb_pack.pady = 8.0;
    tk.pack(combo, comb_pack);

    // Background thread feeding progress bar via TkProxy
    let progress_var = tk.int_var(0);
    let pbar = ProgressBar::new(&mut tk, main_area, 300.0, progress_var)?;
    let mut pbar_pack = PackOpts::default();
    pbar_pack.side = Side::Bottom;
    pbar_pack.fill = Fill::X;
    pbar_pack.pady = 6.0;
    tk.pack(pbar, pbar_pack);

    let proxy = tk.proxy();
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();

    thread::spawn(move || {
        let mut val = 0;
        while r_clone.load(Ordering::Relaxed) && val <= 100 {
            thread::sleep(Duration::from_millis(60));
            val = (val + 2) % 102;
            let p = proxy.clone();
            p.post(move |t| {
                progress_var.set(t, val);
            });
        }
    });

    if let Some(path) = screenshot_path {
        running.store(false, Ordering::Relaxed);
        tk.save_png(path, 960, 620)?;
        return Ok(());
    }

    tk.mainloop()
}
