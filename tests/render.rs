use rtk::{Button, Fill, Frame, Label, PackOpts, Tk};

#[test]
fn hello_renders_without_a_window() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root);
    tk.pack(frame, PackOpts {
        fill: Fill::Both,
        expand: true,
        padx: 12.0,
        pady: 12.0,
        ..PackOpts::default()
    });
    let label = Label::new(&mut tk, frame, "Hello, rtk!");
    let button = Button::new(&mut tk, frame, "Quit");
    let options = PackOpts { pady: 8.0, ..PackOpts::default() };
    tk.pack(label, options);
    tk.pack(button, options);

    let pixmap = tk.render_to_pixmap(320, 160);
    assert_eq!(pixmap.width(), 320);
    assert_eq!(pixmap.height(), 160);
    assert_eq!(pixmap.data().len(), 320 * 160 * 4);
    let first = &pixmap.data()[..4];
    assert!(pixmap.data().chunks_exact(4).any(|pixel| pixel != first));
    assert!(!tk.is_dirty());

    if let Some(path) = std::env::var_os("RTK_PNG") {
        tk.save_png(std::path::PathBuf::from(path), 320, 160).unwrap();
    }
}
