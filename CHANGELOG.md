# Changelog

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
