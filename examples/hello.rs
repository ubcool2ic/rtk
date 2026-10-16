use rtk::{Button, Label, PackOpts, Tk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tk = Tk::new();
    tk.title("Hello, rtk");
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Hello, rtk!");
    let quit = Button::new(&mut tk, root, "Quit");
    let options = PackOpts { padx: 20.0, pady: 10.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(quit, options);
    tk.set_command(quit, |tk| {
        println!("Goodbye!");
        tk.quit();
    });
    tk.mainloop()
}
