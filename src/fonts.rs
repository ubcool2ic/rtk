//! Bundled UI font and default sans-serif selection.
//!
//! rtk ships **Noto Sans** Regular (400) and Bold (700), SIL Open Font License 1.1
//! (see `fonts/OFL.txt`), compiled in with `include_bytes!`. Every built-in theme
//! draws with it, so labels look the same everywhere and never depend on which
//! weights the OS happens to have (Windows' Arial has no 500 face, which garbled
//! button labels before v0.8.2).
//!
//! Load order for the default sans-serif family:
//! 1. the bundled Noto Sans;
//! 2. Arial, if the bundled faces fail to load (a warning is printed once);
//! 3. the platform's usual sans-serif (Segoe UI, Helvetica, DejaVu Sans, ...),
//!    so Linux/macOS machines without Arial still work.
//!
//! System fonts stay loaded either way, so glyphs Noto Sans lacks (emoji, CJK)
//! still fall back to installed fonts.

use cosmic_text::fontdb::{Database, Family, Query, Source, Stretch, Style, Weight};
use std::sync::Arc;

/// Family name of the bundled font.
pub const BUNDLED_FAMILY: &str = "Noto Sans";

/// Bundled faces: Noto Sans Regular (400) and Bold (700), unhinted TTF.
pub(crate) const BUNDLED_FACES: [&[u8]; 2] = [
    include_bytes!("../fonts/NotoSans-Regular.ttf"),
    include_bytes!("../fonts/NotoSans-Bold.ttf"),
];

/// Platform sans-serif families tried (in order) when neither the bundled font nor Arial is available.
const PLATFORM_SANS: [&str; 10] = [
    "Segoe UI", "Helvetica Neue", "Helvetica", "SF Pro Text", "DejaVu Sans", "Liberation Sans", "Noto Sans", "Cantarell", "Ubuntu", "Roboto",
];

/// Which family became the default sans-serif.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultFont {
    /// The bundled Noto Sans (normal case).
    Bundled,
    /// Arial: the bundled font could not be loaded.
    Arial,
    /// Another installed sans-serif family (no bundled font and no Arial).
    Platform(String),
    /// No usable font at all (text is not drawn).
    Missing,
}

impl DefaultFont {
    /// Family name used for `Family::SansSerif`, if any.
    pub fn family(&self) -> Option<&str> {
        match self {
            DefaultFont::Bundled => Some(BUNDLED_FAMILY),
            DefaultFont::Arial => Some("Arial"),
            DefaultFont::Platform(name) => Some(name),
            DefaultFont::Missing => None,
        }
    }
}

/// True if `db` has a face of `family` at exactly `weight` (upright, normal width).
pub(crate) fn has_face(db: &Database, family: &str, weight: u16) -> bool {
    let query = Query { families: &[Family::Name(family)], weight: Weight(weight), stretch: Stretch::Normal, style: Style::Normal };
    db.query(&query).and_then(|id| db.face(id)).is_some_and(|face| face.weight == Weight(weight) && face.families.iter().any(|(n, _)| n == family))
}

/// True if `db` has any face whose family is `family`.
fn has_family(db: &Database, family: &str) -> bool {
    db.faces().any(|face| face.families.iter().any(|(n, _)| n.eq_ignore_ascii_case(family)))
}

/// Prints the bundled-font warning at most once per process.
fn warn_once() {
    static WARNED: std::sync::Once = std::sync::Once::new();
    WARNED.call_once(|| eprintln!("rtk: warning: bundled {BUNDLED_FAMILY} font could not be loaded; falling back to Arial / system sans-serif"));
}

/// Loads `bundled` into `db` and makes the best available family the default
/// sans-serif (bundled -> Arial -> platform sans). Tests pass broken bytes
/// here to exercise the fallback.
pub(crate) fn install(db: &mut Database, bundled: &[&'static [u8]]) -> DefaultFont {
    let mut loaded = !bundled.is_empty();
    for bytes in bundled {
        let data: Arc<dyn AsRef<[u8]> + Sync + Send> = Arc::new(*bytes);
        loaded &= !db.load_font_source(Source::Binary(data)).is_empty();
    }
    let choice = if loaded && has_face(db, BUNDLED_FAMILY, 400) && has_face(db, BUNDLED_FAMILY, 700) {
        DefaultFont::Bundled
    } else {
        warn_once();
        if has_family(db, "Arial") {
            DefaultFont::Arial
        } else if let Some(name) = PLATFORM_SANS.iter().find(|f| has_family(db, f)) {
            DefaultFont::Platform((*name).to_string())
        } else if let Some(name) = db.faces().next().and_then(|f| f.families.first()).map(|(n, _)| n.clone()) {
            DefaultFont::Platform(name)
        } else {
            DefaultFont::Missing
        }
    };
    if let Some(family) = choice.family() {
        db.set_sans_serif_family(family.to_string());
    }
    choice
}

#[cfg(test)]
mod tests {
    //! Bundled font parsing, fallback, and theme coverage.

    use super::*;
    use crate::style::{StyleEngine, Tokens};

    /// Every built-in theme preset.
    fn presets() -> Vec<(&'static str, Tokens)> {
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

    /// Style names the engine registers for every theme.
    const STYLES: [&str; 28] = [
        "TButton", "Accent.TButton", "Ghost.TButton", "Sidebar.TButton", "Sidebar", "Card", "Heading", "Subheading", "Muted", "Mono",
        "Chip.Success", "Chip.Warning", "Chip.Danger", "Chip.Info", "Chip.Neutral", "TLabel", "TEntry", "TFrame", "TCanvas", "TCombobox",
        "TListbox", "TNotebook", "TProgressbar", "TScale", "TScrollbar", "TSpinbox", "TText", "TCheckbutton",
    ];

    /// Both bundled faces parse into an empty database as Noto Sans 400 and 700.
    #[test]
    fn bundled_font_parses() {
        let mut db = Database::new();
        assert_eq!(install(&mut db, &BUNDLED_FACES), DefaultFont::Bundled);
        assert!(has_face(&db, BUNDLED_FAMILY, 400));
        assert!(has_face(&db, BUNDLED_FAMILY, 700));
        assert_eq!(db.family_name(&Family::SansSerif), BUNDLED_FAMILY);
    }

    /// Broken bundled bytes fall back to Arial, then a platform sans, without panicking.
    #[test]
    fn fallback_when_bundled_fails() {
        let broken: [&'static [u8]; 1] = [b"definitely not a font"];
        // Nothing else installed: no font at all.
        let mut empty = Database::new();
        assert_eq!(install(&mut empty, &broken), DefaultFont::Missing);
        // No Arial, but another known sans is installed: the platform path picks it.
        let mut db = Database::new();
        db.load_font_source(Source::Binary(Arc::new(BUNDLED_FACES[0])));
        let choice = install(&mut db, &broken);
        assert_eq!(choice, DefaultFont::Platform("Noto Sans".into()));
        assert_eq!(db.family_name(&Family::SansSerif), "Noto Sans");
        // Real system fonts: whatever is chosen must be an installed family.
        let mut sys = Database::new();
        sys.load_system_fonts();
        let choice = install(&mut sys, &broken);
        assert_ne!(choice, DefaultFont::Bundled);
        if let Some(f) = choice.family() {
            assert!(has_family(&sys, f), "fallback {f} is installed");
        }
    }

    /// Every style of every theme resolves to a real bundled face at its exact weight.
    #[test]
    fn every_theme_resolves_to_available_face() {
        let mut db = Database::new();
        install(&mut db, &BUNDLED_FACES);
        for (theme, tokens) in presets() {
            assert_eq!(tokens.font_family, None, "theme {theme} uses the bundled default font");
            let engine = StyleEngine::new(tokens);
            for name in STYLES {
                let Some(spec) = engine.lookup(name) else { continue };
                let family = spec.font_family.clone().flatten();
                let weight = spec.font_weight.unwrap_or(400);
                let fam = family.as_deref().filter(|f| !f.eq_ignore_ascii_case("monospace")).unwrap_or(BUNDLED_FAMILY);
                if family.as_deref().is_some_and(|f| f.eq_ignore_ascii_case("monospace")) {
                    continue; // monospace resolves through the system monospace family
                }
                assert!(has_face(&db, fam, weight), "theme {theme}: style {name} wants {fam} {weight}");
            }
        }
    }
}
