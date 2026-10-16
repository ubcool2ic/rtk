# Code reviews

## Review 1 — 2026-10-15, google/gemini-3.8-flash, base 2167315

Two independent passes over all of `src/` (12 raw findings). Every finding was
checked against the code by hand before any change. Regression tests live in
`tests/review.rs` and `src/app.rs`.

### Fixed (1)
- **AltGr text dropped on Windows** (`app.rs`). Windows reports AltGr as Ctrl+Alt, and any
  Ctrl chord was discarded, so characters like `@` / `€` on many layouts could not be typed.
  Ctrl+Alt now counts as text input; plain Ctrl/Super chords are still ignored.
  Test: `app::tests::altgr_text_is_accepted_but_shortcuts_are_not`.

### Rejected as false positives (9), with confirming tests where useful
- **Scale i64→f64 overflow/panic**: Rust `as` float→int casts saturate (NaN→0), the span of
  i64 endpoints is finite in f64, and `set_scale` clamps to `[from, to]`. Test
  `scale_with_extreme_range_stays_in_bounds`.
- **Text measurement panics with no fonts**: the no-font path never calls cosmic-text and
  only does arithmetic; nothing can panic.
- **Pack expansion with negative available space**: this matches Tk's packer; the result is
  clamped to ≥0 and the fuzz tests check bounds.
- **Var re-entrant set does not re-notify**: deliberate. It matches Tcl, which disables a
  variable's traces while they run. Widgets read the latest value. Test
  `reentrant_set_in_trace_keeps_last_value_tcl_semantics`.
- **`clusters()` with leading combining mark / cursor between clusters**: the model withdrew
  this itself. `cursor_boundary` always snaps to a valid cluster boundary.
- **Entry destroyed inside its Var trace**: the node is never touched after `write_entry`,
  and every access is generation-checked. Test `entry_destroyed_by_trace_while_typing`.
- **Stale `paint_order` after destroy**: hit testing and focus go through generation-checked
  lookups, and `destroy` marks the UI dirty so every event re-lays out first. Test
  `tab_after_destroy_skips_stale_and_finds_reused_slot`.
- **Press, drag out of the window, release leaves the button stuck pressed**: release
  clears pressed state without firing, and focus loss clears input. Test
  `press_leave_release_does_not_fire_and_does_not_stick`.
- **A panic inside a command leaves `command_running` set**: rtk does not catch panics, so
  a panicking callback unwinds out of `mainloop`. There is no state to recover.

### Deferred (2)
- **Var slots are never reused** (`var.rs`): destroyed Vars leave a small tombstone, about
  40 bytes. This is bounded by how many Vars are created and was chosen for unambiguous
  stale handles. Add a generational free list later.
- **Grid track cap of 4096**: allocates and scans up to 4096 floats per layout when a huge
  row or column index is used. This is bounded and intended; a sparse layout would be an
  optimization.
