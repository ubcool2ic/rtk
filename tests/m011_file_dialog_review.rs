//! Regression tests for the file dialog review findings (milestone 0.11).
//!
//! Covers the backend staying installed while a dialog runs (so callbacks may
//! open further dialogs or reset the backend) and compound extension filters.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rtk::file_dialog::{matches_filters, FileDialog, FileFilter, MockFileDialog};
use rtk::Tk;

/// A callback that opens a second dialog still uses the installed mock backend.
#[test]
fn nested_dialog_uses_same_backend() {
    let mut tk = Tk::new();
    let mock = MockFileDialog::new();
    mock.push_response(Some(vec![PathBuf::from("/first.txt")]));
    mock.push_response(Some(vec![PathBuf::from("/second.txt")]));
    tk.set_file_dialog_backend(mock.clone());

    let inner = Arc::new(Mutex::new(None));
    let inner_for_cb = inner.clone();
    tk.open_file_dialog(&FileDialog::new(), move |t, _p| {
        t.open_file_dialog(&FileDialog::new(), move |_t, p| {
            *inner_for_cb.lock().unwrap() = p;
        });
    });

    assert_eq!(
        inner.lock().unwrap().as_deref(),
        Some(Path::new("/second.txt"))
    );
    assert_eq!(mock.request_count(), 2);
}

/// A `reset_file_dialog_backend` called inside a callback is not undone.
#[test]
fn reset_inside_callback_sticks() {
    let mut tk = Tk::new();
    let mock = MockFileDialog::new();
    mock.push_response(Some(vec![PathBuf::from("/a.txt")]));
    tk.set_file_dialog_backend(mock.clone());

    tk.open_file_dialog(&FileDialog::new(), |t, _p| {
        t.reset_file_dialog_backend();
    });

    // The mock is gone: a further request must not reach it.
    let got = Arc::new(Mutex::new(Some(PathBuf::from("/x"))));
    let g = got.clone();
    tk.open_file_dialog(&FileDialog::new(), move |_t, p| *g.lock().unwrap() = p);
    assert_eq!(mock.request_count(), 1);
    // Without the native feature the default in-app chooser opened a modal.
    #[cfg(not(feature = "native-dialogs"))]
    {
        assert!(rtk::dialog::is_open(&tk));
        assert!(got.lock().unwrap().is_some());
    }
}

/// A `"tar.gz"` filter matches compound extensions case-insensitively.
#[test]
fn compound_extension_filter() {
    let filters = vec![FileFilter::new("Archive", &["tar.gz"])];
    assert!(matches_filters(Path::new("/tmp/a.tar.gz"), &filters));
    assert!(matches_filters(Path::new("/tmp/A.TAR.GZ"), &filters));
    assert!(!matches_filters(Path::new("/tmp/a.gz"), &filters));
    assert!(!matches_filters(Path::new("/tmp/a.tar"), &filters));
    assert!(!matches_filters(Path::new("/tmp/a.tar.bz2"), &filters));
}

/// A plain extension filter still matches through the file-name suffix path.
#[test]
fn plain_extension_filter_still_matches() {
    let filters = vec![FileFilter::new("Text", &["txt"])];
    assert!(matches_filters(Path::new("a.txt"), &filters));
    assert!(matches_filters(Path::new("a.TXT"), &filters));
    assert!(!matches_filters(Path::new("a.md"), &filters));
}
