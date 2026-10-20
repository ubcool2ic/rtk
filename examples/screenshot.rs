//! Example that builds a small window and renders it directly to a PNG file without opening a real window.

use rtk::{Button, Label, PackOpts, Tk};

/// Renders a hello window to a PNG file at the given path (default hello.png) and prints the saved location.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).map(std::path::PathBuf::from)
        .unwrap_or_else(|| "hello.png".into());
    let mut tk = Tk::new();
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Hello, rtk!")?;
    let button = Button::new(&mut tk, root, "Quit")?;
    let options = PackOpts { padx: 20.0, pady: 10.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(button, options);
    tk.save_png(&path, 320, 160)?;
    println!("Saved {}", path.display());
    Ok(())
}
