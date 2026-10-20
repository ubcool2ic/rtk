//! Modal dialog overlays for alerts, confirmations, prompts, and file picking.
//!
//! Dialogs are stacked: opening a dialog while another is already presented
//! pushes the outer dialog's state onto `Tk::modal_stack`, so the inner dialog
//! can be dismissed without orphaning the outer one. All teardown funnels
//! through [`finish_modal`], which restores the previous frame.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::layout::{Anchor, PackOpts, PlaceOpts, Side};
use crate::tk::ModalFrame;
use crate::widgets::{Button, Color, Combobox, Entry, Frame, Label, Listbox, Opt};
use crate::{Tk, WidgetId};

/// Severity / classification icon for informational dialogs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    /// Informational message.
    Info,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

/// Returns true if a modal dialog is currently presented.
pub fn is_open(tk: &Tk) -> bool {
    tk.modal.is_some()
}

/// Returns the number of currently presented modal dialogs (0 when none).
///
/// Nested dialogs each add one level; the top-most dialog is the one that
/// receives Escape/Enter and blocks input to everything beneath it.
pub fn modal_depth(tk: &Tk) -> usize {
    tk.modal_stack.len() + usize::from(tk.modal.is_some())
}

/// Closes the top-most modal dialog, running its cancel callback if pending.
///
/// Only one level is popped; any outer dialog beneath it becomes active again.
pub fn close(tk: &mut Tk) {
    if let Some(mut cancel) = tk.modal_cancel.take() {
        cancel(tk);
    } else if let Some(modal_id) = tk.modal {
        finish_modal(tk, modal_id);
    }
}

/// Destroys `scrim` and restores the previous modal frame from the stack.
///
/// Only the dialog whose scrim is the current top may be finished; if `scrim`
/// is not the active modal (a stale closure fired after the dialog already
/// closed) this is a no-op. When the stack is empty the modal state is cleared.
pub(crate) fn finish_modal(t: &mut Tk, scrim: WidgetId) {
    if t.modal != Some(scrim) {
        return;
    }
    t.modal = None;
    t.modal_cancel = None;
    t.modal_accept = None;
    t.modal_entry_changed = None;
    if t.contains(scrim) {
        t.destroy(scrim);
    }
    if let Some(frame) = t.modal_stack.pop() {
        t.modal = frame.modal;
        t.modal_cancel = frame.cancel;
        t.modal_accept = frame.accept;
        t.modal_entry_changed = frame.entry_changed;
    }
    t.dirty = true;
}

/// Helper that builds the full-screen semi-transparent scrim and centered Card frame.
///
/// If a dialog is already presented, its state is pushed onto the modal stack
/// so it can be restored when this new dialog is dismissed.
fn build_scrim_card(tk: &mut Tk, width: f32) -> Result<(WidgetId, WidgetId), crate::RtkError> {
    if tk.modal.is_some() {
        let frame = ModalFrame {
            modal: tk.modal,
            cancel: tk.modal_cancel.take(),
            accept: tk.modal_accept.take(),
            entry_changed: tk.modal_entry_changed.take(),
        };
        tk.modal_stack.push(frame);
        tk.modal = None;
    }
    let root = tk.root();
    let scrim = Frame::new(tk, root)?;
    tk.config(scrim, Opt::StyleOverride(crate::StyleSpec {
        bg: Some(Color::rgba(0, 0, 0, 110)),
        border_width: Some(0.0),
        ..Default::default()
    }));
    let scrim_opts = PlaceOpts {
        relx: 0.0,
        rely: 0.0,
        relwidth: Some(1.0),
        relheight: Some(1.0),
        ..PlaceOpts::default()
    };
    tk.place(scrim, scrim_opts);

    let card = Frame::new(tk, scrim)?;
    tk.config(card, Opt::Style("Card".into()));
    let card_opts = PlaceOpts {
        relx: 0.5,
        rely: 0.5,
        anchor: Anchor::Center,
        width: Some(width),
        ..PlaceOpts::default()
    };
    tk.place(card, card_opts);

    tk.modal = Some(scrim);
    Ok((scrim, card))
}

/// Displays an alert dialog with an OK button.
pub fn message_box(
    tk: &mut Tk,
    title: &str,
    message: &str,
    _kind: MessageKind,
    on_close: impl FnOnce(&mut Tk) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 380.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_close) as Box<dyn FnOnce(&mut Tk)>)));

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let msg_lbl = Label::new(tk, card, message).unwrap();
    p.pady = 12.0;
    tk.pack(msg_lbl, p);

    let ok_btn = Button::new(tk, card, "OK").unwrap();
    tk.config(ok_btn, Opt::Style("Accent.TButton".into()));
    p.pady = 6.0;
    tk.pack(ok_btn, p);

    let cb_clone = cb.clone();
    tk.set_command(ok_btn, move |t| {
        if let Some(f) = cb_clone.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t);
        }
    });

    let cb_cancel = cb.clone();
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_cancel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t);
        }
    }));
    tk.modal_accept = Some(Box::new(move |t| {
        if let Some(f) = cb.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t);
        }
    }));
    tk.dirty = true;
}

/// Prompts the user with Yes and No buttons, invoking `on_result(tk, bool)`.
pub fn ask_yes_no(
    tk: &mut Tk,
    title: &str,
    message: &str,
    on_result: impl FnOnce(&mut Tk, bool) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 380.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_result) as Box<dyn FnOnce(&mut Tk, bool)>)));

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let msg_lbl = Label::new(tk, card, message).unwrap();
    p.pady = 12.0;
    tk.pack(msg_lbl, p);

    let btn_row = Frame::new(tk, card).unwrap();
    p.pady = 4.0;
    tk.pack(btn_row, p);

    let yes_btn = Button::new(tk, btn_row, "Yes").unwrap();
    tk.config(yes_btn, Opt::Style("Accent.TButton".into()));
    let bp = PackOpts {
        side: Side::Left,
        padx: 6.0,
        ..PackOpts::default()
    };
    tk.pack(yes_btn, bp);

    let no_btn = Button::new(tk, btn_row, "No").unwrap();
    tk.config(no_btn, Opt::Style("TButton".into()));
    tk.pack(no_btn, bp);

    let cb_yes = cb.clone();
    tk.set_command(yes_btn, move |t| {
        if let Some(f) = cb_yes.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, true);
        }
    });

    let cb_no = cb.clone();
    tk.set_command(no_btn, move |t| {
        if let Some(f) = cb_no.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, false);
        }
    });

    let cb_esc = cb.clone();
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_esc.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, false);
        }
    }));
    tk.modal_accept = Some(Box::new(move |t| {
        if let Some(f) = cb.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, true);
        }
    }));
    tk.dirty = true;
}

/// Prompts the user for a text string with an OK and Cancel button.
pub fn ask_string(
    tk: &mut Tk,
    title: &str,
    prompt: &str,
    initial: &str,
    on_result: impl FnOnce(&mut Tk, Option<String>) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 400.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_result) as Box<dyn FnOnce(&mut Tk, Option<String>)>)));

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let prompt_lbl = Label::new(tk, card, prompt).unwrap();
    p.pady = 8.0;
    tk.pack(prompt_lbl, p);

    let entry = Entry::new(tk, card).unwrap();
    tk.entry_set(entry, initial);
    p.fill = crate::Fill::X;
    p.pady = 10.0;
    tk.pack(entry, p);
    tk.focused = Some(entry);

    let btn_row = Frame::new(tk, card).unwrap();
    let rp = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(btn_row, rp);

    let ok_btn = Button::new(tk, btn_row, "OK").unwrap();
    tk.config(ok_btn, Opt::Style("Accent.TButton".into()));
    let bp = PackOpts {
        side: Side::Left,
        padx: 6.0,
        ..PackOpts::default()
    };
    tk.pack(ok_btn, bp);

    let cancel_btn = Button::new(tk, btn_row, "Cancel").unwrap();
    tk.pack(cancel_btn, bp);

    let cb_ok = cb.clone();
    tk.set_command(ok_btn, move |t| {
        let val = t.entry_get(entry);
        if let Some(f) = cb_ok.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, val);
        }
    });

    let cb_cancel = cb.clone();
    tk.set_command(cancel_btn, move |t| {
        if let Some(f) = cb_cancel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    });

    let cb_esc = cb.clone();
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_esc.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    }));

    tk.modal_accept = Some(Box::new(move |t| {
        let val = t.entry_get(entry);
        if let Some(f) = cb.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, val);
        }
    }));
    tk.dirty = true;
}

/// Dangerous action confirmation where the OK button is only enabled when text matches `phrase`.
pub fn ask_typed_confirmation(
    tk: &mut Tk,
    title: &str,
    warning: &str,
    phrase: &str,
    on_result: impl FnOnce(&mut Tk, bool) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 420.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_result) as Box<dyn FnOnce(&mut Tk, bool)>)));

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let warn_lbl = Label::new(tk, card, warning).unwrap();
    p.pady = 6.0;
    tk.pack(warn_lbl, p);

    let prompt_text = format!("Type \"{phrase}\" to confirm:");
    let prompt_lbl = Label::new(tk, card, prompt_text).unwrap();
    tk.config(prompt_lbl, Opt::Style("Muted".into()));
    p.pady = 4.0;
    tk.pack(prompt_lbl, p);

    let entry = Entry::new(tk, card).unwrap();
    p.fill = crate::Fill::X;
    p.pady = 10.0;
    tk.pack(entry, p);
    tk.focused = Some(entry);

    let btn_row = Frame::new(tk, card).unwrap();
    let rp = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(btn_row, rp);

    let ok_btn = Button::new(tk, btn_row, "Confirm").unwrap();
    tk.config(ok_btn, Opt::Style("Accent.TButton".into()));
    tk.config(ok_btn, Opt::Disabled(true));
    let bp = PackOpts {
        side: Side::Left,
        padx: 6.0,
        ..PackOpts::default()
    };
    tk.pack(ok_btn, bp);

    let cancel_btn = Button::new(tk, btn_row, "Cancel").unwrap();
    tk.pack(cancel_btn, bp);

    let target_phrase = phrase.to_string();
    let phrase_for_entry = target_phrase.clone();
    tk.modal_entry_changed = Some(Box::new(move |t, id, text| {
        if id == entry {
            t.config(ok_btn, Opt::Disabled(text != phrase_for_entry));
        }
    }));

    let cb_ok = cb.clone();
    let phrase_check = target_phrase.clone();
    tk.set_command(ok_btn, move |t| {
        let text = t.entry_get(entry).unwrap_or_default();
        if text == phrase_check {
            if let Some(f) = cb_ok.borrow_mut().take() {
                finish_modal(t, scrim);
                f(t, true);
            }
        }
    });

    let cb_cancel = cb.clone();
    tk.set_command(cancel_btn, move |t| {
        if let Some(f) = cb_cancel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, false);
        }
    });

    let cb_esc = cb.clone();
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_esc.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, false);
        }
    }));

    let phrase_accept = target_phrase;
    tk.modal_accept = Some(Box::new(move |t| {
        let text = t.entry_get(entry).unwrap_or_default();
        if text == phrase_accept {
            if let Some(f) = cb.borrow_mut().take() {
                finish_modal(t, scrim);
                f(t, true);
            }
        }
    }));
    tk.dirty = true;
}

// ---------------------------------------------------------------------------
// File and Directory Browser Dialogs
// ---------------------------------------------------------------------------

/// Reads `dir`, returning sorted subdirectory names (with a trailing `/`) and
/// sorted file names that match `filter_exts` (empty or `*` matches all).
fn read_dir_entries(dir: &Path, filter_exts: &[&str]) -> (Vec<String>, Vec<String>) {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                dirs.push(format!("{name}/"));
            } else {
                let matches_filter = if filter_exts.is_empty() || filter_exts.contains(&"*") {
                    true
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    filter_exts.iter().any(|f| f.eq_ignore_ascii_case(&ext_lower))
                } else {
                    false
                };
                if matches_filter {
                    files.push(name);
                }
            }
        }
    }
    dirs.sort();
    files.sort();
    (dirs, files)
}

/// Clears `listbox` and repopulates it with `..`, subdirectories, then files.
fn populate_file_list(
    tk: &mut Tk,
    listbox: WidgetId,
    current_dir: &Path,
    filter_exts: &[&str],
) -> Vec<String> {
    let total = tk.listbox_get(listbox).len();
    if total > 0 {
        tk.listbox_delete(listbox, 0, total);
    }
    let (dirs, files) = read_dir_entries(current_dir, filter_exts);
    let mut all_items = Vec::new();
    if current_dir.parent().is_some() {
        all_items.push("..".to_string());
    }
    all_items.extend(dirs);
    all_items.extend(files);
    for (i, item) in all_items.iter().enumerate() {
        tk.listbox_insert(listbox, i, item);
    }
    all_items
}

/// Built-in modal file chooser for opening files.
pub fn ask_open_filename(
    tk: &mut Tk,
    title: &str,
    start_dir: &Path,
    filters: &[(&str, &[&str])],
    on_result: impl FnOnce(&mut Tk, Option<PathBuf>) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 520.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_result) as Box<dyn FnOnce(&mut Tk, Option<PathBuf>)>)));

    let cur_dir = Rc::new(RefCell::new(start_dir.to_path_buf()));
    let owned_filters: Vec<(String, Vec<String>)> = filters.iter()
        .map(|(n, exts)| (n.to_string(), exts.iter().map(|s| s.to_string()).collect()))
        .collect();
    let filter_var = tk.int_var(0);

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let path_lbl = Label::new(tk, card, start_dir.display().to_string()).unwrap();
    tk.config(path_lbl, Opt::Style("Mono".into()));
    p.pady = 4.0;
    p.fill = crate::Fill::X;
    tk.pack(path_lbl, p);

    let listbox = Listbox::new(tk, card, 10).unwrap();
    p.pady = 8.0;
    tk.pack(listbox, p);

    let combo_vals: Vec<String> = if owned_filters.is_empty() {
        vec!["All Files (*)".into()]
    } else {
        owned_filters.iter().map(|(n, exts)| format!("{n} (*.{})", exts.join(", *."))).collect()
    };
    let combobox = Combobox::new(tk, card, combo_vals, filter_var).unwrap();
    tk.pack(combobox, p);

    let active_filter: Vec<&str> = if let Some((_, exts)) = owned_filters.first() {
        exts.iter().map(|s| s.as_str()).collect()
    } else {
        Vec::new()
    };
    populate_file_list(tk, listbox, start_dir, &active_filter);

    let btn_row = Frame::new(tk, card).unwrap();
    let bp = PackOpts {
        side: Side::Top,
        pady: 6.0,
        ..PackOpts::default()
    };
    tk.pack(btn_row, bp);

    let open_btn = Button::new(tk, btn_row, "Open").unwrap();
    tk.config(open_btn, Opt::Style("Accent.TButton".into()));
    let bbp = PackOpts {
        side: Side::Left,
        padx: 6.0,
        ..PackOpts::default()
    };
    tk.pack(open_btn, bbp);

    let cancel_btn = Button::new(tk, btn_row, "Cancel").unwrap();
    tk.pack(cancel_btn, bbp);

    let make_action = {
        let cur_dir = cur_dir.clone();
        let owned_filters = owned_filters.clone();
        let cb = cb.clone();
        move |t: &mut Tk| {
            let sel = t.listbox_curselection(listbox);
            let items = t.listbox_get(listbox);
            let Some(idx) = sel else { return };
            let Some(item) = items.get(idx) else { return };

            let filter_idx = filter_var.get(t).unwrap_or(0) as usize;
            let filter_exts: Vec<&str> = owned_filters.get(filter_idx)
                .map(|(_, exts)| exts.iter().map(|s| s.as_str()).collect())
                .unwrap_or_default();

            if item == ".." {
                let parent = cur_dir.borrow().parent().map(|p| p.to_path_buf());
                if let Some(p) = parent {
                    *cur_dir.borrow_mut() = p.clone();
                    t.config(path_lbl, Opt::Text(p.display().to_string()));
                    populate_file_list(t, listbox, &p, &filter_exts);
                }
            } else if item.ends_with('/') {
                let child = cur_dir.borrow().join(&item[..item.len() - 1]);
                *cur_dir.borrow_mut() = child.clone();
                t.config(path_lbl, Opt::Text(child.display().to_string()));
                populate_file_list(t, listbox, &child, &filter_exts);
            } else {
                let file_path = cur_dir.borrow().join(item);
                if let Some(f) = cb.borrow_mut().take() {
                    finish_modal(t, scrim);
                    f(t, Some(file_path));
                }
            }
        }
    };

    let act_clone = make_action.clone();
    tk.set_command(open_btn, act_clone);

    let cb_cancel = cb.clone();
    tk.set_command(cancel_btn, move |t| {
        if let Some(f) = cb_cancel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    });

    let cb_esc = cb;
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_esc.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    }));
    tk.modal_accept = Some(Box::new(make_action));
    tk.dirty = true;
}

/// Built-in modal file chooser for saving a file.
pub fn ask_save_filename(
    tk: &mut Tk,
    title: &str,
    start_dir: &Path,
    default_name: &str,
    on_result: impl FnOnce(&mut Tk, Option<PathBuf>) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 520.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_result) as Box<dyn FnOnce(&mut Tk, Option<PathBuf>)>)));
    let cur_dir = Rc::new(RefCell::new(start_dir.to_path_buf()));

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let path_lbl = Label::new(tk, card, start_dir.display().to_string()).unwrap();
    tk.config(path_lbl, Opt::Style("Mono".into()));
    p.fill = crate::Fill::X;
    tk.pack(path_lbl, p);

    let listbox = Listbox::new(tk, card, 8).unwrap();
    p.pady = 6.0;
    tk.pack(listbox, p);
    populate_file_list(tk, listbox, start_dir, &[]);

    let filename_entry = Entry::new(tk, card).unwrap();
    tk.entry_set(filename_entry, default_name);
    p.pady = 6.0;
    tk.pack(filename_entry, p);

    let btn_row = Frame::new(tk, card).unwrap();
    let bp = PackOpts {
        side: Side::Top,
        pady: 6.0,
        ..PackOpts::default()
    };
    tk.pack(btn_row, bp);

    let save_btn = Button::new(tk, btn_row, "Save").unwrap();
    tk.config(save_btn, Opt::Style("Accent.TButton".into()));
    let bbp = PackOpts {
        side: Side::Left,
        padx: 6.0,
        ..PackOpts::default()
    };
    tk.pack(save_btn, bbp);

    let cancel_btn = Button::new(tk, btn_row, "Cancel").unwrap();
    tk.pack(cancel_btn, bbp);

    let cur_dir_clone = cur_dir.clone();
    let cb_save = cb.clone();
    let do_save = move |t: &mut Tk| {
        let name = t.entry_get(filename_entry).unwrap_or_default();
        if !name.trim().is_empty() {
            let path = cur_dir_clone.borrow().join(name.trim());
            if let Some(f) = cb_save.borrow_mut().take() {
                finish_modal(t, scrim);
                f(t, Some(path));
            }
        }
    };

    let do_save_btn = do_save.clone();
    tk.set_command(save_btn, do_save_btn);

    let cb_cancel = cb.clone();
    tk.set_command(cancel_btn, move |t| {
        if let Some(f) = cb_cancel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    });

    let cb_esc = cb;
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_esc.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    }));
    tk.modal_accept = Some(Box::new(do_save));
    tk.dirty = true;
}

/// Built-in modal directory browser dialog.
pub fn ask_directory(
    tk: &mut Tk,
    title: &str,
    start_dir: &Path,
    on_result: impl FnOnce(&mut Tk, Option<PathBuf>) + 'static,
) {
    let (scrim, card) = match build_scrim_card(tk, 480.0) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let cb = Rc::new(RefCell::new(Some(Box::new(on_result) as Box<dyn FnOnce(&mut Tk, Option<PathBuf>)>)));
    let cur_dir = Rc::new(RefCell::new(start_dir.to_path_buf()));

    let title_lbl = Label::new(tk, card, title).unwrap();
    tk.config(title_lbl, Opt::Style("Subheading".into()));
    let mut p = PackOpts {
        side: Side::Top,
        pady: 4.0,
        ..PackOpts::default()
    };
    tk.pack(title_lbl, p);

    let path_lbl = Label::new(tk, card, start_dir.display().to_string()).unwrap();
    tk.config(path_lbl, Opt::Style("Mono".into()));
    p.fill = crate::Fill::X;
    tk.pack(path_lbl, p);

    let listbox = Listbox::new(tk, card, 10).unwrap();
    p.pady = 8.0;
    tk.pack(listbox, p);

    let refresh_dirs = {
        let cur_dir = cur_dir.clone();
        move |t: &mut Tk| {
            let total = t.listbox_get(listbox).len();
            if total > 0 {
                t.listbox_delete(listbox, 0, total);
            }
            let (dirs, _) = read_dir_entries(&cur_dir.borrow(), &[]);
            let mut items = Vec::new();
            if cur_dir.borrow().parent().is_some() {
                items.push("..".to_string());
            }
            items.extend(dirs);
            for (i, d) in items.iter().enumerate() {
                t.listbox_insert(listbox, i, d);
            }
        }
    };
    refresh_dirs(tk);

    let btn_row = Frame::new(tk, card).unwrap();
    let bp = PackOpts {
        side: Side::Top,
        pady: 6.0,
        ..PackOpts::default()
    };
    tk.pack(btn_row, bp);

    let select_btn = Button::new(tk, btn_row, "Select Folder").unwrap();
    tk.config(select_btn, Opt::Style("Accent.TButton".into()));
    let bbp = PackOpts {
        side: Side::Left,
        padx: 6.0,
        ..PackOpts::default()
    };
    tk.pack(select_btn, bbp);

    let cancel_btn = Button::new(tk, btn_row, "Cancel").unwrap();
    tk.pack(cancel_btn, bbp);

    let cur_dir_sel = cur_dir.clone();
    let cb_sel = cb.clone();
    tk.set_command(select_btn, move |t| {
        let path = cur_dir_sel.borrow().clone();
        if let Some(f) = cb_sel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, Some(path));
        }
    });

    let cb_cancel = cb.clone();
    tk.set_command(cancel_btn, move |t| {
        if let Some(f) = cb_cancel.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    });

    let cb_esc = cb;
    tk.modal_cancel = Some(Box::new(move |t| {
        if let Some(f) = cb_esc.borrow_mut().take() {
            finish_modal(t, scrim);
            f(t, None);
        }
    }));
    tk.dirty = true;
}

/// Returns the file/directory `Listbox` of the open file dialog, if any.
/// Useful for tests and automation (e.g. reading the filtered listing).
pub fn file_listbox(tk: &Tk) -> Option<WidgetId> {
    let modal = tk.modal?;
    // Depth-first walk of the modal overlay subtree looking for the Listbox.
    let mut stack = vec![modal];
    while let Some(id) = stack.pop() {
        let node = tk.node(id)?;
        if node.kind == crate::widgets::Kind::Listbox {
            return Some(id);
        }
        stack.extend(node.children.iter().rev().copied());
    }
    None
}
