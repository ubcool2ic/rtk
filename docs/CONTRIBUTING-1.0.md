# Contributing to rtk 1.0 (helper bots)

**Base branch:** `rtk-1.0` in ubcool2ic/rtk. The split landed in commit 5dcc5c1, and the golden-test
harness in 310958c. Branch off the latest `rtk-1.0`, name the branch `m0.10-<topic>`,
`m0.11-<topic>` and so on, and open PRs **into `rtk-1.0`**, never `main`. Use merge commits only.
Never force-push.

## Models and spend
- All code comes from OpenRouter `deepseek/deepseek-v4.1-flash-1017`. Turn **reasoning off**
  (`"reasoning": {"enabled": false}`): with reasoning on, it ran on for 60k+ tokens and returned no code.
  Use the API key file `~/.config/openrouter/env`. The `OPENROUTER_API_KEY` already in some shells is
  an **expired** key, so read the file.
- If tests or clippy still fail after DeepSeek's try plus one self-fix with the error output, send only the
  failing output and the relevant module to `openai/gpt-6.1-sol` at reasoning effort `high` (never max).
  If Sol returns no code or fails twice on the same item, stop on that item and note it in your PR.
- **Credit floor:** check `GET /api/v1/credits` before each task and roughly every $1 of spend. Below **$1.50**
  left, stop, push your WIP, and write `BLOCKED: OpenRouter credits low` in /workspace/progress/rtk-1.0.md.
- Log every call in your PR description: date, model, task, tokens in and out, cost, and a running
  total per model, plus a "Sol fixes" subtotal. The lead copies these into SPEND.md when merging.
- Reference driver, which handles streaming, edit blocks, checks, Sol escalation and the ledger:
  `/workspace/rtk-1.0-tools/ds.py`. Usage:
  `python3 ds.py TASK.md --label 10-ring --ctx Cargo.toml src/lib.rs src/widgets.rs src/tk/progress.rs ...`.
  It works on `/workspace/rtk-1.0`, so copy it and change `REPO` to your own clone.

## Module layout (after 0.9)
```
src/lib.rs          public re-exports; RtkError
src/app.rs          winit event loop, window creation, keyboard/mouse translation (real window only)
src/render.rs       CPU renderer (tiny-skia), render_to_pixmap/save_png, TextEngine (cosmic-text)
src/style.rs        tokens, themes, named styles, state maps, stylesheet parser
src/fonts.rs        bundled Noto Sans + Arial/platform fallback (v0.8.2)
src/widgets.rs      widget structs + ::new(), Kind enum, WidgetNode, Opt, Color, Theme
src/layout.rs       pure pack/grid/place math
src/var.rs          Var<T> handles      src/timer.rs  after/every
src/dialog.rs       in-app modal dialogs   src/icons.rs  vector icons   src/colormap.rs
src/testing.rs      golden-image harness (assert_golden, compare_pixmaps, golden_dir)
src/tk/mod.rs       Tk struct, TkProxy, Key, ScrollUnit, core accessors, child-module decls
src/tk/style.rs     theme/tokens/style API, resolved style, animation
src/tk/window.rs    title/geometry/quit/dirty
src/tk/tree.rs      add/destroy/config/set_command/widget_text
src/tk/geometry.rs  pack/grid/place/forget, measure, update_requested, layout
src/tk/events.rs    hit test, sim_mouse_*, sim_wheel*, sim_key, sim_text, focus traversal
src/tk/scroll.rs    scroll metrics, xview/yview, link_scrollbar
src/tk/{entry,text,listbox,canvas,combobox,notebook,progress,tooltip}.rs   per-widget Tk methods
```

## How to add a widget
1. `src/widgets.rs`: add a `Kind::<Name>` variant, any state fields on `WidgetNode`, and a
   `pub struct <Name>;` with `pub fn new(tk: &mut Tk, parent: WidgetId, ...) -> Result<WidgetId, RtkError>`
   that calls `tk.add(Kind::<Name>, parent, ...)`. Re-export it in `src/lib.rs`.
2. `src/tk/<name>.rs`: start with a `//!` header, then `use super::*;` and `impl Tk { ... }` holding the
   widget's methods, named `<name>_<verb>` in tkinter style (for example `ring_set`, `table_sort_by`).
   Declare it in `src/tk/mod.rs` (`mod <name>;`). Each file gets its own `mod` line, so expect easy
   merge conflicts there and in `widgets.rs` and `lib.rs`; keep those edits small.
3. **Size:** handle the kind's requested size in `src/tk/geometry.rs` (`update_requested`).
4. **Input:** handle hit-testing and input in `src/tk/events.rs` (`sim_mouse_down`, `sim_key`, ...). Mark it
   focusable in `Kind::focusable` if it takes the keyboard.
5. **Drawing:** draw it in `src/render.rs` using the resolved style. **Don't hard-code colours.** Read them
   from `tk.resolved_style(id)` or the theme tokens (`tk.tokens().palette.*`) so every theme, including
   HighContrast, works. Add a default style for the class in `src/style.rs` (`T<Name>`, as `kind_class_name` maps).
6. **Animation** (spinners, indeterminate bars): drive it through the existing animation clock
   (`sim_advance_time`, `is_animating`), so headless tests are deterministic and the idle CPU stays at 0
   when nothing is animating.

## Code style (required)
- Every `.rs` file starts with a `//!` header.
- Every function or method, private ones and tests included, has a `///` doc comment, and every public
  item has rustdoc.
- No `unsafe` and no crate-level `#[allow]`. Add a new dependency only if your milestone lists it
  (for example `arboard`, `accesskit`, `tray-icon`, `resvg`), make it an optional cargo feature where it
  pulls in platform libraries, and say so in the PR.
- Keep the tkinter-style `Tk` + `Opt` API, and prefer additive changes. Mark new public enums
  `#[non_exhaustive]`.

## Tests (all must pass before merge)
```
cargo build --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings          # stable toolchain (1.99)
cargo build --lib --examples --target x86_64-pc-windows-gnu
```
- **Behavior tests:** put them in `tests/<milestone>_<topic>.rs`, headless only. Drive widgets with
  `tk.sim_click`, `sim_mouse_move`, `sim_key`, `sim_text`, `sim_wheel`, `sim_advance_time` and
  `sim_advance_timers`, and check the results through the public API or `Var` values.
- **Screenshot (golden) tests:** for each new widget, add a scene and call
  ```rust
  use rtk::testing::{assert_golden, golden_dir, GoldenOpts};
  assert_golden(&mut tk, golden_dir(), "ring_light_1x", 320, 240, GoldenOpts::default());
  ```
  for Light, Dark and HighContrast at 1x and 2x (`tk.sim_scale_factor(2.0)`, with the size doubled).
  `RTK_BLESS=1 cargo test` writes or refreshes the PNGs under `tests/golden/`. **Look at the PNGs before
  committing them.** A mismatch writes `*.actual.png` and `*.diff.png`, which are git-ignored.
- Never delete or weaken existing tests. If a golden changes on purpose, re-bless it and say why in the PR.

## PR checklist
- [ ] Branched from the latest `rtk-1.0`, and the PR targets `rtk-1.0`.
- [ ] All 4 checks are green. Paste the test count.
- [ ] Goldens added and looked at. Paths are listed in the PR.
- [ ] Spend table (per model, plus the Sol subtotal) is in the PR body.
- [ ] Short CHANGELOG entry under `## 1.0.0 (unreleased)`, in your milestone's section.
