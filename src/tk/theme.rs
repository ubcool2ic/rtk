//! Semantic theme accessors on [`Tk`]: role-based colours derived from the
//! active tokens and theme, the colourblind-safe palette override, and
//! following the operating system's light/dark appearance.

use super::*;
use crate::theme::{ColorRole, SemanticTokens, SystemTheme};

impl Tk {
    /// Returns the semantic tokens derived from the active tokens and theme.
    ///
    /// When colourblind-safe mode is enabled the result has been passed through
    /// [`SemanticTokens::colorblind_safe`].
    pub fn semantic_tokens(&self) -> SemanticTokens {
        let tokens = SemanticTokens::from_tokens(self.tokens(), self.theme);
        if self.colorblind_safe {
            tokens.colorblind_safe()
        } else {
            tokens
        }
    }

    /// Looks up the colour for a semantic [`ColorRole`] in the active theme.
    pub fn color(&self, role: ColorRole) -> Color {
        self.semantic_tokens().color(role)
    }

    /// Enables or disables the colourblind-safe palette override.
    ///
    /// When enabled, [`Tk::semantic_tokens`] returns a colourblind-safe variant
    /// and the live palette's `danger` colour is replaced by the semantic
    /// danger colour, so widgets that draw `palette.danger` follow the mode.
    /// The mode survives [`Tk::set_theme`] and [`Tk::set_tokens`].
    pub fn set_colorblind_safe(&mut self, on: bool) {
        if self.colorblind_safe == on {
            return;
        }
        self.colorblind_safe = on;
        self.apply_colorblind_palette();
        self.dirty = true;
    }

    /// Returns true when the colourblind-safe palette override is active.
    pub fn colorblind_safe(&self) -> bool {
        self.colorblind_safe
    }

    /// Rewrites the live palette's `danger` colour to match the semantic danger
    /// colour when colourblind-safe mode is on, or restores the theme's own
    /// danger colour when it is off.
    ///
    /// The theme's own danger colour is captured into [`Tk::base_danger`] the
    /// first time the override is applied to a given token set (and dropped by
    /// [`Tk::set_theme`]/[`Tk::set_tokens`] when fresh tokens are installed), so
    /// turning the mode off restores the original colour rather than deriving it
    /// from the already-overridden live tokens.
    pub(crate) fn apply_colorblind_palette(&mut self) {
        let mut tokens = self.style_engine.tokens().clone();
        let base = SemanticTokens::from_tokens(&tokens, self.theme);
        let danger = if self.colorblind_safe {
            // Capture the theme's own danger before it is overwritten.
            let original = *self.base_danger.get_or_insert(base.colors.danger);
            let _ = original;
            base.colorblind_safe().colors.danger
        } else {
            // Restore the captured original, falling back to the live tokens
            // when nothing was captured (mode never turned on for this token set).
            self.base_danger.take().unwrap_or(base.colors.danger)
        };
        // Always write the value back: the theme's own danger colour may already
        // equal the colourblind-safe one (e.g. modern_dark), and the mode must
        // still be observable through the live palette.
        tokens.palette.danger = danger;
        self.style_engine.set_tokens(tokens);
        self.dirty = true;
    }

    /// Starts following the operating system's light/dark appearance.
    ///
    /// `light` is applied when the system prefers a light appearance and `dark`
    /// when it prefers a dark one. If the system preference is already known it
    /// is applied immediately. An explicit [`Tk::set_theme`] call by the
    /// application does **not** disable following; call
    /// [`Tk::unfollow_system_theme`] to stop.
    pub fn follow_system_theme(&mut self, light: Theme, dark: Theme) {
        self.follow_system = Some((light, dark));
        if let Some(system) = self.system_theme {
            let target = match system {
                SystemTheme::Light => light,
                SystemTheme::Dark => dark,
            };
            self.set_theme(target);
        }
    }

    /// Stops following the operating system's appearance preference.
    pub fn unfollow_system_theme(&mut self) {
        self.follow_system = None;
    }

    /// Returns true when the toolkit is following the system appearance.
    pub fn is_following_system_theme(&self) -> bool {
        self.follow_system.is_some()
    }

    /// Returns the last system light/dark preference reported by the backend.
    pub fn system_theme(&self) -> Option<SystemTheme> {
        self.system_theme
    }

    /// Records the system light/dark preference and, when following, applies
    /// the matching theme.
    ///
    /// This is the single entry point used by the real window backend (on
    /// window creation and on `ThemeChanged` events) and by headless tests.
    pub fn sim_system_theme(&mut self, theme: SystemTheme) {
        self.system_theme = Some(theme);
        if let Some((light, dark)) = self.follow_system {
            let target = match theme {
                SystemTheme::Light => light,
                SystemTheme::Dark => dark,
            };
            self.set_theme(target);
        }
    }
}
