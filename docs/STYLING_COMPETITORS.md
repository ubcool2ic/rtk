# Styling systems: competitors vs rtk

Researched 2026-10-16 for rtk 0.4. Sources are official docs (links checked to resolve
on that date). Where a point comes from general knowledge of the toolkit rather than the
linked page, it is marked *(gk)*.

## Python ttk (themed Tk)
Sources: https://docs.python.org/3/library/tkinter.ttk.html, https://tkdocs.com/tutorial/styles.html
- **Offers:** `ttk.Style()` with `configure(style, **opts)`, named styles that derive by dotted
  prefix (`Accent.TButton` → `TButton`), `map()` for state-dependent values (`active`,
  `pressed`, `focus`, `disabled`, `selected`…), `layout()` that composes a widget from
  *elements* (border, padding, label, focus), `element_create` (incl. image elements), and
  themes (`clam`, `alt`, `default`, `classic`, plus native `vista`/`aqua`) via `theme_use`.
- **Strengths:** the named-style + state-map model is simple and powerful; per-class styles.
- **Weaknesses:** options differ per theme and element (many silently ignored); no rounded
  corners, shadows, gradients or animation without image tricks; awkward discovery
  (`element_options`); no stylesheet file format; dark mode is DIY.

## CustomTkinter
Source: https://github.com/TomSchimansky/CustomTkinter, https://customtkinter.tomschimansky.com/documentation/appearancemode
- **Offers:** modern-looking widgets drawn on Tk canvas, `corner_radius`, `border_width`,
  `fg_color`/`hover_color` per widget, light/dark/"system" appearance mode, colour themes as
  JSON (`blue`, `green`, `dark-blue`, custom), HiDPI scaling.
- **Strengths:** good-looking defaults, rounded corners, light/dark tuples per colour.
- **Weaknesses:** styling is per-widget kwargs, not a cascade; only hover state is first class;
  no transitions, gradients or shadows; JSON theme must be set before widgets are created *(gk)*.

## Qt Style Sheets (QSS)
Source: https://doc.qt.io/qt-6/stylesheet-reference.html
- **Offers:** CSS-like syntax with type/class/ID/property selectors, sub-controls
  (`::indicator`, `::handle`), pseudo-states (`:hover`, `:pressed`, `:checked`, `:disabled`,
  `:focus`…), box model (margin/border/padding), `border-radius`, `qlineargradient`/`qradialgradient`,
  `border-image`, fonts; cascade from app → parent → widget.
- **Strengths:** most complete declarative styling in a native toolkit; hot-swappable at runtime.
- **Weaknesses:** no transitions/animations; no box-shadow (needs QGraphicsDropShadowEffect);
  mixes poorly with QStyle and has performance costs; C++/Qt-sized dependency.

## GTK4 CSS
Sources: https://docs.gtk.org/gtk4/css-overview.html, https://docs.gtk.org/gtk4/css-properties.html
- **Offers:** real CSS (selectors by node name/class/state, `:hover`, `:checked`, `:disabled`,
  `:focus-visible`, `:backdrop`), `box-shadow`, gradients, `border-radius`, `transition`
  and `animation`, `@define-color`, Adwaita and HighContrast themes.
- **Strengths:** closest to web CSS, including transitions and shadows; theme ecosystem.
- **Weaknesses:** heavy runtime (GTK + GLib); cross-platform deployment pain on Windows/macOS;
  styling relies on node names that change between versions *(gk)*.

## Flutter ThemeData
Source: https://api.flutter.dev/flutter/material/ThemeData-class.html
- **Offers:** `ThemeData` with `ColorScheme` (seed-generated Material 3 palettes), `TextTheme`
  type scale, per-component themes (`ElevatedButtonTheme`, …), `WidgetStateProperty` for states,
  `ThemeExtension` for custom tokens, implicit animations (`AnimatedTheme`).
- **Strengths:** design-token model (colour scheme + type scale), states and animated theme changes.
- **Weaknesses:** Dart and Flutter engine; Material-centric; no external stylesheet.

## egui
Sources: https://docs.rs/egui/latest/egui/style/struct.Style.html, https://docs.rs/egui/latest/egui/style/struct.Visuals.html
- **Offers:** a global `Style` (spacing, text styles, interaction) and `Visuals`
  (`dark()`/`light()`, `widgets.{noninteractive, inactive, hovered, active, open}` each with
  bg fill, stroke, rounding, expansion; window shadow, selection colours).
- **Strengths:** simple, programmatic, per-state visuals, shadows and rounding built in.
- **Weaknesses:** global rather than per-widget-class; no named styles or cascade; no file
  format; theming individual widgets means scoped style overrides *(gk)*.

## iced
Sources: https://docs.rs/iced/latest/iced/theme/index.html, https://docs.rs/iced/latest/iced/widget/button/trait.Catalog.html
- **Offers:** `Theme` with built-in palettes (Light, Dark, Dracula, Nord, Catppuccin…, custom
  `Palette`), per-widget `Catalog` trait and `style(|theme, status| Style{..})` closures
  receiving status (active/hovered/pressed/disabled); `Background::Gradient`, `Border{radius}`, `Shadow`.
- **Strengths:** type-safe, status-aware closures, gradients and shadows.
- **Weaknesses:** styling is code-only; closures per widget get verbose; no cascade/inheritance;
  no built-in transitions *(gk)*.

## Slint
Source: https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/
- **Offers:** compile-time widget styles `fluent`, `material`, `cupertino`, `cosmic`, `qt`,
  `native`, each with `-light`/`-dark` variants following the system; `Palette` and `StyleMetrics`
  globals for custom components; `.slint` DSL with states and animations.
- **Strengths:** polished design-system styles, animations, system dark mode.
- **Weaknesses:** style chosen at **compile time** (env var / build config), not switchable at
  runtime; custom styling is via the DSL, not a stylesheet; GPL/royalty-free/commercial licensing.

## fltk-rs + fltk-theme
Source: https://github.com/fltk-rs/fltk-theme
- **Offers:** `WidgetTheme` (Classic, Aero, Metro, AquaClassic, Greybird, Blue, Dark,
  HighContrast) and `ColorTheme` presets, plus `WidgetScheme` (Fluent, Aqua, Clean, SvgBased)
  that change box drawing *(gk for the exact list)*.
- **Strengths:** many presets, tiny binaries, includes high contrast.
- **Weaknesses:** global, not per-widget-class; no state maps or transitions; customising
  means setting FLTK box types/colours imperatively.

## Dioxus / Tauri (web-tech UIs)
Sources: https://dioxuslabs.com/learn/0.6/guides/assets, https://v2.tauri.app/
- **Offers:** full browser CSS (Tailwind etc.) in a WebView, so everything: variables,
  transitions, shadows, gradients, media queries.
- **Strengths:** unlimited styling, huge ecosystem, hot reload.
- **Weaknesses:** WebView or browser engine dependency, bigger memory footprint, JS/DOM
  semantics, harder to embed in a small native tool.

## Gap table: what rtk 0.4 aims to do better

| Capability | ttk | CTk | QSS | GTK CSS | Flutter | egui | iced | Slint | fltk-theme | **rtk 0.4** |
|---|---|---|---|---|---|---|---|---|---|---|
| tkinter-style API | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Named styles (`Accent.TButton`) | ✅ | ❌ | ~ (classes) | ✅ | ~ | ❌ | ~ | ❌ | ❌ | ✅ |
| Design tokens (palette/spacing/radius/type) | ❌ | ~ | ❌ | ~ | ✅ | ~ | ~ (palette) | ✅ | ❌ | ✅ |
| Cascade / inheritance from parent | ~ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ~ | ❌ | ✅ |
| State maps (hover/pressed/focus/disabled/checked/selected) | ✅ | ~ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| Animated state transitions | ❌ | ❌ | ❌ | ✅ | ✅ | ~ | ❌ | ✅ | ❌ | ✅ |
| Rounded corners / shadows / gradients | ❌ | ~ (radius) | ~ (no shadow) | ✅ | ✅ | ✅ | ✅ | ✅ | ~ | ✅ |
| External stylesheet + hot reload | ❌ | ~ (JSON) | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Runtime theme switching | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (compile time) | ✅ | ✅ |
| High-contrast theme built in | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Pure Rust, no C/web runtime | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ (C++) | ✅ |
| Headless style tests (resolve + render) | ❌ | ❌ | ❌ | ❌ | ✅ | ~ | ~ | ✅ | ❌ | ✅ |

✅ = first-class, ~ = partial/workaround, ❌ = not offered. rtk's pitch: **ttk's
named-style + state-map model, with Flutter-style tokens, GTK-style CSS (incl. transitions,
shadows, gradients) and hot reload, in a small pure-Rust crate with headless testability.**
Where rtk is still behind: no sub-control selectors (`::indicator`), no ID/class selectors
beyond named styles, no radial gradients, approximated (not true Gaussian) blur, and a far
smaller widget set than Qt/GTK/Flutter.
