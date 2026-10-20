# Changelog

## 1.0.0 (unreleased): work in progress on branch `rtk-1.0`

### 0.9 Foundations
- Clippy runs clean on the stable toolchain (1.99) with `-D warnings`.
- `src/tk.rs` is split into `src/tk/` modules (mod, style, window, tree, geometry, events, scroll, entry, text, listbox, canvas, combobox, notebook, progress, tooltip, shortcuts, clipboard, edit). The API is unchanged.
- `rtk::testing`: golden screenshot tests (`assert_golden`, `compare_pixmaps`, `golden_dir`, `RTK_BLESS=1`), with goldens for every widget in the Light, Dark and HighContrast themes at 1x and 2x.
- Window API: `iconphoto_rgba`/`iconphoto_png`, `minsize`/`maxsize`/`resizable` (can be limited to one axis), and `protocol_close`, a close hook that can veto closing, plus `sim_close_request`.
- Keyboard: `Modifiers`, and `Key::Char`/`F(n)`/`Insert` (`Key` is now `#[non_exhaustive]`). `Shortcut::parse`/`label` and `bind_shortcut`/`unbind_shortcut`/`sim_key_with`. Primary maps to Cmd on macOS and Ctrl elsewhere.
- Editing: clipboard (`clipboard_get`/`append`/`clear`; OS clipboard through the default `clipboard` feature, which uses arboard). Text selection in Entry and Text by Shift+keys, mouse drag and Primary+A. Copy, cut and paste. Undo/redo with typing coalescing. New `selection_bg`/`selection_fg` palette tokens.
- CI workflow for Windows, Linux and macOS plus MSRV, in `ci/rust.yml`. Move it to `.github/workflows/` to turn it on.

### 0.13 — theming, accessibility, platform
- **New: semantic theme tokens** (`rtk::theme`). `SemanticTokens` (from `Tk::semantic_tokens()` or `SemanticTokens::for_theme`) gives role-based colours (`SemanticColors`: background, surface, surface_raised, text, text_muted, text_disabled, accent/on_accent, border, focus_ring, success/warning/danger/info with `on_*` text colours, field, selection/on_selection, shadow), `Spacing` (xxs..xxl), `Radii` (none, sm, md, lg, pill), `Typography` (sizes caption..display, weights fixed at 400/700 for the bundled font) and a colourblind-safe `chart` palette (`OKABE_ITO`). `Tk::color(ColorRole)` looks one role up; `SemanticTokens::colorblind_safe()` swaps status colours to Okabe-Ito hues. WCAG helpers `theme::contrast_ratio` and `theme::relative_luminance`. Additive; no rendering change. Tests: `tests/m013_tokens.rs`, `theme::tests`.

## 0.8.2
- **Fix: garbled button labels on Windows.** `TButton`, `Ghost.TButton`, `Sidebar.TButton` and `Chip.*` used font weight 500. Windows' default sans-serif (Arial) has no 500 face, so the matcher picked an unrelated face and labels drew as boxes/symbols. These styles now use 400 (bold variants stay 700). A unit test checks that every built-in theme style and state map uses only 400 or 700.
- **New: Enter-to-submit for Entry.** `Tk::entry_bind_submit(entry, |tk| ...)` runs a callback when Enter is pressed in a focused single-line Entry (tkinter `bind("<Return>")`). It's additive and non-breaking: Enter in an Entry with no callback stays a no-op, and Enter in an open dialog still accepts the dialog first. Tests: `tests/entry_submit.rs`.
- **New: bundled UI font.** rtk now compiles in **Noto Sans** Regular and Bold (unhinted TTF v2.015, SIL OFL 1.1, license in `fonts/OFL.txt`) and makes it the default sans-serif for every widget and theme. Fluent and Material no longer ask for Segoe UI/Roboto; set `Tokens::font_family` to override. Load order: bundled Noto Sans, then Arial, then the platform sans (Segoe UI/Helvetica/DejaVu Sans/...). If the bundled faces fail to load, rtk prints one warning and falls back. System fonts stay loaded for glyph fallback (emoji, CJK). New `rtk::fonts` module, `DefaultFont` and `Tk::default_font()`. Tests: `fonts::tests` (parse, forced fallback, every theme resolves to a real face) and `tests/fonts.rs`.
  - **Binary size:** +~845 KiB (two 422 KiB font files embedded in every rtk app).
  - Why Noto Sans: OFL, real static 400 and 700 faces (cosmic-text 0.12 can't select weights from variable fonts), ~3,900 glyphs covering Latin, Greek and Cyrillic, and a neutral UI design with metrics close to Arial/Segoe UI. Inter was the other candidate but covers fewer scripts.

## 0.8.1

- Code review with claude-fable-5.1 (see docs/REVIEW_FABLE_5_1.md). Fixes: Notebook pages now get layout; UTF-8-safe Text cursor; O(1) canvas item insertion; destroy() clears modal/notebook/scrollbar state; Combobox toggles closed when clicked; yview works on Text; text_set_max_lines(0) means unlimited; consistent dark palette; bounded process_posted; poisoned-mutex recovery; Labels/Frames with tooltips are hoverable; non-finite sizes sanitized; correct colormap tables; Var no longer uses unsafe and gained Eq/Hash; more docs.

## 0.8.0
- **Text** widget: multi-line editing, wheel scrolling, read-only mode, `text_get/set/insert_end/clear`, `text_set_max_lines` for logs.
- **Listbox** `listbox_bind_select` callback (mouse, keyboard and programmatic changes).
- **Combobox** (index-bound `Var<i64>`, overlay dropdown, keys/wheel), `combobox_set_values`, `combobox_get`.
- **Notebook** tabs (`notebook_add/select/selected/bind_changed`) and `pack_forget` / `grid_forget` / `place_forget` / `is_mapped`.
- **ProgressBar** bound to `Var<f64>` (`f64_var`, `progress_bind`) and indeterminate mode.
- `Opt::Monospace(bool)`; **tooltips** (`tooltip`, `tooltip_remove`, `active_tooltip`).
- **Canvas RGBA images** (`canvas_create_image_rgba`, `canvas_update_image`, nearest/bilinear) and `colormap` module (Viridis, Inferno, Magma, Plasma, Jet, Turbo, Grayscale; `heatmap_to_rgba`, `legend_rgba`).
- **Thread-safe `TkProxy`** (`tk.proxy().post(..)` wakes the event loop), `process_posted`, `after_idle`. `Var<T>` is now `Send + Sync` (identity handle only).
- **Dialogs** (`dialog` module, in-app modal overlay): message box, yes/no, ask string, typed confirmation, open/save file, directory.
- **Modern theming**: `Theme::ModernDark` / `Theme::ModernLight`, `set_accent`, selected state (`set_selected`), named styles `Card`, `Sidebar`, `Sidebar.TButton`, `Accent.TButton`, `Ghost.TButton`, `Heading`, `Subheading`, `Muted`, `Mono`, `Chip.*`.
- **Icons** (`icons` module, 28 vector icons) + `register_image_rgba`; `Opt::Compound` for image + text.
- Examples: `modern.rs`. Tests: `tests/v08_widgets.rs`, `tests/v08_extras.rs`.
- Note: `Theme` gained variants; exhaustive `match`es need a wildcard arm.

## 0.7.0
- Scrollable Canvas: `canvas_scrollregion`, `canvas_view`, `yview`/`xview`, `yview_moveto`/`xview_moveto`, `yview_scroll`/`xview_scroll` with `ScrollUnit::{Units, Pages}`.
- `link_scrollbar` accepts a Canvas. The scrollbar thumb, track clicks and dragging now share one model for Listbox and Canvas.
- Wheel: Canvas scrolls 48 px per line, Shift+wheel scrolls horizontally, touchpad `PixelDelta` is exact, and wheeling over a scrollbar forwards to its target. New `sim_wheel_xy`.
- Keyboard: new `Key::PageUp`/`Key::PageDown`. A focused scrollable Canvas handles PgUp/PgDn/Space/Home/End/arrows.
- Canvas callbacks get content coordinates (scroll offset applied).
- Added `examples/scrolling.rs` and `tests/scrolling.rs`.


## 0.6.0
- Canvas: `canvas_create_round_rect`, `canvas_create_round_rect_outline`, `canvas_create_polyline`, `canvas_create_polygon`, `canvas_create_ring`, `canvas_create_text_ex` (size + weight), and `Tk::text_size` for measuring text.
- `canvas_move` and `canvas_coords` support the new items. Added `tests/canvas_shapes.rs`.


## 0.5.0
- Timer API: `Tk::after` (one-shot), `Tk::every` (repeating), `Tk::after_cancel`, `TimerId`, `pending_timers`, `sim_advance_timers`.
- Event loop sleeps until the next timer deadline (no polling); timers can quit the app.
- `examples/timers.rs`, `tests/timers.rs`, timer unit tests.


## 0.4.0

### Added
- Advanced styling layer (`src/style.rs`) on top of the unchanged basic API: design tokens
  (`Tokens`, `Palette`, `Radius`, `TypeScale`), ttk-style named styles (`style_configure`,
  `Opt::Style`), per-widget `Opt::StyleOverride`, Frame inheritance of text properties,
  and `resolved_style`.
- State maps (`style_map`) for hover/pressed/focused/disabled/checked/selected, `Opt::Disabled`,
  and animated colour transitions (`sim_advance_time`, `is_animating`; the window keeps redrawing while animating).
- Rounded corners, borders, drop shadows (layered approximation), linear gradients, padding,
  margin, opacity; font family, weight, italic and size per widget.
- Themes `Fluent`, `Material`, `HighContrast` (yellow focus ring), plus improved Dark indicators;
  runtime switching keeps named styles.
- CSS-like stylesheets: `load_stylesheet`, `load_stylesheet_file`, `watch_stylesheet` +
  `poll_stylesheet` hot reload; errors carry the line number.
- Images on Buttons/Labels (`load_image`, `Opt::Image`).
- `examples/styling.rs`, `examples/styles/custom.css`, `tests/styling.rs`, and
  `docs/STYLING_COMPETITORS.md`.

### Fixed after verifying the model output
- Transitions started from the *new* colour (no visible animation); they now start from the
  previous state's resolved style, including press/release and click focus changes.
- Layout measured text without the resolved font family/weight/slant, clipping bold titles; Entry caret hit-testing too.
- Stylesheet `border` rejected `1px` and `solid`; lengths accept `px`.
- Disabled widgets had no visual change; every class now has a muted, 55 %-opacity disabled map.
- Compile errors (missing `ImageId`, private fields, borrow errors), clippy warnings, and test access to private internals.

## 0.3.0

### Added
- `Listbox`: text row list with selection, Up/Down/Home/End key movement, mouse wheel scrolling (`sim_wheel`), and selection change command callbacks.
- `Scrollbar`: vertical scrollbar linked to `Listbox`, supporting trough page clicks and thumb dragging.
- `Radiobutton`: circular radio button bound to a `Var<i64>` and specific value.
- `Canvas`: vector drawing canvas supporting rectangle, oval, line, and text items with generational `CanvasItem` handles and `canvas_bind_drag` callback handling.
- `place` geometry manager: absolute and relative layout (`x`, `y`, `relx`, `rely`, `width`, `height`, `relwidth`, `relheight`, `anchor`). Pure math function `place_rect`.
- Themes: `Theme::Light` and `Theme::Dark` via `set_theme` and `theme()`.
- `Spinbox`: entry-like number display with increment/decrement arrow buttons and arrow key stepping.
- `ProgressBar`: horizontal progress bar bound to a `Var<i64>` (0..=100).
- Extended Tab focus traversal to Listbox, Radiobutton, and Spinbox (excluding Canvas).
- Comprehensive headless tests in `tests/features.rs` and rich demo in `examples/demo.rs`.
- `canvas_clear` (like `delete("all")`).

### Fixed / hardened (after verifying the model output)
- Canvas items drawn in creation order, so a new item reusing a freed slot stacks on top.
- Canvas item slots with exhausted generations are retired; at most 2^20 live items per canvas.
- `canvas_move` on text items now moves y as well as x.
- Borrow errors, tiny-skia 0.11 mask API misuse, and clippy warnings fixed in generated code; the layout unit tests it dropped were restored.
- Accept AltGr (Ctrl+Alt) text input on Windows; independent review in REVIEW.md with regression tests.

## 0.2.0

### Fixed after review

- Zero-area clipped fills no longer leak pixels outside widgets (Entry text overflowed its box at scroll 0).
- Restored layout unit tests; added tests/vars.rs (get/set, equal-set no-op, trace loop guard, destroyed Var).
- Verified counter and form interactively on an X display with xdotool.

### Stability

- Generational widget IDs, subtree destruction, slot reuse, and stale-ID checks.
- Fallible constructors/rendering, optional queries, and boolean mutations.
- Typed variable ownership checks, destruction, and equal-value no-op sets.
- Reentrancy-safe command replacement and bounded variable trace notification.
- Zero-size and occluded-window guards; recoverable, logged surface errors.
- Empty-font fallback and bounded pixmap allocation.
- Iterative tree processing and safe rendering order after slot reuse.
- Finite, nonnegative, parent-contained layout; bounded grid indices and spans.
- Logical-pixel layout and HiDPI scaling, including text metrics and input.

### Widgets and input

- Boolean Checkbutton and horizontal integer Scale.
- Tab/Shift+Tab traversal, Space/Enter activation, and focus rings.
- Entry cluster-aware navigation/deletion for combining marks, variation
  selectors, skin tones, and ZWJ sequences.
- Control filtering, 64 KiB Entry limit, clipped text, and caret scrolling.
- Public headless event simulation using the real backend's handlers.

### Tests and documentation

- Deterministic 500-case layout fuzz test.
- Unicode, stale-ID, callback reentrancy, variable-cycle, event, rendering,
  zero-size, and HiDPI tests.
- Updated examples and added `examples/widgets.rs`.
- Documented migration, handle/error semantics, and remaining limitations.
- Kept the existing dependency requirements and Rust 1.85 compatibility.

## 0.1.0

- Initial tkinter-style arena API.
- Frame, Label, Button, and Entry.
- String/integer variables and text bindings.
- Pack/grid layout and recursive requested-size calculation.
- cosmic-text shaping, tiny-skia software rendering, softbuffer presentation.
- winit event loop and headless PNG screenshots.
