//! Tests that the entry widget handles Unicode grapheme clusters correctly across navigation and deletion.

use rtk::{Entry, Key, Opt, PackOpts, Tk};

/// Checks that grapheme clusters survive cursor navigation and deletion in an Entry.
#[test]
fn unicode_clusters_survive_navigation_and_deletion() {
    let mut tk = Tk::new();
    let root = tk.root();
    let value = tk.string_var("");
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.config(entry, Opt::TextVariable(value));
    tk.pack(entry, PackOpts::default());
    tk.sim_resize(400, 100);
    tk.sim_key(Key::Tab);
    tk.sim_text("héllo 👍🏽 世界");
    assert_eq!(tk.entry_get(entry).as_deref(), Some("héllo 👍🏽 世界"));
    tk.sim_key(Key::Left);
    tk.sim_key(Key::Backspace);
    assert_eq!(value.get(&tk).as_deref(), Some("héllo 👍🏽 界"));
    tk.sim_key(Key::Delete);
    assert_eq!(value.get(&tk).as_deref(), Some("héllo 👍🏽 "));
    tk.sim_key(Key::Left);
    tk.sim_key(Key::Backspace);
    assert_eq!(value.get(&tk).as_deref(), Some("héllo  "));
    tk.sim_key(Key::Home);
    tk.sim_key(Key::Right);
    tk.sim_key(Key::Delete);
    assert_eq!(value.get(&tk).as_deref(), Some("hllo  "));
    tk.sim_key(Key::End);
    tk.sim_text("e\u{0301}");
    tk.sim_key(Key::Left);
    tk.sim_key(Key::Delete);
    assert_eq!(value.get(&tk).as_deref(), Some("hllo  "));
    tk.sim_text("👩🏽\u{200d}💻");
    tk.sim_key(Key::Backspace);
    assert_eq!(value.get(&tk).as_deref(), Some("hllo  "));
    tk.sim_text("x\u{fe0f}");
    tk.sim_key(Key::Backspace);
    assert_eq!(value.get(&tk).as_deref(), Some("hllo  "));
}

/// Verifies external variable changes and large input are handled safely by an Entry.
#[test]
fn external_changes_controls_and_large_input_are_safe() {
    let mut tk = Tk::new();
    let root = tk.root();
    let value = tk.string_var("long original text");
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.config(entry, Opt::TextVariable(value));
    tk.pack(entry, PackOpts::default());
    tk.sim_key(Key::Tab);
    tk.sim_key(Key::End);
    value.set(&mut tk, "é".into());
    tk.sim_key(Key::Backspace);
    assert_eq!(value.get(&tk).as_deref(), Some(""));
    tk.sim_text("\n\t\r\u{0000}a");
    assert_eq!(value.get(&tk).as_deref(), Some("a"));
    tk.sim_text(&"界".repeat(30_000));
    let text = tk.entry_get(entry).expect("text");
    assert!(text.len() <= 65_536);
    assert!(text.is_char_boundary(text.len()));
    tk.sim_key(Key::End);
    for _ in 0..20 {
        tk.sim_key(Key::Left);
        tk.sim_key(Key::Delete);
    }
    assert!(tk.entry_get(entry).expect("text").len() < text.len());
}

/// Checks that long entry text is clipped and the caret scrolls the view.
#[test]
fn entry_text_is_clipped_and_caret_can_scroll() {
    let mut tk = Tk::new();
    let root = tk.root();
    let entry = Entry::new(&mut tk, root).expect("entry");
    tk.config(entry, Opt::Width(4));
    tk.pack(entry, PackOpts::default());
    tk.sim_key(Key::Tab);
    tk.sim_text("a very long single line 👍🏽");
    let end = tk.render_to_pixmap(220, 80).expect("image");
    tk.sim_key(Key::Home);
    let home = tk.render_to_pixmap(220, 80).expect("image");
    assert_eq!(end.width(), home.width());
    let rect = tk.widget_rect(entry).expect("rect");
    // Outside the Entry, moving the caret cannot alter pixels.
    for (index, (a, b)) in end.data().chunks_exact(4).zip(home.data().chunks_exact(4)).enumerate() {
        let x = (index % 220) as f32;
        let y = (index / 220) as f32;
        if !rect.contains(x, y) {
            assert_eq!(a, b, "px {x},{y} rect {rect:?}");
        }
    }
}
