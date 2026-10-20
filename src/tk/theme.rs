//! Semantic theme accessors on [`Tk`]: role-based colours derived from the
//! active tokens and theme.

use super::*;
use crate::theme::{ColorRole, SemanticTokens};

impl Tk {
    /// Returns the semantic tokens derived from the active tokens and theme.
    pub fn semantic_tokens(&self) -> SemanticTokens {
        SemanticTokens::from_tokens(self.tokens(), self.theme)
    }

    /// Looks up the colour for a semantic [`ColorRole`] in the active theme.
    pub fn color(&self, role: ColorRole) -> Color {
        self.semantic_tokens().color(role)
    }
}
