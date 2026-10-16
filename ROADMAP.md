# Roadmap

## Completed in 0.4

- Design tokens, named styles, cascade with Frame inheritance, per-widget overrides.
- State maps with animated transitions; disabled state.
- Rounded corners, borders, shadows, linear gradients, padding/margin, opacity, per-widget fonts.
- Fluent, Material and High Contrast themes; runtime theme switching.
- CSS-like stylesheet loader with hot reload; images in Buttons/Labels.

## Completed in 0.3

- Listbox with row selection, Up/Down/Home/End navigation, and wheel scrolling.
- Vertical Scrollbar linked to Listbox with trough paging and thumb dragging.
- Radiobutton with shared `Var<i64>` group selection.
- Canvas with retained items (rect, oval, line, text) and reentrant drag events.
- Place geometry manager with absolute and relative positioning math.
- Light and Dark themes with runtime switching.
- Spinbox with up/down arrows and integer clamping.
- ProgressBar bound to `0..=100` values.
- Tab focus navigation covering all interactive widgets except Canvas.

## Completed in 0.2

- Generational widget IDs, safe stale-handle rejection, subtree destruction.
- Iterative tree layout/destruction and slot-reuse-safe paint/focus ordering.
- Safe variable queries/destruction, ownership validation, equal-set elision.
- Reentrant commands and bounded synchronous variable traces.
- Zero-size/occluded rendering guards and recoverable surface errors.
- Empty-font-database fallback and bounded headless image allocation.
- Logical-pixel layout, HiDPI input/rendering, and scaled text metrics.
- Bounded grid indices/spans and proportional grid shrinking.
- Entry cluster approximation, control filtering, size guard, clipping/scrolling.
- Tab/Shift+Tab traversal, keyboard activation, and focus rings.
- Checkbutton with boolean binding and command.
- Horizontal integer Scale with dragging, keyboard stepping, and command.
- Shared real/headless event handlers and deterministic integration tests.

## Future Work

- Complete Unicode grapheme segmentation and bidirectional caret movement.
- Text selection, clipboard, undo/redo.
- Floating-point Scale and custom step sizes.
- Multiline Text widget.
- Clipboard copy/paste in Entry (skipped in 0.3 to avoid a new dependency; `arboard` is the candidate).
- Horizontal Scrollbar, Scrollbar for Canvas, Canvas tags/hit-testing per item, polygons and arcs.
- Menus, dialogs (messagebox, file chooser), multiple top-level windows.
- Accessibility via `accesskit`, IME composition, ttk-style themable styles beyond Light/Dark.
- Better dark-theme contrast for radio/check indicators.
- Styling follow-ups: sub-control selectors (`TCheckbutton::indicator`), ID selectors,
  radial gradients, true Gaussian blur, transitions for size/radius, theme-aware Scale/Progressbar
  colours in stylesheets, colour transitions on keyboard (Tab) focus changes.
