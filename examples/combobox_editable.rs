//! Runnable example for editable comboboxes: a filter-as-you-type country
//! picker with a status label, plus a read-only combobox that switches the
//! active theme.
//!
//! Keys: type to filter the dropdown, Up/Down to move the highlight, Enter to
//! pick the highlighted value, Escape to close the dropdown.
//!
//! Run with `cargo run --example combobox_editable`.

use std::error::Error;

use rtk::{Combobox, Frame, Label, Opt, PackOpts, Side, Theme, Tk};

/// The country names offered by the editable combobox.
const COUNTRIES: &[&str] = &[
    "Argentina",
    "Australia",
    "Austria",
    "Belgium",
    "Brazil",
    "Canada",
    "Chile",
    "China",
    "Denmark",
    "Egypt",
    "Finland",
    "France",
    "Germany",
    "Greece",
    "India",
    "Ireland",
    "Italy",
    "Japan",
    "Kenya",
    "Mexico",
];

/// Builds the editable country combobox and wires its callbacks to `status`.
fn build_country_picker(tk: &mut Tk, parent: rtk::WidgetId, status: rtk::WidgetId) {
    let label = Label::new(tk, parent, "Country").expect("country label");
    tk.pack(
        label,
        PackOpts {
            side: Side::Top,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );

    let values: Vec<String> = COUNTRIES.iter().map(|s| (*s).to_string()).collect();
    let var = tk.int_var(-1);
    let combo = Combobox::editable(tk, parent, values, var).expect("country combobox");
    tk.pack(
        combo,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );

    tk.combobox_on_select(combo, move |tk, index, text| {
        let msg = format!("Selected: {text} (#{index})");
        tk.config(status, Opt::Text(msg));
    });
    tk.combobox_on_text_change(combo, move |tk, text| {
        let n = tk.combobox_filtered(combo).len();
        let msg = format!("Typing: {text} — {n} matches");
        tk.config(status, Opt::Text(msg));
    });
}

/// Builds the read-only theme combobox that switches the active theme.
fn build_theme_picker(tk: &mut Tk, parent: rtk::WidgetId) {
    let label = Label::new(tk, parent, "Theme").expect("theme label");
    tk.pack(
        label,
        PackOpts {
            side: Side::Top,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );

    let values = vec![
        "Light".to_string(),
        "Dark".to_string(),
        "HighContrast".to_string(),
    ];
    let var = tk.int_var(0);
    let combo = Combobox::new(tk, parent, values, var).expect("theme combobox");
    tk.pack(
        combo,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );

    tk.combobox_on_select(combo, move |tk, index, _text| {
        let theme = match index {
            1 => Theme::Dark,
            2 => Theme::HighContrast,
            _ => Theme::Light,
        };
        tk.set_theme(theme);
    });
}

/// Entry point: builds the window and runs the event loop.
fn main() -> Result<(), Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.title("rtk editable combobox");
    tk.geometry(420, 320);
    let root = tk.root();

    let body = Frame::new(&mut tk, root)?;
    tk.pack(
        body,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            ..PackOpts::default()
        },
    );

    let status = Label::new(&mut tk, root, "Type to filter countries")?;
    tk.pack(
        status,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );

    build_country_picker(&mut tk, body, status);
    build_theme_picker(&mut tk, body);

    tk.mainloop()
}
