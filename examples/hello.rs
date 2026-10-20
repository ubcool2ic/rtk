//! Minimal hello-world example with a label and a quit button wired to a command that exits the main loop.

use rtk::{Button, Label, PackOpts, Tk};

/// Runs a hello-world window with a label and a quit button that prints a farewell and exits.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Hello, rtk");
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Hello, rtk!")?;
    let quit = Button::new(&mut tk, root, "Quit")?;
    let options = PackOpts { padx: 20.0, pady: 10.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(quit, options);
    tk.set_command(quit, |tk| {
        println!("Goodbye!");
        tk.quit();
    });
    tk.mainloop()
}
