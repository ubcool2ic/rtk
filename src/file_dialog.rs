//! Native and in-app file dialogs behind a small, test-friendly backend abstraction.
//!
//! The [`FileDialog`] builder describes a request (title, start directory,
//! default file name and extension filters). A [`FileDialogBackend`] turns that
//! request into a user interaction and reports the outcome through a callback,
//! so both blocking native dialogs and the existing non-blocking in-app modal
//! dialogs (`crate::dialog`) fit the same shape.
//!
//! Three backends ship with the crate:
//!
//! * [`InAppFileDialog`] — the default; reuses the modal choosers in
//!   [`crate::dialog`] and works headlessly.
//! * [`NativeFileDialog`] — available with the `native-dialogs` cargo feature;
//!   wraps [`rfd`] and uses the platform's own dialogs.
//! * [`MockFileDialog`] — a scripted backend for tests.
//!
//! # Example
//!
//! ```no_run
//! use rtk::file_dialog::{FileDialog, MockFileDialog};
//! use rtk::Tk;
//!
//! let mut tk = Tk::new();
//! let mock = MockFileDialog::new();
//! mock.push_response(Some(vec!["/tmp/a.txt".into()]));
//! tk.set_file_dialog_backend(mock.clone());
//!
//! let opts = FileDialog::new().title("Open").add_filter("Text", &[".txt"]);
//! tk.open_file_dialog(&opts, |_tk, path| {
//!     assert_eq!(path.as_deref(), Some(std::path::Path::new("/tmp/a.txt")));
//! });
//! ```

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::dialog;
use crate::Tk;

/// A named extension filter for a file dialog.
///
/// Extensions are stored without a leading dot; `FileFilter::new("Text",
/// &[".txt", "md"])` yields `["txt", "md"]`. Construct one with
/// [`FileFilter::new`]; the struct is `#[non_exhaustive]` so fields may be
/// added without a breaking change.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFilter {
    /// Human-readable filter name, e.g. `"Images"`.
    pub name: String,
    /// Extensions without a leading dot, e.g. `["png", "jpg"]`.
    pub extensions: Vec<String>,
}

impl FileFilter {
    /// Creates a filter from a display name and a list of extensions.
    ///
    /// A single leading `.` is stripped from each extension so callers may
    /// write either `"txt"` or `".txt"`.
    #[must_use]
    pub fn new(name: impl Into<String>, extensions: &[&str]) -> Self {
        Self {
            name: name.into(),
            extensions: extensions
                .iter()
                .map(|ext| ext.strip_prefix('.').unwrap_or(ext).to_string())
                .collect(),
        }
    }
}

/// Which kind of file dialog to present.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDialogKind {
    /// Choose a single existing file.
    OpenFile,
    /// Choose one or more existing files.
    OpenFiles,
    /// Choose a destination path for saving.
    SaveFile,
    /// Choose a directory.
    PickFolder,
}

impl FileDialogKind {
    /// Returns the default dialog title used when [`FileDialog::title`] is unset.
    fn default_title(self) -> &'static str {
        match self {
            Self::OpenFile => "Open File",
            Self::OpenFiles => "Open Files",
            Self::SaveFile => "Save File",
            Self::PickFolder => "Select Folder",
        }
    }
}

/// A description of a file dialog request.
///
/// Build one with [`FileDialog::new`] and the chainable setters. The struct is
/// `#[non_exhaustive]` so fields may be added without a breaking change.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileDialog {
    /// Window title; a per-kind default is used when `None`.
    pub title: Option<String>,
    /// Directory the dialog starts in.
    pub directory: Option<PathBuf>,
    /// Pre-filled file name (used by save dialogs).
    pub file_name: Option<String>,
    /// Extension filters offered to the user.
    pub filters: Vec<FileFilter>,
}

impl FileDialog {
    /// Creates an empty dialog description.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the window title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the directory the dialog starts in.
    #[must_use]
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    /// Sets the pre-filled file name.
    #[must_use]
    pub fn file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    /// Appends an extension filter.
    #[must_use]
    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter::new(name, extensions));
        self
    }
}

/// The outcome of a file dialog: `None` when cancelled, otherwise the chosen
/// paths. `OpenFile`, `SaveFile` and `PickFolder` return at most one path.
pub type FileDialogResult = Option<Vec<PathBuf>>;

/// Completion callback handed to a [`FileDialogBackend`]; it must be called exactly once.
pub type FileDialogDone = Box<dyn FnOnce(&mut Tk, FileDialogResult)>;

/// A pluggable file dialog implementation.
///
/// A backend must invoke `done` exactly once. Blocking native dialogs call it
/// before returning; in-app modal dialogs call it later, when the user
/// dismisses the modal.
pub trait FileDialogBackend {
    /// Presents a dialog of `kind` described by `opts` and reports the result.
    fn show(
        &mut self,
        tk: &mut Tk,
        kind: FileDialogKind,
        opts: &FileDialog,
        done: FileDialogDone,
    );
}

/// Returns true when `path` satisfies `filters`.
///
/// An empty filter list matches everything, as does any filter containing the
/// `"*"` extension. Otherwise a filter matches when the path's extension equals
/// one of its extensions, or when the lowercased file name ends with
/// `"." + lowercased extension` (so a `"tar.gz"` filter matches `a.tar.gz`).
/// Comparisons are case-insensitive.
#[must_use]
pub fn matches_filters(path: &Path, filters: &[FileFilter]) -> bool {
    if filters.is_empty() {
        return true;
    }
    if filters
        .iter()
        .any(|f| f.extensions.iter().any(|ext| ext == "*"))
    {
        return true;
    }
    let ext = path.extension().and_then(|e| e.to_str());
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_lowercase);
    filters.iter().any(|f| {
        f.extensions.iter().any(|candidate| {
            if let Some(ext) = ext {
                if candidate.eq_ignore_ascii_case(ext) {
                    return true;
                }
            }
            match &file_name {
                Some(name) => {
                    let suffix = format!(".{}", candidate.to_lowercase());
                    name.ends_with(&suffix)
                }
                None => false,
            }
        })
    })
}

/// The default backend: reuses the in-app modal choosers from [`crate::dialog`].
///
/// The in-app chooser is single-select, so [`FileDialogKind::OpenFiles`]
/// returns a one-element vector. The in-app save chooser does not offer
/// extension filters, so [`FileDialog::filters`] is ignored for
/// [`FileDialogKind::SaveFile`].
#[derive(Clone, Copy, Debug, Default)]
pub struct InAppFileDialog;

impl InAppFileDialog {
    /// Resolves the start directory: `opts.directory`, else the process CWD,
    /// else `"."`.
    fn start_dir(opts: &FileDialog) -> PathBuf {
        opts.directory
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Resolves the dialog title, falling back to the per-kind default.
    fn title(kind: FileDialogKind, opts: &FileDialog) -> String {
        opts.title
            .clone()
            .unwrap_or_else(|| kind.default_title().to_string())
    }
}

impl FileDialogBackend for InAppFileDialog {
    /// Opens the matching in-app modal dialog and forwards its result.
    fn show(
        &mut self,
        tk: &mut Tk,
        kind: FileDialogKind,
        opts: &FileDialog,
        done: FileDialogDone,
    ) {
        let title = Self::title(kind, opts);
        let dir = Self::start_dir(opts);
        match kind {
            FileDialogKind::OpenFile | FileDialogKind::OpenFiles => {
                let ext_refs: Vec<Vec<&str>> = opts
                    .filters
                    .iter()
                    .map(|f| f.extensions.iter().map(String::as_str).collect())
                    .collect();
                let filters: Vec<(&str, &[&str])> = opts
                    .filters
                    .iter()
                    .zip(&ext_refs)
                    .map(|(f, exts)| (f.name.as_str(), exts.as_slice()))
                    .collect();
                dialog::ask_open_filename(tk, &title, &dir, &filters, move |t, path| {
                    done(t, path.map(|p| vec![p]));
                });
            }
            FileDialogKind::SaveFile => {
                let default_name = opts.file_name.clone().unwrap_or_default();
                dialog::ask_save_filename(tk, &title, &dir, &default_name, move |t, path| {
                    done(t, path.map(|p| vec![p]));
                });
            }
            FileDialogKind::PickFolder => {
                dialog::ask_directory(tk, &title, &dir, move |t, path| {
                    done(t, path.map(|p| vec![p]));
                });
            }
        }
    }
}

/// A scripted backend for tests.
///
/// Responses are queued with [`MockFileDialog::push_response`] and handed out
/// in FIFO order; once the queue is empty every request answers `None`. Every
/// request is recorded so tests can assert on the exact options passed.
#[derive(Clone, Debug, Default)]
pub struct MockFileDialog {
    /// Shared state so a test keeps a handle after cloning into `Tk`.
    shared: Rc<RefCell<MockState>>,
}

/// Interior state of a [`MockFileDialog`].
#[derive(Debug, Default)]
struct MockState {
    /// Queued responses, consumed front-to-back.
    responses: std::collections::VecDeque<FileDialogResult>,
    /// Every request received, in order.
    requests: Vec<(FileDialogKind, FileDialog)>,
}

impl MockFileDialog {
    /// Creates a mock with an empty response queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a response to be returned by the next request.
    pub fn push_response(&self, response: FileDialogResult) {
        self.shared.borrow_mut().responses.push_back(response);
    }

    /// Returns every request received so far, in order.
    #[must_use]
    pub fn requests(&self) -> Vec<(FileDialogKind, FileDialog)> {
        self.shared.borrow().requests.clone()
    }

    /// Returns the number of requests received so far.
    #[must_use]
    pub fn request_count(&self) -> usize {
        self.shared.borrow().requests.len()
    }
}

impl FileDialogBackend for MockFileDialog {
    /// Records the request and answers it from the response queue.
    fn show(
        &mut self,
        tk: &mut Tk,
        kind: FileDialogKind,
        opts: &FileDialog,
        done: FileDialogDone,
    ) {
        let response = {
            let mut state = self.shared.borrow_mut();
            state.requests.push((kind, opts.clone()));
            state.responses.pop_front().unwrap_or(None)
        };
        done(tk, response);
    }
}

/// The native backend, available with the `native-dialogs` cargo feature.
///
/// Wraps [`rfd::FileDialog`]'s blocking API and reports the result before
/// returning.
#[cfg(feature = "native-dialogs")]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeFileDialog;

#[cfg(feature = "native-dialogs")]
impl FileDialogBackend for NativeFileDialog {
    /// Shows the platform's native dialog and forwards its result.
    fn show(
        &mut self,
        tk: &mut Tk,
        kind: FileDialogKind,
        opts: &FileDialog,
        done: FileDialogDone,
    ) {
        let mut builder = rfd::FileDialog::new();
        if let Some(title) = opts.title.clone().or_else(|| Some(kind.default_title().to_string())) {
            builder = builder.set_title(title);
        }
        if let Some(dir) = &opts.directory {
            builder = builder.set_directory(dir);
        }
        if let Some(name) = &opts.file_name {
            builder = builder.set_file_name(name);
        }
        for filter in &opts.filters {
            let exts: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
            builder = builder.add_filter(&filter.name, &exts);
        }
        let result = match kind {
            FileDialogKind::OpenFile => builder.pick_file().map(|p| vec![p]),
            FileDialogKind::OpenFiles => builder.pick_files(),
            FileDialogKind::SaveFile => builder.save_file().map(|p| vec![p]),
            FileDialogKind::PickFolder => builder.pick_folder().map(|p| vec![p]),
        };
        done(tk, result);
    }
}

impl Tk {
    /// Installs a custom file dialog backend.
    pub fn set_file_dialog_backend(&mut self, backend: impl FileDialogBackend + 'static) {
        let boxed: Box<dyn FileDialogBackend> = Box::new(backend);
        self.file_dialog_backend = Some(Rc::new(RefCell::new(boxed)));
    }

    /// Removes any custom backend, restoring the default.
    pub fn reset_file_dialog_backend(&mut self) {
        self.file_dialog_backend = None;
    }

    /// Returns true when dialogs use the platform's native implementation.
    ///
    /// This is only the case when no custom backend is installed and the
    /// `native-dialogs` feature is enabled.
    #[must_use]
    pub fn file_dialog_uses_native(&self) -> bool {
        self.file_dialog_backend.is_none() && cfg!(feature = "native-dialogs")
    }

    /// Presents a file dialog of `kind` and reports the result through `done`.
    ///
    /// The installed backend stays in place while it runs, so a callback may
    /// open another dialog (which reuses the same backend) or call
    /// [`Tk::reset_file_dialog_backend`] / [`Tk::set_file_dialog_backend`] and
    /// have that change stick. A backend must not re-enter itself
    /// synchronously (i.e. call `Tk::file_dialog` from inside its own `show`);
    /// in that case the default backend is used for the nested call instead of
    /// panicking.
    pub fn file_dialog(
        &mut self,
        kind: FileDialogKind,
        opts: &FileDialog,
        done: impl FnOnce(&mut Tk, FileDialogResult) + 'static,
    ) {
        let done: FileDialogDone = Box::new(done);
        let showing = Rc::new(std::cell::Cell::new(true));
        let pending = Rc::new(RefCell::new(None));
        let showing_for_done = Rc::clone(&showing);
        let pending_for_done = Rc::clone(&pending);
        let wrapped: FileDialogDone = Box::new(move |t, result| {
            if showing_for_done.get() {
                // Deliver synchronous completions after the backend borrow ends.
                *pending_for_done.borrow_mut() = Some((done, result));
            } else {
                done(t, result);
            }
        });

        // Keep the installed backend in place, including during callbacks.
        let custom = self.file_dialog_backend.clone();
        if let Some(shared) = custom {
            match shared.try_borrow_mut() {
                Ok(mut backend) => backend.show(self, kind, opts, wrapped),
                Err(_) => {
                    // Direct re-entry from inside `show` uses the default.
                    let mut fallback = Self::default_file_dialog_backend();
                    fallback.show(self, kind, opts, wrapped);
                }
            };
        } else {
            let mut backend = Self::default_file_dialog_backend();
            backend.show(self, kind, opts, wrapped);
        }

        showing.set(false);
        let completed = pending.borrow_mut().take();
        if let Some((done, result)) = completed {
            done(self, result);
        }
    }

    /// Returns the default backend for the current feature set.
    fn default_file_dialog_backend() -> Box<dyn FileDialogBackend> {
        #[cfg(feature = "native-dialogs")]
        {
            Box::new(NativeFileDialog)
        }
        #[cfg(not(feature = "native-dialogs"))]
        {
            Box::new(InAppFileDialog)
        }
    }

    /// Opens a single-file chooser and reports the chosen path.
    pub fn open_file_dialog(
        &mut self,
        opts: &FileDialog,
        done: impl FnOnce(&mut Tk, Option<PathBuf>) + 'static,
    ) {
        self.file_dialog(FileDialogKind::OpenFile, opts, move |t, result| {
            done(t, first_path(result));
        });
    }

    /// Opens a multi-file chooser and reports the chosen paths.
    pub fn open_files_dialog(
        &mut self,
        opts: &FileDialog,
        done: impl FnOnce(&mut Tk, Option<Vec<PathBuf>>) + 'static,
    ) {
        self.file_dialog(FileDialogKind::OpenFiles, opts, move |t, result| {
            done(t, non_empty(result));
        });
    }

    /// Opens a save chooser and reports the destination path.
    pub fn save_file_dialog(
        &mut self,
        opts: &FileDialog,
        done: impl FnOnce(&mut Tk, Option<PathBuf>) + 'static,
    ) {
        self.file_dialog(FileDialogKind::SaveFile, opts, move |t, result| {
            done(t, first_path(result));
        });
    }

    /// Opens a folder chooser and reports the chosen directory.
    pub fn pick_folder_dialog(
        &mut self,
        opts: &FileDialog,
        done: impl FnOnce(&mut Tk, Option<PathBuf>) + 'static,
    ) {
        self.file_dialog(FileDialogKind::PickFolder, opts, move |t, result| {
            done(t, first_path(result));
        });
    }
}

/// Normalises an empty vector result to `None`.
fn non_empty(result: FileDialogResult) -> Option<Vec<PathBuf>> {
    result.filter(|paths| !paths.is_empty())
}

/// Extracts the first path of a result, treating an empty vector as `None`.
fn first_path(result: FileDialogResult) -> Option<PathBuf> {
    non_empty(result).and_then(|paths| paths.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `"tar.gz"` filter matches a compound extension.
    #[test]
    fn compound_extension_matches() {
        let filters = vec![FileFilter::new("Archive", &["tar.gz"])];
        assert!(matches_filters(Path::new("/a/b.tar.gz"), &filters));
        assert!(matches_filters(Path::new("/a/B.TAR.GZ"), &filters));
        assert!(!matches_filters(Path::new("/a/b.gz"), &filters));
        assert!(!matches_filters(Path::new("/a/b.tar"), &filters));
    }

    /// `FileFilter::new` strips a single leading dot from each extension.
    #[test]
    fn filter_strips_leading_dot() {
        let f = FileFilter::new("Text", &[".txt", "md", "..weird"]);
        assert_eq!(f.name, "Text");
        assert_eq!(f.extensions, vec!["txt", "md", ".weird"]);
    }

    /// An empty filter list matches any path.
    #[test]
    fn empty_filters_match_all() {
        assert!(matches_filters(Path::new("/a/b.xyz"), &[]));
    }

    /// A `"*"` extension matches any path.
    #[test]
    fn star_matches_all() {
        let filters = vec![FileFilter::new("All", &["*"])];
        assert!(matches_filters(Path::new("/a/b.xyz"), &filters));
    }

    /// Extensions match case-insensitively and reject non-matches.
    #[test]
    fn extension_matching() {
        let filters = vec![FileFilter::new("Images", &["png", "JPG"])];
        assert!(matches_filters(Path::new("a.PNG"), &filters));
        assert!(matches_filters(Path::new("a.jpg"), &filters));
        assert!(!matches_filters(Path::new("a.gif"), &filters));
        assert!(!matches_filters(Path::new("noext"), &filters));
    }

    /// The builder records every field.
    #[test]
    fn builder_fields() {
        let opts = FileDialog::new()
            .title("Pick")
            .directory("/tmp")
            .file_name("out.txt")
            .add_filter("Text", &[".txt"]);
        assert_eq!(opts.title.as_deref(), Some("Pick"));
        assert_eq!(opts.directory.as_deref(), Some(Path::new("/tmp")));
        assert_eq!(opts.file_name.as_deref(), Some("out.txt"));
        assert_eq!(opts.filters.len(), 1);
        assert_eq!(opts.filters[0].extensions, vec!["txt"]);
    }
}
