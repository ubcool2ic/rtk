//! Tests for newer features including cross-thread proxies, dialogs, and icons.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use rtk::dialog;
use rtk::icons::{self, Icon};
use rtk::{Button, Color, Key, Opt, State, Theme, Tk};

/// Checks that a proxy can post a closure from another thread and have it execute.
#[test]
fn proxy_post_from_another_thread() {
    let mut tk = Tk::new();
    let proxy = tk.proxy();
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();

    let handle = thread::spawn(move || {
        proxy.post(move |_t| {
            flag_clone.store(true, Ordering::SeqCst);
        });
    });
    handle.join().unwrap();

    assert!(!flag.load(Ordering::SeqCst));
    let processed = tk.process_posted();
    assert_eq!(processed, 1);
    assert!(flag.load(Ordering::SeqCst));
}

/// Ensures after_idle callbacks only run when posted events are processed.
#[test]
fn after_idle_runs_on_process_posted() {
    let mut tk = Tk::new();
    let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let c = counter.clone();

    tk.after_idle(move |_t| {
        c.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(counter.load(Ordering::SeqCst), 0);
    tk.process_posted();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

/// Tests the ask_string dialog's accept and cancel callback results.
#[test]
fn dialog_ask_string_accept_and_cancel() {
    let mut tk = Tk::new();
    let root = tk.root();
    let _bg = Button::new(&mut tk, root, "Background").unwrap();

    let result = Arc::new(std::sync::Mutex::new(None::<Option<String>>));
    let r_clone = result.clone();

    dialog::ask_string(&mut tk, "Test Prompt", "Enter your name:", "Alice", move |_t, res| {
        *r_clone.lock().unwrap() = Some(res);
    });

    assert!(dialog::is_open(&tk));

    // Type text into the pre-filled and focused entry
    tk.sim_text(" Cooper");
    tk.sim_key(Key::Enter);

    assert!(!dialog::is_open(&tk));
    let res = result.lock().unwrap().take().expect("callback should have fired");
    assert_eq!(res, Some("Alice Cooper".into()));

    // Now test Escape returns None
    let result_esc = Arc::new(std::sync::Mutex::new(None::<Option<String>>));
    let re_clone = result_esc.clone();

    dialog::ask_string(&mut tk, "Test Escape", "Type something:", "", move |_t, res| {
        *re_clone.lock().unwrap() = Some(res);
    });

    assert!(dialog::is_open(&tk));
    tk.sim_text("Ignored");
    tk.sim_key(Key::Escape);

    assert!(!dialog::is_open(&tk));
    let res_esc = result_esc.lock().unwrap().take().expect("callback fired on escape");
    assert_eq!(res_esc, None);
}

/// Verifies ask_typed_confirmation only accepts an exact match of the required phrase.
#[test]
fn dialog_ask_typed_confirmation_exact_match() {
    let mut tk = Tk::new();
    let confirmed = Arc::new(AtomicBool::new(false));
    let c_clone = confirmed.clone();

    dialog::ask_typed_confirmation(&mut tk, "Delete Item", "This cannot be undone.", "DELETE", move |_t, ok| {
        c_clone.store(ok, Ordering::SeqCst);
    });

    assert!(dialog::is_open(&tk));

    // Incomplete phrase doesn't accept
    tk.sim_text("DEL");
    tk.sim_key(Key::Enter);
    assert!(dialog::is_open(&tk));
    assert!(!confirmed.load(Ordering::SeqCst));

    // Complete exact phrase enables and accepts on Enter
    tk.sim_text("ETE");
    tk.sim_key(Key::Enter);
    assert!(!dialog::is_open(&tk));
    assert!(confirmed.load(Ordering::SeqCst));
}

/// Confirms a modal dialog blocks clicks from reaching background widgets.
#[test]
fn modal_blocks_background_clicks() {
    let mut tk = Tk::new();
    let root = tk.root();
    let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let c = counter.clone();

    let btn = Button::new(&mut tk, root, "Target").unwrap();
    tk.set_command(btn, move |_t| {
        c.fetch_add(1, Ordering::SeqCst);
    });
    tk.pack(btn, Default::default());
    tk.sim_resize(600, 400);

    // Clicking before dialog works
    let rect = tk.widget_rect(btn).unwrap();
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Open modal dialog
    dialog::message_box(&mut tk, "Blocked", "Clicking behind is ignored", dialog::MessageKind::Info, |_| {});
    assert!(dialog::is_open(&tk));

    // Click at the same background button location
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    // Close dialog
    dialog::close(&mut tk);
    assert!(!dialog::is_open(&tk));

    // Clicking now works again
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

/// Tests that the file browser dialog lists directory contents and returns an accepted path.
#[test]
fn dialog_file_browser_lists_and_accepts() {
    let tmp = std::env::temp_dir().join("rtk_test_file_dialog");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("subdir")).unwrap();
    std::fs::write(tmp.join("data.bin"), b"1234").unwrap();
    std::fs::write(tmp.join("notes.txt"), b"text").unwrap();

    let mut tk = Tk::new();
    let accepted = Arc::new(std::sync::Mutex::new(None::<Option<PathBuf>>));
    let a_clone = accepted.clone();

    dialog::ask_open_filename(
        &mut tk,
        "Open Binary",
        &tmp,
        &[("Binary", &["bin"])],
        move |_t, res| {
            *a_clone.lock().unwrap() = Some(res);
        },
    );

    assert!(dialog::is_open(&tk));

    // Find the Listbox within modal overlay and check filtered listing
    let listbox_id = dialog::file_listbox(&tk);
    let listbox = listbox_id.expect("listbox must be in dialog");
    let items = tk.listbox_get(listbox);

    // Dirs first, matching file, non-matching filtered out
    assert!(items.contains(&"subdir/".to_string()));
    assert!(items.contains(&"data.bin".to_string()));
    assert!(!items.contains(&"notes.txt".to_string()));

    // Select data.bin and press Enter
    let bin_idx = items.iter().position(|s| s == "data.bin").unwrap();
    tk.listbox_select(listbox, Some(bin_idx));
    tk.sim_key(Key::Enter);

    assert!(!dialog::is_open(&tk));
    let path = accepted.lock().unwrap().take().expect("callback ran").expect("file chosen");
    assert_eq!(path.file_name().unwrap(), "data.bin");

    let _ = std::fs::remove_dir_all(tmp);
}

/// Checks that the ModernDark theme applies the expected accent button background color.
#[test]
fn modern_dark_and_accent_theming() {
    let mut tk = Tk::new();
    tk.set_theme(Theme::ModernDark);
    assert_eq!(tk.theme(), Theme::ModernDark);

    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Accent Button").unwrap();
    tk.config(btn, Opt::Style("Accent.TButton".into()));

    let style = tk.resolved_style(btn).unwrap();
    assert_eq!(style.bg, Color::rgb(79, 140, 255));

    // Change accent dynamically
    let emerald = Color::rgb(16, 185, 129);
    tk.set_accent(emerald);
    let updated_style = tk.resolved_style(btn).unwrap();
    assert_eq!(updated_style.bg, emerald);
}

/// Verifies set_selected toggles the SELECTED state bit on a widget.
#[test]
fn set_selected_state_bit() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Item").unwrap();
    assert!(!tk.widget_state(btn).contains(State::SELECTED));

    tk.set_selected(btn, true);
    assert!(tk.widget_state(btn).contains(State::SELECTED));

    tk.set_selected(btn, false);
    assert!(!tk.widget_state(btn).contains(State::SELECTED));
}

/// Ensures every built-in icon renders to RGBA and registers without error.
#[test]
fn all_icons_render_and_register() {
    let mut tk = Tk::new();
    let all = [
        Icon::Home, Icon::Chip, Icon::Usb, Icon::Library, Icon::Sliders, Icon::Wrench,
        Icon::Bolt, Icon::Settings, Icon::Book, Icon::Shield, Icon::Download, Icon::Upload,
        Icon::Check, Icon::Warning, Icon::Error, Icon::Info, Icon::Refresh, Icon::Folder,
        Icon::Save, Icon::Play, Icon::Stop, Icon::Sun, Icon::Moon, Icon::Search,
        Icon::Memory, Icon::Partition, Icon::Battery, Icon::Clock,
    ];

    for icon in all {
        let rgba = icons::render_rgba(icon, 24, Color::WHITE);
        assert_eq!(rgba.len(), 24 * 24 * 4);
        let has_alpha = rgba.chunks_exact(4).any(|p| p[3] > 0);
        assert!(has_alpha, "icon {:?} rendered completely transparent", icon);

        let img_id = icons::icon(&mut tk, icon, 24, Color::WHITE);
        assert!(img_id.is_some(), "icon {:?} failed to register in tk", icon);
    }
}
