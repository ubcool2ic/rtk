# Roadmap

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

## Geometry and core behavior

- Place manager with absolute and relative positioning.
- More complete Tk-compatible pack ordering/reordering.
- Visibility controls and reparenting.
- Cross-toolkit widget-handle validation.
- Runtime window geometry requests and richer constraints.
- Event bindings, timers, and scheduled callbacks.
- Cached measurements, shaped buffers, and partial redraws.

## Widgets

- Canvas with retained drawing items and hit testing.
- Radiobutton and grouped selection.
- Floating-point Scale and configurable step sizes.
- Listbox and selection models.
- Multiline Text.
- Scrollbars and scrollable containers.

## Text and platform integration

- Complete Unicode grapheme segmentation and bidirectional caret movement.
- Text selection, undo/redo, and clipboard support.
- IME composition, preedit, and surrounding-text integration.
- Accessibility trees, actions, focus, and announcements.
- Native menus, context menus, and keyboard accelerators.
- File, color, message, and other dialogs.
- Multiple windows and shared application-level state.

## Styling

- Themes and ttk-like styling.
- State-dependent typography, borders, spacing, and colors.
- Consistent dark mode and high-contrast behavior.
- Optional vector icons and richer relief styles.

## Quality

- Cross-platform CI and interactive backend/minimize/HiDPI smoke tests.
- Deterministic font fixtures and additional fontless rendering tests.
- Wider property testing of text, geometry, and event sequences.
- Performance benchmarks for large widget trees and long entry text.
