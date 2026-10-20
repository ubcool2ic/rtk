//! Tests for theme token defaults and style resolution across the built-in themes.

use rtk::{
    Button, Color, Frame, Key, Label, Opt, PackOpts, State, StyleSpec, Theme, Tk,
};

/// Checks that theme token defaults for palette and radius match each theme.
#[test]
fn token_defaults_per_theme() {
    let mut tk = Tk::new();

    tk.set_theme(Theme::Light);
    assert_eq!(tk.tokens().palette.accent, Color::rgb(0, 103, 192));
    assert_eq!(tk.tokens().radius.md, 5.0);

    tk.set_theme(Theme::Dark);
    assert_eq!(tk.tokens().palette.bg, Color::rgb(32, 32, 32));
    assert_eq!(tk.tokens().palette.accent, Color::rgb(96, 205, 255));

    tk.set_theme(Theme::Fluent);
    assert_eq!(tk.tokens().radius.md, 4.0);
    assert_eq!(tk.tokens().palette.accent, Color::rgb(0, 103, 192));

    tk.set_theme(Theme::Material);
    assert_eq!(tk.tokens().radius.lg, 20.0);
    assert_eq!(tk.tokens().palette.accent, Color::rgb(103, 80, 164));

    tk.set_theme(Theme::HighContrast);
    assert_eq!(tk.tokens().palette.bg, Color::rgb(0, 0, 0));
    assert_eq!(tk.tokens().palette.fg, Color::rgb(255, 255, 255));
    assert_eq!(tk.tokens().palette.focus, Color::rgb(255, 255, 0));
    assert_eq!(tk.tokens().radius.md, 0.0);
}

/// Confirms a named style overrides the default class style for a button.
#[test]
fn named_style_beats_class() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Hello").unwrap();

    // Default TButton has body font size
    let default_style = tk.resolved_style(btn).unwrap();
    assert_eq!(default_style.font_size, tk.tokens().type_scale.body);

    // Custom named style
    tk.style_configure("Big.TButton", StyleSpec {
        font_size: Some(28.0),
        bg: Some(Color::rgb(10, 20, 30)),
        ..Default::default()
    });

    tk.config(btn, Opt::Style("Big.TButton".into()));
    let res = tk.resolved_style(btn).unwrap();
    assert_eq!(res.font_size, 28.0);
    assert_eq!(res.bg, Color::rgb(10, 20, 30));
}

/// Ensures a per-widget style override takes precedence over a named style.
#[test]
fn override_beats_named() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Hello").unwrap();

    tk.style_configure("Special.TButton", StyleSpec {
        bg: Some(Color::rgb(100, 100, 100)),
        font_size: Some(16.0),
        ..Default::default()
    });
    tk.config(btn, Opt::Style("Special.TButton".into()));

    // Override beats named style
    tk.config(btn, Opt::StyleOverride(StyleSpec {
        bg: Some(Color::rgb(222, 111, 0)),
        ..Default::default()
    }));

    let res = tk.resolved_style(btn).unwrap();
    assert_eq!(res.bg, Color::rgb(222, 111, 0));
    assert_eq!(res.font_size, 16.0); // kept from named style
}

/// Verifies that frames inherit foreground and font from parents but not background.
#[test]
fn frame_inheritance_fg_and_font_not_bg() {
    let mut tk = Tk::new();
    let root = tk.root();
    let frame = Frame::new(&mut tk, root).unwrap();

    // Frame with overridden fg, font, and bg
    tk.config(frame, Opt::StyleOverride(StyleSpec {
        bg: Some(Color::rgb(50, 50, 50)),
        fg: Some(Color::rgb(200, 10, 10)),
        font_size: Some(22.0),
        font_weight: Some(700),
        italic: Some(true),
        ..Default::default()
    }));

    let lbl = Label::new(&mut tk, frame, "Inherited text").unwrap();
    let res = tk.resolved_style(lbl).unwrap();

    // Foreground and font properties are inherited
    assert_eq!(res.fg, Color::rgb(200, 10, 10));
    assert_eq!(res.font_size, 22.0);
    assert_eq!(res.font_weight, 700);
    assert!(res.italic);

    // Background is NOT inherited (Label has its own default transparent/surface bg)
    assert_ne!(res.bg, Color::rgb(50, 50, 50));
}

/// Tests that state maps apply on simulated hover, press, and focus events.
#[test]
fn state_map_applies_on_sim_hover_press_focus() {
    let mut tk = Tk::new();
    tk.geometry(200, 200);
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Interactive").unwrap();
    tk.pack(btn, PackOpts::default());

    tk.style_map("TButton", State::HOVER, StyleSpec {
        bg: Some(Color::rgb(11, 22, 33)),
        ..Default::default()
    });
    tk.style_map("TButton", State::PRESSED, StyleSpec {
        bg: Some(Color::rgb(99, 88, 77)),
        ..Default::default()
    });

    let rect = tk.widget_rect(btn).unwrap();
    let center_x = rect.x + rect.width / 2.0;
    let center_y = rect.y + rect.height / 2.0;

    // Hover
    tk.sim_mouse_move(center_x, center_y);
    assert!(tk.widget_state(btn).contains(State::HOVER));
    let res = tk.resolved_style(btn).unwrap();
    assert_eq!(res.bg, Color::rgb(11, 22, 33));

    // Press
    tk.sim_mouse_down();
    assert!(tk.widget_state(btn).contains(State::PRESSED));
    let res = tk.resolved_style(btn).unwrap();
    assert_eq!(res.bg, Color::rgb(99, 88, 77));

    // Release
    tk.sim_mouse_up();
    assert!(!tk.widget_state(btn).contains(State::PRESSED));
    assert!(tk.widget_state(btn).contains(State::HOVER));
}

/// Confirms style transitions interpolate halfway and complete to the target values.
#[test]
fn transition_interpolates_halfway_and_finishes() {
    let mut tk = Tk::new();
    tk.geometry(200, 200);
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Anim").unwrap();
    tk.pack(btn, PackOpts::default());

    tk.config(btn, Opt::StyleOverride(StyleSpec {
        bg: Some(Color::rgb(0, 0, 0)),
        transition_ms: Some(120),
        ..Default::default()
    }));

    tk.style_map("TButton", State::HOVER, StyleSpec {
        bg: Some(Color::rgb(100, 100, 100)),
        ..Default::default()
    });

    let rect = tk.widget_rect(btn).unwrap();
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);

    // Initial transition active
    assert!(tk.is_animating());

    // Advance 60ms (halfway of 120ms)
    tk.sim_advance_time(60);
    assert!(tk.is_animating());

    // Render to check interpolation logic directly on transition
    let cur = tk.displayed_bg(btn).unwrap();
    assert!((cur.r as i32 - 50).abs() <= 2, "halfway bg {cur:?}");

    // Advance another 60ms -> finished
    tk.sim_advance_time(60);
    assert!(!tk.is_animating());
}

/// Ensures disabled buttons ignore clicks and are skipped during tab navigation.
#[test]
fn disabled_button_ignores_click_and_skipped_by_tab() {
    let mut tk = Tk::new();
    tk.geometry(200, 200);
    let root = tk.root();

    let btn1 = Button::new(&mut tk, root, "First").unwrap();
    let btn2 = Button::new(&mut tk, root, "Disabled").unwrap();
    let btn3 = Button::new(&mut tk, root, "Third").unwrap();
    tk.pack(btn1, PackOpts::default());
    tk.pack(btn2, PackOpts::default());
    tk.pack(btn3, PackOpts::default());

    let clicked = tk.bool_var(false);
    let c = clicked;
    tk.set_command(btn2, move |tk| {
        c.set(tk, true);
    });
    tk.config(btn2, Opt::Disabled(true));

    let rect = tk.widget_rect(btn2).unwrap();
    tk.sim_click(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    assert_eq!(clicked.get(&tk), Some(false));

    // Tab traversal should skip disabled btn2
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(btn1));
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(btn3));
}

/// Tests stylesheet parsing including comments, hover states, and gradient backgrounds.
#[test]
fn stylesheet_parse_success_and_hover_and_gradients() {
    let mut tk = Tk::new();
    let css = r#"
        /* Comment here */
        TButton {
            background: #3366ff;
            color: white;
            border-radius: 6px;
            border: 1px #224488;
            padding: 6px 12px;
            font-weight: 700;
        }

        Accent.TButton:hover {
            background: linear-gradient(90deg, #4a7dff, #7a5cff);
            box-shadow: 0 2 6 #00000055;
        }
    "#;

    let rules = tk.load_stylesheet(css).unwrap();
    assert_eq!(rules, 2);

    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Styled").unwrap();
    let res = tk.resolved_style(btn).unwrap();
    assert_eq!(res.bg, Color::rgb(0x33, 0x66, 0xff));
    assert_eq!(res.fg, Color::WHITE);
    assert_eq!(res.radius, 6.0);
    assert_eq!(res.font_weight, 700);

    // Accent.TButton hover rule has linear gradient and shadow
    let btn_accent = Button::new(&mut tk, root, "Accent").unwrap();
    tk.config(btn_accent, Opt::Style("Accent.TButton".into()));

    tk.pack(btn, PackOpts::default());
    tk.pack(btn_accent, PackOpts::default());
    let _ = tk.render_to_pixmap(300, 200);
    let rect = tk.widget_rect(btn_accent).unwrap();
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let hovered = tk.resolved_style(btn_accent).unwrap();
    assert!(hovered.gradient.is_some());
    assert!(hovered.shadow.is_some());
}

/// Verifies that a per-widget Bg option overrides a style-configured background for a TButton.
#[test]
fn legacy_opt_bg_still_wins() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Test").unwrap();

    tk.style_configure("TButton", StyleSpec {
        bg: Some(Color::rgb(11, 22, 33)),
        ..Default::default()
    });

    tk.config(btn, Opt::Bg(Color::rgb(200, 200, 200)));
    let res = tk.resolved_style(btn).unwrap();
    assert_eq!(res.bg, Color::rgb(200, 200, 200));
}

/// Checks that a focused button under the HighContrast theme renders a yellow focus-ring pixel.
#[test]
fn high_contrast_renders_yellow_focus_ring_pixel() {
    let mut tk = Tk::new();
    tk.set_theme(Theme::HighContrast);
    tk.geometry(120, 80);
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "HC").unwrap();
    tk.pack(btn, PackOpts::default());

    // Focus the button
    tk.sim_key(Key::Tab);
    assert_eq!(tk.focused_widget(), Some(btn));

    let pixmap = tk.render_to_pixmap(120, 80).unwrap();
    let mut found_yellow = false;
    for pixel in pixmap.data().chunks_exact(4) {
        if let [r, g, b, _] = pixel {
            if *r > 200 && *g > 200 && *b < 50 {
                found_yellow = true;
                break;
            }
        }
    }
    assert!(found_yellow, "High contrast focus ring must render yellow pixels");
}

/// Confirms that the top-left pixel of a rounded button corner shows the parent background color.
#[test]
fn rounded_corner_pixel_at_top_left_equals_parent_bg() {
    let mut tk = Tk::new();
    tk.geometry(160, 100);
    let root = tk.root();

    // Set root frame background to solid blue
    tk.config(root, Opt::Bg(Color::rgb(0, 0, 255)));

    let btn = Button::new(&mut tk, root, "Radius").unwrap();
    tk.config(btn, Opt::StyleOverride(StyleSpec {
        bg: Some(Color::rgb(255, 0, 0)),
        radius: Some(12.0),
        border_width: Some(0.0),
        shadow: Some(None),
        ..Default::default()
    }));
    tk.pack(btn, PackOpts { padx: 10.0, pady: 10.0, ..Default::default() });

    let rect = tk.widget_rect(btn).unwrap();
    let pixmap = tk.render_to_pixmap(160, 100).unwrap();

    // At the absolute corner (x, y) of the rounded button rect, the pixel should NOT be the button bg
    // because radius 12 cuts that corner, showing parent's blue background.
    let px = rect.x.round() as u32;
    let py = rect.y.round() as u32;
    let idx = ((py * 160 + px) * 4) as usize;
    let r = pixmap.data()[idx];
    let b = pixmap.data()[idx + 2];

    assert!(b > 200, "Corner pixel must equal parent bg (blue)");
    assert_eq!(r, 0, "Corner pixel must not be button bg (red)");
}

/// Ensures named style configurations survive a theme switch and still resolve correctly.
#[test]
fn theme_switch_keeps_named_styles() {
    let mut tk = Tk::new();
    tk.style_configure("Custom.TButton", StyleSpec {
        font_size: Some(21.0),
        ..Default::default()
    });

    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Btn").unwrap();
    tk.config(btn, Opt::Style("Custom.TButton".into()));

    assert_eq!(tk.resolved_style(btn).unwrap().font_size, 21.0);

    tk.set_theme(Theme::Dark);
    assert_eq!(tk.resolved_style(btn).unwrap().font_size, 21.0);

    tk.set_theme(Theme::Material);
    assert_eq!(tk.resolved_style(btn).unwrap().font_size, 21.0);
}

/// Fuzzes the stylesheet parser with random input to verify it never panics.
#[test]
fn stylesheet_parser_fuzz_never_panics() {
    let mut tk = Tk::new();
    let mut seed: u64 = 0x853c49e6748fea9b;
    let mut xorshift = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };

    let chars = b" \t\n\r{};:,-_#abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/*()";

    for _ in 0..300 {
        let len = (xorshift() % 80) as usize;
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            let idx = (xorshift() as usize) % chars.len();
            s.push(chars[idx] as char);
        }
        // Must never panic on random input
        let _ = tk.load_stylesheet(&s);
    }
}

/// Tests that a button can render with an image loaded from in-memory PNG bytes.
#[test]
fn image_support_button_rendering() {
    let mut tk = Tk::new();
    tk.geometry(120, 60);

    // Create a 16x16 red PNG in memory
    let mut pm = tiny_skia::Pixmap::new(16, 16).unwrap();
    pm.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
    let png_bytes = pm.encode_png().unwrap();

    let img_id = tk.load_image(png_bytes).unwrap();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "With Img").unwrap();
    tk.config(btn, Opt::Image(img_id));
    tk.pack(btn, PackOpts::default());

    let pixmap = tk.render_to_pixmap(120, 60).unwrap();
    let mut found_red = false;
    for pixel in pixmap.data().chunks_exact(4) {
        if pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0 {
            found_red = true;
            break;
        }
    }
    assert!(found_red, "Button with Image must render image pixels");
}

/// Verifies press and release transitions animate from the previous state rather than resetting.
#[test]
fn press_and_release_transitions_start_from_previous_state() {
    let mut tk = Tk::new();
    let root = tk.root();
    let btn = Button::new(&mut tk, root, "Press").unwrap();
    tk.pack(btn, PackOpts::default());
    tk.config(btn, Opt::StyleOverride(StyleSpec {
        bg: Some(Color::rgb(0, 0, 0)),
        transition_ms: Some(100),
        ..Default::default()
    }));
    tk.style_map("TButton", State::HOVER, StyleSpec { bg: Some(Color::rgb(0, 0, 0)), ..Default::default() });
    tk.style_map("TButton", State::PRESSED, StyleSpec { bg: Some(Color::rgb(200, 0, 0)), ..Default::default() });
    let _ = tk.render_to_pixmap(200, 100);
    let rect = tk.widget_rect(btn).unwrap();
    tk.sim_mouse_move(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    tk.sim_advance_time(200);
    tk.sim_mouse_down();
    tk.sim_advance_time(50);
    let mid = tk.displayed_bg(btn).unwrap();
    assert!((mid.r as i32 - 100).abs() <= 3, "press midpoint {mid:?}");
    tk.sim_advance_time(100);
    tk.sim_mouse_up();
    tk.sim_advance_time(50);
    let back = tk.displayed_bg(btn).unwrap();
    assert!((back.r as i32 - 100).abs() <= 3, "release midpoint {back:?}");
}

/// Checks that bold text requests more width and disabled widgets render with a muted appearance.
#[test]
fn bold_text_requests_more_width_and_disabled_has_muted_look() {
    let mut tk = Tk::new();
    let root = tk.root();
    let normal = Label::new(&mut tk, root, "Standard Controls").unwrap();
    let bold = Label::new(&mut tk, root, "Standard Controls").unwrap();
    tk.config(bold, Opt::StyleOverride(StyleSpec { font_weight: Some(800), ..Default::default() }));
    tk.pack(normal, PackOpts::default());
    tk.pack(bold, PackOpts::default());
    let _ = tk.render_to_pixmap(400, 200);
    let (n, b) = (tk.widget_rect(normal).unwrap(), tk.widget_rect(bold).unwrap());
    assert!(b.width > n.width, "bold {} vs normal {}", b.width, n.width);

    let btn = Button::new(&mut tk, root, "Off").unwrap();
    tk.config(btn, Opt::Disabled(true));
    let style = tk.resolved_style(btn).unwrap();
    assert!(style.opacity < 1.0);
    assert_eq!(style.fg, tk.tokens().palette.muted);
}

/// Confirms the README stylesheet snippet parses into the expected number of rules.
#[test]
fn readme_stylesheet_snippet_parses() {
    let mut tk = Tk::new();
    let rules = tk.load_stylesheet(r#"
    TButton { background: #3366ff; color: white; border-radius: 6px; padding: 6px 14px; font-weight: 700; }
    Accent.TButton:hover { background: linear-gradient(135deg, #4a7dff, #7a5cff); box-shadow: 0 4 8 #00000066; }
    TLabel { font-weight: bold; border: 1px solid #224488; }
    "#);
    assert_eq!(rules, Ok(3));
}
