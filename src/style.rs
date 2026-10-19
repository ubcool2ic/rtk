//! Design tokens, styles, state cascades, transitions, and CSS stylesheet parser.

use std::collections::BTreeMap;
use std::fmt;

use crate::layout::Rect;
use crate::widgets::Color;

/// Flags representing the interaction state of a widget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct State(pub(crate) u8);

impl State {
    pub const HOVER: Self = Self(1 << 0);
    pub const PRESSED: Self = Self(1 << 1);
    pub const FOCUSED: Self = Self(1 << 2);
    pub const DISABLED: Self = Self(1 << 3);
    pub const CHECKED: Self = Self(1 << 4);
    pub const SELECTED: Self = Self(1 << 5);

    /// This set minus `other`.
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & 0b0011_1111)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for State {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for State {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub const fn all(v: f32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }

    pub const fn symmetrical(vertical: f32, horizontal: f32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
}

impl Default for Edges {
    fn default() -> Self {
        Self::all(0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub dx: f32,
    pub dy: f32,
    pub blur: f32,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    pub angle_deg: f32,
    pub stops: Vec<(f32, Color)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub surface2: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub accent_fg: Color,
    pub border: Color,
    pub focus: Color,
    pub danger: Color,
    pub field_bg: Color,
    pub select_bg: Color,
    pub select_fg: Color,
    pub shadow: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub small: f32,
    pub body: f32,
    pub title: f32,
    pub heading: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub palette: Palette,
    pub spacing: [f32; 5],
    pub radius: Radius,
    pub type_scale: TypeScale,
    pub font_family: Option<String>,
}

impl Tokens {
    pub fn light() -> Self {
        Self {
            palette: Palette {
                bg: Color::rgb(243, 243, 243),
                surface: Color::rgb(255, 255, 255),
                surface2: Color::rgb(238, 238, 238),
                fg: Color::rgb(30, 30, 30),
                muted: Color::rgb(115, 115, 115),
                accent: Color::rgb(0, 103, 192),
                accent_fg: Color::WHITE,
                border: Color::rgb(209, 209, 209),
                focus: Color::rgb(0, 103, 192),
                danger: Color::rgb(209, 52, 56),
                field_bg: Color::rgb(255, 255, 255),
                select_bg: Color::rgb(0, 103, 192),
                select_fg: Color::WHITE,
                shadow: Color::rgba(0, 0, 0, 35),
            },
            spacing: [4.0, 8.0, 12.0, 16.0, 24.0],
            radius: Radius { sm: 3.0, md: 5.0, lg: 8.0 },
            type_scale: TypeScale { small: 11.0, body: 14.0, title: 18.0, heading: 24.0 },
            font_family: None,
        }
    }

    pub fn dark() -> Self {
        Self {
            palette: Palette {
                bg: Color::rgb(32, 32, 32),
                surface: Color::rgb(45, 45, 45),
                surface2: Color::rgb(55, 55, 55),
                fg: Color::rgb(240, 240, 240),
                muted: Color::rgb(160, 160, 160),
                accent: Color::rgb(96, 205, 255),
                accent_fg: Color::rgb(20, 20, 20),
                border: Color::rgb(60, 60, 60),
                focus: Color::rgb(96, 205, 255),
                danger: Color::rgb(255, 153, 164),
                field_bg: Color::rgb(40, 40, 40),
                select_bg: Color::rgb(96, 205, 255),
                select_fg: Color::rgb(15, 15, 15),
                shadow: Color::rgba(0, 0, 0, 80),
            },
            spacing: [4.0, 8.0, 12.0, 16.0, 24.0],
            radius: Radius { sm: 3.0, md: 5.0, lg: 8.0 },
            type_scale: TypeScale { small: 11.0, body: 14.0, title: 18.0, heading: 24.0 },
            font_family: None,
        }
    }

    pub fn fluent() -> Self {
        let mut t = Self::light();
        t.palette.accent = Color::rgb(0, 103, 192);
        t.palette.surface2 = Color::rgb(240, 240, 240);
        t.radius = Radius { sm: 2.0, md: 4.0, lg: 6.0 };
        t.font_family = Some("Segoe UI".into());
        t
    }

    pub fn material() -> Self {
        Self {
            palette: Palette {
                bg: Color::rgb(254, 247, 255),
                surface: Color::rgb(238, 232, 244),
                surface2: Color::rgb(226, 218, 236),
                fg: Color::rgb(29, 27, 32),
                muted: Color::rgb(121, 116, 126),
                accent: Color::rgb(103, 80, 164),
                accent_fg: Color::WHITE,
                border: Color::rgb(202, 196, 208),
                focus: Color::rgb(103, 80, 164),
                danger: Color::rgb(179, 38, 30),
                field_bg: Color::rgb(247, 242, 250),
                select_bg: Color::rgb(103, 80, 164),
                select_fg: Color::WHITE,
                shadow: Color::rgba(0, 0, 0, 50),
            },
            spacing: [4.0, 8.0, 12.0, 16.0, 24.0],
            radius: Radius { sm: 4.0, md: 10.0, lg: 20.0 },
            type_scale: TypeScale { small: 12.0, body: 14.0, title: 20.0, heading: 26.0 },
            font_family: Some("Roboto".into()),
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            palette: Palette {
                bg: Color::rgb(0, 0, 0),
                surface: Color::rgb(0, 0, 0),
                surface2: Color::rgb(20, 20, 20),
                fg: Color::rgb(255, 255, 255),
                muted: Color::rgb(200, 200, 200),
                accent: Color::rgb(26, 235, 255),
                accent_fg: Color::BLACK,
                border: Color::rgb(255, 255, 255),
                focus: Color::rgb(255, 255, 0),
                danger: Color::rgb(255, 50, 50),
                field_bg: Color::rgb(0, 0, 0),
                select_bg: Color::rgb(26, 235, 255),
                select_fg: Color::BLACK,
                shadow: Color::rgba(0, 0, 0, 0),
            },
            spacing: [4.0, 8.0, 12.0, 16.0, 24.0],
            radius: Radius { sm: 0.0, md: 0.0, lg: 0.0 },
            type_scale: TypeScale { small: 12.0, body: 14.0, title: 18.0, heading: 24.0 },
            font_family: None,
        }
    }

    /// Modern dark theme tokens (#0F1115 bg, #171A21 surface, #4F8CFF accent).
    pub fn modern_dark() -> Self {
        let accent = Color::rgb(79, 140, 255);
        Self {
            palette: Palette {
                bg: Color::rgb(15, 17, 21),
                surface: Color::rgb(23, 26, 33),
                surface2: Color::rgb(31, 36, 48),
                fg: Color::rgb(230, 232, 238),
                muted: Color::rgb(154, 163, 178),
                accent,
                accent_fg: Color::WHITE,
                border: Color::rgb(42, 48, 64),
                focus: accent,
                danger: Color::rgb(248, 113, 113),
                field_bg: Color::rgb(23, 26, 33),
                select_bg: Color::rgba(accent.r, accent.g, accent.b, 64),
                select_fg: Color::rgb(230, 232, 238),
                shadow: Color::rgba(0, 0, 0, 110),
            },
            spacing: [4.0, 8.0, 12.0, 16.0, 24.0],
            radius: Radius { sm: 6.0, md: 10.0, lg: 14.0 },
            type_scale: TypeScale { small: 12.0, body: 14.0, title: 18.0, heading: 22.0 },
            font_family: None,
        }
    }

    /// Modern light theme tokens (#F5F7FB bg, #FFFFFF surface, #4F8CFF accent).
    pub fn modern_light() -> Self {
        let accent = Color::rgb(79, 140, 255);
        Self {
            palette: Palette {
                bg: Color::rgb(245, 247, 251),
                surface: Color::rgb(255, 255, 255),
                surface2: Color::rgb(235, 239, 248),
                fg: Color::rgb(27, 31, 42),
                muted: Color::rgb(122, 132, 150),
                accent,
                accent_fg: Color::WHITE,
                border: Color::rgb(226, 230, 239),
                focus: accent,
                danger: Color::rgb(220, 53, 69),
                field_bg: Color::rgb(255, 255, 255),
                select_bg: Color::rgba(accent.r, accent.g, accent.b, 64),
                select_fg: Color::rgb(27, 31, 42),
                shadow: Color::rgba(0, 0, 0, 25),
            },
            spacing: [4.0, 8.0, 12.0, 16.0, 24.0],
            radius: Radius { sm: 6.0, md: 10.0, lg: 14.0 },
            type_scale: TypeScale { small: 12.0, body: 14.0, title: 18.0, heading: 22.0 },
            font_family: None,
        }
    }
}

/// Sparse style specification with all-Option fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleSpec {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<f32>,
    pub radius: Option<f32>,
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub shadow: Option<Option<Shadow>>,
    pub gradient: Option<Option<LinearGradient>>,
    pub opacity: Option<f32>,
    pub font_family: Option<Option<String>>,
    pub font_size: Option<f32>,
    pub font_weight: Option<u16>,
    pub italic: Option<bool>,
    pub transition_ms: Option<u32>,
}

impl StyleSpec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge_into(&self, target: &mut ResolvedStyle) {
        if let Some(c) = self.bg { target.bg = c; }
        if let Some(c) = self.fg { target.fg = c; }
        if let Some(c) = self.border_color { target.border_color = c; }
        if let Some(v) = self.border_width { target.border_width = v; }
        if let Some(v) = self.radius { target.radius = v; }
        if let Some(v) = self.padding { target.padding = v; }
        if let Some(v) = self.margin { target.margin = v; }
        if let Some(v) = &self.shadow { target.shadow = *v; }
        if let Some(v) = &self.gradient { target.gradient = v.clone(); }
        if let Some(v) = self.opacity { target.opacity = v.clamp(0.0, 1.0); }
        if let Some(v) = &self.font_family { target.font_family = v.clone(); }
        if let Some(v) = self.font_size { target.font_size = v; }
        if let Some(v) = self.font_weight { target.font_weight = v.clamp(100, 900); }
        if let Some(v) = self.italic { target.italic = v; }
        if let Some(v) = self.transition_ms { target.transition_ms = v; }
    }
}

/// Fully resolved concrete widget style.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedStyle {
    pub bg: Color,
    pub fg: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub radius: f32,
    pub padding: Edges,
    pub margin: Edges,
    pub shadow: Option<Shadow>,
    pub gradient: Option<LinearGradient>,
    pub opacity: f32,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: u16,
    pub italic: bool,
    pub transition_ms: u32,
}

impl Default for ResolvedStyle {
    fn default() -> Self {
        Self {
            bg: Color::BG,
            fg: Color::FG,
            border_color: Color::rgb(180, 180, 180),
            border_width: 1.0,
            radius: 0.0,
            padding: Edges::default(),
            margin: Edges::default(),
            shadow: None,
            gradient: None,
            opacity: 1.0,
            font_family: None,
            font_size: 14.0,
            font_weight: 400,
            italic: false,
            transition_ms: 120,
        }
    }
}

/// Handle to an image loaded with `Tk::load_image`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageId(pub(crate) u32);

/// Error type when parsing a stylesheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for StyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stylesheet error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for StyleError {}

/// Database of styles and token registry.
pub struct StyleEngine {
    tokens: Tokens,
    /// Base styles registered via configure, keyed by style name (e.g. "TButton", "Accent.TButton")
    styles: BTreeMap<String, StyleSpec>,
    /// State maps: (style_name, state_bitmask) -> StyleSpec
    state_maps: BTreeMap<(String, u8), StyleSpec>,
}

impl Default for StyleEngine {
    fn default() -> Self {
        Self::new(Tokens::light())
    }
}

impl StyleEngine {
    pub fn new(tokens: Tokens) -> Self {
        let mut engine = Self {
            tokens,
            styles: BTreeMap::new(),
            state_maps: BTreeMap::new(),
        };
        engine.rebuild_theme_defaults();
        engine
    }

    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.tokens = tokens;
        self.rebuild_theme_defaults();
    }

    pub fn configure(&mut self, name: impl Into<String>, spec: StyleSpec) {
        let existing = self.styles.entry(name.into()).or_default();
        merge_spec(existing, &spec);
    }

    pub fn map(&mut self, name: impl Into<String>, state: State, spec: StyleSpec) {
        let name = name.into();
        let key = (name, state.bits());
        self.state_maps.entry(key.clone()).or_default();
        if let Some(existing) = self.state_maps.get_mut(&key) {
            merge_spec(existing, &spec);
        } else {
            self.state_maps.insert(key, spec);
        }
    }

    pub fn rebuild_theme_defaults(&mut self) {
        let p = self.tokens.palette;
        let r = self.tokens.radius;
        let ff = self.tokens.font_family.clone();
        let ts = self.tokens.type_scale;

        // Base TButton
        self.styles.insert("TButton".into(), StyleSpec {
            bg: Some(p.surface),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(if r.sm == 0.0 { 2.0 } else { 1.0 }),
            radius: Some(r.md),
            padding: Some(Edges::symmetrical(6.0, 14.0)),
            margin: Some(Edges::default()),
            shadow: Some(if p.shadow.a > 0 { Some(Shadow { dx: 0.0, dy: 1.0, blur: 2.0, color: p.shadow }) } else { None }),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(500),
            italic: Some(false),
            transition_ms: Some(120),
        });

        // Hover TButton
        let hover_bg = blend(p.surface, p.fg, 0.06);
        self.state_maps.insert(("TButton".into(), State::HOVER.bits()), StyleSpec {
            bg: Some(hover_bg),
            ..Default::default()
        });

        // Pressed TButton
        let pressed_bg = blend(p.surface, p.fg, 0.12);
        self.state_maps.insert(("TButton".into(), State::PRESSED.bits()), StyleSpec {
            bg: Some(pressed_bg),
            ..Default::default()
        });

        // Accent.TButton
        let accent_hover = blend(p.accent, Color::WHITE, 0.08);
        let accent_pressed = blend(p.accent, Color::BLACK, 0.08);
        self.styles.insert("Accent.TButton".into(), StyleSpec {
            bg: Some(p.accent),
            fg: Some(p.accent_fg),
            border_color: Some(p.accent),
            border_width: Some(1.0),
            radius: Some(r.md),
            padding: Some(Edges::symmetrical(6.0, 14.0)),
            margin: Some(Edges::default()),
            shadow: Some(if p.shadow.a > 0 { Some(Shadow { dx: 0.0, dy: 1.5, blur: 3.0, color: p.shadow }) } else { None }),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(700),
            italic: Some(false),
            transition_ms: Some(120),
        });
        self.state_maps.insert(("Accent.TButton".into(), State::HOVER.bits()), StyleSpec {
            bg: Some(accent_hover),
            ..Default::default()
        });
        self.state_maps.insert(("Accent.TButton".into(), State::PRESSED.bits()), StyleSpec {
            bg: Some(accent_pressed),
            ..Default::default()
        });

        // Built-in named styles: Card, Sidebar, Sidebar.TButton, Ghost.TButton, Heading, Subheading, Muted, Mono, Chips
        self.styles.insert("Card".into(), StyleSpec {
            bg: Some(p.surface),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(1.0),
            radius: Some(12.0),
            padding: Some(Edges::all(16.0)),
            margin: Some(Edges::default()),
            shadow: Some(if p.shadow.a > 0 { Some(Shadow { dx: 0.0, dy: 2.0, blur: 6.0, color: p.shadow }) } else { None }),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        self.styles.insert("Sidebar".into(), StyleSpec {
            bg: Some(p.surface),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(1.0),
            radius: Some(0.0),
            padding: Some(Edges::all(8.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        self.styles.insert("Sidebar.TButton".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(8.0),
            padding: Some(Edges::symmetrical(8.0, 12.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(500),
            italic: Some(false),
            transition_ms: Some(100),
        });
        self.state_maps.insert(("Sidebar.TButton".into(), State::HOVER.bits()), StyleSpec {
            bg: Some(p.surface2),
            ..Default::default()
        });
        let selected_bg = Color::rgba(p.accent.r, p.accent.g, p.accent.b, 51);
        self.state_maps.insert(("Sidebar.TButton".into(), State::SELECTED.bits()), StyleSpec {
            bg: Some(selected_bg),
            fg: Some(p.accent),
            ..Default::default()
        });

        self.styles.insert("Ghost.TButton".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(r.md),
            padding: Some(Edges::symmetrical(6.0, 14.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(500),
            italic: Some(false),
            transition_ms: Some(100),
        });
        self.state_maps.insert(("Ghost.TButton".into(), State::HOVER.bits()), StyleSpec {
            bg: Some(p.surface2),
            ..Default::default()
        });
        self.state_maps.insert(("Ghost.TButton".into(), State::PRESSED.bits()), StyleSpec {
            bg: Some(blend(p.surface2, p.fg, 0.1)),
            ..Default::default()
        });

        self.styles.insert("Heading".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(0.0),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(22.0),
            font_weight: Some(700),
            italic: Some(false),
            transition_ms: Some(0),
        });

        self.styles.insert("Subheading".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(0.0),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(16.0),
            font_weight: Some(700),
            italic: Some(false),
            transition_ms: Some(0),
        });

        self.styles.insert("Muted".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.muted),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(0.0),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(13.0),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        self.styles.insert("Mono".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(0.0),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(Some("monospace".into())),
            font_size: Some(13.0),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        // Pill chips
        let chip_specs = [
            ("Chip.Success", Color::rgba(46, 204, 113, 38), Color::rgb(39, 174, 96)),
            ("Chip.Warning", Color::rgba(241, 196, 15, 38), Color::rgb(214, 137, 16)),
            ("Chip.Danger", Color::rgba(231, 76, 60, 38), Color::rgb(214, 48, 49)),
            ("Chip.Info", Color::rgba(52, 152, 219, 38), Color::rgb(41, 128, 185)),
            ("Chip.Neutral", Color::rgba(128, 128, 128, 38), p.muted),
        ];
        for (name, bg, fg) in chip_specs {
            self.styles.insert(name.into(), StyleSpec {
                bg: Some(bg),
                fg: Some(fg),
                border_color: Some(Color::rgba(0, 0, 0, 0)),
                border_width: Some(0.0),
                radius: Some(999.0),
                padding: Some(Edges::symmetrical(3.0, 10.0)),
                margin: Some(Edges::default()),
                shadow: Some(None),
                gradient: Some(None),
                opacity: Some(1.0),
                font_family: Some(ff.clone()),
                font_size: Some(12.0),
                font_weight: Some(500),
                italic: Some(false),
                transition_ms: Some(0),
            });
        }

        // TLabel
        self.styles.insert("TLabel".into(), StyleSpec {
            bg: Some(Color::rgba(0, 0, 0, 0)),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(0.0),
            padding: Some(Edges::symmetrical(2.0, 4.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        // TEntry
        self.styles.insert("TEntry".into(), StyleSpec {
            bg: Some(p.field_bg),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(if r.sm == 0.0 { 2.0 } else { 1.0 }),
            radius: Some(r.sm),
            padding: Some(Edges::symmetrical(5.0, 8.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(120),
        });
        self.state_maps.insert(("TEntry".into(), State::FOCUSED.bits()), StyleSpec {
            border_color: Some(p.focus),
            border_width: Some(if r.sm == 0.0 { 3.0 } else { 2.0 }),
            ..Default::default()
        });

        // TFrame
        self.styles.insert("TFrame".into(), StyleSpec {
            bg: Some(p.bg),
            fg: Some(p.fg),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(0.0),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        // TCheckbutton & TRadiobutton
        for class in ["TCheckbutton", "TRadiobutton"] {
            self.styles.insert(class.into(), StyleSpec {
                bg: Some(Color::rgba(0, 0, 0, 0)),
                fg: Some(p.fg),
                border_color: Some(p.border),
                border_width: Some(1.0),
                radius: Some(if class == "TRadiobutton" { 8.0 } else { r.sm }),
                padding: Some(Edges::symmetrical(3.0, 4.0)),
                margin: Some(Edges::default()),
                shadow: Some(None),
                gradient: Some(None),
                opacity: Some(1.0),
                font_family: Some(ff.clone()),
                font_size: Some(ts.body),
                font_weight: Some(400),
                italic: Some(false),
                transition_ms: Some(120),
            });
        }

        // TScale
        self.styles.insert("TScale".into(), StyleSpec {
            bg: Some(p.border),
            fg: Some(p.accent),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(r.sm),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(120),
        });

        // TListbox
        self.styles.insert("TListbox".into(), StyleSpec {
            bg: Some(p.field_bg),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(1.0),
            radius: Some(r.sm),
            padding: Some(Edges::all(2.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        // TSpinbox
        self.styles.insert("TSpinbox".into(), StyleSpec {
            bg: Some(p.field_bg),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(1.0),
            radius: Some(r.sm),
            padding: Some(Edges::symmetrical(4.0, 6.0)),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(120),
        });

        // TProgressbar
        self.styles.insert("TProgressbar".into(), StyleSpec {
            bg: Some(p.border),
            fg: Some(p.accent),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(r.sm),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(120),
        });

        // TCanvas
        self.styles.insert("TCanvas".into(), StyleSpec {
            bg: Some(p.field_bg),
            fg: Some(p.fg),
            border_color: Some(p.border),
            border_width: Some(1.0),
            radius: Some(0.0),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff.clone()),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(0),
        });

        // TScrollbar
        self.styles.insert("TScrollbar".into(), StyleSpec {
            bg: Some(blend(p.bg, p.border, 0.4)),
            fg: Some(p.muted),
            border_color: Some(Color::rgba(0, 0, 0, 0)),
            border_width: Some(0.0),
            radius: Some(r.sm),
            padding: Some(Edges::default()),
            margin: Some(Edges::default()),
            shadow: Some(None),
            gradient: Some(None),
            opacity: Some(1.0),
            font_family: Some(ff),
            font_size: Some(ts.body),
            font_weight: Some(400),
            italic: Some(false),
            transition_ms: Some(120),
        });
        // v0.8 widget classes: Text and Combobox look like input fields; the
        // Notebook strip inherits the surrounding background (like TFrame).
        if let Some(entry) = self.styles.get("TEntry").cloned() {
            self.styles.insert("TText".into(), entry.clone());
            self.styles.insert("TCombobox".into(), entry);
        }
        if let Some(focus) = self.state_maps.get(&("TEntry".into(), State::FOCUSED.bits())).cloned() {
            self.state_maps.insert(("TText".into(), State::FOCUSED.bits()), focus.clone());
            self.state_maps.insert(("TCombobox".into(), State::FOCUSED.bits()), focus);
        }
        if let Some(frame) = self.styles.get("TFrame").cloned() {
            self.styles.insert("TNotebook".into(), frame);
        }
        // Disabled look for every class (theme- and stylesheet-overridable).
        for class in [
            "TButton", "TLabel", "TEntry", "TCheckbutton", "TRadiobutton", "TScale",
            "TListbox", "TSpinbox", "TProgressbar", "TText", "TCombobox", "TNotebook",
        ] {
            self.state_maps
                .entry((class.into(), State::DISABLED.bits()))
                .or_insert_with(|| StyleSpec {
                    fg: Some(p.muted),
                    opacity: Some(0.55),
                    shadow: Some(None),
                    ..Default::default()
                });
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&StyleSpec> {
        self.styles.get(name)
    }

    pub fn lookup_state(&self, name: &str, state: State) -> Option<&StyleSpec> {
        self.state_maps.get(&(name.to_string(), state.bits()))
    }
}

fn merge_spec(dst: &mut StyleSpec, src: &StyleSpec) {
    if src.bg.is_some() { dst.bg = src.bg; }
    if src.fg.is_some() { dst.fg = src.fg; }
    if src.border_color.is_some() { dst.border_color = src.border_color; }
    if src.border_width.is_some() { dst.border_width = src.border_width; }
    if src.radius.is_some() { dst.radius = src.radius; }
    if src.padding.is_some() { dst.padding = src.padding; }
    if src.margin.is_some() { dst.margin = src.margin; }
    if src.shadow.is_some() { dst.shadow = src.shadow; }
    if src.gradient.is_some() { dst.gradient = src.gradient.clone(); }
    if src.opacity.is_some() { dst.opacity = src.opacity; }
    if src.font_family.is_some() { dst.font_family = src.font_family.clone(); }
    if src.font_size.is_some() { dst.font_size = src.font_size; }
    if src.font_weight.is_some() { dst.font_weight = src.font_weight; }
    if src.italic.is_some() { dst.italic = src.italic; }
    if src.transition_ms.is_some() { dst.transition_ms = src.transition_ms; }
}

/// Blends two colors with factor `t` in [0, 1] (0 = c1, 1 = c2).
pub fn blend(c1: Color, c2: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    Color::rgba(
        (c1.r as f32 * inv + c2.r as f32 * t).round().clamp(0.0, 255.0) as u8,
        (c1.g as f32 * inv + c2.g as f32 * t).round().clamp(0.0, 255.0) as u8,
        (c1.b as f32 * inv + c2.b as f32 * t).round().clamp(0.0, 255.0) as u8,
        (c1.a as f32 * inv + c2.a as f32 * t).round().clamp(0.0, 255.0) as u8,
    )
}

/// Active transition tracking for smooth state changes.
#[derive(Clone, Debug)]
pub struct TransitionState {
    pub from_bg: Color,
    pub to_bg: Color,
    pub from_fg: Color,
    pub to_fg: Color,
    pub from_border: Color,
    pub to_border: Color,
    pub duration_ms: u32,
    pub elapsed_ms: u32,
}

impl TransitionState {
    pub fn new(
        from_bg: Color, to_bg: Color,
        from_fg: Color, to_fg: Color,
        from_border: Color, to_border: Color,
        duration_ms: u32,
    ) -> Self {
        Self {
            from_bg, to_bg,
            from_fg, to_fg,
            from_border, to_border,
            duration_ms: duration_ms.max(1),
            elapsed_ms: 0,
        }
    }

    pub fn advance(&mut self, ms: u32) -> bool {
        self.elapsed_ms = (self.elapsed_ms + ms).min(self.duration_ms);
        self.is_done()
    }

    pub fn is_done(&self) -> bool {
        self.elapsed_ms >= self.duration_ms
    }

    pub fn progress(&self) -> f32 {
        if self.duration_ms == 0 {
            1.0
        } else {
            (self.elapsed_ms as f32 / self.duration_ms as f32).clamp(0.0, 1.0)
        }
    }

    pub fn current_bg(&self) -> Color {
        blend(self.from_bg, self.to_bg, self.progress())
    }

    pub fn current_fg(&self) -> Color {
        blend(self.from_fg, self.to_fg, self.progress())
    }

    pub fn current_border(&self) -> Color {
        blend(self.from_border, self.to_border, self.progress())
    }
}

// ---------------------------------------------------------------------------
// Stylesheet Parser
// ---------------------------------------------------------------------------

pub struct StylesheetParser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> StylesheetParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0, line: 1 }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.input.len() {
            let rest = &self.input[self.pos..];
            if rest.starts_with("/*") {
                if let Some(end) = rest.find("*/") {
                    let comment = &rest[..end + 2];
                    self.line += comment.matches('\n').count();
                    self.pos += end + 2;
                } else {
                    self.pos = self.input.len();
                    break;
                }
            } else if let Some(ch) = rest.chars().next() {
                if ch.is_whitespace() {
                    if ch == '\n' {
                        self.line += 1;
                    }
                    self.pos += ch.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn parse(mut self, engine: &mut StyleEngine) -> Result<usize, StyleError> {
        let mut count = 0;
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.input.len() {
                break;
            }
            // Parse selector
            let selector = self.parse_selector()?;
            self.skip_whitespace_and_comments();
            if self.pos >= self.input.len() || !self.input[self.pos..].starts_with('{') {
                return Err(StyleError { line: self.line, message: "expected '{' after selector".into() });
            }
            self.pos += 1; // skip '{'
            let spec = self.parse_block()?;
            // Apply spec according to selector
            for (name, state) in selector {
                if let Some(st) = state {
                    engine.map(name, st, spec.clone());
                } else {
                    engine.configure(name, spec.clone());
                }
            }
            count += 1;
        }
        Ok(count)
    }

    fn parse_selector(&mut self) -> Result<Vec<(String, Option<State>)>, StyleError> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos..].chars().next().unwrap_or('\0');
            if ch == '{' {
                break;
            }
            if ch == '\n' {
                self.line += 1;
            }
            self.pos += ch.len_utf8();
        }
        let raw = self.input[start..self.pos].trim();
        if raw.is_empty() {
            return Err(StyleError { line: self.line, message: "empty selector".into() });
        }
        let mut selectors = Vec::new();
        for part in raw.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (name, state) = if let Some((base, st_str)) = part.split_once(':') {
                let st = parse_state(st_str).ok_or_else(|| StyleError {
                    line: self.line,
                    message: format!("unknown pseudo-class ':{st_str}'"),
                })?;
                (base.trim().to_string(), Some(st))
            } else {
                (part.to_string(), None)
            };
            selectors.push((name, state));
        }
        if selectors.is_empty() {
            return Err(StyleError { line: self.line, message: "no valid selectors".into() });
        }
        Ok(selectors)
    }

    fn parse_block(&mut self) -> Result<StyleSpec, StyleError> {
        let mut spec = StyleSpec::default();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.input.len() {
                return Err(StyleError { line: self.line, message: "unclosed '{'".into() });
            }
            if self.input[self.pos..].starts_with('}') {
                self.pos += 1;
                break;
            }
            // Parse property: value;
            let decl_start = self.pos;
            while self.pos < self.input.len() {
                let ch = self.input[self.pos..].chars().next().unwrap_or('\0');
                if ch == ';' || ch == '}' {
                    break;
                }
                if ch == '\n' {
                    self.line += 1;
                }
                self.pos += ch.len_utf8();
            }
            let decl = self.input[decl_start..self.pos].trim();
            if self.pos < self.input.len() && self.input[self.pos..].starts_with(';') {
                self.pos += 1;
            }
            if decl.is_empty() {
                continue;
            }
            let Some((prop, val)) = decl.split_once(':') else {
                return Err(StyleError { line: self.line, message: format!("invalid declaration: '{decl}'") });
            };
            parse_declaration(prop.trim(), val.trim(), &mut spec, self.line)?;
        }
        Ok(spec)
    }
}

fn parse_state(s: &str) -> Option<State> {
    match s.trim().to_ascii_lowercase().as_str() {
        "hover" => Some(State::HOVER),
        "pressed" | "active" => Some(State::PRESSED),
        "focus" | "focused" => Some(State::FOCUSED),
        "disabled" => Some(State::DISABLED),
        "checked" => Some(State::CHECKED),
        "selected" => Some(State::SELECTED),
        _ => None,
    }
}

fn parse_declaration(prop: &str, val: &str, spec: &mut StyleSpec, line: usize) -> Result<(), StyleError> {
    let err = |msg: &str| StyleError { line, message: format!("{msg} in '{prop}: {val}'") };
    match prop.to_ascii_lowercase().as_str() {
        "background" | "bg" => {
            if val.starts_with("linear-gradient") {
                let grad = parse_linear_gradient(val).ok_or_else(|| err("invalid linear-gradient"))?;
                spec.gradient = Some(Some(grad));
            } else {
                let color = parse_color(val).ok_or_else(|| err("invalid color"))?;
                spec.bg = Some(color);
                spec.gradient = Some(None);
            }
        }
        "color" | "fg" => {
            let color = parse_color(val).ok_or_else(|| err("invalid color"))?;
            spec.fg = Some(color);
        }
        "border" => {
            // CSS-like: any order of width, colour and an ignored style keyword.
            let mut any = false;
            for part in val.split_whitespace() {
                if let Some(width) = css_number(part) {
                    spec.border_width = Some(width);
                    any = true;
                } else if let Some(color) = parse_color(part) {
                    spec.border_color = Some(color);
                    any = true;
                } else if !matches!(part, "solid" | "none") {
                    return Err(err("invalid border value"));
                }
            }
            if !any {
                return Err(err("invalid border value"));
            }
        }
        "border-color" => {
            spec.border_color = Some(parse_color(val).ok_or_else(|| err("invalid color"))?);
        }
        "border-width" => {
            let w = val.trim_end_matches("px").trim().parse::<f32>().map_err(|_| err("invalid number"))?;
            spec.border_width = Some(w);
        }
        "border-radius" => {
            let r = val.trim_end_matches("px").trim().parse::<f32>().map_err(|_| err("invalid number"))?;
            spec.radius = Some(r);
        }
        "padding" => {
            spec.padding = Some(parse_edges(val).ok_or_else(|| err("invalid padding"))?);
        }
        "margin" => {
            spec.margin = Some(parse_edges(val).ok_or_else(|| err("invalid margin"))?);
        }
        "box-shadow" | "shadow" => {
            if val.eq_ignore_ascii_case("none") {
                spec.shadow = Some(None);
            } else {
                let s = parse_shadow(val).ok_or_else(|| err("invalid box-shadow"))?;
                spec.shadow = Some(Some(s));
            }
        }
        "opacity" => {
            let op = val.parse::<f32>().map_err(|_| err("invalid opacity number"))?;
            spec.opacity = Some(op);
        }
        "font-family" => {
            let family = val.trim_matches('"').trim_matches('\'').trim().to_string();
            spec.font_family = Some(Some(family));
        }
        "font-size" => {
            let sz = val.trim_end_matches("pt").trim_end_matches("px").trim().parse::<f32>().map_err(|_| err("invalid font size"))?;
            spec.font_size = Some(sz);
        }
        "font-weight" => {
            let w = match val.to_ascii_lowercase().as_str() {
                "normal" => 400,
                "bold" => 700,
                other => other.parse::<u16>().map_err(|_| err("invalid font-weight"))?,
            };
            spec.font_weight = Some(w);
        }
        "font-style" => {
            spec.italic = Some(val.eq_ignore_ascii_case("italic"));
        }
        "transition" | "transition-duration" => {
            let ms = if let Some(stripped) = val.strip_suffix("ms") {
                stripped.trim().parse::<u32>().map_err(|_| err("invalid transition"))?
            } else if let Some(stripped) = val.strip_suffix('s') {
                let s = stripped.trim().parse::<f32>().map_err(|_| err("invalid transition"))?;
                (s * 1000.0) as u32
            } else {
                val.trim().parse::<u32>().map_err(|_| err("invalid transition"))?
            };
            spec.transition_ms = Some(ms);
        }
        _ => {}
    }
    Ok(())
}

fn parse_edges(val: &str) -> Option<Edges> {
    let parts: Vec<&str> = val.split_whitespace().collect();
    let nums: Vec<f32> = parts.iter().filter_map(|p| p.trim_end_matches("px").parse::<f32>().ok()).collect();
    match nums.len() {
        1 => Some(Edges::all(nums[0])),
        2 => Some(Edges::symmetrical(nums[0], nums[1])),
        4 => Some(Edges::new(nums[0], nums[1], nums[2], nums[3])),
        _ => None,
    }
}

fn parse_shadow(val: &str) -> Option<Shadow> {
    let parts: Vec<&str> = val.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let dx = parts[0].trim_end_matches("px").parse::<f32>().ok()?;
    let dy = parts[1].trim_end_matches("px").parse::<f32>().ok()?;
    let blur = parts[2].trim_end_matches("px").parse::<f32>().ok()?;
    let color = if parts.len() >= 4 {
        parse_color(parts[3])?
    } else {
        Color::rgba(0, 0, 0, 100)
    };
    Some(Shadow { dx, dy, blur, color })
}

fn parse_linear_gradient(val: &str) -> Option<LinearGradient> {
    let start = val.find('(')?;
    let end = val.rfind(')')?;
    let inner = &val[start + 1..end];
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return None;
    }
    let (angle_deg, stops_start) = if let Ok(angle) = parts[0].trim_end_matches("deg").trim().parse::<f32>() {
        (angle, 1)
    } else {
        (180.0, 0)
    };
    let mut stops = Vec::new();
    let num_stops = parts.len() - stops_start;
    for (i, p) in parts[stops_start..].iter().enumerate() {
        let sub: Vec<&str> = p.split_whitespace().collect();
        if sub.is_empty() {
            continue;
        }
        let color = parse_color(sub[0])?;
        let pos = if sub.len() >= 2 {
            let raw = sub[1].trim_end_matches('%');
            raw.parse::<f32>().ok().map(|v| if sub[1].ends_with('%') { v / 100.0 } else { v }).unwrap_or(0.0)
        } else if num_stops > 1 {
            i as f32 / (num_stops - 1) as f32
        } else {
            0.0
        };
        stops.push((pos, color));
    }
    if stops.is_empty() {
        return None;
    }
    Some(LinearGradient { angle_deg, stops })
}

pub fn parse_color(val: &str) -> Option<Color> {
    let s = val.trim();
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(Color::rgb(r * 17, g * 17, b * 17))
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                Some(Color::rgba(r * 17, g * 17, b * 17, a * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::rgba(r, g, b, a))
            }
            _ => None,
        }
    } else {
        match s.to_ascii_lowercase().as_str() {
            "white" => Some(Color::rgb(255, 255, 255)),
            "black" => Some(Color::rgb(0, 0, 0)),
            "red" => Some(Color::rgb(255, 0, 0)),
            "green" => Some(Color::rgb(0, 128, 0)),
            "blue" => Some(Color::rgb(0, 0, 255)),
            "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
            "transparent" => Some(Color::rgba(0, 0, 0, 0)),
            "yellow" => Some(Color::rgb(255, 255, 0)),
            "cyan" => Some(Color::rgb(0, 255, 255)),
            "magenta" => Some(Color::rgb(255, 0, 255)),
            _ => None,
        }
    }
}

/// Helper to create a rounded rectangle tiny-skia Path.
pub fn build_rounded_rect_path(rect: Rect, radius: f32) -> Option<tiny_skia::Path> {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return None;
    }
    let r = radius.max(0.0).min(rect.width / 2.0).min(rect.height / 2.0);
    let mut pb = tiny_skia::PathBuilder::new();
    if r <= 0.001 {
        pb.move_to(rect.x, rect.y);
        pb.line_to(rect.x + rect.width, rect.y);
        pb.line_to(rect.x + rect.width, rect.y + rect.height);
        pb.line_to(rect.x, rect.y + rect.height);
        pb.close();
        return pb.finish();
    }
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let k = 0.552_284_8 * r;
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

/// Parses a finite, non-negative CSS length such as `2`, `2.5` or `2px`.
fn css_number(text: &str) -> Option<f32> {
    let value = text.trim().trim_end_matches("px").parse::<f32>().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}
