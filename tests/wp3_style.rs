//! WP3 Style/theme regression tests (B4, B5, B19, B21, P2, L1).
//!
//! Probe cases for B4 and B5 are ported from
//! `/workspace/rtk-audit-notes/zz_audit_probe.rs`.

use rtk::{Button, Color, Frame, Label, Opt, PackOpts, State, StyleSpec, Switch, Theme, Tk};

/// B4: a Heading inside a Card/Sidebar must keep the Heading size/weight,
/// not inherit the frame's 14px/400 after its own named style is applied.
#[test]
fn heading_inside_card_keeps_size() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lone = Label::new(&mut tk, root, "H").unwrap();
    tk.config(lone, Opt::Style("Heading".into()));
    let card = Frame::new(&mut tk, root).unwrap();
    tk.config(card, Opt::Style("Card".into()));
    let h = Label::new(&mut tk, card, "H").unwrap();
    tk.config(h, Opt::Style("Heading".into()));
    let a = tk.resolved_style(lone).unwrap();
    let b = tk.resolved_style(h).unwrap();
    assert_eq!(a.font_size, b.font_size);
    assert_eq!(a.font_weight, b.font_weight);
}

/// B4: Frame inheritance must not clobber a named Accent.TButton foreground.
#[test]
fn accent_button_inside_card_keeps_fg() {
    let mut tk = Tk::new();
    let root = tk.root();
    let lone = Button::new(&mut tk, root, "A").unwrap();
    tk.config(lone, Opt::Style("Accent.TButton".into()));
    let card = Frame::new(&mut tk, root).unwrap();
    tk.config(card, Opt::Style("Card".into()));
    let inner = Button::new(&mut tk, card, "A").unwrap();
    tk.config(inner, Opt::Style("Accent.TButton".into()));
    let a = tk.resolved_style(lone).unwrap();
    let b = tk.resolved_style(inner).unwrap();
    assert_eq!(a.fg, b.fg);
}

/// B5: disabled foreground must follow the active theme (not stay Light's muted).
#[test]
fn disabled_fg_follows_theme() {
    let mut tk = Tk::new();
    let root = tk.root();
    let b = Button::new(&mut tk, root, "x").unwrap();
    tk.pack(b, Default::default());
    tk.config(b, Opt::Disabled(true));
    let light = tk.resolved_style(b).unwrap().fg;
    tk.set_theme(Theme::HighContrast);
    let hc = tk.resolved_style(b).unwrap().fg;
    tk.set_theme(Theme::Dark);
    let dark = tk.resolved_style(b).unwrap().fg;
    assert_ne!(light, hc, "disabled fg did not change with HighContrast");
    assert_ne!(light, dark, "disabled fg did not change with Dark");
}

/// B5: a user `style_map` on `:disabled` must survive a theme change.
#[test]
fn user_disabled_map_survives_theme_change() {
    let mut tk = Tk::new();
    let root = tk.root();
    let b = Button::new(&mut tk, root, "x").unwrap();
    tk.config(b, Opt::Disabled(true));
    let custom = Color::rgb(1, 2, 3);
    tk.style_map(
        "TButton",
        State::DISABLED,
        rtk::StyleSpec {
            fg: Some(custom),
            ..Default::default()
        },
    );
    assert_eq!(tk.resolved_style(b).unwrap().fg, custom);
    tk.set_theme(Theme::Dark);
    assert_eq!(tk.resolved_style(b).unwrap().fg, custom);
}

/// B19: a checked Switch must include State::CHECKED so TSwitch:checked rules apply.
#[test]
fn switch_checked_state_applies_stylesheet() {
    let mut tk = Tk::new();
    let root = tk.root();
    let on_var = tk.bool_var(true);
    let off_var = tk.bool_var(false);
    let on = Switch::new(&mut tk, root, "On", on_var).unwrap();
    let off = Switch::new(&mut tk, root, "Off", off_var).unwrap();
    tk.load_stylesheet("TSwitch:checked { color: #ff0000; }").unwrap();
    assert_eq!(tk.resolved_style(on).unwrap().fg, Color::rgb(255, 0, 0));
    assert_ne!(tk.resolved_style(off).unwrap().fg, Color::rgb(255, 0, 0));
}

/// B21: negative or non-finite transition durations and negative shadow blur
/// must be rejected, not saturated.
#[test]
fn css_rejects_invalid_transition_and_shadow() {
    let mut tk = Tk::new();
    assert!(
        tk.load_stylesheet("TButton { transition: -1s; }").is_err(),
        "negative transition should be rejected"
    );
    assert!(
        tk.load_stylesheet("TButton { transition: infs; }").is_err(),
        "non-finite transition should be rejected"
    );
    assert!(
        tk.load_stylesheet("TButton { box-shadow: 0 0 -1px #000; }").is_err(),
        "negative shadow blur should be rejected"
    );
    assert!(
        tk.load_stylesheet("TButton { transition: 1e30s; }").is_err(),
        "out-of-range seconds must be rejected, not saturated"
    );
    assert!(
        tk.load_stylesheet("TButton { transition: 1e30ms; }").is_err(),
        "out-of-range milliseconds must be rejected, not saturated"
    );
    assert!(tk.load_stylesheet("TButton { transition: 120ms; }").is_ok());
    assert!(tk.load_stylesheet("TButton { transition: 0.12s; }").is_ok());
    assert!(tk.load_stylesheet("TButton { box-shadow: 0 1px 2px #000; }").is_ok());
}

fn pixmap_has_rgb(pixmap: &tiny_skia::Pixmap, r: u8, g: u8, b: u8) -> bool {
    pixmap.data().chunks_exact(4).any(|p| p[0] == r && p[1] == g && p[2] == b)
}

/// L1: unload_image / update_image_rgba must replace or drop a registered image,
/// including the pixels observed when that image is shown on a Button.
#[test]
fn image_unload_and_update() {
    let mut tk = Tk::new();
    tk.geometry(120, 60);
    let red = [255u8, 0, 0, 255].repeat(16 * 16);
    let id = tk.register_image_rgba(16, 16, &red).expect("register");
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "With Img").unwrap();
    tk.config(btn, Opt::Image(id));
    tk.pack(btn, PackOpts::default());

    let pixmap = tk.render_to_pixmap(120, 60).unwrap();
    assert!(pixmap_has_rgb(&pixmap, 255, 0, 0), "registered red image must appear");

    let green = [0u8, 255, 0, 255].repeat(16 * 16);
    assert!(tk.update_image_rgba(id, 16, 16, &green));
    let pixmap = tk.render_to_pixmap(120, 60).unwrap();
    assert!(pixmap_has_rgb(&pixmap, 0, 255, 0), "updated image must show new pixels");
    assert!(!pixmap_has_rgb(&pixmap, 255, 0, 0), "old red pixels must be gone after update");

    assert!(!tk.update_image_rgba(id, 16, 16, &[0u8; 3]), "wrong length must fail");
    assert!(tk.unload_image(id));
    let pixmap = tk.render_to_pixmap(120, 60).unwrap();
    assert!(!pixmap_has_rgb(&pixmap, 0, 255, 0), "unloaded image must not still paint");
    assert!(!pixmap_has_rgb(&pixmap, 255, 0, 0), "unloaded image must not restore old pixels");
    assert!(!tk.unload_image(id), "second unload must fail");
    assert!(!tk.update_image_rgba(id, 16, 16, &green), "update after unload must fail");
}

/// P2: there is no resolved-style cache; resolved_style must still recompute
/// after a theme change when asked repeatedly.
#[test]
fn resolved_style_updates_after_theme_change() {
    let mut tk = Tk::new();
    let root = tk.root();
    let b = Button::new(&mut tk, root, "x").unwrap();
    let light = tk.resolved_style(b).unwrap().fg;
    let _ = tk.resolved_style(b);
    tk.set_theme(Theme::Dark);
    let dark = tk.resolved_style(b).unwrap().fg;
    assert_ne!(light, dark);
}


/// User hover/pressed/focus/selected maps on built-in classes must survive
/// a theme rebuild, not only `:disabled`.
#[test]
fn user_hover_map_survives_theme_change() {
    let mut tk = Tk::new();
    tk.geometry(200, 200);
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Interactive").unwrap();
    tk.pack(btn, PackOpts::default());
    let custom = Color::rgb(11, 22, 33);
    tk.style_map(
        "TButton",
        State::HOVER,
        StyleSpec {
            bg: Some(custom),
            ..Default::default()
        },
    );
    tk.set_theme(Theme::Dark);
    let rect = tk.widget_rect(btn).unwrap();
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert!(tk.widget_state(btn).contains(State::HOVER));
    assert_eq!(tk.resolved_style(btn).unwrap().bg, custom);
}
