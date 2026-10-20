# Sonnet 5.5 review/fix pass (rtk 1.0)

Model `anthropic/claude-sonnet-5.5`, reasoning budget 16k tokens (never xhigh/max), max_tokens 32k, ~350-line chunks reviewed bottom-up. A chunk whose edits break build/test/doctest/clippy/doc/Windows checks is reverted and noted. A no-code reply is split once, then skipped (never retried at the same settings).

### src/render.rs lines 1352-1397
- [high] [bug] Tooltip placement uses `f32::clamp(2.0, max)`, which panics when `max < 2.0`. That happens in a small or narrow window, or with a tooltip wider than the window, and also if `max` is NaN. The upper bounds are now floored at 2.0 with `.max(2.0)` (lines 1366-1367) -> fixed
- [low] [bug] The `save_png` chunk (lines 1384-1395) and the rest of the chunk showed no other issues; `save_png` has rustdoc and propagates errors rather than panicking -> not fixed (nothing to fix)

### src/render.rs lines 1002-1351
- [high] [bug] Combobox dropdown height uses `f32::clamp(item_h, 240.0)`. It panics when `item_h > 240` (a large `font_size`, about 182 or more) and also on NaN (line 1314) -> fixed
- [medium] [bug] Combobox rows beyond the 240px dropdown height are painted outside the dropdown box, because `drop_clip` is the whole window (lines 1316-1336) -> fixed by clipping to the dropdown rect. This is only visible in the fill and outline calls shown here. Rows past 240px are still clipped rather than scrollable, and the text-draw call is cut off at line 1351, so I couldn't check its clip argument.
- [medium] [bug] Compound button layout (Left, Top, ImageOnly) treats the image pixmap size as logical units. The pixmap is blitted unscaled in physical pixels, so on HiDPI the image is mispositioned and the text/image gap and centering are wrong (line 1189) -> fixed by converting the size to logical (`/ scale`). This assumes `scale` is nonzero and that images are meant to be 1:1 physical pixels. The layout code that sizes the button isn't in this chunk, so check it uses the same convention.
- [low] [bug] Compound button images are blitted with no clip mask, so they can bleed outside a scrolled or clipped parent (lines 1203-1267) -> not fixed (needs a mask built per draw and a wider refactor)
- NOTE: edits for src/render.rs:1002-1351 were REVERTED (checks failed or SEARCH mismatch): '[test failed]\n\n.................................\ntest result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s\n\n\n..\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s\n\n\n.................\ntest result: ok. 17 passed; 0 failed;'

### src/render.rs lines 652-1001
- [high] [bug] Horizontal scrollbar thumb: `f32::clamp(16.0, track_w)` panics when the track is narrower than 16 px (min > max). `visible / total` is also NaN or inf when `total` is 0, and the scroll fraction is unclamped (line 901) -> fixed
- [high] [bug] Vertical scrollbar thumb has the same `clamp` panic when the track is shorter than 16 px, the same division by zero, and the same unclamped fraction (line 913) -> fixed
- [low] [bug] Scrollbar thumb rect width/height (`logical.height - 2.0`, `logical.width - 2.0`) can go negative on very thin tracks (lines 905, 917) -> fixed
- [low] [bug] Canvas/Chart/Image sub-pixmap is allocated at the full widget size with no upper bound, so a huge widget could allocate a very large buffer (line 963) -> not fixed (the clip-aware sizing needs context outside this chunk)

### src/render.rs lines 302-651
- [high] [bug] `render_styled_box` only uses `clip` for an early-out. Shadow, background and border were painted unclipped, so widgets partly outside a scroll or frame clip bled over neighbours (lines 312-394). -> fixed with a clip mask, built only when the shadow, border or rect can cross the clip.
- [medium] [bug] The border was stroked centred on the rect edge, so half of it fell outside the widget and a 1px border blurred over 2px (line 390). -> fixed by stroking a path inset by half the border width.
- [high] [bug] When an IME preedit is active, `lines` and `line_starts` come from `display` (text with preedit inserted). The selection offsets refer to `full_text`, so the `&line[..]` slices can land off a UTF-8 boundary and panic (lines 548-616). -> fixed by skipping selection painting during preedit.
- [high] [bug] The caret code sliced `full_text[..cur]` with `cur = text_cursor.min(len)`. That panics if the cursor is not on a char boundary, although the preedit path already uses `floor_char_boundary` (line 627). -> fixed.
- [low] [bug] `render_to_pixmap` called `sim_resize(width, height)` before validating the size, so zero or huge dimensions reached layout (lines 403-404). -> fixed by validating first.
- [low] [bug] The text caret is drawn at the un-preedited cursor position while a preedit is shown (line 626). -> not fixed (cosmetic; the preedit underline marks the composition).
- [low] [bug] The focus ring strokes (lines 486, 489) are not clipped to the widget clip. -> not fixed (outside the minimal scope; the ring is inset inside the widget rect).
- NOTE: edits for src/render.rs:302-651 were REVERTED (checks failed or SEARCH mismatch): '[test failed]\n\n.................................\ntest result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s\n\n\n..\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s\n\n\n....... 7/17\nedit_selection_golden --- FAILED\n.........'

### src/render.rs lines 1-301
- [severity low] [bug] `draw_styled` rounds `origin` but never bounds it. A huge or infinite origin saturates to `i32::MAX` on the `as i32` cast, and `origin.0 as i32 + x` and `left + w as i32` then overflow, panicking in debug builds (lines 190, 206-209) -> fixed (origin clamped to ±1e7 px, far outside any real pixmap, so drawing is unchanged).
- [severity low] [docs] The crate-private `TextEngine` struct has no doc comment (line 27) -> fixed
- [severity low] [bug] `fill_circle` allocates a full-pixmap `Mask` on every call. This is only a performance cost and is correct, so I left it (line 291) -> not fixed (no correctness impact; a refactor is out of scope)
- [severity low] [bug] `outline` draws the corner pixels twice, so a translucent outline colour double-blends there (lines 262-273) -> not fixed (cosmetic and only for translucent colours; changing the geometry risks pixel-test regressions)

### (re-run with golden re-bless allowed) src/render.rs lines 1003-1352
- [severity medium] [bug] Combobox dropdown height used `f32::clamp(item_h, 240.0)`, which panics when `item_h > 240.0` (font_size above about 178 px, or a NaN) (line 1317) -> fixed
- [severity medium] [bug] Combobox dropdown drew every row, even when the list was taller than the 240 px popup. Rows and highlights spilled below the popup box over other widgets (lines 1328-1331) -> fixed (stop drawing at the popup's bottom edge)
- [severity low] [bug] Compound-button images (Left, Top and ImageOnly) are drawn at their raw pixel size but laid out as if that size were logical, so they look wrong when `scale != 1`. They are also drawn without a clip mask or opacity (lines 1203-1270) -> not fixed (the intended image-scaling semantics aren't visible in this chunk, and a fix would change pixel output broadly)
- [severity low] [bug] The canvas clip path allocates a full-pixmap `Mask` for every canvas on every frame (line 1142) -> not fixed (a performance cost only, not a correctness bug)

### (re-run with golden re-bless allowed) src/render.rs lines 303-652
- [severity high] [bug] Text widget: while an IME preedit is active, `line_starts` is computed from `display` (text plus preedit), but the selection offsets are byte offsets into `full_text`. The mismatch can slice `line[..]` off a UTF-8 boundary and panic, and otherwise highlights the wrong range. The selection is now ignored while composing (line 549) -> fixed
- [severity high] [bug] Text caret: `full_text[..cur]` used `text_cursor.min(len)`, which is not guaranteed to be a char boundary and can panic on multibyte text. The code already computed the floored `caret` for the preedit path; the caret now uses it (line 628) -> fixed
- [severity medium] [bug] `render_to_pixmap` called `sim_resize(width, height)` before validating the size. A 0 or huge size left the toolkit resized to an invalid size even though the call returned `Err`. Validation now happens first (line 404) -> fixed
- [severity medium] [bug] `render_styled_box` computes `int_clip` but only uses it for the early return. The shadow, background, gradient and border are never clipped to `clip`, so partially visible widgets in scroll or clipped parents can paint outside their clip. -> not fixed (needs a `tiny_skia::Mask`, and the API differs between tiny-skia versions; this needs a deliberate change that can be compiled and checked)
- [severity low] [bug] Text caret is drawn at the pre-preedit column while a preedit is shown, so it overlaps the composition text. -> not fixed (cosmetic; needs a design decision)
- [severity low] [bug] The gradient branch ignores `bg_override`, so transitions do not animate background colour on gradient widgets. -> not fixed (consistent with the current design)

### src/layout.rs lines 694-807
- none

This chunk (lines 694-807) contains only unit tests, with no production code. I checked each test's expected values by hand against the layout semantics. The helpers `slave` and `cell` are outside the chunk, so I inferred their signatures from the assertions. `cell` appears to take `(w, row, col)`, and the assertions are consistent with that.

- **Pack tests:** the stacking, left-fill, expand-sharing, padding and orthogonal-request expectations are all arithmetically correct.
- **Grid tests:**
  - Column maxima give 50 wide, and the third item lands at x=30.
  - Weights 1:3 over 80 extra pixels give widths 30 and 70.
  - Spans plus padding give a 50×16 request and a placed rect of (5, 3, 40, 10).
  - The spans test assumes the helper's default cell height is 10, which I could not confirm.
- **Docs:** every test fn has a `///` doc comment.

### src/layout.rs lines 351-693
- none

I read lines 351-693 and found nothing to fix. The checks below cover each function in the chunk.

- **`pack_request` and `expansion`:** both follow Tk's packer rules for pack requests and expand slack, so I left them alone.
- **`pack`:** the cavity arithmetic is clamped with `.max(0.0)` and `.min(available)`, so it cannot go negative. Parcels are then bounded by `within(container)`.
- **Grid track bounds:** `span` keeps `start + count <= MAX_TRACKS`, so the `start..start + count` slices cannot overflow. `tracks` and `weighted` never grow past `MAX_TRACKS`. `track_range` only gets a start beyond the track list's length if `grid` were called with inconsistent inputs, and `grid` builds the list from the same items, so that cannot happen.
- **Weights:** `weighted` ignores out-of-range indices consistently in both the `last` calculation and the `total`.
- **`sticky_axis`:** it handles the low-only, high-only, both and neither cases correctly.
- **Docs and API:** every function in the chunk has a `///` doc comment. `PackItem` and `GridItem` are plain data structs that users construct with struct literals, so I did not add `#[non_exhaustive]`.

Two things I could not check from this chunk:
- **`MAX_TRACKS`:** its value is defined outside this chunk. If it is very large, a single item at a huge column index allocates a track vector of that size.
- **`pack` cost:** `pack` is O(n²) when many items use `expand`. That is acceptable at normal widget counts.

### src/layout.rs lines 1-350
- [low] [docs] `Rect::normalized` doc comment described `inset` (copy-paste error) (line 70) -> fixed
- [medium] [api] Public enums `Side`, `Fill` and `Anchor` lacked `#[non_exhaustive]` (lines 141, 155, 169) -> fixed
- [low] [bug] `place_rect` used `requested` unnormalized. Its fields are public, so a NaN or negative size made the anchor math produce NaN and silently drop the anchor offset (line 249, 261) -> fixed
- [low] [bug] `place_rect` did not clamp `relx`, `rely`, `relwidth` or `relheight`. Huge values overflowed to inf, and a nonfinite result collapsed to 0 instead of clamping (lines 235-259) -> fixed
- [low] [docs] The `Rect::within` doc comment was unclear about what it does (line 103) -> fixed

### src/style.rs lines 1640-1766
- [high] [bug] `parse_color` slices `hex` by byte offsets after matching on `hex.len()` (a byte length). Non-ASCII input such as `#é1` has length 3, so `&hex[0..1]` splits a UTF-8 character and panics. -> fixed (reject anything that is not all ASCII hex digits before slicing)
- [low] [bug] `u8::from_str_radix` accepts a leading `+`, so `#+f+f+f` or `#+f` forms parsed as valid colours. -> fixed (same hex-digit check)
- [low] [bug] `css_number` uses `trim_end_matches("px")`, which strips repeated suffixes (`2pxpx` parses as 2) and does not allow whitespace before the unit (`2 px` fails). -> fixed (strip the suffix once, then trim)
- [low] [docs] `build_rounded_rect_path` and `css_number` have `///` docs. The path-builder curve geometry checked out, so nothing needed fixing there. -> not fixed (nothing to fix)

### src/style.rs lines 1300-1639
- [high] [bug] `parse_linear_gradient` slices `val[start+1..end]` without checking `end > start`. Input like `linear-gradient)(` panics on a user-supplied stylesheet (line 1606) -> fixed
- [medium] [bug] `/*/` was treated as a complete comment, because `find("*/")` started at the `/*` itself and matched its own `*`. This swallows following rules (line 1328) -> fixed
- [medium] [bug] `border: none` returned an error even though `none` is listed as an accepted keyword. It now sets the border width to 0 (line 1501) -> fixed
- [medium] [bug] `parse_edges` silently dropped unparsable tokens, so `padding: 10px abc` became `10px` on all sides. It also accepted NaN and inf. It now rejects any bad token (line 1576) -> fixed
- [medium] [bug] `border-width`, `border-radius`, `opacity`, `font-size` and the shadow numbers accepted NaN, inf and negative values, because `f32::from_str` parses `nan` and `inf`. Non-finite or zero font sizes can break layout and text shaping. These are now validated (lines 1513, 1517, 1535, 1543, 1591) -> fixed
- [medium] [bug] `StylesheetParser::parse` applied rules to the engine as it went. A syntax error late in the sheet left it half-applied. Rules are now collected and applied only after the whole sheet parses (line 1368) -> fixed
- [low] [bug] A bad gradient stop position such as `red abc` silently became 0.0 -> not fixed (lenient behaviour, not a crash)
- [low] [bug] `rgb(a, b, c)` colours inside gradients or shadows are split on commas or whitespace and so are unsupported -> not fixed (feature limitation)
- [low] [api] The argument order of `Edges::symmetrical(nums[0], nums[1])` against CSS's vertical-then-horizontal order can't be verified from this chunk -> not fixed (definition not visible)
- NOTE: edits for src/style.rs:1300-1639 were REVERTED (checks failed or SEARCH mismatch): '[clippy failed]\nerror: stripping a prefix manually\n    --> src/style.rs:1328:36\n     |\n1328 |                 if let Some(end) = rest[2..].find("*/").map(|e| e + 2) {\n     |                                    ^^^^^^^^^\n     |\nnote: the prefix was tested here\n    --> src/style.rs:1327:13\n     |\n1327 '

### src/style.rs lines 956-1299
- [medium] [bug] `TransitionState::advance` computes `elapsed_ms + ms` in u32, which panics in debug builds and wraps in release (restarting the transition) on a huge `ms` (line 1272) -> fixed (saturating_add)
- [medium] [bug] `blend` interpolates non-premultiplied RGB. A transition from or to a transparent colour such as `rgba(0,0,0,0)` passes through a dark fringe, because the transparent side's black RGB is weighted in (lines 1220-1229) -> fixed (alpha-weighted RGB when the alphas differ; unchanged for equal alphas)
- [low] [docs] The `TransitionState` doc comment is duplicated (line 1231-1232) -> fixed
- [low] [docs] `progress()` is documented as "eased" but the progress is linear (line 1281) -> fixed
- [low] [bug] The disabled state maps use `entry().or_insert_with`. If the enclosing function does not clear `state_maps` before re-applying a theme, the old theme's disabled colour would persist after a theme switch (lines 1178-1185). The clearing happens outside this chunk, so I could not verify it -> not fixed (outside chunk)
- [low] [bug] `lookup_state` allocates a `String` on every call because of the tuple key (line 1196) -> not fixed (changing the key type is a refactor beyond a minimal edit)

- NOTE: the call for src/style.rs lines 606-955 hung upstream (no stream data for over 20 minutes; nothing billed in the ledger). It was killed, not retried at the same settings, and the range was split in half instead.

### (re-run with golden re-bless allowed) src/style.rs lines 781-955
- none

### (re-run with golden re-bless allowed) src/style.rs lines 606-780
- none

I found no real bugs in lines 606-780. The code is a series of `StyleSpec` inserts with no indexing, arithmetic that could overflow, or string slicing. Every fn in the chunk keeps its `///` doc comment, and the chunk adds no public enums or naming that would break the `<widget>_<verb>` pattern.

Four things look inconsistent, but I left them alone because I can't tell from this chunk whether they are intentional:
- **Hardcoded radii:** `Card` (12.0) and `Sidebar.TButton` (8.0) don't use the radius tokens `r.*`, while `TButton` and `Ghost.TButton` use `r.md`. A theme with zero radii would still get rounded corners on those two styles.
- **Hardcoded heading size:** `Heading` uses 22.0 instead of a `TypeScale` token.
- **Missing pressed state:** `Sidebar.TButton` has hover and selected states but no pressed state, unlike `TButton`, `Accent.TButton` and `Ghost.TButton`.
- **Selected-background alpha:** `selected_bg` uses a literal alpha of 51 (about 20%). This is fine as long as `Color` is non-premultiplied.

### src/style.rs lines 339-605
- [low] [docs] `StyleSpec` has its rustdoc line duplicated, so the rendered docs repeat the sentence (lines 405-406) -> fixed
- [low] [docs] `ResolvedStyle` has its rustdoc line duplicated (lines 467-468) -> fixed
- [low] [docs] `StyleError` has its rustdoc line duplicated (lines 530-531) -> fixed
- [medium] [bug] `merge_into` copies `font_size`, `border_width` and `radius` unchecked. A NaN, zero or negative font size reaches cosmic-text metrics, and a negative radius or border width reaches tiny-skia path building, which can panic or produce degenerate output. `opacity.clamp` also passes NaN through. Invalid values are now ignored (lines 452-453, 458, 460) -> fixed
- [low] [bug] `StyleEngine::map` has an unreachable `else { insert }` branch after `entry().or_default()`. It is harmless and works correctly, so I left it alone per the minimal-edit rule (lines 598-603) -> not fixed (working code)
- [low] [api] The public enums in this chunk are fine. There are none here, and `State` is defined outside this chunk -> not fixed (out of chunk)

### src/style.rs lines 1-338
- none

### src/render/data.rs lines 533-670
- [severity medium] [bug] The sort-indicator triangle is drawn with no clip. For a column that is only partly visible (cell_clip.width > 0), it can paint outside the cell or widget bounds (line 589-595) -> fixed (only drawn when it lies fully inside cell_clip)
- [severity low] [bug] The column separators were sized to the full table height but drawn before the body rows. Each row's full-width background fill then painted over them, so they showed only in the header (lines 598-599, 623) -> fixed (second separator pass over the body area after the rows are drawn)
- NOTE: edits for src/render/data.rs:533-670 were REVERTED (checks failed or SEARCH mismatch): '[build failed]\nerror[E0425]: cannot find value `body_top` in this scope\n   --> src/render/data.rs:519:46\n    |\n519 |         let sep = Rect::new(sep_x + w - 1.0, body_top, 1.0, (logical.height - hh).max(0.0));\n    |                                              ^^^^^^^^ not found in this scope\n\nerror'

### src/render/data.rs lines 287-532
- [high] [bug] `draw_menu_overlay` builds `full_clip` by dividing the pixmap size by `scale`, so it is in logical units. `fill` and `draw_styled` take physical-pixel clips, so on HiDPI (scale > 1) menu panels, rows and text were cut off to the top-left 1/scale of the window (line 297) -> fixed
- [low] [bug] `draw_tree` finds each row's display index with `position(|r| r.0 == *key)`, which is O(n²) per frame and would put duplicate keys on the wrong row. It now uses the enumerate index (lines 478-480) -> fixed
- [low] [docs] `tree_triangle` doc says the apex is at `(cx, cy)`, but the triangle is centred on that point (line 424) -> fixed
- NOTE: 1 golden(s) changed by these fixes and were re-blessed (diffs in /workspace/rtk-1.0-shots/sonnet-diffs/src_render_data.rs-287): m011_toast_dark_2x.png, m011_toast_highcontrast_2x.png, m011_toast_light_2x.png
- NOTE: edits for src/render/data.rs:287-532 were REVERTED (checks failed or SEARCH mismatch): '[clippy failed]\nerror: field `top` is never read\n   --> src/render/data.rs:123:5\n    |\n119 | struct TreeSnapshot {\n    |        ------------ field in this struct\n...\n123 |     top: usize,\n    |     ^^^\n    |\n    = note: `-D dead-code` implied by `-D warnings`\n    = help: to override `-D warnings` ad'

### src/render/data.rs lines 1-286
- [high] [bug] `draw_toast_overlay` builds `full_clip` in logical px (`pixmap.width()/scale`) but intersects it with `rect.scaled(scale)`, which is in device px. At scale > 1 toasts near the right or bottom edge (the default bottom-right placement) are clipped or skipped entirely, and their text is clipped too (line 168) -> fixed
- [low] [bug] `top + visible + 1` can overflow `usize` in `TableSnapshot::capture` (line 95) -> fixed
- [low] [bug] The same `top + visible + 1` overflow in `TreeSnapshot::capture` (line 145) -> fixed
- [low] [docs] The doc comment on `fill` says `rect` is logical but also says both arguments are device px. It is inconsistent but harmless, since the function only intersects and fills in a single space -> not fixed (docs only, no behavioural effect)

### src/render/inputs.rs lines 338-567
- [severity high] [bug] Slider tick loop is unbounded: a tiny `every`, or an infinite span, makes `count` huge (`inf as i64` is `i64::MAX`), so the render thread hangs (lines 445-448) -> fixed (ticks are skipped when more than 1000 would be drawn)
- [severity medium] [bug] `draw_slider` track, fill and thumb are drawn with `fill_path`/`stroke_path` and no mask, so they ignore `clip` and bleed outside a scrolled or clipped container (lines 417-491) -> fixed (a rect clip mask is built from `clip` and passed to the three paint calls)
- [severity low] [bug] `draw_dspin` arrow glyphs use `resolved.fg` even when disabled, while the text uses the muted colour (lines 547, 558) -> not fixed (changes visuals; the comment says this is intentional to match the integer Spinbox)
- NOTE: edits for src/render/inputs.rs:338-567 were REVERTED (checks failed or SEARCH mismatch): '[build failed]\nerror[E0425]: cannot find value `clip_mask` in this scope\n   --> src/render/inputs.rs:167:94\n    |\n167 |         pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), clip_mask.as_ref());\n    |                                                             '

### src/render/inputs.rs lines 1-337
- [severity medium] [bug] `draw_ring` and `draw_switch` ignore `clip`: the strokes and fills pass `None` as the mask, so a ring or switch partly scrolled out of a `ScrollArea` or clipped frame paints outside its clip (lines 49, 87-125, 163-186) -> fixed (a pixel mask built from `clip`, applied to every stroke and fill)
- [severity low] [bug] The smooth-mode `progress_drawn` fraction is not clamped, so an overshoot or NaN can make the fill wider than the bar or empty (line 242) -> fixed
- [severity low] [bug] The indeterminate fill is drawn in accent, but the overlaid percent/label text picks its contrast against `resolved.fg` (lines 232, 295) -> fixed (the contrast colour now matches the colour actually drawn)

- RECOVERED: the reverted src/style.rs 1300-1639 edits (comment parsing, gradient slicing panic, etc.) were re-applied from the saved Sonnet reply; DeepSeek fixed the clippy `manual_strip` error (recover-style-1300). Checks green.

- RECOVERED: src/render/inputs.rs 338-567 edits re-applied; they had only failed because they used `clip_mask` from the not-yet-reviewed chunk above. Green with no extra calls.

- RECOVERED: src/render/data.rs 287-670 edits re-applied from the saved Sonnet replies, plus DeepSeek recover-data round 0 (build fix). Later DeepSeek and Sol rounds chased a golden "failure" that was a genuine fix and were discarded. **Bug found:** menus and toasts at 2x were clipped or missing (the old 2x goldens were wrong). The m011_menu_*_2x and m011_toast_*_2x goldens were re-blessed and checked visually. Hand glue: removed the now-dead `TreeSnapshot.top` field (clippy dead_code).

### src/app.rs lines 651-925
- [severity medium] [bug] `sync_window_props` pins a non-resizable axis to `inner_size()`, which is 0×0 while the window is minimized (Windows, some X11 WMs). That sets min = max = 0 and leaves the window unusable or mis-sized on restore. (line 786) -> fixed (pinning is skipped when the size is zero)
- [severity low] [bug] The logical size is derived from `self.tk.scale_factor`, not the window's own scale factor. A zero or non-finite value gives `inf` or `NaN`, which saturate to `u32::MAX` or 0 in `as u32`. Using `window.scale_factor()` matches the units winit applies to `LogicalSize`, with a finite-and-positive guard. (line 780) -> fixed
- [severity low] [bug] `sync_ime` caches `ime_allowed`, so it must be reset when the window is destroyed or re-created, otherwise IME stays disabled on the new window. The reset would live in `open` or the destroy path, which are outside this chunk. (line 751) -> not fixed (outside chunk)
- [severity low] [bug] F16/F17 (per-frame title/env calls, redraws while occluded): nothing in this chunk does either. `apply_window_props` is dirty-flag gated. `pump_a11y` calls `request_redraw` only after handled actions, so the occlusion check belongs in `schedule` or `RedrawRequested`. -> not fixed (not present in this chunk)
- [severity low] [api] No public API is touched in this chunk. Every fn has a `///` comment. -> none

### src/app.rs lines 308-650
- [medium] [bug] `draw()` called `set_title` on every frame, and `RTK_STATS` was read from the environment on every present (F16) (lines 311, 394) -> fixed. The title is now cached in a thread-local and pushed only when it changes. `RTK_STATS` is read once through a `OnceLock`. The title cache is reset before the window is (re)opened.
- [medium] [bug] Damage rects were converted with raw `as u32` casts and `rect.x + rect.width.get()` (lines 353-372). Negative or oversized rects could overflow u32 (a debug panic) or be passed outside the surface to `present_with_damage`. Fractional rects were also truncated, which under-covered them. -> fixed. Rects are now floor/ceil-ed and clamped to the surface size, and empty results are dropped.
- [medium] [bug] `suspended()` did not reset `occluded`, `modifiers` or `dirty` (line 444). If the window was suspended while occluded, the recreated window would never draw. -> fixed.
- [medium] [bug] `Focused(false)` cleared input but left `self.modifiers` as it was (line 637). A modifier released while another window had focus stayed "held" and made `accepts_key_input` reject typing. -> fixed. Modifiers are reset on focus loss.
- [low] [bug] `ThemeChanged` requested a redraw while occluded (line 484, F17) -> fixed. The `Occluded(false)` event already sets `dirty`, so the redraw happens on un-occlusion.
- [low] [bug] `DroppedFile` calls `sim_file_drop` once per file (line 630), so a multi-file drop fires the drop callback several times instead of once with the whole batch -> not fixed. Batching needs state outside this chunk.
- [low] [bug] The `RedrawRequested` error path sets `dirty = false`, so the "will retry" message only holds once another event arrives (line 643) -> not fixed. It deliberately avoids a busy loop on a persistent surface error.
- [low] [bug] The per-frame `apply_window_props` and `sync_ime` calls, and any redraw scheduling while occluded, live outside this chunk (lines 312-313) -> not fixed. `draw()` already returns early when occluded.
- [low] [bug] The AltGr handling (ctrl+alt treated as a shortcut) depends on `accepts_key_input`, which is outside this chunk -> not fixed.

### src/app.rs lines 1-307
- [medium] [bug] `open` reads the OS theme from `self.window`, which is still `None` at that point, so `follow_system_theme` never receives the initial light/dark preference (line 221) -> fixed (now reads the new window).
- [medium] [bug] F17: while animating, `schedule` requested redraws and woke at ~60 Hz even when the window was occluded (lines 262-275) -> fixed (the animating branch is skipped while occluded, so it falls through to the idle/Wait branches; `last_frame` is reset there so no time jump on un-occlude).
- [medium] [bug] `start + Duration::from_millis(due)` can panic on `Instant` overflow if a timer is scheduled with a huge delay (line 299) -> fixed (`checked_add`).
- [low] [bug] `mainloop` leaves the dead event loop's proxy in `proxy_shared.waker` after `run_app` returns, so later `TkProxy::post` calls target a closed loop (lines 134, 161) -> fixed (waker cleared after the loop exits, on both success and error).
- [low] [docs] `App` and its non-feature-gated fields are undocumented (lines 105-116) -> not fixed (private items, so `missing_docs` does not apply; no behaviour impact).
- [low] [bug] F16 (per-frame window title/env calls): none present in lines 1-307. `sync_window_props` is called once from `open`, and its body is outside this chunk -> not fixed (not in this chunk).

