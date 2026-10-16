# rtk performance (0.4.x)

Measured 2026-10-16 on the shared build box. Numbers are honest for this machine, not a
benchmark lab: other agents were using the box at the same time, so expect ±20 % noise.

## Machine

| | |
|---|---|
| CPU | Intel Xeon (virtualised, model not exposed), 8 vCPUs |
| RAM | 15 GiB total (about 11 GiB in use by other jobs during the runs) |
| OS | Debian 13, Linux 6.12, X11 display `:10` (virtual desktop) |
| GPU | **none**: rtk renders on the CPU (tiny-skia) and presents with softbuffer |
| Toolchain | rustc/cargo 1.85.1 (Debian), Python 3.13.5 + Tk 8.6 for comparison |

## Results

rtk "before" = 0.4.0 at b471a80. rtk "after" = the fixes in this commit (see below).

| Metric | rtk hello | rtk demo | rtk styling | Python tkinter hello | Python tkinter demo | fltk-rs 1.5 hello |
|---|---|---|---|---|---|---|
| Binary size, stripped release | 5.9 MB (4.2 MB with LTO + opt-level s) | 6.0 MB | 6.0 MB | n/a (interpreter + 3.4 MB libtk/libtcl) | n/a | 1.1 MB |
| Shared libs needed | libc, libm, libgcc_s (X11/Wayland loaded at runtime) | same | same | python3, libtk, libtcl, libX11… | same | 44 (X11, fontconfig, Xft…) |
| Startup to first frame, cold cache | 294 ms | 220 ms | n/m | 275 ms | 149 ms | n/m |
| Startup to first frame, warm (median of 5) | 82 ms | 98 ms | 102 ms | 99 ms | 131 ms | 15 ms |
| Peak RSS (start + first frame) | 19.7 MB | 23.2 MB | 24.9 MB | 28.8 MB | 30.1 MB | 9.6 MB |
| RSS while running idle | 20.7 MB | 22.6 MB | 25.8 MB | 28.9 MB | 30.2 MB | 21.9 MB |
| Idle CPU over 30 s | 0 % | 0 % | 0 % | 0 % | 0 % | 0 % |
| CPU, hover sweep with transitions (10 s) | | | 98 % before → **26 %** after | | | |
| CPU, canvas drag drawing ovals | | 36 % → **6.7 %** | | | 1.0 % | |
| CPU, Scale drag | | 44 % → **10.5 %** | | | | |
| Clean release build (all examples, deps cached in registry) | 1 min 14 s wall (6 min 44 s CPU on 8 vCPUs) | | | | | 22 s (prebuilt FLTK) |

CPU % is one core = 100 %, for the app process only (the X server's share is not counted for
any toolkit). Tk and FLTK redraw only damaged regions and let the X server draw; rtk redraws the
whole window in software every frame, which is why tkinter is still cheaper while dragging.
fltk-rs and tkinter numbers are for hello/demo programs written to match rtk's examples
(`/tmp/pyb/*.py`, `fltk-rs` hello with a label and a button).

### Offscreen render benchmark (`cargo run --release --example bench`)

Average ms per frame of `render_to_pixmap` (layout cached unless stated):

| Scene | before | after |
|---|---|---|
| demo-like window 800×600 | 6.79 | **1.44** |
| demo-like window 1920×1080 | 7.78 | **2.45** |
| styled cards 960×600, Light / Material | n/m | 1.58 / 2.05 (animating hover 1.70 / 1.45) |
| 1,000 Labels: layout + render | 55.0 | **1.0** |
| 10,000 Labels: layout + render | 505 | **8.6** |
| Listbox with 1,000 rows: layout + render | 67.6 | **0.5** |
| Listbox with 10,000 rows: layout + render | 379 | **1.3** |

In the real window (styling example during the hover sweep) a frame costs about 3.5 ms
render + 0.6 ms convert/present (was 4.1 ms) at 58 fps (was 119 fps).

## What was fixed (this commit)

1. **Text shaping every frame**: every label was reshaped with cosmic-text on every frame.
   Added a shaped-buffer cache (2,048 entries) and a separate measurement cache (65,536 entries).
2. **Glyph pixels drawn as 1×1 tiny-skia rectangles**: glyph coverage is now blended directly into
   the pixmap with integer clipping.
3. **Listbox cost grew with total rows**: rendering cloned every row each frame, and layout
   measured every row. Now only visible rows are cloned and at most the first 256 rows are
   measured for the requested width (tkinter sizes Listbox in characters, not by content).
4. **Uncapped redraw rate**: every mouse event could trigger a full redraw (119 fps while
   hovering). Redraws are now capped at about 60 Hz.
5. **Idle wake-ups**: the loop woke every 500 ms even when nothing was watched; it now sleeps
   until an event arrives unless a stylesheet is being hot-reloaded.
6. **Window present**: an opaque-pixel fast path skips the per-pixel un-premultiply division.
7. **Animation clock**: the first transition frame after a pause used the time since the *last*
   animation (could skip a transition entirely); the clock now restarts when animations start.

Opt-in diagnostics: `RTK_FIRST_FRAME=1` prints time from `Tk::new` to the first presented frame,
`RTK_EXIT_AFTER_FIRST_FRAME=1` exits right after it, `RTK_STATS=1` prints per-frame render and
present times.

## Known costs / not fixed

- Full-window software redraw on every change (no damage regions). Fine at 60 Hz for typical
  windows; a 4K window will cost several ms per frame on CPU.
- Startup is dominated by loading the system font database (about 60–80 ms warm).
- Creating 10,000 widgets takes about 94 ms (creation is roughly quadratic); 1,000 takes 1 ms.
- Binary size is about 6 MB (cosmic-text + swash + font database + winit's X11/Wayland
  backends); about 4 MB with LTO and `opt-level = "s"`.

## Minimum requirements (estimate)

- **OS:** Linux with X11 or Wayland (tested on X11); Windows 10+ and macOS 11+ should work
  through winit/softbuffer, but this has not been tested.
- **CPU:** any x86-64 or ARM64 from the last ~10 years; a single core at ~1.5 GHz is plenty for
  typical windows (2–4 ms per frame here at 800×600–1080p).
- **RAM:** about 25 MB per app at runtime; 64 MB free is a comfortable minimum.
- **GPU:** not required (pure CPU rendering).
- **Fonts:** at least one system font (fontconfig/fontdb); without fonts rtk runs but draws no text.
- **Build:** Rust 1.85+, about 2 GB disk for a release `target/` with dependencies.
