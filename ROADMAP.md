# Roadmap

The first release establishes the arena API, pure layout layer, variable
bindings, software rendering, and a working event loop.

## Geometry and core behavior

- Place manager with absolute and relative positioning.
- More complete Tk-compatible pack ordering/reordering and grid shrink behavior.
- Explicit widget destruction, visibility controls, and generational IDs.
- Cross-toolkit handle validation.
- Runtime geometry requests and richer size constraints.
- Event bindings, focus traversal, keyboard activation, and timers.
- Cached measurement, shaped buffers, and partial redraws.

## Widgets

- Canvas with retained drawing items and hit testing.
- Checkbutton and Radiobutton with variable bindings.
- Scale.
- Listbox and selection models.
- Multiline Text with selection and editing commands.
- Scrollbars and scrollable containers.

## Text and platform integration

- Grapheme-aware and bidirectional cursor movement.
- Text selection, undo/redo, and clipboard support.
- IME composition, preedit rendering, and surrounding-text integration.
- HiDPI scale-factor handling for layout, text, input, and screenshots.
- accesskit accessibility trees, actions, focus, and announcements.
- Native menus, context menus, and keyboard accelerators.
- File, color, message, and other dialogs.
- Multiple windows and shared application-level state.

## Styling

- Themes and ttk-like styling.
- State-dependent colors, typography, borders, and spacing.
- Consistent dark mode and high-contrast behavior.
- Optional vector icons and richer relief styles.

## Quality

- Cross-platform CI and interactive backend smoke tests.
- Deterministic font fixtures for rendering tests.
- Property tests for clipping, geometry, and Unicode editing.
- Performance benchmarks for large widget trees and long entry text.
