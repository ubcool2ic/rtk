//! Example showing a checkbutton and scale bound to variables, with labels reflecting their state and a quit button.

use rtk::{Button, Checkbutton, Label, Opt, PackOpts, Scale, Tk};

/// Runs a checkbutton and scale example that updates status labels from bound variables.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Checkbutton and Scale");
    tk.geometry(360, 220);
    let root = tk.root();
    let enabled = tk.bool_var(true);
    let level = tk.int_var(25);
    let status = tk.string_var("Enabled");
    let check = Checkbutton::new(&mut tk, root, "Enabled", enabled)?;
    let scale = Scale::new(&mut tk, root, 0, 100, level)?;
    let caption = Label::new(&mut tk, root, "")?;
    let number = Label::new(&mut tk, root, "")?;
    let quit = Button::new(&mut tk, root, "Quit")?;
    tk.config(caption, Opt::TextVariable(status));
    tk.config(number, Opt::TextVariableInt(level));
    enabled.trace(&mut tk, move |tk| {
        let text = if enabled.get(tk).unwrap_or(false) { "Enabled" } else { "Disabled" };
        status.set(tk, text.into());
    });
    tk.set_command(quit, Tk::quit);
    for widget in [check, scale, caption, number, quit] {
        tk.pack(widget, PackOpts { pady: 5.0, ..PackOpts::default() });
    }
    tk.mainloop()
}
