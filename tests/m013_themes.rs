//! Milestone 0.13 step 2: WCAG AA palette checks, colourblind-safe mode and
//! following the system light/dark appearance. All tests are headless.

use rtk::theme::contrast_ratio;
use rtk::{ColorRole, SystemTheme, Theme, Tk, Tokens};

/// Every built-in token preset paired with a human-readable name.
fn all_presets() -> Vec<(&'static str, Tokens)> {
    vec![
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("fluent", Tokens::fluent()),
        ("material", Tokens::material()),
        ("high_contrast", Tokens::high_contrast()),
        ("modern_dark", Tokens::modern_dark()),
        ("modern_light", Tokens::modern_light()),
    ]
}

/// Asserts that `a`/`b` reach at least `min` contrast, with a descriptive message.
fn assert_contrast(theme: &str, what: &str, a: rtk::Color, b: rtk::Color, min: f32) {
    let ratio = contrast_ratio(a, b);
    assert!(
        ratio >= min,
        "theme {theme}: {what} contrast {ratio:.2} < {min} ({a:?} vs {b:?})"
    );
}

/// Every preset meets the WCAG AA contrast requirements for text, muted text,
/// accent, focus ring, accent foreground and danger.
#[test]
fn all_presets_meet_wcag_aa() {
    for (name, tokens) in all_presets() {
        let p = tokens.palette;
        assert_contrast(name, "fg/bg", p.fg, p.bg, 4.5);
        assert_contrast(name, "fg/surface", p.fg, p.surface, 4.5);
        assert_contrast(name, "muted/bg", p.muted, p.bg, 4.5);
        assert_contrast(name, "muted/surface", p.muted, p.surface, 4.5);
        assert_contrast(name, "accent/bg", p.accent, p.bg, 3.0);
        assert_contrast(name, "focus/bg", p.focus, p.bg, 3.0);
        assert_contrast(name, "accent_fg/accent", p.accent_fg, p.accent, 4.5);
        assert_contrast(name, "danger/bg", p.danger, p.bg, 4.5);
    }
}

/// The high-contrast preset additionally reaches AAA (7:1) for text and muted text.
#[test]
fn high_contrast_meets_aaa() {
    let p = Tokens::high_contrast().palette;
    assert_contrast("high_contrast", "fg/bg", p.fg, p.bg, 7.0);
    assert_contrast("high_contrast", "muted/bg", p.muted, p.bg, 7.0);
}

/// Toggling colourblind-safe mode changes the semantic danger colour and the
/// mode survives a subsequent `set_theme`.
#[test]
fn colorblind_toggle_changes_danger_and_survives_set_theme() {
    let mut tk = Tk::new();
    assert!(!tk.colorblind_safe());
    let normal = tk.color(ColorRole::Danger);

    tk.set_colorblind_safe(true);
    assert!(tk.colorblind_safe());
    let safe = tk.color(ColorRole::Danger);
    assert_ne!(normal, safe, "colourblind-safe danger should differ");

    tk.set_theme(Theme::Dark);
    assert!(tk.colorblind_safe(), "mode must survive set_theme");
    let safe_dark = tk.color(ColorRole::Danger);
    // The live palette's danger follows the semantic danger.
    assert_eq!(tk.tokens().palette.danger, safe_dark);

    tk.set_colorblind_safe(false);
    assert!(!tk.colorblind_safe());
    assert_ne!(tk.color(ColorRole::Danger), safe_dark);
}

/// Following the system theme applies the matching theme on each report.
#[test]
fn follow_system_theme_switches_theme() {
    let mut tk = Tk::new();
    tk.follow_system_theme(Theme::Light, Theme::Dark);
    assert!(tk.is_following_system_theme());

    tk.sim_system_theme(SystemTheme::Dark);
    assert_eq!(tk.theme(), Theme::Dark);
    assert_eq!(tk.system_theme(), Some(SystemTheme::Dark));

    tk.sim_system_theme(SystemTheme::Light);
    assert_eq!(tk.theme(), Theme::Light);
    assert_eq!(tk.system_theme(), Some(SystemTheme::Light));
}

/// Unfollowing stops theme changes but keeps reporting the last system value.
#[test]
fn unfollow_stops_theme_changes() {
    let mut tk = Tk::new();
    tk.follow_system_theme(Theme::Light, Theme::Dark);
    tk.sim_system_theme(SystemTheme::Light);
    assert_eq!(tk.theme(), Theme::Light);

    tk.unfollow_system_theme();
    assert!(!tk.is_following_system_theme());
    tk.sim_system_theme(SystemTheme::Dark);
    assert_eq!(tk.theme(), Theme::Light, "theme must not change after unfollow");
    assert_eq!(tk.system_theme(), Some(SystemTheme::Dark));
}

/// An explicit `set_theme` does not disable following the system theme.
#[test]
fn explicit_set_theme_keeps_following() {
    let mut tk = Tk::new();
    tk.follow_system_theme(Theme::Light, Theme::Dark);
    tk.set_theme(Theme::Material);
    assert!(tk.is_following_system_theme());
    tk.sim_system_theme(SystemTheme::Dark);
    assert_eq!(tk.theme(), Theme::Dark);
}
