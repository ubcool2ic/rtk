use rtk::{Button, Label, PackOpts, Tk};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).map(std::path::PathBuf::from)
        .unwrap_or_else(|| "hello.png".into());
    let mut tk = Tk::new();
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Hello, rtk!");
    let button = Button::new(&mut tk, root, "Quit");
    let options = PackOpts { padx: 20.0, pady: 10.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(button, options);
    tk.save_png(&path, 320, 160)?;
    println!("Saved {}", path.display());
    Ok(())
}
