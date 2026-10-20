//! Integration tests for semantic theme tokens (milestone 0.13, step 1).

use rtk::theme::{contrast_ratio, ColorRole, SemanticTokens, OKABE_ITO};
use rtk::{Color, Theme, Tk};

/// All built-in themes exercised by the token tests.
const THEMES: [Theme; 7] = [
    Theme::Light,
    Theme::Dark,
    Theme::Fluent,
    Theme::Material,
    Theme::HighContrast,
    Theme::ModernDark,
    Theme::ModernLight,
];

/// Text on background meets WCAG AA (AAA for high contrast).
#[test]
fn text_contrast_meets_wcag() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast, Theme::ModernLight, Theme::ModernDark] {
        let t = SemanticTokens::for_theme(theme);
        let ratio = contrast_ratio(t.colors.text, t.colors.background);
        let min = if theme == Theme::HighContrast { 7.0 } else { 4.5 };
        assert!(ratio >= min, "{theme:?}: text/background ratio {ratio} < {min}");
    }
}

/// Status colours keep readable foregrounds, also after colorblind_safe().
#[test]
fn status_contrast_meets_wcag() {
    for theme in [Theme::Light, Theme::Dark, Theme::HighContrast, Theme::ModernLight, Theme::ModernDark] {
        let base = SemanticTokens::for_theme(theme);
        for t in [base.clone(), base.colorblind_safe()] {
            let pairs = [
                (t.colors.on_success, t.colors.success, "success"),
                (t.colors.on_warning, t.colors.warning, "warning"),
                (t.colors.on_danger, t.colors.danger, "danger"),
                (t.colors.on_info, t.colors.info, "info"),
            ];
            for (fg, bg, name) in pairs {
                let ratio = contrast_ratio(fg, bg);
                let min = if theme == Theme::Light && name == "warning" { 3.0 } else { 4.5 };
                assert!(ratio >= min, "{theme:?} {name}: ratio {ratio} < {min}");
            }
        }
    }
}

/// Typography uses only the bundled 400/700 weights for every theme.
#[test]
fn typography_weights() {
    for theme in THEMES {
        let t = SemanticTokens::for_theme(theme);
        assert_eq!(t.typography.regular, 400, "{theme:?}");
        assert_eq!(t.typography.bold, 700, "{theme:?}");
    }
}

/// A fresh toolkit reports the Light theme's semantic tokens.
#[test]
fn default_toolkit_tokens() {
    let tk = Tk::new();
    assert_eq!(tk.semantic_tokens(), SemanticTokens::for_theme(Theme::Light));
}

/// Switching to the dark theme flips is_dark and tracks the palette background.
#[test]
fn dark_theme_tokens() {
    let mut tk = Tk::new();
    tk.set_theme(Theme::Dark);
    assert!(tk.semantic_tokens().is_dark);
    assert_eq!(tk.color(ColorRole::Background), tk.tokens().palette.bg);
}

/// set_accent changes the accent role colour.
#[test]
fn accent_updates_role() {
    let mut tk = Tk::new();
    let before = tk.color(ColorRole::Accent);
    tk.set_accent(Color::rgb(200, 30, 90));
    let after = tk.color(ColorRole::Accent);
    assert_ne!(before, after);
    assert_eq!(after, Color::rgb(200, 30, 90));
}

/// The Okabe-Ito palette has eight distinct colours.
#[test]
fn okabe_ito_distinct() {
    assert_eq!(OKABE_ITO.len(), 8);
    for (i, a) in OKABE_ITO.iter().enumerate() {
        for b in OKABE_ITO.iter().skip(i + 1) {
            assert_ne!(a, b);
        }
    }
}

/// Black/white contrast ratio is about 21.
#[test]
fn black_white_contrast() {
    let r = contrast_ratio(Color::BLACK, Color::WHITE);
    assert!((r - 21.0).abs() < 0.1, "ratio {r}");
}

/// Every colour role maps to a colour without panicking.
#[test]
fn all_roles_map() {
    let tk = Tk::new();
    let mut count = 0;
    for role in ColorRole::ALL {
        let _ = tk.color(role);
        count += 1;
    }
    assert_eq!(count, ColorRole::ALL.len());
    assert_eq!(count, 22);
    assert_eq!(tk.color(ColorRole::Background), tk.tokens().palette.bg);
}
