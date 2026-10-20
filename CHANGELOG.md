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

### 0.10 Progress and inputs
- **ProgressRing** (`ProgressRing`, `RingMode::Percent`/`Spinner`): `ring_set`/`ring_get`, `ring_set_mode`/`ring_mode`, `ring_set_show_text` (centred "NN%"), `ring_set_thickness`. The spinner animates on the existing animation clock and only while visible.
- **ProgressBar polish** (opt-in, default look unchanged): `progress_set_show_percent`, `progress_set_label`, `progress_set_smooth` (eased value changes), `progress_get`. Text colour is picked per region for WCAG contrast.
- **Switch** (`Switch`): toggle bound to `Var<bool>`, click/Space/Enter, animated knob, `switch_get`/`switch_set`, `set_command`.
- **Slider** (`Slider`, `Orient`): `f64` slider with `slider_set_orient` (vertical), `slider_set_step`, `slider_set_ticks`, `slider_set_length`, `slider_get`/`slider_set`; drag, arrow/Home/End/PageUp/PageDown keys and wheel.
- **DoubleSpinbox** (`DoubleSpinbox`): `f64` spin box with `dspin_set_step`, `dspin_set_decimals`, `dspin_set_suffix`, `dspin_get`/`dspin_set`/`dspin_text`; arrows, keys, wheel and text entry (`,` accepted as decimal separator), no float drift.
- **Horizontal Scrollbar**: `Scrollbar::horizontal`, `scrollbar_orient`; drives the X view of a scrollable Canvas or a ScrollArea.
- **ScrollArea** (`ScrollArea`): scrollable frame of widgets. `scrollarea_content`, `scrollarea_scroll_to`, `scrollarea_offset`, `scrollarea_content_size`, `scrollarea_attach_scrollbars`; wheel/Shift+wheel, clipping, Tab focus scrolls into view.
- **Tri-state Checkbutton** (opt-in, default look unchanged): `checkbutton_set_tristate`, `checkbutton_state`/`checkbutton_set_state` with `CheckState::Unchecked`/`Checked`/`Mixed`, and `checkbutton_set_cycle_mixed`. By default a click on a mixed box makes it checked; with cycling on, clicks go Unchecked → Checked → Mixed. Mixed draws a dash in the box. `A11yNode::mixed` is new, and AccessKit reports `Toggled::Mixed`. Tests: `tests/m0_10_tristate.rs`, goldens `tristate_*`.
- Tests: `tests/m0_10_*.rs` and goldens `ring_*`, `progressbar_*`, `switch_*`, `slider_*`, `dspin_*`, `hscroll_*`, `scrollarea_*`. Example: `gallery_progress_inputs`.

### 0.11 — Data and navigation
- `Table` data grid: virtualized rows, numeric-aware stable sort (header click or `table_sort_by`), resizable columns, selection, keyboard and wheel navigation. 10k rows scroll at ~2 ms/frame (release, 1280x720; `examples/bench_table.rs`).
- `TreeView`: virtualized hierarchical list with expand/collapse, keyboard navigation and selection.
- Menus: `Menubar`, popup and context menus (`menu_popup`, `bind_context_menu`, `sim_right_click`), submenus, check items, display-only accelerator labels.
- Toast notifications (`toast`, `toast_with`, four kinds shown by icon shape as well as color, positions, max stack), driven by the animation clock.
- Fix: nested modal dialogs now use a modal stack, so closing an inner dialog restores the outer one (review finding F5). New `dialog::modal_depth`.
- `examples/gallery_m011.rs` gallery and goldens for all of the above in Light, Dark and HighContrast at 1x and 2x.
### 0.12 Charts and media
- **Chart widget** (`Chart`, `Sparkline`): `ChartKind::Line`/`Area`/`Bar`, the `chart_*` API (`chart_add_series`, `chart_set_data`, `chart_append`, `chart_set_max_points`, `chart_set_time_axis`, `chart_set_legend`, `chart_set_y_range`, `chart_set_series_color`, `chart_clear`, plus `chart_series_count`/`chart_series_len`/`chart_point`/`chart_bounds`). Time-axis labels are formatted in UTC. Live `chart_append` keeps only the last `max_points` per series. Optional legend row. `Sparkline` is a compact single-series chart. Series colours default to the colourblind-safe Okabe-Ito palette (`theme::OKABE_ITO`) and can be overridden per series or taken from theme tokens.
- **Image widget** (`Image`): RGBA (`image_set_rgba`), PNG (`image_load_png`, from a path or in-memory bytes) and SVG (`image_set_svg`) sources, with `ImageFit::Contain`/`Fill`/`Center`. SVG is rasterized at device resolution so it stays crisp at any scale. `Tk::load_svg` rasterizes SVG bytes into an `ImageId` for use as a Button/Label icon via `Opt::Image`. The default `svg` feature uses resvg 0.45 without text/system fonts.
- **File drag-and-drop**: `bind_file_drop`/`unbind_file_drop` and `bind_file_hover`/`DropPhase` (`Enter`/`Leave`/`Drop`). Drops bubble from the deepest widget up to the root. Headless harness: `sim_file_hover`, `sim_file_hover_cancel`, `sim_file_drop`, `drop_target_at`, `file_hover_target`, `hovered_files`. winit reports no drop position, so the last pointer position is used.
- **Optional `tray` feature**: system tray icon via tray-icon 0.26.1 (ksni backend on Linux, no GTK). tray-icon needs Rust 1.90, above the crate MSRV 1.85, so it is feature-gated and off by default.
- Tests: `tests/m012_*.rs` and goldens `charts_*`, `image_*`. Example: `gallery_charts_media`.

### 0.13 — theming, accessibility, platform
- **New: semantic theme tokens** (`rtk::theme`). `SemanticTokens` (from `Tk::semantic_tokens()` or `SemanticTokens::for_theme`) gives role-based colours (`SemanticColors`: background, surface, surface_raised, text, text_muted, text_disabled, accent/on_accent, border, focus_ring, success/warning/danger/info with `on_*` text colours, field, selection/on_selection, shadow), `Spacing` (xxs..xxl), `Radii` (none, sm, md, lg, pill), `Typography` (sizes caption..display, weights fixed at 400/700 for the bundled font) and a colourblind-safe `chart` palette (`OKABE_ITO`). `Tk::color(ColorRole)` looks one role up; `SemanticTokens::colorblind_safe()` swaps status colours to Okabe-Ito hues. WCAG helpers `theme::contrast_ratio` and `theme::relative_luminance`. Additive; no rendering change. Tests: `tests/m013_tokens.rs`, `theme::tests`.
- **Themes meet WCAG AA.** All 7 built-in token sets now pass text/background ≥ 4.5:1 (muted text included), accent/focus ≥ 3:1 and danger ≥ 4.5:1; HighContrast ≥ 7:1. Changed colours: Light muted + danger, Material muted, ModernLight muted/accent/danger (ModernLight gets a darker accent, #2563EB). Tests: `tests/m013_themes.rs`.
- **New: colourblind-safe mode.** `Tk::set_colorblind_safe(true)` switches status colours (success/warning/danger/info) to Okabe-Ito hues and applies danger to the live palette; it survives `set_theme`.
- **New: follow the OS dark mode.** `Tk::follow_system_theme(light, dark)`, `unfollow_system_theme`, `system_theme()`, `SystemTheme`; the window applies winit `Window::theme()` at start and `ThemeChanged` later. Headless: `sim_system_theme`.
- **New: IME composition** for Entry and Text (CJK input): winit `Ime::Preedit/Commit/Disabled`, `set_ime_allowed` only while an editable widget has focus, candidate window placed at the caret (`ime_cursor_area`). Preedit is drawn inline with an underline and never touches the value/Var until commit. Headless: `sim_ime_preedit`, `sim_ime_commit`. Tests: `tests/m013_ime.rs`.
- **New: dirty-rect (partial) redraws.** `Tk::render_frame` returns `FrameDamage::{Full, Partial(rects), None}`; hover/press/transition/Entry/Text changes repaint only the affected widget (inflated for focus ring and drop shadow) and the window presents with `present_with_damage`. Overlays, theme/layout changes and resizes still redraw fully. `invalidate_rect`/`invalidate_widget` for custom invalidation. Under Xvfb, a hover repaints 10–57k px instead of 307k. Tests check every partial frame is pixel-identical to a full render (1x and 1.5x): `tests/m013_damage.rs`.
- **HiDPI verified headlessly** at 100/125/150/175/200% (physical size, logical layout invariance, logical hit testing, text ink scaling, scale round-trip); goldens at 1.25x/1.5x; `docs/HIDPI.md` lists the remaining real-display checks.
- **New: accessibility.** Always-on `Tk::accessibility_tree()` (`rtk::a11y`: roles, labels, values, numeric ranges, toggled/focused/disabled, bounds, actions), `a11y_set_label`, `a11y_do_action`. The optional cargo feature **`accesskit`** (accesskit 0.25 + accesskit_winit 0.34) exposes it to screen readers (UIA / NSAccessibility / AT-SPI) and routes their actions back. Tests: `tests/m013_a11y.rs`, `a11y_accesskit::tests` (with the feature).
- Theme goldens: `tests/golden/theme_*` (Light, Dark, HighContrast, ModernLight, ModernDark, Light/Dark colourblind, 1x and 2x), `ime_preedit_light_1x`, `hidpi_*`.

### 1.0 Docs, gallery and review
- Docs: `docs/GUIDE.md` user guide (all 42 snippets compiled as doctests); README docs/gallery/features sections; `///` on every fn and `//!` on every file; `#![deny(missing_docs)]`; `cargo doc -D warnings` clean.
- Gallery: `examples/gallery.rs` shows every widget on 5 pages (Basics, Inputs & progress, Data, Charts & media, Theme) in all 7 themes; `--screenshot-dir DIR` writes 36 PNGs (incl. 2x). Smoke test `tests/gallery_smoke.rs` (RTK_GALLERY_TEST=1).
- New API: `Tk::listbox_bind_activate` (fires on Enter / double-click, not on selection change).
- Checkbutton/Radiobutton indicators use theme tokens (`field_bg`, `muted`, `accent`) and are dimmed (alpha 0.38) when disabled.
- `#[non_exhaustive]` added to `Theme` and `MessageKind` (match them with a `_` arm).
- Fixes from the Sonnet 5.5 review (see `docs/REVIEW_SONNET_5_5.md`): in-app dialogs drew off-centre and clipped (scrim anchor) and placed widgets painted under pack/grid siblings (paint order); menus and toasts clipped or missing at 2x; several `f32::clamp` panics (tooltip, scrollbars, combobox dropdown) and UTF-8 boundary panics (IME preedit selection, text caret, colour parsing); unclipped widget box painting; stylesheet comment/gradient parsing; transition overflow; drag-and-drop hover-only targets swallowing drops; degenerate damage rects dropped; `ask_directory` could not navigate (Fable F6) plus file-dialog filter syntax, save list and empty typed-confirm fixes (F7/F8/F11); TreeView on a not-yet-selected Notebook page rendered empty.
- Tests: Fable review regression tests (`tests/tk_review_regressions.rs`, `_2.rs`), `dialog_anchor`, `paint_order`, `listbox_activate`, `treeview_notebook`; 398 tests + 42 doctests.

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
