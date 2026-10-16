use rtk::{Button, Entry, GridOpts, Label, Sticky, Tk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Form");
    tk.geometry(420, 160);
    let root = tk.root();
    let name_label = Label::new(&mut tk, root, "Name")?;
    let email_label = Label::new(&mut tk, root, "Email")?;
    let name = Entry::new(&mut tk, root)?;
    let email = Entry::new(&mut tk, root)?;
    for (row, label, entry) in [(0, name_label, name), (1, email_label, email)] {
        tk.grid(label, GridOpts {
            row,
            column: 0,
            sticky: Sticky::E,
            padx: 8.0,
            pady: 8.0,
            ..GridOpts::default()
        });
        tk.grid(entry, GridOpts {
            row,
            column: 1,
            sticky: Sticky::EW,
            padx: 8.0,
            pady: 8.0,
            ..GridOpts::default()
        });
    }
    let submit = Button::new(&mut tk, root, "Submit")?;
    tk.grid(submit, GridOpts {
        row: 2,
        columnspan: 2,
        pady: 8.0,
        ..GridOpts::default()
    });
    tk.grid_columnconfigure(root, 1, 1);
    tk.set_command(submit, move |tk| {
        println!(
            "Name: {}\nEmail: {}",
            tk.entry_get(name).unwrap_or_default(),
            tk.entry_get(email).unwrap_or_default(),
        );
    });
    tk.mainloop()
}
