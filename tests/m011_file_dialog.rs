//! Headless tests for the file dialog abstraction (milestone 0.11).
//!
//! These never open a real dialog: the mock backend scripts every response and
//! the in-app backend is driven through the existing modal simulation helpers.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rtk::file_dialog::{
    matches_filters, FileDialog, FileDialogBackend, FileDialogKind, FileFilter,
    InAppFileDialog, MockFileDialog,
};
use rtk::{Key, Tk};

/// The builder records every field and strips dots from extensions.
#[test]
fn builder_and_filter_fields() {
    let opts = FileDialog::new()
        .title("Choose")
        .directory("/tmp")
        .file_name("a.txt")
        .add_filter("Text", &[".txt", "md"]);
    assert_eq!(opts.title.as_deref(), Some("Choose"));
    assert_eq!(opts.directory.as_deref(), Some(Path::new("/tmp")));
    assert_eq!(opts.file_name.as_deref(), Some("a.txt"));
    assert_eq!(opts.filters.len(), 1);
    assert_eq!(opts.filters[0].name, "Text");
    assert_eq!(opts.filters[0].extensions, vec!["txt", "md"]);
}

/// `matches_filters` handles empty, star, case-insensitive and miss cases.
#[test]
fn matches_filters_cases() {
    assert!(matches_filters(Path::new("x.bin"), &[]));
    let star = vec![FileFilter::new("All", &["*"])];
    assert!(matches_filters(Path::new("x.bin"), &star));
    let imgs = vec![FileFilter::new("Images", &["png", "JPG"])];
    assert!(matches_filters(Path::new("a.PNG"), &imgs));
    assert!(matches_filters(Path::new("a.jpg"), &imgs));
    assert!(!matches_filters(Path::new("a.gif"), &imgs));
    assert!(!matches_filters(Path::new("noext"), &imgs));
}

/// The mock answers each convenience method in FIFO order and records requests.
#[test]
fn mock_answers_all_kinds_in_order() {
    let mut tk = Tk::new();
    let mock = MockFileDialog::new();
    mock.push_response(Some(vec![PathBuf::from("/a.txt")]));
    mock.push_response(Some(vec![PathBuf::from("/b.txt"), PathBuf::from("/c.txt")]));
    mock.push_response(Some(vec![PathBuf::from("/save.txt")]));
    mock.push_response(Some(vec![PathBuf::from("/dir")]));
    tk.set_file_dialog_backend(mock.clone());

    let opts = FileDialog::new().title("T").directory("/tmp");

    let got_open = Arc::new(Mutex::new(None));
    let g = got_open.clone();
    tk.open_file_dialog(&opts, move |_t, p| *g.lock().unwrap() = p);
    assert_eq!(got_open.lock().unwrap().as_deref(), Some(Path::new("/a.txt")));

    let got_many = Arc::new(Mutex::new(None));
    let g = got_many.clone();
    tk.open_files_dialog(&opts, move |_t, p| *g.lock().unwrap() = p);
    assert_eq!(
        got_many.lock().unwrap().as_deref(),
        Some(&[PathBuf::from("/b.txt"), PathBuf::from("/c.txt")][..])
    );

    let got_save = Arc::new(Mutex::new(None));
    let g = got_save.clone();
    tk.save_file_dialog(&opts, move |_t, p| *g.lock().unwrap() = p);
    assert_eq!(got_save.lock().unwrap().as_deref(), Some(Path::new("/save.txt")));

    let got_dir = Arc::new(Mutex::new(None));
    let g = got_dir.clone();
    tk.pick_folder_dialog(&opts, move |_t, p| *g.lock().unwrap() = p);
    assert_eq!(got_dir.lock().unwrap().as_deref(), Some(Path::new("/dir")));

    let reqs = mock.requests();
    assert_eq!(reqs.len(), 4);
    assert_eq!(reqs[0].0, FileDialogKind::OpenFile);
    assert_eq!(reqs[1].0, FileDialogKind::OpenFiles);
    assert_eq!(reqs[2].0, FileDialogKind::SaveFile);
    assert_eq!(reqs[3].0, FileDialogKind::PickFolder);
    for (_, o) in &reqs {
        assert_eq!(o, &opts);
    }
}

/// An empty response queue answers `None`.
#[test]
fn mock_empty_queue_returns_none() {
    let mut tk = Tk::new();
    let mock = MockFileDialog::new();
    tk.set_file_dialog_backend(mock.clone());
    let got = Arc::new(Mutex::new(Some(PathBuf::from("/x"))));
    let g = got.clone();
    tk.open_file_dialog(&FileDialog::new(), move |_t, p| *g.lock().unwrap() = p);
    assert!(got.lock().unwrap().is_none());
    assert_eq!(mock.request_count(), 1);
}

/// An empty vector result is normalised to `None`.
#[test]
fn empty_vec_normalises_to_none() {
    let mut tk = Tk::new();
    let mock = MockFileDialog::new();
    mock.push_response(Some(Vec::new()));
    tk.set_file_dialog_backend(mock.clone());
    let got = Arc::new(Mutex::new(Some(vec![PathBuf::from("/x")])));
    let g = got.clone();
    tk.open_files_dialog(&FileDialog::new(), move |_t, p| *g.lock().unwrap() = p);
    assert!(got.lock().unwrap().is_none());
}

/// A backend installed from inside a callback survives the outer call.
#[test]
fn backend_installed_in_callback_survives() {
    let mut tk = Tk::new();
    let first = MockFileDialog::new();
    first.push_response(None);
    tk.set_file_dialog_backend(first.clone());

    let second = MockFileDialog::new();
    second.push_response(Some(vec![PathBuf::from("/second.txt")]));
    let second_for_cb = second.clone();
    tk.open_file_dialog(&FileDialog::new(), move |t, _p| {
        t.set_file_dialog_backend(second_for_cb.clone());
    });

    // The newly installed backend is now in use.
    let got = Arc::new(Mutex::new(None));
    let g = got.clone();
    tk.open_file_dialog(&FileDialog::new(), move |_t, p| *g.lock().unwrap() = p);
    assert_eq!(got.lock().unwrap().as_deref(), Some(Path::new("/second.txt")));
    assert_eq!(second.request_count(), 1);
    assert_eq!(first.request_count(), 1);
}

/// Without the native feature the default backend is the in-app chooser.
#[cfg(not(feature = "native-dialogs"))]
#[test]
fn default_backend_is_in_app() {
    let tk = Tk::new();
    assert!(!tk.file_dialog_uses_native());
}

/// The in-app open dialog opens a modal; Escape cancels with `None`.
#[cfg(not(feature = "native-dialogs"))]
#[test]
fn in_app_open_cancels_with_escape() {
    let mut tk = Tk::new();
    let fired = Arc::new(AtomicBool::new(false));
    let got = Arc::new(Mutex::new(Some(PathBuf::from("/x"))));
    let f = fired.clone();
    let g = got.clone();
    tk.open_file_dialog(&FileDialog::new(), move |_t, p| {
        *g.lock().unwrap() = p;
        f.store(true, Ordering::SeqCst);
    });
    assert!(rtk::dialog::is_open(&tk));
    tk.sim_key(Key::Escape);
    assert!(!rtk::dialog::is_open(&tk));
    assert!(fired.load(Ordering::SeqCst));
    assert!(got.lock().unwrap().is_none());
}

/// The in-app folder dialog opens a modal; Escape cancels with `None`.
#[cfg(not(feature = "native-dialogs"))]
#[test]
fn in_app_folder_cancels_with_escape() {
    let mut tk = Tk::new();
    let fired = Arc::new(AtomicBool::new(false));
    let got = Arc::new(Mutex::new(Some(PathBuf::from("/x"))));
    let f = fired.clone();
    let g = got.clone();
    tk.pick_folder_dialog(&FileDialog::new(), move |_t, p| {
        *g.lock().unwrap() = p;
        f.store(true, Ordering::SeqCst);
    });
    assert!(rtk::dialog::is_open(&tk));
    tk.sim_key(Key::Escape);
    assert!(!rtk::dialog::is_open(&tk));
    assert!(fired.load(Ordering::SeqCst));
    assert!(got.lock().unwrap().is_none());
}

/// The in-app save dialog opens a modal; Escape cancels with `None`.
#[cfg(not(feature = "native-dialogs"))]
#[test]
fn in_app_save_cancels_with_escape() {
    let mut tk = Tk::new();
    let fired = Arc::new(AtomicBool::new(false));
    let got = Arc::new(Mutex::new(Some(PathBuf::from("/x"))));
    let f = fired.clone();
    let g = got.clone();
    tk.save_file_dialog(&FileDialog::new().file_name("out.txt"), move |_t, p| {
        *g.lock().unwrap() = p;
        f.store(true, Ordering::SeqCst);
    });
    assert!(rtk::dialog::is_open(&tk));
    tk.sim_key(Key::Escape);
    assert!(!rtk::dialog::is_open(&tk));
    assert!(fired.load(Ordering::SeqCst));
    assert!(got.lock().unwrap().is_none());
}

/// A custom backend can be reset back to the default.
#[test]
fn reset_restores_default() {
    let mut tk = Tk::new();
    tk.set_file_dialog_backend(MockFileDialog::new());
    assert!(!tk.file_dialog_uses_native());
    tk.reset_file_dialog_backend();
    assert_eq!(tk.file_dialog_uses_native(), cfg!(feature = "native-dialogs"));
}

/// The in-app backend can be used directly through the trait object.
#[test]
fn in_app_backend_direct_use() {
    let mut tk = Tk::new();
    let mut backend = InAppFileDialog;
    let fired = Arc::new(AtomicBool::new(false));
    let f = fired.clone();
    let done: rtk::file_dialog::FileDialogDone = Box::new(move |_t, r| {
        assert!(r.is_none());
        f.store(true, Ordering::SeqCst);
    });
    backend.show(&mut tk, FileDialogKind::OpenFile, &FileDialog::new(), done);
    assert!(rtk::dialog::is_open(&tk));
    tk.sim_key(Key::Escape);
    assert!(fired.load(Ordering::SeqCst));
}
