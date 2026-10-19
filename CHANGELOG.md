# Changelog

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
