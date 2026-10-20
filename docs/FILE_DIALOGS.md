# File dialogs (`rtk::file_dialog`)

Added in 0.11. One API covers four dialog kinds and hides which implementation shows them.

| Kind | Convenience method | Result |
|---|---|---|
| `FileDialogKind::OpenFile` | `tk.open_file_dialog(&opts, \|tk, Option<PathBuf>\| ..)` | one file |
| `FileDialogKind::OpenFiles` | `tk.open_files_dialog(&opts, \|tk, Option<Vec<PathBuf>>\| ..)` | one or more files |
| `FileDialogKind::SaveFile` | `tk.save_file_dialog(&opts, \|tk, Option<PathBuf>\| ..)` | destination path |
| `FileDialogKind::PickFolder` | `tk.pick_folder_dialog(&opts, \|tk, Option<PathBuf>\| ..)` | one directory |

`None` means the user cancelled. An empty selection also comes back as `None`.
The generic entry point is `tk.file_dialog(kind, &opts, |tk, FileDialogResult| ..)`.

## Options

```rust,ignore
use rtk::file_dialog::FileDialog;

let opts = FileDialog::new()
    .title("Open a source file")
    .directory("/home/me/project")      // start directory (default path)
    .file_name("untitled.txt")          // suggested name (save dialogs)
    .add_filter("Rust sources", &["rs"])
    .add_filter("All files", &["*"]);
tk.open_file_dialog(&opts, |tk, path| {
    if let Some(path) = path {
        println!("picked {}", path.display());
    }
});
```

Extensions can be given with or without the dot. `file_dialog::matches_filters(path, &filters)`
checks a path against filters. `"*"` matches everything.

## Backends

The callback style lets one API serve both blocking native dialogs and rtk's non-blocking in-app
modal dialogs. A backend implements `FileDialogBackend::show` and must call `done` exactly once.

| Backend | When | Notes |
|---|---|---|
| `NativeFileDialog` | default when the `native-dialogs` feature is on | Wraps `rfd` 0.17.2 (blocking API). Windows and macOS use the system dialogs. Linux uses the XDG desktop portal (no GTK). |
| `InAppFileDialog` | default without the feature | The existing `dialog::ask_open_filename` / `ask_save_filename` / `ask_directory` overlays. Single-select only: `OpenFiles` returns at most one path. |
| `MockFileDialog` | tests | Scripted FIFO responses (`push_response`) plus a record of every request (`requests`, `request_count`). Never opens a window. |

```toml
rtk = { version = "0.8", features = ["native-dialogs"] }
```

`tk.set_file_dialog_backend(backend)` installs a custom backend, and `tk.reset_file_dialog_backend()`
goes back to the default. `tk.file_dialog_uses_native()` reports whether the native backend is active.

### Testing

```rust,ignore
use rtk::file_dialog::MockFileDialog;
use std::path::PathBuf;

let mock = MockFileDialog::new();
mock.push_response(Some(vec![PathBuf::from("/tmp/a.txt")]));
tk.set_file_dialog_backend(mock.clone());   // clones share state
// ... drive the UI that opens the dialog ...
assert_eq!(mock.request_count(), 1);
```

See `tests/m011_file_dialog.rs` and `examples/file_dialogs.rs`.

## Limits

- Native dialogs are modal to the application but are not parented to the rtk window (no
  `set_parent` yet), so on some desktops they may open behind it.
- The native dialog blocks the event loop while it is open, so animations pause.
- `native-dialogs` does not build on the 1.85 MSRV toolchain (`rfd` 0.17.2 fails to compile there).
  It builds on current stable. The default feature set still builds on 1.85.
- With the default xdg-portal backend, Linux needs a running desktop portal
  (`xdg-desktop-portal`). Without one the dialog returns `None`.
