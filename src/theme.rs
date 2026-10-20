//! Semantic theme tokens: role-based colours, spacing, radii, typography,
//! colourblind-safe chart palettes and WCAG contrast helpers.
//!
//! This module is purely additive: it derives a [`SemanticTokens`] view from
//! the existing [`Tokens`] and [`Theme`] without changing any rendering
//! behaviour. Widgets and stylesheets can migrate to semantic roles over time.

use crate::style::{blend, Tokens};
use crate::widgets::{Color, Theme};

/// Role-based colour set used by semantic styling.
///
/// Every field is a concrete [`Color`]; use [`SemanticTokens::color`] to look
/// one up by [`ColorRole`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SemanticColors {
    /// Window/page background.
    pub background: Color,
    /// Default surface (cards, panels).
    pub surface: Color,
    /// Raised surface (popovers, hovered rows).
    pub surface_raised: Color,
    /// Primary text colour.
    pub text: Color,
    /// Secondary/muted text colour.
    pub text_muted: Color,
    /// Text colour for disabled controls.
    pub text_disabled: Color,
    /// Brand accent colour.
    pub accent: Color,
    /// Text/icon colour drawn on top of `accent`.
    pub on_accent: Color,
    /// Default border colour.
    pub border: Color,
    /// Focus ring colour.
    pub focus_ring: Color,
    /// Success status colour.
    pub success: Color,
    /// Text/icon colour drawn on top of `success`.
    pub on_success: Color,
    /// Warning status colour.
    pub warning: Color,
    /// Text/icon colour drawn on top of `warning`.
    pub on_warning: Color,
    /// Danger/error status colour.
    pub danger: Color,
    /// Text/icon colour drawn on top of `danger`.
    pub on_danger: Color,
    /// Informational status colour.
    pub info: Color,
    /// Text/icon colour drawn on top of `info`.
    pub on_info: Color,
    /// Input field background.
    pub field: Color,
    /// Selection highlight background.
    pub selection: Color,
    /// Text colour drawn on top of `selection`.
    pub on_selection: Color,
    /// Shadow colour (usually translucent).
    pub shadow: Color,
}

/// Spacing scale in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Spacing {
    /// Extra-extra-small spacing.
    pub xxs: f32,
    /// Extra-small spacing.
    pub xs: f32,
    /// Small spacing.
    pub sm: f32,
    /// Medium spacing.
    pub md: f32,
    /// Large spacing.
    pub lg: f32,
    /// Extra-large spacing.
    pub xl: f32,
    /// Extra-extra-large spacing.
    pub xxl: f32,
}

/// Corner radius scale in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct Radii {
    /// No rounding (0).
    pub none: f32,
    /// Small radius.
    pub sm: f32,
    /// Medium radius.
    pub md: f32,
    /// Large radius.
    pub lg: f32,
    /// Fully rounded / pill radius (9999).
    pub pill: f32,
}

/// Typography scale and weights.
///
/// `regular` is always 400 and `bold` always 700 because the bundled Noto Sans
/// ships only those two weights.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Typography {
    /// Preferred font family, if any.
    pub family: Option<String>,
    /// Caption size.
    pub caption: f32,
    /// Body text size.
    pub body: f32,
    /// Title size.
    pub title: f32,
    /// Heading size.
    pub heading: f32,
    /// Display (largest) size.
    pub display: f32,
    /// Regular font weight (always 400).
    pub regular: u16,
    /// Bold font weight (always 700).
    pub bold: u16,
    /// Line height multiplier (always 1.4).
    pub line_height: f32,
}

/// A semantic colour role, one per [`SemanticColors`] field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ColorRole {
    /// [`SemanticColors::background`].
    Background,
    /// [`SemanticColors::surface`].
    Surface,
    /// [`SemanticColors::surface_raised`].
    SurfaceRaised,
    /// [`SemanticColors::text`].
    Text,
    /// [`SemanticColors::text_muted`].
    TextMuted,
    /// [`SemanticColors::text_disabled`].
    TextDisabled,
    /// [`SemanticColors::accent`].
    Accent,
    /// [`SemanticColors::on_accent`].
    OnAccent,
    /// [`SemanticColors::border`].
    Border,
    /// [`SemanticColors::focus_ring`].
    FocusRing,
    /// [`SemanticColors::success`].
    Success,
    /// [`SemanticColors::on_success`].
    OnSuccess,
    /// [`SemanticColors::warning`].
    Warning,
    /// [`SemanticColors::on_warning`].
    OnWarning,
    /// [`SemanticColors::danger`].
    Danger,
    /// [`SemanticColors::on_danger`].
    OnDanger,
    /// [`SemanticColors::info`].
    Info,
    /// [`SemanticColors::on_info`].
    OnInfo,
    /// [`SemanticColors::field`].
    Field,
    /// [`SemanticColors::selection`].
    Selection,
    /// [`SemanticColors::on_selection`].
    OnSelection,
    /// [`SemanticColors::shadow`].
    Shadow,
}

impl ColorRole {
    /// Every colour role, in [`SemanticColors`] field order.
    pub const ALL: [ColorRole; 22] = [
        ColorRole::Background,
        ColorRole::Surface,
        ColorRole::SurfaceRaised,
        ColorRole::Text,
        ColorRole::TextMuted,
        ColorRole::TextDisabled,
        ColorRole::Accent,
        ColorRole::OnAccent,
        ColorRole::Border,
        ColorRole::FocusRing,
        ColorRole::Success,
        ColorRole::OnSuccess,
        ColorRole::Warning,
        ColorRole::OnWarning,
        ColorRole::Danger,
        ColorRole::OnDanger,
        ColorRole::Info,
        ColorRole::OnInfo,
        ColorRole::Field,
        ColorRole::Selection,
        ColorRole::OnSelection,
        ColorRole::Shadow,
    ];
}

/// The operating system's light/dark appearance preference.
///
/// Reported by the window backend via [`crate::Tk::sim_system_theme`] and used
/// by [`crate::Tk::follow_system_theme`] to pick between a light and a dark
/// [`Theme`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemTheme {
    /// The system prefers a light appearance.
    Light,
    /// The system prefers a dark appearance.
    Dark,
}

/// Okabe-Ito colourblind-safe categorical palette (8 colours).
pub const OKABE_ITO: [Color; 8] = [
    Color::rgb(0, 0, 0),
    Color::rgb(230, 159, 0),
    Color::rgb(86, 180, 233),
    Color::rgb(0, 158, 115),
    Color::rgb(240, 228, 66),
    Color::rgb(0, 114, 178),
    Color::rgb(213, 94, 0),
    Color::rgb(204, 121, 167),
];

/// WCAG 2.x relative luminance of an sRGB colour (alpha ignored).
pub fn relative_luminance(c: Color) -> f32 {
    /// Linearises one sRGB channel in 0..=255 to 0..=1.
    fn channel(v: u8) -> f32 {
        let s = v as f32 / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

/// WCAG 2.x contrast ratio between two colours (always >= 1).
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Picks white or black, whichever contrasts more against `bg`.
fn best_on(bg: Color) -> Color {
    if contrast_ratio(Color::WHITE, bg) >= contrast_ratio(Color::BLACK, bg) {
        Color::WHITE
    } else {
        Color::BLACK
    }
}

/// Fully resolved semantic design tokens for one theme.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SemanticTokens {
    /// Role-based colours.
    pub colors: SemanticColors,
    /// Spacing scale.
    pub spacing: Spacing,
    /// Corner radius scale.
    pub radii: Radii,
    /// Typography scale.
    pub typography: Typography,
    /// Colourblind-safe categorical chart palette.
    pub chart: [Color; 8],
    /// True when the theme is dark.
    pub is_dark: bool,
    /// True when the theme is the high-contrast theme.
    pub high_contrast: bool,
}

impl SemanticTokens {
    /// Derives semantic tokens from legacy [`Tokens`] and the active [`Theme`].
    pub fn from_tokens(tokens: &Tokens, theme: Theme) -> Self {
        let p = tokens.palette;
        let background = p.bg;
        let is_dark = relative_luminance(background) < 0.2;
        let high_contrast = theme == Theme::HighContrast;

        let text_disabled = Color::rgba(
            blend(p.fg, p.bg, 0.45).r,
            blend(p.fg, p.bg, 0.45).g,
            blend(p.fg, p.bg, 0.45).b,
            255,
        );

        let (success, warning, info) = if high_contrast {
            (Color::rgb(0, 255, 0), Color::rgb(255, 255, 0), Color::rgb(0, 255, 255))
        } else if is_dark {
            (Color::rgb(74, 222, 128), Color::rgb(251, 191, 36), Color::rgb(56, 189, 248))
        } else {
            (Color::rgb(21, 128, 61), Color::rgb(180, 83, 9), Color::rgb(3, 105, 161))
        };
        let (on_success, on_warning, on_info) = if high_contrast {
            (Color::BLACK, Color::BLACK, Color::BLACK)
        } else if is_dark {
            (Color::rgb(15, 15, 15), Color::rgb(15, 15, 15), Color::rgb(15, 15, 15))
        } else {
            (Color::WHITE, Color::WHITE, Color::WHITE)
        };
        let danger = p.danger;
        let on_danger = best_on(danger);
        let accent = p.accent;
        let on_accent = if contrast_ratio(p.accent_fg, accent) < 4.5 {
            best_on(accent)
        } else {
            p.accent_fg
        };

        let s = tokens.spacing;
        let spacing = Spacing {
            xxs: s[0] / 2.0,
            xs: s[0],
            sm: s[1],
            md: s[2],
            lg: s[3],
            xl: s[4],
            xxl: s[4] * 4.0 / 3.0,
        };
        let radii = Radii {
            none: 0.0,
            sm: tokens.radius.sm,
            md: tokens.radius.md,
            lg: tokens.radius.lg,
            pill: 9999.0,
        };
        let ts = tokens.type_scale;
        let typography = Typography {
            family: tokens.font_family.clone(),
            caption: ts.small,
            body: ts.body,
            title: ts.title,
            heading: ts.heading,
            display: ts.heading * 1.333,
            regular: 400,
            bold: 700,
            line_height: 1.4,
        };

        let mut chart = OKABE_ITO;
        if is_dark {
            chart[0] = Color::rgb(255, 255, 255);
        }

        Self {
            colors: SemanticColors {
                background,
                surface: p.surface,
                surface_raised: p.surface2,
                text: p.fg,
                text_muted: p.muted,
                text_disabled,
                accent,
                on_accent,
                border: p.border,
                focus_ring: p.focus,
                success,
                on_success,
                warning,
                on_warning,
                danger,
                on_danger,
                info,
                on_info,
                field: p.field_bg,
                selection: p.select_bg,
                on_selection: p.select_fg,
                shadow: p.shadow,
            },
            spacing,
            radii,
            typography,
            chart,
            is_dark,
            high_contrast,
        }
    }

    /// Builds semantic tokens for a built-in [`Theme`] using its default tokens.
    pub fn for_theme(theme: Theme) -> Self {
        let tokens = match theme {
            Theme::Light => Tokens::light(),
            Theme::Dark => Tokens::dark(),
            Theme::Fluent => Tokens::fluent(),
            Theme::Material => Tokens::material(),
            Theme::HighContrast => Tokens::high_contrast(),
            Theme::ModernDark => Tokens::modern_dark(),
            Theme::ModernLight => Tokens::modern_light(),
        };
        Self::from_tokens(&tokens, theme)
    }

    /// Looks up the colour for a semantic [`ColorRole`].
    pub fn color(&self, role: ColorRole) -> Color {
        let c = &self.colors;
        match role {
            ColorRole::Background => c.background,
            ColorRole::Surface => c.surface,
            ColorRole::SurfaceRaised => c.surface_raised,
            ColorRole::Text => c.text,
            ColorRole::TextMuted => c.text_muted,
            ColorRole::TextDisabled => c.text_disabled,
            ColorRole::Accent => c.accent,
            ColorRole::OnAccent => c.on_accent,
            ColorRole::Border => c.border,
            ColorRole::FocusRing => c.focus_ring,
            ColorRole::Success => c.success,
            ColorRole::OnSuccess => c.on_success,
            ColorRole::Warning => c.warning,
            ColorRole::OnWarning => c.on_warning,
            ColorRole::Danger => c.danger,
            ColorRole::OnDanger => c.on_danger,
            ColorRole::Info => c.info,
            ColorRole::OnInfo => c.on_info,
            ColorRole::Field => c.field,
            ColorRole::Selection => c.selection,
            ColorRole::OnSelection => c.on_selection,
            ColorRole::Shadow => c.shadow,
        }
    }

    /// Returns a copy whose status colours are distinguishable for protan/deutan users.
    pub fn colorblind_safe(mut self) -> Self {
        let (success, warning, danger, info) = if self.high_contrast {
            (
                self.colors.success,
                self.colors.warning,
                self.colors.danger,
                self.colors.info,
            )
        } else if self.is_dark {
            (
                Color::rgb(0, 190, 140),
                Color::rgb(230, 159, 0),
                Color::rgb(240, 120, 40),
                Color::rgb(86, 180, 233),
            )
        } else {
            (
                Color::rgb(0, 158, 115),
                Color::rgb(230, 159, 0),
                Color::rgb(213, 94, 0),
                Color::rgb(0, 114, 178),
            )
        };
        self.colors.success = success;
        self.colors.on_success = best_on(success);
        self.colors.warning = warning;
        self.colors.on_warning = best_on(warning);
        self.colors.danger = danger;
        self.colors.on_danger = best_on(danger);
        self.colors.info = info;
        self.colors.on_info = best_on(info);
        self
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the semantic token helpers.

    use super::*;

    /// Relative luminance of white is 1 and of black is 0.
    #[test]
    fn luminance_extremes() {
        assert!((relative_luminance(Color::WHITE) - 1.0).abs() < 1e-4);
        assert!(relative_luminance(Color::BLACK).abs() < 1e-4);
    }

    /// Contrast between black and white is about 21.
    #[test]
    fn contrast_black_white() {
        let r = contrast_ratio(Color::BLACK, Color::WHITE);
        assert!((r - 21.0).abs() < 0.1, "ratio {r}");
    }

    /// The Okabe-Ito palette has eight distinct colours.
    #[test]
    fn okabe_ito_distinct() {
        for (i, a) in OKABE_ITO.iter().enumerate() {
            for b in OKABE_ITO.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }

    /// Every colour role resolves without panicking.
    #[test]
    fn all_roles_resolve() {
        let t = SemanticTokens::for_theme(Theme::Light);
        for role in ColorRole::ALL {
            let _ = t.color(role);
        }
    }
}
