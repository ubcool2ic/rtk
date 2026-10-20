# HiDPI in rtk

rtk separates **logical** layout from **physical** rendering. Every widget
rectangle, pointer coordinate, padding and font size you write in code is in
logical pixels; the renderer multiplies by the window's scale factor only when
it rasterises into the physical framebuffer.

## How it works

- **Logical layout.** `Tk::layout` computes widget rectangles in logical pixels
  from the viewport divided by `scale_factor`. `widget_rect(id)` returns those
  logical rectangles, so geometry is identical at 100 %, 125 %, 150 %, 175 % and
  200 %.
- **Physical rendering.** `render_to_pixmap(w, h)` / `render_frame(w, h)` take
  the *physical* size. Text is measured and shaped at `font_size * scale_factor`
  physical pixels (`TextEngine::measure_styled`), so glyphs stay sharp instead
  of being upscaled.
- **Scale changes.** In `src/app.rs`, `WindowEvent::ScaleFactorChanged` calls
  `Tk::sim_scale_factor(new_factor)` and then re-reads `window.inner_size()` and
  calls `Tk::sim_resize`, so layout and the framebuffer stay in sync when the
  window moves between monitors with different DPI.
- **Mouse coordinates.** `WindowEvent::CursorMoved` divides the physical cursor
  position by `scale_factor` before calling `sim_mouse_move`, so hit testing and
  drags operate in logical pixels. Wheel deltas are converted the same way.

## What the headless tests verify

`tests/m013_hidpi.rs` builds one scene (Frame, Label, Button, Entry,
Checkbutton, Scale, ProgressBar, Listbox, Combobox) and checks, for scale
factors **1.0, 1.25, 1.5, 1.75 and 2.0**:

- rendering at `round(320*s) × round(240*s)` yields a pixmap of exactly that
  physical size and never panics;
- every widget's logical `widget_rect` is identical across scales;
- hit testing is in logical pixels: clicking the logical centre of the Button
  runs its command and of the Checkbutton toggles its variable;
- the Label's rendered ink area grows roughly with `s` (within ±20 %);
- `sim_scale_factor` rejects `0`, negative, `NaN` and values `> 16`;
- switching 1.0 → 1.5 → 1.0 reproduces pixel-identical output.

Golden images `hidpi_light_1_25x` (400×300), `hidpi_light_1_5x` (480×360) and
`hidpi_dark_1_5x` (480×360) are asserted via `rtk::testing::assert_golden`.

## What still needs a real-display check

Headless tests cannot exercise the OS compositor. Verify manually:

- **Windows 125 % / 150 % per-monitor DPI.** Drag the window between a 100 %
  and a 150 % monitor; text should re-sharpen and layout should not jump.
- **macOS Retina 2×.** Confirm crisp text and correct logical hit targets.
- **Wayland fractional scaling** (e.g. GNOME 125 %). Confirm the window is not
  blurry and that pointer coordinates map to the right widgets.

### Manual checklist

```sh
RTK_STATS=1 cargo run --example demo
```

1. Move the window between monitors with different DPI; watch the
   `rtk: frame render … ms` line and confirm no layout jump.
2. Click buttons, drag the slider and scroll a list; targets should match the
   visible widgets at every scale.
3. Resize the window; text should stay sharp (no upscaled bitmap look).
4. On Wayland, toggle fractional scaling and repeat steps 1–3.
