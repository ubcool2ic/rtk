//! Runnable example for the file dialog backends: four buttons that open the
//! open-file, open-files, save-file and pick-folder dialogs, a label showing
//! the chosen path(s) and a label naming the active backend.
//!
//! Run with `cargo run --example file_dialogs`.
//! Run with native platform dialogs via
//! `cargo run --example file_dialogs --features native-dialogs`.

use std::error::Error;
use std::path::PathBuf;

use rtk::{Button, FileDialog, Frame, Label, Opt, PackOpts, Side, Tk};

/// Formats a dialog result for the status label: `"Cancelled"` for `None`,
/// otherwise the paths joined by `", "`.
fn describe(result: Option<Vec<PathBuf>>) -> String {
    match result {
        None => "Cancelled".to_string(),
        Some(paths) if paths.is_empty() => "Cancelled".to_string(),
        Some(paths) => paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Builds the row of four dialog buttons under `parent`.
fn build_buttons(tk: &mut Tk, parent: rtk::WidgetId, status: rtk::WidgetId) {
    let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let open_file = Button::new(tk, parent, "Open file…").expect("open file button");
    tk.pack(
        open_file,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    let open_file_dir = dir.clone();
    tk.set_command(open_file, move |tk| {
        let opts = FileDialog::new()
            .title("Open file")
            .directory(open_file_dir.clone())
            .add_filter("Rust sources", &["rs"])
            .add_filter("All files", &["*"]);
        tk.file_dialog_open(&opts, move |tk, path| {
            let text = describe(path.map(|p| vec![p]));
            tk.config(status, Opt::Text(text));
        });
    });

    let open_files = Button::new(tk, parent, "Open files…").expect("open files button");
    tk.pack(
        open_files,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    let open_files_dir = dir.clone();
    tk.set_command(open_files, move |tk| {
        let opts = FileDialog::new()
            .title("Open files")
            .directory(open_files_dir.clone())
            .add_filter("Rust sources", &["rs"])
            .add_filter("All files", &["*"]);
        tk.file_dialog_open_many(&opts, move |tk, paths| {
            let text = describe(paths);
            tk.config(status, Opt::Text(text));
        });
    });

    let save_as = Button::new(tk, parent, "Save as…").expect("save as button");
    tk.pack(
        save_as,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    let save_as_dir = dir.clone();
    tk.set_command(save_as, move |tk| {
        let opts = FileDialog::new()
            .title("Save as")
            .directory(save_as_dir.clone())
            .file_name("untitled.txt")
            .add_filter("Rust sources", &["rs"])
            .add_filter("All files", &["*"]);
        tk.file_dialog_save(&opts, move |tk, path| {
            let text = describe(path.map(|p| vec![p]));
            tk.config(status, Opt::Text(text));
        });
    });

    let pick_folder = Button::new(tk, parent, "Pick folder…").expect("pick folder button");
    tk.pack(
        pick_folder,
        PackOpts {
            side: Side::Left,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );
    let pick_folder_dir = dir;
    tk.set_command(pick_folder, move |tk| {
        let opts = FileDialog::new()
            .title("Pick folder")
            .directory(pick_folder_dir.clone());
        tk.file_dialog_pick_folder(&opts, move |tk, path| {
            let text = describe(path.map(|p| vec![p]));
            tk.config(status, Opt::Text(text));
        });
    });
}

/// Entry point: builds the window and runs the event loop.
fn main() -> Result<(), Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.title("rtk file dialogs");
    tk.geometry(520, 260);
    let root = tk.root();

    let buttons = Frame::new(&mut tk, root)?;
    tk.pack(
        buttons,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            ..PackOpts::default()
        },
    );

    let status = Label::new(&mut tk, root, "No selection yet")?;
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

    build_buttons(&mut tk, buttons, status);

    let backend_text = if tk.file_dialog_uses_native() {
        "Backend: native (rfd)"
    } else {
        "Backend: in-app (build with --features native-dialogs for native)"
    };
    let backend = Label::new(&mut tk, root, backend_text)?;
    tk.pack(
        backend,
        PackOpts {
            side: Side::Top,
            fill: rtk::Fill::X,
            padx: 6.0,
            pady: 4.0,
            ..PackOpts::default()
        },
    );

    tk.mainloop()
}
