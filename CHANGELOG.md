# Changelog

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
