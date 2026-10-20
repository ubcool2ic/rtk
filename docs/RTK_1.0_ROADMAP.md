# rtk → 1.0 roadmap (audit of v0.8.1 @ 61c0a11, 2026-10-20)

Read-only audit. No code changed in rtk. All findings come from reading the source and running
`cargo test --all-targets` locally (103 tests pass, 0 fail). No paid model calls were made.

**Decision already made:** rtk bundles one OFL font (Inter or Noto Sans, Regular and Bold), with
fallback to Arial and then the platform's sans font. This is being built in the v0.8.2 font-fix PR,
so it is counted as in progress, not as a gap.

---------------------------------------------------------------------------------------------------
## 1. Current state (v0.8.1)

Size: about 12.3k lines of Rust (354 KB in `src/`). `src/tk.rs` alone is 2,923 lines and 120 KB, a
single "god object" with 110 `pub fn`s. Dependencies: winit 0.30, softbuffer 0.4, tiny-skia 0.11 and
cosmic-text 0.12 (`Cargo.toml`). MSRV is 1.85, edition 2021.

| Area | State | Files |
|---|---|---|
| Widgets (15 kinds) | Frame, Label, Button, Entry (single line), Checkbutton, Radiobutton, Scale (integer only), Spinbox (integer only), ProgressBar (`Var<f64>`, plus an indeterminate mode), Listbox, Scrollbar (vertical only), Canvas, Text (multi-line), Combobox, Notebook (tabs). Tooltips. In-app dialogs. 28 vector icons. | `src/widgets.rs` (`enum Kind` at line 431), `src/tk.rs`, `src/dialog.rs`, `src/icons.rs` |
| Canvas | rect, oval, line, text, round-rect, polyline (sparklines), polygon, ring, RGBA images, colormaps, scrollregion, drag. This is the only way to draw charts today, all by hand. | `src/tk.rs`, `src/colormap.rs` |
| Layout | Tk-style pack, grid (weighted tracks, capped at 4,096) and place, plus `*_forget`. Fuzz-tested. **There is no scrollable Frame.** Only Canvas, Listbox and Text scroll. | `src/layout.rs`, `tests/layout_fuzz.rs` |
| Theming | Tokens (palette, spacing, radius, type scale, font family), ttk-style named styles, cascade, state maps with animated transitions, shadows, gradients, CSS-like stylesheets with hot reload. Themes: Light, Dark, Fluent, Material, HighContrast, ModernDark, ModernLight, plus an accent color. Fairly mature. | `src/style.rs` |
| Text and fonts | cosmic-text. `FontSystem::new()` loads only system fonts (`src/render.rs:33`), and themes name fonts like "Segoe UI" and "Roboto" that may not be installed (`style.rs:204`, `:229`). **Known bug:** button styles ask for weight 500 (`style.rs:489`, `587`, `615`, `722`). On Windows, Arial has no 500 weight, so cosmic-text picks the wrong face and the labels come out garbled. Fixed in the v0.8.2 PR along with the bundled font. | `src/render.rs`, `src/style.rs` |
| Keyboard and mouse | Tab/Shift+Tab focus order, focus rings, Space/Enter activate buttons, AltGr fix. The key set is small (`src/app.rs:305-323`). There is **no Ctrl/Cmd shortcut API**. **Known gap:** Enter in an Entry does nothing; there is no submit callback (`tk.rs:2027` passes Enter to `edit_entry`, which ignores it). Fixed in the v0.8.2 PR. | `src/app.rs`, `src/tk.rs` |
| IME, clipboard, selection, undo | **None.** winit `Ime` events are not handled, and there is no `arboard`. Entry and Text have no text selection and no undo. | `ROADMAP.md` "Future Work" |
| Window | `title`, `geometry` and `quit` (`tk.rs:655-667`). **No window icon API**, no min size, no resizable flag, no multiple windows or Toplevel, no close-request hook. The window is built with `.with_title` only (`app.rs:69`). | `src/app.rs` |
| Rendering | CPU only: tiny-skia and softbuffer, redrawing the **whole window** every frame at up to 60 Hz. About 1.4 ms per frame at 800×600 and 2.5 ms at 1080p (`docs/PERFORMANCE.md`). Idle CPU is 0%. No damage regions and no GPU. | `src/render.rs`, `docs/PERFORMANCE.md` |
| Platforms | winit should cover Windows, Linux (X11 and Wayland) and macOS, but **only Linux X11 is tested**. Windows has been used by Tyler with real bugs (the font bug). macOS has never been run. | — |
| HiDPI | Logical-pixel layout, and `scale_factor` and `ScaleFactorChanged` are handled (`app.rs:72`, `:266`). Never checked on a real 150% Windows display. | `src/app.rs` |
| Accessibility | **None.** No AccessKit and no screen reader support. HighContrast is the only accessibility feature. | — |
| Threading | Single-threaded, like Tk. `TkProxy::post` wakes the loop from other threads. `after`, `every` and `after_idle` timers. `Var<T>` is a handle that is `Send + Sync`. No async or futures integration. | `src/timer.rs`, `src/var.rs`, `src/tk.rs` |
| Tests | 103 tests, all headless (`sim_*` event simulation, PNG rendering). **No screenshot golden tests.** `docs/REVIEW_FABLE_5_1.md` says `tests/tk_review_regressions.rs` was added, but **that file does not exist**. | `tests/` |
| CI | `ci/rust.yml` exists, but **it is not active** (there is no `.github/workflows`), and it only targets ubuntu. | `ci/rust.yml` |
| Docs and examples | README, CHANGELOG, ROADMAP, a performance doc and a styling comparison. The v0.8.1 review added rustdoc to most items but did not check that every public item has one. 11 examples (`demo`, `modern`, `styling` and others), but no single gallery of every widget. | `docs/`, `examples/` |
| API stability | tkinter-style `Opt` enum plus about 110 `Tk` methods, all named `<widget>_<verb>`. Enums are not `#[non_exhaustive]`, and v0.8 already broke exhaustive matches on `Theme`. There is no semver policy. | `src/lib.rs`, `src/widgets.rs` |
| Open review debt | The Fable review's pass 2 returned nothing, so **`render.rs`, `style.rs` and `layout.rs` have never been reviewed**. Pass 1 F24–F31 were deferred. Pass 3 F2, F5 and F6 were truncated and never applied: icon buffers under 8 px, nested dialogs overwriting modal state, and `ask_directory`. | `docs/REVIEW_FABLE_5_1.md` |
| Publishing | Used as a git dependency. **The name `rtk` is already taken on crates.io** (an unrelated "Rust Type Kit" CLI, v0.1.0). | — |

Apps that depend on rtk and exist on GitHub: `rtk-weather`, `esp-boot-manager`, `pulsegrow-climate`,
`macchina`, `dropbox-extractor`. `rtk-llm` and `pulsegrow` were not found under those names.

---------------------------------------------------------------------------------------------------
## 2. Gaps compared with a credible 1.0 desktop toolkit

**MUST** = needed for 1.0. **SHOULD** = strongly wanted, can slip to 1.x. **LATER** = 1.x or 2.0.

| Feature | Today | Priority |
|---|---|---|
| Bundled font + weight fallback (fixes this whole class of bug) | v0.8.2 PR in progress | done / in progress |
| Enter-to-submit on Entry (`on_submit`) | v0.8.2 PR in progress | done / in progress |
| Linear progress: determinate + indeterminate, themed, with label/percent, smooth animation | basic version exists | MUST (polish) |
| Circular / ring progress: determinate + spinner | none (only `canvas_create_ring`) | MUST |
| Slider: `f64`, step, vertical, ticks | integer horizontal only | MUST |
| Toggle / switch | none | MUST |
| Checkbox tri-state, radio group helper | checkbox and radio exist | SHOULD |
| Dropdown / combobox: editable, filter-as-you-type | read-only combobox | done in 0.11 (`Combobox::editable`) |
| Numeric spin box: `f64`, step, format | integer only | MUST |
| Tabs | Notebook exists | done (closable tabs LATER) |
| Table / data grid: columns, sorting, resizable columns, virtualized rows | none | MUST |
| Tree view | none | SHOULD |
| Scroll area (scrollable Frame of widgets), horizontal Scrollbar | none | MUST |
| Modal dialogs / message boxes | in-app overlay exists, nesting bug (F5) | MUST (fix nesting) |
| File dialogs | in-app only; native through `rfd` (v0.17.2) | done in 0.11 (`rtk::file_dialog`, opt-in `native-dialogs` feature) |
| Tooltips | done | done |
| Menus: menubar, context menu, accelerators | none | MUST |
| Notifications / toasts (in-app) | none | SHOULD |
| Charts: line, area, bar, time axis, live append, sparkline, legend (weather, climate and CAN logging) | hand-drawn on Canvas | MUST |
| Image widget (PNG/JPEG), SVG icons | RGBA and PNG on buttons | SHOULD |
| **Window icon API** (`tk.iconphoto`), min size, resizable, close hook | none | MUST |
| Multiple windows / Toplevel | none | SHOULD |
| System tray (`tray-icon` v0.26.1) | none | LATER (optional feature) |
| Clipboard: copy, cut, paste (`arboard` v3.7.0) | none | MUST |
| Text selection + undo/redo in Entry and Text | none | MUST |
| IME composition (CJK input) | none | SHOULD |
| Drag and drop of files onto the window | none | SHOULD |
| Keyboard shortcuts / accelerators API (Ctrl/Cmd mapping) | none | MUST |
| Theme tokens v2: semantic color roles, spacing, radius, typography; light, dark and high-contrast that pass WCAG AA; colorblind-safe chart palette (Okabe-Ito); follow the OS dark mode | tokens exist, no semantic roles or chart palette | MUST |
| HiDPI checked at 100, 125, 150 and 200% on Windows and macOS | implemented, unverified | MUST |
| Accessibility through AccessKit (`accesskit` v0.25.1, `accesskit_winit` v0.34.1; check it supports winit 0.30) | none | SHOULD (MUST if public) |
| Localization basics: UTF-8 everywhere, RTL layout mirroring, bidi caret | cosmic-text shapes bidi, layout does not mirror | LATER (strings) / SHOULD (bidi caret) |
| Damage-region rendering (CPU budget while dragging) | full redraw | SHOULD |
| GPU backend | none | LATER |

---------------------------------------------------------------------------------------------------
## 3. Criteria for 1.0

1. **API freeze and semver.** Settle the API shape first (see decisions). Mark public enums
   `#[non_exhaustive]` (`Opt`, `Theme`, `Key`, `Kind`-like enums). Split `tk.rs` into per-widget
   modules behind the same `Tk` facade. Write a written semver policy: no breaking change without
   2.0, deprecated APIs live one minor release.
2. **Deprecations removed.** Remove the legacy `Bg`/`Fg` palette paths that duplicate tokens, the
   integer-only Scale/Spinbox/ProgressBar variants once the `f64` versions land, and every
   `#[deprecated]` item.
3. **Docs.** `#![deny(missing_docs)]` so every public item has rustdoc and a short example. A guide
   (`docs/guide/`) covering layout, styling, threading, charts and packaging a Windows exe. One
   **gallery** example that shows every widget in every theme; it doubles as the screenshot source.
4. **Visual regression tests.** Golden PNGs for each widget × theme (Light, Dark, HighContrast)
   × scale (1.0, 1.5, 2.0), rendered headless with the bundled font so pixels match on every OS.
   Compare with a small per-pixel tolerance.
5. **CI.** Move `ci/rust.yml` into `.github/workflows/`. Build matrix: windows-latest,
   ubuntu-latest and macos-latest. Steps: build, test, clippy `-D warnings`, doc build, golden tests,
   MSRV check.
6. **Performance budget**, enforced in CI by `examples/bench.rs`: frame ≤ 4 ms at 1080p, idle CPU
   0%, startup ≤ 150 ms warm, RSS ≤ 30 MB for the gallery, 10k-row table scroll at 60 fps.
7. **Review debt cleared.** Review `render.rs`, `style.rs` and `layout.rs` (never reviewed). Apply
   or reject the deferred F24–F31 and pass-3 F2/F5/F6 items. Add the missing regression test file.
8. **Publishing decision.** Either stay a private git or path dependency (simplest), or publish
   to crates.io under a new name, since `rtk` is taken.
9. **All five apps** (rtk-weather, esp-boot-manager, pulsegrow-climate, macchina,
   dropbox-extractor) port to the 1.0 API and build on Windows.

---------------------------------------------------------------------------------------------------
## 4. Milestones

LOC means new or changed Rust lines, including tests and docs comments.

| Version | Theme | Contents | Size | Est. LOC |
|---|---|---|---|---|
| **0.8.2** | Hotfix (in progress) | Bundled OFL font with Arial/sans fallback, button weight fix, Entry Enter-to-submit | S | ~0.4k |
| **0.9** | Foundations | Split `tk.rs` into modules, `#[non_exhaustive]`, window API (icon, min size, resizable, close hook), clipboard, Entry/Text selection + undo, shortcuts API, activate CI on 3 OSes, golden-test harness, review `render`/`style`/`layout` | L | ~5k |
| **0.10** | Progress and inputs | Linear progress polish, **ProgressRing + Spinner**, Switch, `f64` Slider (vertical, step, ticks), `f64` SpinBox, tri-state checkbox, horizontal Scrollbar, **ScrollArea** (scrollable Frame) | M/L | ~4k |
| **0.11** ✅ done | Data and navigation | **Table/DataGrid** (sort, resize, virtualized), TreeView, menubar + context menus, toasts, nested-modal fix, editable combobox, optional native file dialogs (`rfd`) | L | ~7k |
| **0.12** | Charts and media | **Chart** widget (line, area, bar, time axis, live append, sparkline, legend, colorblind palette), Image widget, SVG icons, file drag-and-drop, optional tray | M/L | ~4.5k |
| **0.13** | Theming, a11y, platform | Semantic token v2 + light/dark/HC/colorblind palettes + follow OS dark mode, AccessKit, IME, bidi caret, HiDPI verification on Windows and macOS, damage-region rendering | L | ~5k |
| **1.0-rc → 1.0** | Freeze | Remove deprecations, `deny(missing_docs)`, guide, gallery app, full golden suite, perf budget in CI, port the 5 apps, publishing | M | ~3k (+ docs) |
| **Total** | | | | **~28.9k** |

---------------------------------------------------------------------------------------------------
## 5. Cost with Claude Sonnet 5.5 (OpenRouter `anthropic/claude-sonnet-5.5`)

Prices: $2 per million input tokens, $10 per million output tokens. Reasoning tokens are billed as
output. Assumes `reasoning_effort: xhigh`, falling back to `max`.

**Assumptions for each 1,000 LOC delivered:**
- About 12 output tokens per line of code, so about 12k tokens of final code.
- **Low case:** reasoning ≈ 3× the code, plus one fix round, for about 70k output tokens. About
  400k input tokens (relevant modules resent each call, after the `tk.rs` split).
  Cost: 0.4M × $2 + 0.07M × $10 = **$1.50 per 1k LOC**.
- **High case:** xhigh/max reasoning ≈ 10×, plus 2–3 test-fix rounds, for about 300k output tokens.
  About 1.5M input tokens (big files resent, retries, max fallbacks).
  Cost: 1.5M × $2 + 0.3M × $10 = **$6.00 per 1k LOC**.
- One Sonnet review pass per milestone: **$3–10**.
- Prompt caching, if OpenRouter applies it, would cut input cost further. That is not counted.
- Splitting `tk.rs` first (0.9) matters for cost: today every widget change resends about
  30k tokens of `tk.rs`.

| Milestone | LOC | Low | High |
|---|---|---|---|
| 0.8.2 (already under way) | 0.4k | $1 | $5 |
| 0.9 | 5k | $11 | $40 |
| 0.10 | 4k | $9 | $34 |
| 0.11 | 7k | $14 | $52 |
| 0.12 | 4.5k | $10 | $37 |
| 0.13 | 5k | $11 | $40 |
| 1.0 | 3k | $8 | $28 |
| **Total** | **~29k** | **≈ $64** | **≈ $236** |

A realistic middle is about $120–150. Sonnet's code still has to be checked by building and testing
it on the box, as was done for the Gemini and Fable output (see `REVIEW.md`).

---------------------------------------------------------------------------------------------------
## 6. Decisions for Tyler

1. **Lift the rtk hold, milestone by milestone?** Every item above changes rtk.
2. **API shape for 1.0:** keep the tkinter-style `Tk` + `Opt` API (fewer breaks for the 5 apps), or
   move to typed builders per widget (cleaner, but a bigger migration). The recommendation is to
   keep the tkinter style, and add typed helpers.
3. **Platforms for 1.0:** Windows + Linux only, or macOS too? macOS adds a CI runner and real
   Cmd-key, menubar and HiDPI work.
4. **Publishing:** stay a private git or path dependency, or publish to crates.io under a new name.
5. **Native vs in-app file dialogs** (`rfd` adds platform dependencies).
6. **Accessibility (AccessKit):** required for 1.0, or 1.1?
7. **Budget:** approve about $65–240 of Sonnet 5.5 spend across all milestones, or set a
   per-milestone cap and review screenshots at each step.
8. **Rendering:** CPU only for 1.0 (recommended, with damage regions), with GPU later or never.
