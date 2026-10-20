//! Headless screenshot generator for the PR #9 review.
//!
//! Renders the in-app file dialogs (open, save, folder) and the editable
//! combobox across the light, dark and high-contrast themes, writing one PNG
//! per scene into the directory given on the command line.
//!
//! Usage:
//!
//! ```text
//! cargo run --example dialog_shots -- DIR
//! ```
//!
//! Every written path is printed to stdout.

use std::error::Error;
use std::path::{Path, PathBuf};

use rtk::file_dialog::{FileDialog, InAppFileDialog};
use rtk::{Combobox, Fill, Key, Label, PackOpts, Theme, Tk};

/// The themes rendered by every scene.
const THEMES: [Theme; 3] = [Theme::Light, Theme::Dark, Theme::HighContrast];

/// The fruit values shown by the editable combobox scene.
const FRUITS: [&str; 8] = [
    "Apple",
    "Apricot",
    "Banana",
    "Blackberry",
    "Cherry",
    "Grape",
    "Mango",
    "Papaya",
];

/// Returns the lowercase slug used in file names for `theme`.
fn slug(theme: Theme) -> String {
    format!("{theme:?}").to_lowercase()
}

/// Returns the crate's `examples` directory as the dialogs' start directory.
fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
}

/// Creates a fresh 820×580 toolkit with `theme` and the in-app dialog backend.
fn new_dialog_tk(theme: Theme) -> Tk {
    let mut tk = Tk::new();
    tk.set_theme(theme);
    tk.geometry(820, 580);
    tk.sim_resize(820, 580);
    tk.set_file_dialog_backend(InAppFileDialog);
    tk
}

/// Renders the open-file dialog for `theme` and saves it into `dir`.
fn shot_open(dir: &Path, theme: Theme) -> Result<PathBuf, Box<dyn Error>> {
    let mut tk = new_dialog_tk(theme);
    let opts = FileDialog::new()
        .title("Open image")
        .directory(examples_dir())
        .add_filter("Rust source", &["rs"])
        .add_filter("All files", &["*"]);
    tk.open_file_dialog(&opts, |_tk, _path| {});
    let path = dir.join(format!("dialog_open_{}.png", slug(theme)));
    tk.save_png(&path, 820, 580)?;
    Ok(path)
}

/// Renders the save-file dialog for `theme` and saves it into `dir`.
fn shot_save(dir: &Path, theme: Theme) -> Result<PathBuf, Box<dyn Error>> {
    let mut tk = new_dialog_tk(theme);
    let opts = FileDialog::new()
        .title("Save report")
        .directory(examples_dir())
        .file_name("report.csv");
    tk.save_file_dialog(&opts, |_tk, _path| {});
    let path = dir.join(format!("dialog_save_{}.png", slug(theme)));
    tk.save_png(&path, 820, 580)?;
    Ok(path)
}

/// Renders the folder-picker dialog for `theme` and saves it into `dir`.
fn shot_folder(dir: &Path, theme: Theme) -> Result<PathBuf, Box<dyn Error>> {
    let mut tk = new_dialog_tk(theme);
    let opts = FileDialog::new()
        .title("Choose folder")
        .directory(examples_dir());
    tk.pick_folder_dialog(&opts, |_tk, _path| {});
    let path = dir.join(format!("dialog_folder_{}.png", slug(theme)));
    tk.save_png(&path, 820, 580)?;
    Ok(path)
}

/// Renders the open editable combobox for `theme` and saves it into `dir`.
fn shot_combobox(dir: &Path, theme: Theme) -> Result<PathBuf, Box<dyn Error>> {
    let mut tk = Tk::new();
    tk.set_theme(theme);
    let root = tk.root();
    let label = Label::new(&mut tk, root, "Fruit")?;
    tk.pack(label, PackOpts::default());
    let fruits: Vec<String> = FRUITS.iter().map(|s| (*s).to_string()).collect();
    let var = tk.int_var(-1);
    let combo = Combobox::editable(&mut tk, root, fruits, var)?;
    tk.pack(
        combo,
        PackOpts {
            fill: Fill::X,
            padx: 8.0,
            pady: 8.0,
            ..PackOpts::default()
        },
    );
    tk.sim_resize(320, 240);
    let rect = tk.widget_rect(combo).ok_or("combobox has no rect")?;
    tk.sim_click(rect.x + 10.0, rect.y + rect.height / 2.0);
    tk.sim_text("ap");
    tk.sim_key(Key::Down);
    tk.sim_key(Key::Down);
    let path = dir.join(format!("combobox_editable_{}.png", slug(theme)));
    tk.save_png(&path, 320, 240)?;
    Ok(path)
}

/// Writes every screenshot into `dir` and prints the resulting paths.
fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::args()
        .nth(1)
        .ok_or("usage: dialog_shots DIR")?;
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir)?;

    for theme in THEMES {
        let paths = [
            shot_open(&dir, theme)?,
            shot_save(&dir, theme)?,
            shot_folder(&dir, theme)?,
            shot_combobox(&dir, theme)?,
        ];
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}
