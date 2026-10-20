//! Example app demonstrating a simple counter with a label bound to a string variable and a button whose command increments an integer variable.

use rtk::{Button, Label, Opt, PackOpts, Tk};

/// Runs a counter app with a label bound to a variable and a button that increments it.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Counter");
    let root = tk.root();
    let count = tk.int_var(0);
    let text = tk.string_var("Count: 0");
    let label = Label::new(&mut tk, root, "")?;
    tk.config(label, Opt::TextVariable(text));
    let increment = Button::new(&mut tk, root, "+1")?;
    let options = PackOpts { padx: 16.0, pady: 10.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(increment, options);
    tk.set_command(increment, move |tk| {
        if let Some(value) = count.get(tk) {
            let next = value.saturating_add(1);
            count.set(tk, next);
            text.set(tk, format!("Count: {next}"));
        }
    });
    tk.mainloop()
}
