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

### src/widgets.rs lines 675-888
- none

I found nothing in lines 675-888 that needs fixing. The chunk holds the tail of the `WidgetNode` field list, `EditSnapshot`, `WidgetNode::new`, `WidgetSlot` and `CanvasItemKind::translate`.

- **`WidgetNode::new`:** `cursor: text.len()` is evaluated before `text` is moved, so there is no borrow or move problem and the cursor starts on a UTF-8 boundary. The code also uses `kind` after moving it into the struct. That compiles only if `Kind` is `Copy`, which I assumed because the definition is outside this chunk.
- **`CanvasItemKind::translate`:** every variant is handled, so a missing variant would fail to compile rather than slip through. The coordinate math is plain addition on `f32`, with no overflow or panic risk.
- **API surface:** `WidgetNode` and `WidgetSlot` are crate-private, and `EditSnapshot` sits in the private `widgets` module and is not re-exported. The `missing_docs` lint and the `#[non_exhaustive]` requirement therefore do not apply to them, and every fn already has a `///` comment.
- **State reset on destroy:** the fields that would need clearing, such as `scroll_area_owner`, the undo and redo stacks, and the `*_running` flags, are reset by destroy logic outside this chunk, so I could not check them here.

### src/widgets.rs lines 340-674
- [severity medium] [bug] `Spinbox::new` stores `range = (from, to)` without checking order. If a caller passes `from > to`, any later `i64::clamp(from, to)` on the spin box panics ("min > max") (line 401) -> fixed. The bounds are now normalised by swapping them, and the doc comment says so.
- [severity low] [bug] `Canvas`, `Image`, `Chart`, `Sparkline` and `ProgressBar` only clamp the lower bound of their sizes. An infinite size is accepted and could reach layout or pixmap allocation (lines 380, 422, 505, 532, 547) -> not fixed. NaN and negative values are already handled by `max(0.0)`. Allocation limits belong at the render/layout boundary outside this chunk, and changing it here would be speculative.
- [severity low] [api] `Kind` is `pub(crate)`, so `#[non_exhaustive]` and rustdoc are not required on it (line 554) -> not fixed (not public API).
- No other bugs, naming inconsistencies or missing docs found in this chunk. All `fn`s carry `///` docs.

### src/widgets.rs lines 1-339
- [medium] [api] Public enum `Theme` lacks `#[non_exhaustive]`, so adding a theme later would be a breaking change (line 77) -> fixed
- [medium] [api] Public enum `Compound` lacks `#[non_exhaustive]` (line 97) -> fixed
- [medium] [api] Public enum `DragPhase` lacks `#[non_exhaustive]` (line 111) -> fixed
- [medium] [api] Public enum `Opt` lacks `#[non_exhaustive]`, and it is the most likely enum to gain variants (line 164) -> fixed
- [low] [api] `ListSelectCallback` and `NotebookChangedCallback` are `pub` aliases in a private module and are not re-exported in lib.rs, so users cannot name them (lines 159, 161) -> not fixed (the re-export belongs in lib.rs, outside this chunk)
- [low] [api] The `Scrollbar::new` doc references `Tk::link_scrollbar`, which looks like `<verb>_<widget>` rather than the `<widget>_<verb>` pattern (line 335) -> not fixed (the method is defined outside this chunk, so I can't confirm it or rename it safely)
- [low] [bug] `Scale::new` stores `from`/`to` unchecked and does not clamp the variable's current value. Consumers outside this chunk may assume `from <= to` (line 315) -> not fixed (handling lives outside this chunk, and tkinter allows reversed ranges)
- NOTE: edits for src/widgets.rs:1-339 were REVERTED (checks failed or SEARCH mismatch): '[build failed]\nerror[E0004]: non-exhaustive patterns: `_` not covered\n  --> examples/gallery.rs:50:11\n   |\n50 |     match theme {\n   |           ^^^^^ pattern `_` not covered\n   |\nnote: `Theme` defined here\n  --> src/widgets.rs:78:1\n   |\n78 | pub enum Theme {\n   | ^^^^^^^^^^^^^^\n   = note: the match'

### src/tk/events.rs lines 523-817
- [severity low] [bug] Combobox Up/Down on a combobox with no values writes index 0 to its variable, which then points at a nonexistent item (lines 723-733) -> fixed
- [severity low] [bug] `clear_input` leaves `open_combobox` set and the node's `combo_open` true, so a dropdown can stay open after input state is cleared (lines 805-814) -> fixed
- [severity low] [bug] `modal_accept`/`modal_cancel` restore logic can re-install a stale handler if the callback closes the modal and opens a new one without handlers (lines 646-667) -> not fixed (needs a modal generation counter, which is a larger change than this chunk allows)
- [severity low] [bug] `Spinbox` Up/Down sets the variable without running the widget's command, unlike `Scale` (lines 771-784) -> not fixed (cannot tell from this chunk whether Spinbox has a command; leaving as is)

### src/tk/events.rs lines 188-522
- [severity high] [bug] Horizontal scrollbar thumb width used `f32::clamp(16.0, bar_w)`, which panics when the scrollbar is narrower than 16px (min > max). A zero `total` also gave a NaN ratio. Now `.max(16.0).min(bar_w)`, which never panics and handles NaN (line 363) -> fixed
- [severity high] [bug] Vertical scrollbar thumb height had the same `clamp(16.0, bar_h)` panic on short scrollbars (line 382) -> fixed
- [severity low] [bug] Combobox dropdown height `clamp(item_h, 240.0)` panics if `item_h` exceeds 240 (font size above about 178) (line 212) -> fixed. This assumes the dropdown drawing code computes its height the same way; I could not check it.
- [severity low] [bug] `run_command` leaves `command_running` set if the command panics (line 507) -> not fixed (the panic would propagate and the toolkit is not expected to survive it; a guard is out of scope for a minimal edit)
- [severity low] [bug] Clicking in a multi-line `Text` always places the cursor at the start of the clicked line rather than at the clicked column (line 407) -> not fixed (behaviour limitation, not a crash or corruption, and fixing it needs new text-measurement code)

### src/tk/events.rs lines 1-187
- [severity high] [bug] The horizontal scrollbar drag calls `f32::clamp(16.0, bar_w)`, which panics when the bar is narrower than 16 px (`min > max`). That can happen with a tiny or collapsed scrollbar during a drag (line 103) -> fixed
- [severity high] [bug] The vertical scrollbar drag has the same `clamp(16.0, bar_h)` panic when `bar_h < 16` (line 116) -> fixed
- [severity low] [bug] `drag_text_selection` for `Kind::Text` always puts the cursor at the start of the dragged line and ignores the x column. This gives coarse selection but no panic or UTF-8 error, and a proper fix needs a per-line column hit-test, which is a larger change (lines 170-176) -> not fixed (behavioural limitation, not a crash; out of scope for minimal edits)

### src/tk/menu.rs lines 693-698
- none

I reviewed only lines 693-698, which hold `Menubar::new`. It has a `///` doc comment and delegates to `tk.add(Kind::Menubar, parent, String::new())`, so parent validation and arena-full errors are handled in `Tk::add` and returned through `RtkError`. This chunk has no geometry, HiDPI scaling, UTF-8 handling, arithmetic, or state-reset logic. The constructor name `new` matches the other widgets' public constructors. I did not review the HiDPI clipping or the dialog-close fix, since they are outside this chunk.

### src/tk/menu.rs lines 345-692
- [severity medium] [bug] `menu_pointer_move` truncates deeper menu levels before checking whether the hovered item is the submenu that is already open. Every pointer move over a submenu item closed and reopened the submenu, which reset its highlight and caused redraw churn, and the `already` check could never be true (lines 422-426) -> fixed
- [severity medium] [bug] `menu_activate` activated disabled Check and Submenu items on click, because only Command checked `enabled` (lines 522-529) -> fixed
- [severity low] [bug] The Right key opened a disabled submenu when the highlight had been set by the mouse (lines 589-594) -> fixed
- [severity low] [bug] After a command runs, `menu_activate` re-inserted it and overwrote any command the callback had installed at the same index (lines 536-538) -> fixed
- [severity low] [bug] `menu_pointer_move` highlights separator rows. I did not change it because the separator and disabled-item drawing is outside this chunk -> not fixed
- [severity low] [bug] The menu item geometry (`4.0 + idx * ih`) assumes every row has the same height. If `panel_height` gives separators a different height, hit-testing and submenu anchoring drift. That code is outside this chunk -> not fixed
- [severity low] [bug] I could not check the 2x clipping from this chunk. The geometry here is all in logical px and consistent with `item_height(font_size)`. `panel_width`, `panel_height` and `clamp_menu_anchor` are outside the chunk and are the place to look -> not fixed

### src/tk/menu.rs lines 1-344
- [severity low] [docs] `MenuId` is described as "Generational", but it is a plain monotonically increasing `u32` with no generation counter (line 15) -> fixed
- [severity low] [api] `bind_context_menu` breaks the `<widget>_<verb>` naming pattern; `menu_bind_context` would fit better (line 276) -> not fixed (a rename would break callers outside this chunk, and the rename belongs in a coordinated API change)
- [severity low] [bug] `next_id` uses `saturating_add`, so at `u32::MAX` a new menu would overwrite the last one (line 190) -> not fixed (unreachable in practice)
- [severity low] [api] `menu_add` returns the `usize::MAX` sentinel for an unknown menu instead of an `Option` (line 196) -> not fixed (changing the return type would break the public API)
- [severity low] [bug] HiDPI menu geometry (`clamp_menu_anchor`, `panel_width`, `panel_height`, `menubar_open`, `menubar_hit`) -> not fixed. The anchors, widths and heights are all in logical px and the viewport is divided by `scale_factor` consistently, so I found no clipping bug in this chunk. If `viewport` turns out to be logical rather than physical, the division on lines 247-248 is the one to remove. The dialog-closing behaviour from PR #2 is not in this chunk.

### src/tk/treeview.rs lines 350-595
- [severity medium] [bug] `tree_see` calls `tree_clamp_top` even when the widget has no laid-out rows, as on a not-yet-selected Notebook page. `tree_visible_rows` is then 0, so the clamp can push `top` to the end of the list and the body renders empty. This is the chunk-visible root cause of the band-aid; the clamp is now skipped until real geometry exists (line 413-433) -> fixed. `tree_clamp_top` and `ensure_visible_robust` are outside this chunk, so I could not check whether they also clear `dirty` too early or build `visible` before nodes exist.
- [severity low] [bug] `tree_mouse_down` computes `top + (y / row_h).floor() as usize`. A huge or infinite `y` saturates the cast to `usize::MAX`, and the addition overflows (panics in debug builds) (line 466) -> fixed
- [severity low] [bug] `tree_scroll` computes `state.top as i64 + delta`. An infinite `lines` value gives `i64::MAX` and the addition overflows (line 489) -> fixed
- [severity low] [bug] Collapsing a node by clicking its triangle or pressing a key does not re-clamp `top`, so the view may show blank rows (lines 476, 554-587) -> not fixed (the clamp helper is outside this chunk and render likely clamps; changing it here would be speculative)

### src/tk/treeview.rs lines 1-349
- [medium] [bug] `tree_clamp_top` read `state.visible.len()` without rebuilding a dirty cache. After inserts or expands, `total` was stale (often 0), so `top` was clamped against the wrong row count. This is the one stale-`visible` hazard in this chunk, and it can leave the view scrolled to the top or empty. (line 204) -> fixed (calls `ensure_visible` before reading the length).
- [low] [bug] The `ensure_visible_robust` and `tree_visible_rows` requested-height fallbacks are band-aids, but the root cause is not in this chunk. Every mutator here (`tree_insert`, `tree_remove`, `tree_clear`, `tree_set_open`, `tree_expand_all`, `tree_collapse_all`) sets `state.dirty = true` after changing nodes. `rebuild_visible` clears `dirty` only when called, and `tree_insert` re-sets it afterwards, so no ordering here clears it before nodes exist. The remaining suspects are the layout and render code that calls `rebuild_visible` or builds `visible`, outside these lines. (lines 112-126, 186-198) -> not fixed (removing them blind could break callers elsewhere; the band-aids stay until the real cause is found in the other chunks).
- [low] [bug] `push_visible` recurses once per tree level, so a pathologically deep tree could overflow the stack. (line 129) -> not fixed (unrealistic depth, and fixing it would be a refactor).

### src/tk/mod.rs lines 306-577
- [low] [docs] Doc comments are duplicated verbatim on `new`, `proxy`, `after_idle`, `process_posted`, `root`, `contains`, `theme`, `default_font`, `tokens`, `set_theme` and `set_accent`. Rustdoc would render each sentence twice, and `process_posted` would show its whole paragraph twice (lines 306-307, 384-385, 390-391, 396-405, 451-452, 471-472, 477-478, 483-486, 491-492, 508-511, 564-565) -> fixed
- [low] [bug] `set_theme` returns early when the theme is unchanged. Calling `set_theme(current)` therefore cannot reset custom tokens installed through `set_tokens`, although the doc says it resets tokens (line 513). This looks deliberate, as it avoids a needless full restyle, so I left it -> not fixed (intentional early-out, behaviour change not warranted pre-1.0)
- [low] [bug] `process_posted` takes whole queues into locals. If a task panics, the rest of that batch is dropped. Unwinding through the GUI loop is already fatal, so this is not worth extra machinery -> not fixed
- [low] [bug] `is_descendant_of` returns true for `id == ancestor` even when the id is stale. It is crate-private and callers pass live ids, so I left it -> not fixed
- [low] [api] The `Theme` enum is matched exhaustively in `set_theme`. Whether it carries `#[non_exhaustive]` cannot be checked from this chunk; it is defined in `widgets` -> not fixed (outside this chunk)

### src/tk/mod.rs lines 1-305
- [low] [docs] `TkProxy::post` has its doc line duplicated (lines 129-130) -> fixed
- [medium] [api] Public enum `ScrollUnit` lacks `#[non_exhaustive]`, so adding a variant after 1.0 would be a breaking change (line 186) -> fixed
- [low] [api] `Tk::modal` is a `pub` field while every other state field is `pub(crate)`. External code can corrupt the modal invariants by writing to it, but changing the visibility could break external examples or tests I can't see (line 236) -> not fixed (cannot verify out-of-chunk uses; consider an accessor before 1.0)
- [low] [bug] `TkProxy::post` queues tasks without bound if the event loop has exited or never started. The closure type is `FnOnce`, so there is no safe way to drop them early without a loop-lifecycle design change (line 131) -> not fixed (design change out of scope)

### src/theme.rs lines 283-507
- [low] [bug] `chart[0]` is forced to white whenever `high_contrast` is set, even if the tokens have a light background, which would make the first series invisible on white (line 351). Only `is_dark` should decide this: a dark high-contrast theme already has `is_dark` true, and a light one keeps the dark first colour. -> fixed
- [low] [bug] `text_disabled` calls `blend` three times, once per channel (lines 290-295). The result is correct, so this is style only. -> not fixed (working code, no behavioural change)
- [low] [docs] No other problems found in this chunk. Every fn has a `///` doc comment, and the `ColorRole` match is exhaustive over the roles listed. -> not fixed (nothing to fix)

### src/theme.rs lines 1-282
- none

### src/chart.rs lines 269-494
- [severity low] [docs] The `build_items` doc claims an empty list for a too-small plot area and that axes are still drawn with no finite points. In the non-sparkline path, the legend is pushed before the `bounds()` and size checks, so the list can be non-empty when the plot is small. With no finite points only the legend is drawn, with no axes. (line 271) -> fixed
- [severity low] [bug] I could not check the following because `bounds()`, `nice_ticks` and `time_ticks` are outside this chunk: whether the x/y span is non-zero (divisions at lines 296-297 and 353-354), and whether ticks stay inside the data range. This chunk has no panic, overflow or UTF-8 problems. Bar index/slot handling, the `continue` on an empty bar series, and the NEG_INFINITY label-gap logic are correct. -> not fixed (nothing to change in this chunk; the guarantees belong to `bounds()` and the tick helpers)

### src/chart.rs lines 1-268
- [medium] [bug] `nice_ticks` computes `k as f64 * step` for sub-unit steps, so it yields values like `0.30000000000000004`, which `format_tick` then prints as ugly labels (line 167) -> fixed (sub-unit ticks are now computed as an integer divided by an exact power of ten)
- [medium] [bug] `k += 1` in the `nice_ticks` and `time_ticks` loops can overflow `i64` and panic in debug builds when `last` saturates at `i64::MAX` (lines 171, 212) -> fixed (now iterates over an inclusive range)
- [medium] [bug] `time_ticks` falls back to the 1-week step for huge ranges, which can produce millions of ticks (unbounded allocation) (lines 194-201) -> fixed (the fallback step is scaled up so the tick count stays within `max_ticks`)
- [low] [bug] `bounds` uses `y_range` as given, so a reversed (`lo > hi`) or non-finite range gives inverted or NaN bounds (lines 115-118) -> fixed (a finite range is normalised so lo ≤ hi; a non-finite range is ignored)

### src/tk/geometry.rs lines 325-483
- [severity low] [docs] `widget_rect` has its `///` doc line duplicated, so rustdoc renders the sentence twice (lines 449-450) -> fixed
- [severity low] [docs] `focused_widget` has its `///` doc line duplicated (lines 456-457) -> fixed
- [severity low] [api] `is_mapped` and `focused_widget` don't follow the strict `<widget>_<verb>` pattern (tkinter's `winfo_ismapped` and `focus_get`). Renaming would be a breaking change to existing callers beyond this chunk's scope -> not fixed
- [severity low] [bug] When pack children exist, grid children of the same parent are silently left unmapped (line 396-401). This is a deliberate `else if` that stands in for tkinter's "cannot mix managers" error, and the behaviour is consistent -> not fixed

I found no panics, overflow, UTF-8 or reentrancy bugs in the layout, focus-reset or notebook/scroll-area paths of this chunk.

### src/tk/geometry.rs lines 1-324
- none

I read lines 1-324 and found nothing I could confirm as a bug or API problem.

- **Layout order:** the bottom-up measurement order is correct. It is a pre-order walk, reversed, so every descendant is measured before its parent. Notebook pages and ScrollArea content are included in that walk.
- **Borrows:** the listbox and menubar code takes and restores or clones state without borrow or ordering problems.
- **Size conversion:** the `initial_size` float-to-`u32` cast is NaN-safe, because `NaN.max(200.0)` gives 200.
- **Docs and naming:** every fn has a `///` comment, and no public enum in this chunk needs `#[non_exhaustive]`. The method names (`pack`, `grid`, `place`, `grid_columnconfigure`, `grid_rowconfigure`) follow tkinter's own names.
- **Unverified (radiobutton text, line 222):** the Radiobutton branch reads `n.text` directly, while the generic branch uses `widget_text`. If `widget_text` resolves a bound text variable, a Radiobutton's size would ignore it. I can't see `widget_text` from this chunk, so I left it alone rather than risk an empty-string regression.
- **Unverified (zero window size):** `initial_size` returns `window_size` unclamped. A zero size is only a problem if the setter doesn't already reject it, and I can't see the setter.

### src/tk/table.rs lines 339-445
- [medium] [bug] `table_scroll`: `state.top as i64 + delta` can overflow when `lines` is huge or infinite, because `delta` saturates to `i64::MAX`. This panics in debug builds and wraps in release. (line 411) -> fixed (`saturating_add`)
- [low] [bug] `table_key`: with no selection, or a selected row hidden from the display, `cur_display` defaulted to 0. Down/PageDown then skipped the first row and selected row 1. (lines 425-437) -> fixed (with no current selection, Down/PageDown/Up/PageUp now select the first row)
- [info] Selection after sort: the selection is stored as a model index and resolved through `display_of`/`model_at` (lines 380, 427, 438). It therefore follows the row identity, not the display position. The sort itself (`table_sort_by`) is outside this chunk, so I could not check it. -> no change needed here
- [low] [bug] `table_mouse_down` clicks beyond the last row do nothing, which is fine. `row_h` of zero would give an infinite quotient, but `as usize` saturates and `model_at` returns `None`, so there is no panic. -> not fixed (harmless)

### src/tk/table.rs lines 1-338
- [low] [docs] `table_clear` doc claims it clears the sort, but the code keeps it (and the header indicator stays) (line 181) -> fixed (doc now matches the code)
- [medium] [bug] `table_clear` silently reset the selection without firing the on_select callback, so listeners kept a stale selection (line 182) -> fixed
- [medium] [bug] `table_set_rows` cleared an out-of-range selection without firing the on_select callback (line 154) -> fixed
- [none] Selection after sort: `selected` holds a model row index, and sorting only permutes `order`, so the selection follows row identity. `table_insert_row` also shifts the model index correctly. Verified, no change needed.

### src/tk/style.rs lines 314-432
- [severity medium] [bug] `check_state_transition` leaves an in-flight transition running when the new state has `transition_ms == 0`. The widget keeps animating toward the previous state's colors and shows a stale style, and `is_animating` stays true until the old transition finishes (lines 415-428) -> fixed (the transition is now cleared in that case)
- [severity low] [bug] `is_animating` and `sim_advance_time` test only the widget's own `visible` flag. If that flag is not already the effective visibility, an indeterminate bar inside a hidden ancestor would keep requesting frames. I can't tell from this chunk, and the fix would sit outside it (lines 341, 385) -> not fixed (outside this chunk)

### src/tk/style.rs lines 1-313
- [medium] [bug] `poll_stylesheet` only reloaded when `mtime > last_mtime`, so replacing the file with an older one (git checkout, `mv`, backup restore) was never picked up (line 117) -> fixed (now `!=`)
- [low] [bug] `poll_stylesheet` returned true even when the reload failed (parse error or a read caught mid-write), contradicting its "Returns true when reloaded" doc (line 119-120) -> fixed (returns the load result; mtime is still recorded so a broken file is not re-parsed every poll)
- [medium] [bug] `register_image_rgba` computed `w*h*4` unchecked. On 32-bit targets this can overflow and wrap to match a short buffer, which would then panic or mis-size the pixmap (line 148) -> fixed (checked multiplication)
- [low] [bug] `register_image_rgba` premultiplied with truncating division, which systematically darkens semi-transparent pixels (lines 155-157) -> fixed (rounded division)
- [low] [api] `load_stylesheet`, `watch_stylesheet`, `poll_stylesheet`, `load_image` and `register_image_rgba` are `verb_noun`, while `style_configure` and `style_map` follow the `<widget>_<verb>` pattern -> not fixed (renaming public API this late is out of scope for a minimal review; the doc comments are present)
- [low] [bug] `watch_stylesheet` ignores the initial load error and still returns true (line 104) -> not fixed (the return value is documented as stat-only; changing it alters behaviour)
- [low] [bug] `ImageId` allocation uses `saturating_add`, and no image unload exists in this chunk, so `images` grows without bound -> not fixed (an unload API would live outside this chunk)

### src/icons.rs lines 351-417
- [severity medium] [bug] F2: `icon()` registers the image with the requested `size_px` for both width and height, but `render_rgba` (not visible in this chunk) may render a different size for icons under 8px, e.g. by clamping or returning an empty buffer for 0. The buffer length then does not match `size_px²·4`, so registration fails or produces a garbled image. (lines 413-416) -> fixed. The registered dimensions are now derived from the actual buffer, and an empty or non-square buffer returns `None`. I could not see the clamp in the top part of `render_rgba`, so I did not touch it.
- [severity low] [docs] `icon()` doc comment did not say what happens for tiny or zero sizes (line 412) -> fixed
- [severity low] [bug] The unpremultiply loop (lines 394-409) and the `from_xywh(..).unwrap()` calls use constant positive sizes scaled by `k`, which is only positive if `size_px` is non-zero. Zero-size input is now rejected in `icon()`, which makes these safe. -> not fixed (no change needed beyond the `icon()` guard)

### src/icons.rs lines 1-350
- [medium] [bug] `render_rgba` clamps `size_px` up to 8, so a request for 1–7 px returns an 8×8 buffer. The caller expects `size_px*size_px*4` bytes, so the buffer length is wrong (F2). Size 0 and over-large sizes hit the same path, because `Pixmap::new` fails and the fallback is again 8×8 (lines 72-73). -> fixed: the requested size is honored, and 0 or an unallocatable size returns an empty `Vec`.
- [low] [api] The public enum `Icon` lacks `#[non_exhaustive]`, so adding icons later would be a breaking change (line 10). -> fixed
- [low] [docs] The `render_rgba` doc did not state the buffer length or the empty-buffer cases (line 70). -> fixed

### src/tk/toast.rs lines 1-337
- [severity low] [api] `ToastId` derives only `Clone, Copy, Debug, PartialEq, Eq`, so it cannot be used as a `HashMap` or `HashSet` key. This is inconsistent with other handle types (line 52) -> fixed (added `Hash`)
- [severity low] [docs] `toast_dismiss` says it returns false only if the toast is unknown, but it also returns false when the toast is already fading out (line 225) -> fixed (doc clarified)
- [severity low] [docs] `toasts()` and `toast_rect` say "visible", but they also include toasts that are fading out (lines 242, 267) -> not fixed (the wording is acceptable and fading toasts are still drawn)
- [severity low] [bug] `toast_layout` divides by `scale_factor` with no zero guard (lines 280-281) -> not fixed (winit never reports 0, and the guard belongs where `scale_factor` is set)
- [severity medium] [bug] The reported 2x clipping cannot be traced to this chunk -> not fixed (see below)
  - `toast_layout` is internally consistent in logical px. It uses `viewport / scale_factor` and the logical constants, and it clamps width and position.
  - The remaining suspects are `measure_with` returning physical-scaled widths, or `draw_toast_overlay` mixing logical rects with a physical clip. Both are outside this chunk and I could not verify either.
- [severity none] [bug] Lifecycle (PR #7) checked, no defects:
  - **Expiry:** a non-sticky toast starts its fade-out once `elapsed_ms >= duration_ms`, carrying the overshoot into `fade_out_ms`.
  - **Removal:** `retain(!is_finished)` runs after every advance, and large time steps cannot overflow because the adds saturate.
  - **Timers:** `is_animating` stays true for any toast that has a duration, is fading out, or is still in fade-in. A sticky toast stops requesting frames once it has faded in.
  - **Eviction:** `toast_set_max` and the over-max eviction in `toast_with` drop the oldest toasts correctly. `toast_clear` resets the list.
  - **Clicks:** `toast_click` is index-aligned with the layout and only fades a toast out once.

### src/tk/scroll.rs lines 1-333
- [medium] [bug] A horizontal-only wheel event (`dy == 0`, e.g. `sim_wheel_shift` or a sideways touchpad swipe) over a Slider, DoubleSpinbox or closed Combobox was treated as "scroll down". It decremented the value or selection. (lines 64, 72, 84) -> fixed (these arms now require `dy != 0.0`)
- [low] [bug] `canvas_set_view` with a NaN offset (e.g. `yview_moveto(id, NaN)`) stored NaN in `view`, because `clamp` passes NaN through, and then marked the widget dirty on every call. (line 182) -> fixed (NaN input is ignored)
- [low] [api] `link_scrollbar` breaks the `<widget>_<verb>` pattern used by `scrollbar_orient`, and its parameter is misleadingly named `listbox`. (line 319) -> fixed (added `scrollbar_link(scrollbar, target)` as the primary name; `link_scrollbar` stays as a thin wrapper so existing callers keep compiling)
- [low] [docs] The `yview` doc omits ScrollArea, which the method supports. (line 222) -> fixed
- [low] [bug] Pixel-mode wheel deltas on Listbox, Text, Table and TreeView are divided by the row height and rounded, so deltas under half a row are lost. This needs a per-widget accumulator and is a design change. (lines 44, 59) -> not fixed (out of scope for a minimal pre-1.0 edit)
- [low] [api] The `yview*`, `xview*` and `*_scroll` names deliberately mirror tkinter, so I left them unchanged. -> not fixed (intentional)

### src/timer.rs lines 1-318
- none

I found nothing in lines 1-318 that needs a fix:
- **Timer queue:** the `saturating_add` calls and the `p.max(1)` clamp on the period prevent overflow and a zero-period spin.
- **Reentrancy:** a repeating timer's callback is taken out of the list while it runs, and its slot is looked up again by id afterwards. Self-cancel, scheduling from inside a callback, and nested advance calls are all safe.
- **Runaway timers:** the 10,000-callback cap bounds `after(0)` self-rescheduling loops.
- **API and docs:** the public items follow tkinter naming, `TimerId` is a struct rather than an enum so `#[non_exhaustive]` does not apply, and every public fn has a `///` doc comment.

### src/tray.rs lines 1-317
- [medium] [bug] `dispatch` re-installs the handler after the callback returns if the callback dropped the `Tray` (which clears the slot). A stale callback then survives the `Tray`, and later events still reach it. It now uses a generation counter and only restores the handler if nothing was installed or cleared during dispatch (lines 149-161) -> fixed
- [low] [docs] The module example binds `let _tray = Tray::new(...)` inside the `after_idle` closure, so the tray is dropped immediately when the closure ends. It also has an unused `proxy` variable. The example now says to keep the `Tray` alive and drops the unused variable (lines 27-38) -> fixed
- [low] [bug] The event handlers are global in `tray_icon`, so a second `Tray` replaces the first one's handlers, and dropping either `Tray` clears them. Fixing this would need a redesign, so only one `Tray` at a time is supported (lines 241-272) -> not fixed (design limitation)

### src/tk/edit.rs lines 1-310
- [high] [bug] `extend_selection_to` and `move_cursor_to` clamp only to the text length, not to a char boundary. A cursor inside a multi-byte char later makes `replace_range` in `delete_selection` panic, or makes `selected_text_of` return `None` (lines 296, 304) -> fixed
- [high] [bug] `selection_of` does not clamp a stale anchor or cursor to the current text. If the text was replaced externally (Var or set), `delete_selection` panics in `replace_range` with an out-of-range or non-boundary index (lines 113-121) -> fixed
- [medium] [bug] `replace_selection_with` uses the raw cursor as the insertion point. A stale or non-boundary cursor makes `get(..start)` return `None`, which silently drops the whole text (lines 221-225) -> fixed
- [low] [bug] `replace_selection_with` coalesces typing even when it replaces a selection. The replacement should be its own undo step (line 226) -> fixed
- [low] [bug] `sync_after_edit` clones the full text for every widget kind and clamps the cursor without checking char boundaries. It now clones only for Entry and floors the cursor to a boundary (lines 277-288) -> fixed
- [low] [api] `selection_clear` breaks the `<widget>_<verb>` naming pattern, while the sibling methods are `edit_*`, `entry_*` and `text_*`. A rename would break callers outside this chunk -> not fixed (the call sites are not visible here)
- [low] [bug] After an external text set, `last_edit_typing` is not reset, so the next typing may coalesce into a stale undo step. The setter lives outside this chunk -> not fixed

### src/tk/a11y.rs lines 1-297
- [medium] [bug] `a11y_labels` overrides are never removed when a widget is destroyed, so the map can grow without bound and keep stale entries (lines 86, 199). The destroy path is outside this chunk, so I mitigated it here by pruning entries for dead widgets in `a11y_tree_changed` (lines 288-295), which runs once per a11y poll. The proper fix is still to remove the entry in the widget destroy routine. -> fixed (mitigated)
- [low] [api] `accessibility_tree` does not follow the `a11y_*` naming used by the neighbouring methods (line 44). Renaming it would break `a11y_accesskit.rs` and any callers outside this chunk. -> not fixed (public rename outside this chunk's scope)
- [low] [bug] `A11yAction::Focus` and the Click/Toggle/Increment paths set `self.focused` directly, which may bypass focus-change handling such as blur callbacks and cursor reset (lines 226-272). The central focus setter is not visible in this chunk, so I could not verify this. -> not fixed (cannot verify)
- [low] [api] `A11yAction`, `A11yRole` and `A11yNode` may need `#[non_exhaustive]`, but they are defined in `src/a11y.rs`, outside this chunk. -> not fixed (outside this chunk)
- NOTE: edits for src/tk/a11y.rs:1-297 were REVERTED (checks failed or SEARCH mismatch): '[test failed]\n\n.................................\ntest result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s\n\n\n..\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s\n\n\n.................\ntest result: ok. 17 passed; 0 failed;'

### src/tk/tree.rs lines 1-296
- [medium] [bug] `destroy` compared `id == self.root()` only after the slot generations had been bumped. If `root()` is built from the live generation of slot 0, the comparison fails and destroying the root never requests quit. The check is now computed before the teardown loop (line 101) -> fixed
- [low] [bug] `config(Opt::Disabled(true))` leaves `hovered` and `pressed` pointing at the now-disabled widget (line 178) -> not fixed (the event code that reads those fields is outside this chunk, so I can't tell whether it is a real problem)
- [low] [api] `set_command` and `set_selected` do not follow the `<widget>_<verb>` pattern (lines 207, 274) -> not fixed (renaming public API is a larger change than a minimal edit, and both are documented and consistent with each other)

### src/tk/double_spinbox.rs lines 1-287
- [medium] [bug] `dspin_type`/`dspin_backspace` seeded the edit buffer from the formatted text including the suffix. Typing after the seed gave e.g. "1.50 V5", which fails to parse and silently reverts on Enter. Backspace ate the suffix first. (lines 182-186, 204-208) -> fixed (seed now excludes the suffix)
- [medium] [bug] Stepping with Up/Down/wheel left a stale edit buffer in place, so the display kept showing the old typed text instead of the new value. (line 149) -> fixed (`dspin_step` clears the buffer)
- [medium] [bug] `dspin_set` documents returning false for dead variables but always returned true. It also left a stale edit buffer showing. (lines 96-106) -> fixed
- [low] [bug] `dspin_round` overflowed to infinity for huge ranges (`clamped * 10^decimals`). It could also yield `-0.0`, which displays as "-0.00". (lines 126-129) -> fixed
- [low] [bug] `dspin_user_set` never wrote the variable when its current value was NaN and the target equalled the lower bound, because `NaN.abs() > EPS` is false. (line 142) -> fixed

### src/shortcut.rs lines 1-277
- [severity low] [bug] `parse_key` accepted malformed function-key names such as `F+1` and `F01`, because `u8::from_str` allows a leading `+` -> fixed. The function-key suffix now has to be 1–2 ASCII digits.
- [severity low] [api] I found no other problems in lines 1-277. `ShortcutError` is already `#[non_exhaustive]`, every fn has a `///` doc, and the UTF-8 slicing of the macOS modifier symbols is safe because it advances by `len_utf8`. The literal-plus handling (`Ctrl++`, `+`) and the `Ctrl+` missing-key case behave correctly. -> not fixed (nothing to fix)

### src/tk/text.rs lines 1-275
- [medium] [bug] `trim_lines` with `max == 0` wipes all text, but `text_set_max_lines(0)` treats 0 as "no trimming", and later `text_set` / `text_insert_end` would clear everything (lines 7-13) -> fixed (0 now means unlimited)
- [high] [bug] `text_clear` leaves `selection_anchor` set while the cursor goes to 0, so a stale anchor beyond the empty text could cause out-of-range slicing in selection consumers (lines 238-246) -> fixed (anchor cleared)
- [high] [bug] `text_insert_end` trims from the front but leaves `selection_anchor` stale (possibly out of range or mid-char), and `text_scroll_row` can exceed the new line count (lines 220-235) -> fixed
- [medium] [bug] `text_set_max_lines` trims leading bytes but only clamps the cursor, so it can land mid-char or on the wrong text. The anchor and scroll row are also left stale (lines 262-272) -> fixed (cursor shifted by removed bytes, anchor cleared, scroll clamped)
- [low] [bug] `text_set` does not clamp `text_scroll_row`, so replacing the text with a shorter one can leave the view scrolled past the end (lines 203-216) -> fixed
- [medium] [bug] Caret-only moves (Left/Right/Up/Down/Home/End) return before the scroll-follow logic, so the caret can leave the visible rows (lines 166-174) -> fixed (scroll adjusted on that path too)

### src/tk/canvas.rs lines 1-255
- [medium] [bug] `canvas_create_image_rgba` computes `src_w * src_h * 4` in `usize` without overflow checks. Huge dimensions (e.g. `u32::MAX` squared) panic in debug builds and can wrap in release, or on 32-bit targets, so a mismatched buffer could pass validation (line 227) -> fixed (`checked_mul`)
- [medium] [bug] `canvas_update_image` has the same unchecked size multiplication (line 235) -> fixed
- [low] [bug] `text_size` passes the raw `size` to the text engine. NaN, zero or negative sizes are not sanitized, unlike `canvas_create_text_ex`, which clamps with `size.max(1.0)` (line 202) -> fixed (same clamp)
- [low] [bug] `canvas_add_item` scans all slots twice per insert (`max` over `seq` and a free-slot search), so filling a canvas is O(n²) up to the 1M-item cap (lines 73-74) -> not fixed (a counter would need a new `Node` field outside this chunk; correct as is)
- [low] [api] `text_size` and `canvas_items` (which returns a count) do not follow `<widget>_<verb>` naming exactly (lines 200, 207) -> not fixed (renaming public methods breaks callers outside this chunk; left for a deliberate API decision)
- [low] [bug] Non-finite or huge `size`, `width` and coordinates in the `canvas_create_*` and `canvas_move` functions are stored unvalidated (lines 123-197) -> not fixed (the renderer is outside this chunk; tiny-skia rejects non-finite geometry)

### src/tk/damage.rs lines 1-251
- [medium] [bug] `invalidate_rect` coalescing clears `self.damage` and pushes the union only if it passes a validity check. If the check failed (for example `x + width` overflowing to infinity or rounding to `x`), the damage was silently dropped while `partial_dirty` stayed set, so `render_frame` returned `None` and the screen went stale. Now falls back to a full redraw (`self.dirty = true`). (lines 67-69) -> fixed
- [low] [bug] The scratch-allocation-failure fallback in `render_frame` did not reset `self.coalesced`. After the full render, new damage would then keep taking the coalescing path even with an empty list. (line 213) -> fixed
- [info] Coalescing otherwise never drops damage and is bounded. The list is folded into its union at 16 rects. While `coalesced` is set, every new rect merges into that single union, so the list never exceeds one rect. `coalesced` is reset on every path that clears the damage list. -> no change needed
- NOTE: edits for src/tk/damage.rs:1-251 were REVERTED (checks failed or SEARCH mismatch): 'EDIT src/tk/damage.rs: SEARCH text not found:\n        let Some(mut scratch) = self.scratch.take() else {\n            // Scratch allocation failed; fall back to a full render.\n            let pixmap = self.render_to_pixmap(width, height)?;\n            self.frame = Some(pixmap);\n            self.damag'

### src/tk/scroll_area.rs lines 1-240
- [severity low] [bug] `scrollarea_scroll_to` sends ±infinity to offset 0 instead of clamping to the nearest bound. A huge `scroll_by` delta that overflows to infinity therefore jumps to the top instead of the end (lines 100-101) -> fixed (only NaN maps to 0; infinities now clamp).
- [severity low] [docs] `scrollarea_vbar_width` doc says a hidden bar reserves 0 width, but the code does no visibility check (line 202). The hidden-state field is not visible in this chunk, so I cannot add the check without guessing a field name -> not fixed (needs the visibility field from the other module).
- [severity low] [bug] A scrollbar's `scroll_target` can outlive its destroyed area. The destroy path is outside this chunk, and `scrollarea_vbar_width` already verifies that the target is a live ScrollArea, so it stays safe here -> not fixed (outside chunk).

### src/testing.rs lines 1-232
- none

I found nothing that needs fixing in this chunk.

- **`compare_pixmaps`:** it indexes each image by its own stride and clamps to the overlapping region, so differing sizes cannot cause an out-of-bounds slice.
- **`build_diff_image`:** it indexes both images with `actual`'s stride, which is safe because `assert_golden` panics on any size mismatch before calling it.
- **Missing golden file:** `assert_golden` writes the golden and returns instead of failing. This is documented, so I left it.
- **`GoldenOpts` (not fixed):** it is a public struct with public fields and no `#[non_exhaustive]`. Adding the attribute would break struct-literal construction in downstream tests, and the requirement covers enums only.
- **Stale artifacts (not fixed):** `.actual.png` and `.diff.png` files from an earlier failure are not deleted when a later run passes or blesses. This is cosmetic, so I did not change it.
- **Unused `name` parameter (not fixed):** `write_png` takes `name` only to discard it. It is harmless and in a private function, so I left it.
- **Docs:** every function has a `///` comment.

### src/tk/slider.rs lines 1-228
- [medium] [bug] `slider_key` ignores inverted ranges (`from > to`). Right/Up and Home/End moved the opposite way from the pointer mapping and the documented direction inversion (lines 210-223) -> fixed
- [medium] [bug] `slider_user_set` used an absolute `f64::EPSILON` threshold, so changes on small ranges (e.g. `0..1e-12`) were silently dropped and the command never fired (line 150) -> fixed
- [low] [bug] `slider_set` is documented to return false for dead variables but always returned true (lines 118-122) -> fixed
- [low] [bug] `slider_fraction` returned NaN as `f32` when the bound variable held NaN, whereas `slider_get` maps NaN to `from` (line 165) -> fixed
- [low] [bug] `slider_set_length` replaced `±inf` with 160 instead of clamping as documented (line 98) -> fixed
- [low] [docs] `slider_fraction` doc mentioned a `value` parameter that does not exist (line 156) -> fixed
- [low] [bug] The `slider_set_orient`, `slider_set_length` and `slider_set_ticks` setters only set `dirty`. This may not trigger relayout, though they change the requested size. Layout invalidation lives outside this chunk -> not fixed (cannot verify)

### src/colormap.rs lines 1-218
- [low] [api] The public `Colormap` enum lacks `#[non_exhaustive]`, so adding a palette after 1.0 would be a breaking change (line 6) -> fixed. The `match` in `sample` is in the same crate, so it is unaffected.
- [low] [bug] `legend_rgba` computes `width * height * 4` for the capacity, which can overflow or abort on absurd dimensions (line 198) -> not fixed. The caller controls the size, and a request that large would fail to allocate anyway. Clamping the capacity would only change the failure mode.
- [low] [docs] `heatmap_to_rgba` maps infinite values to `t = 0.5` when `min == max`, which the doc does not say (line 173) -> not fixed. This is harmless and consistent with the degenerate-span handling.
- No other bugs found. NaN is handled in `sample` before the table lookup, so `interpolate_table` never sees it. Its index is clamped to `len - 2`, so there is no out-of-bounds access. The degenerate-span, infinite and NaN cases in `heatmap_to_rgba` behave as documented.

### src/a11y_accesskit.rs lines 1-207
- [severity medium] [bug] The focus node is not checked against the emitted node set (line 69-72). A focused widget that is hidden or unmapped, or otherwise missing from `accessibility_tree()`, makes AccessKit receive a focus id that is not in the tree. Platform adapters can panic or reject that update. -> fixed (falls back to the root)
- [severity medium] [bug] Child ids are emitted without checking that they exist in the update (line 53). A child pruned from the tree, such as a hidden widget, leaves a dangling `NodeId`, which AccessKit treats as an invalid tree. -> fixed (children are filtered to emitted nodes)
- [severity medium] [bug] `map_action` drops `ActionData::NumericValue` (line 133-136). Screen readers set sliders and spin buttons through `SetValue` with a numeric payload, so the toolkit received `SetValue` with `None` and the request was silently lost. -> fixed (the number is passed as a string)

### src/tk/progress.rs lines 1-203
- [severity low] [bug] `progress_advance` called `get_or_insert_with(Default::default)` on every tick, so it allocated an `InputsState` for every visible progress bar even when smooth easing was never enabled (line 152). `progress_is_animating` already treats a missing state as "not smooth". -> fixed
- [severity low] [docs] The `progress_bind` rustdoc says "live in `tk`", but there is no `tk` parameter (line 14). -> not fixed (wording only, no behaviour impact)
- [severity low] [api] `progress_bind` is slightly off the `progress_set_*` naming used by its siblings. -> not fixed (renaming a public method this late is churn, and I can't see the other widgets' conventions from this chunk)
- [severity low] [bug] `progress_requested` measures `"{label} 100%"` even when `show_percent` is false, so the bar can be slightly wider than needed (line 120). -> not fixed (this only over-reserves width and is harmless; changing it would shift layout and goldens for little gain)
- [severity none] [bug] I found no panics, overflow or borrow problems in the rest of the chunk. The easing math is stable for `ms = 0` and for very large `ms`, and NaN or out-of-range values are clamped. -> none

### src/tk/dnd.rs lines 1-202
- [high] [bug] `drop_target_at` also counted hover-only bindings, so a widget with only a hover binding swallowed drops. Its drop callback never ran, yet `sim_file_drop` returned true, and it shadowed a bound ancestor (line 81-99). Drop resolution now uses drop bindings only. Hover resolution (Enter/Leave/Drop) still considers either kind of binding. -> fixed
- [medium] [bug] The walk-up ignored the active modal. Ancestors of the modal, and the root fallback, could be returned as targets (lines 83-92). The walk now stops at the modal, and the root fallback is skipped while a modal is active. -> fixed
- [medium] [bug] Bindings for destroyed widgets were never pruned, so the `dnd` vectors grew without bound. `file_hover_target` could also return a dead id (lines 13-55, 102). `bind_*` now prunes dead entries, and `file_hover_target` filters dead ids. I can't see the destroy path in this chunk, so the pruning lives here. -> fixed
- [low] [api] There was `unbind_file_drop` but no `unbind_file_hover`, which breaks the bind/unbind symmetry (line 31). -> fixed (added)
- NOTE: edits for src/tk/dnd.rs:1-202 were REVERTED (checks failed or SEARCH mismatch): '[test failed]\n\n.................................\ntest result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s\n\n\n..\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s\n\n\n.................\ntest result: ok. 17 passed; 0 failed;'

### src/tk/window.rs lines 1-200
- [severity low] [bug] `geometry(0, h)` or `geometry(w, 0)` passes a zero dimension to `sim_resize` and to the requested window size. A zero-sized pixmap or surface is invalid in tiny-skia and softbuffer (line 19) -> fixed (clamped to at least 1 px)
- [severity low] [docs] `geometry` documents physical px while `minsize` and `maxsize` use logical px. This is probably intentional, so only the docs were adjusted, not the behaviour (line 18) -> fixed (doc clarified)
- [severity low] [bug] `handle_close_request` takes the callback out before calling it. A `sim_close_request` made from inside the callback therefore finds no callback and quits without consulting the veto. This reentrant edge case is acceptable and documented behaviour, and fixing it would need a redesign (line 177) -> not fixed (edge case, needs redesign)
- [severity low] [api] The tkinter-style names (`title`, `geometry`, `minsize`, `resizable`) intentionally do not follow `<widget>_<verb>`, and `clear_minsize`/`clear_maxsize` are consistent with each other -> not fixed (intentional)

### src/var.rs lines 1-199
- [low] [bug] A handle from a dropped `Tk` can alias a variable in a later `Tk`, because the owner token is the address of `var_owner`, which the allocator may reuse after the first `Tk` drops (lines 26-28, 64-66, 161-165). The comment only guarantees the address stays unique while the owning `Tk` is alive. The proper fix is a process-wide atomic counter for the owner id, but `Tk` construction lives outside this chunk. -> not fixed (the fix needs changes outside this chunk)
- [low] [bug] Variable slots are never reused, so creating and destroying variables dynamically grows `vars` without bound (lines 158-177). The design is documented and intentional. A linear free-slot scan would make creation O(n), and a free list needs a new `Tk` field outside this chunk. -> not fixed (deliberate design; slots are small)
- [low] [bug] If a trace panics, `notifying` stays true and the trace is dropped (lines 111-128). Later changes to that variable then never notify. This only matters when the caller catches the unwind, so I left it alone. -> not fixed (unwind-only edge case; a guard would be a refactor)
- [low] [api] `#[derive(Debug)]` on `Var<T>` adds an unwanted `T: Debug` bound (line 22). It is harmless for the supported types (`String`, `i64`, `bool`, `f64`), so I did not change it. -> not fixed (no practical impact)

No other defects found in this chunk:
- The generation, owner and type checks reject destroyed, foreign and mistyped handles.
- `destroy` clears the value, trace and `notifying` flag, so the generation saturating at `u32::MAX` is harmless because the value is gone.
- Reentrant writes and traces that replace or destroy their own variable are handled correctly.
- The borrow flow in `set` is sound, and every fn has a `///` doc.

### src/fonts.rs lines 1-187
- [severity low] [api] Public enum `DefaultFont` lacked `#[non_exhaustive]`, so adding a variant later would be a breaking change (line 36) -> fixed
- No other bugs found in this chunk. The fallback logic, `has_face` and `has_family` are sound, with no panics or UTF-8 hazards. The `warn_once` guard is correct, and every fn has a doc comment.

### src/image.rs lines 1-181
- [medium] [bug] `ImageFit::Center` clamped width and height independently (`nat.min(w)` and `nat.min(h)`). An image larger than the widget in only one axis was squashed non-uniformly, so its aspect ratio was distorted. Now it is scaled down uniformly, and only when it doesn't fit (line 130-131) -> fixed
- [low] [docs] The `Center` variant doc said "no scaling" without mentioning the downscale when the image exceeds the widget (line 16) -> fixed
- [low] [bug] `build_items` clones the whole RGBA buffer on every call. This is a performance concern rather than a correctness bug, and fixing it would need a `CanvasItemKind` change outside this chunk (line 157) -> not fixed (outside the chunk, no correctness impact)

### src/tk/entry.rs lines 1-162
- [severity medium] [bug] When the entry is at `MAX_ENTRY_BYTES` and a selection is active, typing or pasting is rejected. The insert limit ignores the bytes the selection will free (line 137) -> fixed
- [severity low] [bug] Input that cleans down to an empty string (for example only control characters, or a full entry) falls through to the tail of the function. That clears the selection anchor and repaints without any change to the text (lines 138-143) -> fixed
- [severity low] [bug] `join_text` silently drops the prefix or suffix if an index is out of range or off a char boundary (lines 9, 11) -> not fixed (documented as tolerant; callers pass cluster boundaries)

### src/tk/chart.rs lines 1-155
- none

I found nothing in lines 1-155 that needs an edit.

- **Borrows:** `self.dirty = true` comes after the last use of `state` in every method, so the borrows are fine.
- **Slicing:** `&points[start..]` uses a `saturating_sub` start, so it cannot go out of range.
- **Retention limits:** `max_points` is clamped to at least 1 in `chart_set_data`, `chart_append` and `chart_set_max_points`, so a zero limit cannot occur.
- **`chart_set_y_range`:** it rejects non-finite bounds and `lo >= hi`, and it does this before touching state.
- **Docs:** every function has a `///` comment, and the naming follows `chart_<verb>`.
- **Not fixed, outside this chunk:** non-finite `(x, y)` values are accepted by `chart_append` and `chart_set_data`. Whether they can break `bounds()` or drawing depends on code outside this chunk, so I left the behaviour alone.
- **Not fixed, outside this chunk:** `ChartKind` is defined in `crate::chart`, so any `#[non_exhaustive]` attribute belongs there, not in this file.

### src/tk/ime.rs lines 1-151
- [low] [bug] `ime_cursor_area` passed the raw `node.cursor` to `floor_char_boundary` without clamping to the value length. A stale or out-of-range cursor could panic or index past the end, depending on how the helper is written (line 127) -> fixed
- [low] [bug] `ime_preedit()` returned a composition for a widget that had since been disabled or made read-only, so a stale preedit kept rendering (lines 34-40) -> fixed
- [low] [bug] `sim_ime_preedit` (not-wanted branch) and `sim_ime_commit` cleared a stored composition without setting `dirty`. The stale preedit stayed on screen until some other redraw (lines 60-63, 88) -> fixed
- [low] [bug] A stored preedit is only dropped lazily on the next `sim_ime_*` call. If focus leaves widget A and returns to A with no IME event in between, the old composition reappears (lines 34-49) -> not fixed (the fix belongs in the focus-change code, outside this chunk; the backend should send an empty Preedit on focus loss)
- [low] [bug] `ime_cursor_area` ignores the preedit text and any Text horizontal scroll, and measures the raw value even for masked Entries. This is documented as approximate -> not fixed (approximation by design)

### src/tk/image.rs lines 1-147
- [medium] [bug] `image_set_rgba` computes `w * h * 4` in `usize` without overflow checking (line 23). Large `w`/`h` can overflow on 32-bit targets, or on 64-bit with `w` and `h` near `u32::MAX`. That panics in debug builds and wraps in release, so a mismatched buffer could be accepted. -> fixed (checked multiplication)
- [low] [bug] `image_set_fit` on an image with no source inserts a 1x1 transparent RGBA placeholder (lines 107-113). `image_natural_size` then reports `Some((1, 1))` for an image that is logically empty, and `image_clear` drops the stored fit. Fixing this needs an empty `ImageSource` variant or an `Option` source in `ImageData`, which lies outside this chunk. -> not fixed (needs a change outside this chunk)
- [low] [api] The verbs are inconsistent: `image_load_png` and `image_set_svg` differ from the `image_set_*` pattern, and `load_svg` has no widget prefix. The latter is deliberate, since it registers a shared `ImageId` like `register_image_rgba`. -> not fixed (renaming is a breaking change with no functional bug)

### src/tk/shortcuts.rs lines 1-146
- [severity high] [bug] Shortcut dispatch never restores the real callback (lines 84-91). The entry is removed, and its callback is swapped for a no-op. The no-op is then reinserted and the real `cb` is dropped, so a shortcut works exactly once and is then dead. -> fixed
- [severity medium] [bug] A callback that unbinds its own shortcut has no effect (lines 84-91). The entry was already removed, so `unbind_shortcut` returns false. The `still_bound` check is then always false and the entry is reinserted anyway. -> fixed (the entry stays in place during the call, and the callback is restored only if the id is still bound)
- [severity low] [bug] The entry's position was lost when callbacks bound or unbound other shortcuts during dispatch, because the entry was reinserted by a stale index (line 90). -> fixed (the callback is now restored by id, and binding order is preserved)
- [severity low] [bug] `next_shortcut_id` uses `wrapping_add` (line 21), so ids could collide after 2^32 or 2^64 binds. -> not fixed (practically unreachable, and the field type is defined outside this chunk)

### src/tk/ring.rs lines 1-129
- [medium] [bug] `ring_set_thickness` doc promises a clamp to `1.0..=diameter/2`, but the code only floors at 0. A huge or zero thickness is stored unclamped (line 92) -> fixed (clamps finite values to `1.0..=max(diameter/2, 1.0)`; non-finite values still store 0, and the doc now says so)
- [medium] [bug] `ring_set` is documented to return false for dead variables but returns true, and sets dirty, even when the variable is dead (line 68) -> fixed (checks the variable is live first)
- [low] [bug] An infinite `diameter` passed to `ProgressRing::new` is stored as-is and breaks layout (line 28) -> fixed (non-finite falls back to 48.0)
- [low] [bug] `ring_advance` allocates an `InputsState` on every tick for non-spinner rings (line 111) -> not fixed (harmless, one allocation per ring, and the behaviour matches the other widgets)

### src/tk/listbox.rs lines 1-128
- [severity low] [bug] `listbox_delete` and `listbox_insert` change or clear the selection without firing the select callback (lines 19-23, 38-44). `listbox_select` does fire it on change, so this is slightly inconsistent. Tkinter does not fire `<<ListboxSelect>>` for programmatic edits, and firing here would add a reentrancy path inside delete. -> not fixed (behavior decision, not a defect)
- [severity low] [bug] `list_select_running` stays `true` if the callback panics (line 117-120). The next dispatch for that widget is then silently skipped. This only matters if panics are caught and the toolkit keeps running, and the callback is already taken out of the node at that point. -> not fixed (the callback is lost on panic anyway, and the toolkit is not expected to survive one)
- [severity low] [api] The public methods follow the `listbox_<verb>` pattern, have `///` docs, and use no public enums. No arithmetic can overflow: `last.saturating_add(1)` is used, and `idx + 1 - vis` is safe because `idx >= scroll + vis`. Indices are clamped, and `node_mut` guards stale ids. -> nothing to fix

### src/tk/theme.rs lines 1-126
- [severity low] [api] `sim_system_theme` (line 115) has a `sim_` (simulate) prefix, but its doc says it is the single entry point for the real window backend as well as for tests. A rename would touch call sites outside this chunk and break the public API, so it is better done in a dedicated change. -> not fixed (the rename belongs outside this chunk)
- [severity low] [docs] Lines 61-62 (`let original = ...; let _ = original;`) are redundant but correct. -> not fixed (working code, no behavioural effect)

I found no other defects in lines 1-126. The colourblind capture and restore logic is consistent with the documented behaviour. Every function has a `///` doc comment. No panics, indexing, or arithmetic risks appear in this chunk.

### src/tk/switch.rs lines 1-106
- [low] [bug] `switch_set` is documented to return false for dead variables, but it called `var.set` and returned true unconditionally, so a dead or foreign variable was reported as success and the tree was marked dirty (lines 45-48) -> fixed
- [low] [api] No other API, docs or `#[non_exhaustive]` issues in this chunk. `Switch` is a constructor namespace with no public enums, and the method names follow `switch_<verb>`. The animation code has no panics, overflow, or state-reset problems here.

### src/a11y.rs lines 1-97
- [low] [api] `A11yNode` is `#[non_exhaustive]`, so external users and tests cannot construct it. This is acceptable for a snapshot type produced only by the toolkit. -> not fixed (intentional)
- [low] [docs] `A11yNode::value` doc says "Current text/selection value", which is vague, but it is not wrong. -> not fixed (cosmetic)
- [low] [api] `A11yNode` derives `PartialEq` with `f64` fields, so `Eq` cannot be derived. This is correct as written. -> not fixed (n/a)
- [low] [api] All enums are already `#[non_exhaustive]`, every item has rustdoc, and the derives are consistent. No bugs are present in this type-only chunk. -> not fixed (nothing to fix)

### src/tk/notebook.rs lines 1-92
- [medium] [bug] `notebook_add` accepts the same page twice, which creates duplicate tabs that point at one frame (line 34) -> fixed
- [low] [bug] If a changed callback panics, `notebook_changed_running` stays true and the callback is lost (line 82). This would need a drop guard and the surrounding node/widget code, which is outside this chunk -> not fixed (out of scope)
- [low] [api] `grid_forget` delegates to `pack_forget` and unmaps any manager, not only grid. This is harmless and consistent with tkinter's observable behavior -> not fixed
- [low] [bug] A `notebook_select` call made from inside the changed callback updates the selection but does not re-fire the callback. This is deliberate reentrancy protection -> not fixed

### src/tk/inputs_state.rs lines 1-116
- none

This chunk contains only three public enums and a `pub(crate)` state struct, so there is no executable logic to hold a panic, overflow, UTF-8 or reentrancy bug.
- All three public enums (`RingMode`, `Orient`, `CheckState`) are `#[non_exhaustive]` and fully documented.
- Every struct field has a `///` doc comment.
- The struct is stored inside the widget node, so it is dropped with the widget and is not reset separately on destroy.
- Two things could not be checked from this chunk. One is that `Default` gives `slider_step` and `dspin_step` a value of 0.0, so their constructors must set a sensible step. The other is that `scroll_area_content` could dangle if the content frame is destroyed independently of its ScrollArea. Both are outside the lines reviewed.

### src/lib.rs lines 1-88
- [severity medium] [api] Public enum `RtkError` lacked `#[non_exhaustive]`, so adding a variant after 1.0 (e.g. a new construction failure) would be a semver break for downstream exhaustive matches (line 59) -> fixed
- [severity low] [docs] The other public enums re-exported here (`Side`, `Key`, `State`, and so on) are defined in other modules, so I cannot check their `#[non_exhaustive]` attributes or docs from this chunk -> not fixed (outside the reviewed chunk)
- [severity low] [bug] The rest of the chunk is correct. The `Display` impl matches all variants, every fn has a `///` doc, and the `cfg` gating on `a11y_accesskit`, `tray` and the doctest struct is consistent. -> not fixed (nothing to fix)

### src/tk/clipboard.rs lines 1-69
- [severity low] [bug] `clipboard_get` returned `Some("")` when the OS clipboard held empty text, while the in-memory path returns `None` for empty, so the two paths disagreed (lines 56-59) -> fixed
- [severity low] [api] None of the public methods carry `#[non_exhaustive]`, naming, or docs problems. `clipboard_clear`, `clipboard_append` and `clipboard_get` follow the tkinter-style naming pattern and all have `///` docs -> not fixed (nothing to fix)

### src/dnd.rs lines 1-43
- none

The chunk has only type definitions and a struct, and I found nothing to fix in it.
- `DropPhase` is `#[non_exhaustive]` and has rustdoc on every variant.
- Both callback type aliases and every `DndState` field are documented.
- `DndState` is `pub(crate)`, so it adds nothing to the public API.
- Whether state is cleared on widget destroy depends on `tk/dnd.rs`, which is outside this chunk. The struct supplies the fields (`drop`, `hover`, `target`) that cleanup would have to handle.

### src/tk/tooltip.rs lines 1-37
- [low] [bug] `active_tooltip` returns `Some("")` when a tooltip is set to an empty string, so an empty tooltip bubble could be shown (line 31) -> fixed (empty text is now treated as no tooltip).
- [low] [api] `tooltip` (line 12) is a bare noun and does not follow the `<widget>_<verb>` pattern; `tooltip_set` would fit better -> not fixed. Renaming a public method would break callers outside this chunk, and it is a deliberate API decision rather than a local fix.
- [low] [bug] `tooltip` and `tooltip_remove` change what is displayed without requesting a redraw or damage (lines 14, 23) -> not fixed. The redraw/invalidation mechanism is not visible in this chunk, so I can't safely call it. If the tooltip text changes or is removed while shown, the caller should invalidate.
- [low] [docs] The `bool` return values (false when the widget id is not live) are undocumented (lines 11, 18) -> fixed.

- RECOVERED: src/widgets.rs 1-339 edits re-applied (they include `#[non_exhaustive]` on a public enum). DeepSeek added the wildcard arm that examples/gallery.rs then needed (recover-widgets). Green.

- RECOVERED: src/tk/a11y.rs 1-297 edits re-applied. Checks are green with no extra calls, so the earlier test failure came from the concurrent PR #8 merge or flakiness, not from these edits.

- RECOVERED: src/tk/dnd.rs 1-202 edits re-applied. After the fix, a hover-only binding no longer swallows drops. tests/m012_dnd.rs expected the old behaviour, where `sim_file_drop` returned true on a hover-only target, so DeepSeek updated it to expect false and to check that the Drop phase still reaches the hover callback (recover-dnd). The lead reviewed this. It matches the intended semantics of the BizyBot5 hand fix flagged for review.

- RECOVERED: src/tk/damage.rs 1-251. One SEARCH block was stale; DeepSeek re-targeted it (recover-damage r1). Result: degenerate coalesced damage now forces a full repaint instead of being silently dropped, and the scratch-failure fallback resets `coalesced`. This confirms the BizyBot6 coalesce hand fix otherwise never drops damage and stays bounded.

### src/tk/tristate.rs lines 1-172
- none

I found no bugs or API problems in lines 1-172 of `src/tk/tristate.rs`.

- **Borrows:** the immutable `node` borrows at lines 59 and 138-145 end before `var.get(self)` and `var.set(self, …)` need `self`, so there is no borrow conflict.
- **Cycle logic:** the transitions in `tristate_activate` match the rustdoc on `checkbutton_set_cycle_mixed`. With cycling on, the order is `Unchecked -> Checked -> Mixed -> Unchecked`. With cycling off, a mixed box goes to `Checked`.
- **Mixed flag on disable:** `checkbutton_set_tristate(false)` clears both `check_mixed` and `check_cycle_mixed`, so no stale mixed state survives.
- **API:** all public methods follow `checkbutton_<verb>` naming and have `///` docs. `CheckState` is re-exported from `lib.rs`, and I did not check it for `#[non_exhaustive]` because its definition is outside this chunk.

I did not check whether `inputs` is dropped when a widget is destroyed, because that code is outside this chunk.

- NOTE: the first Sonnet call writing tests/tk_review_regressions.rs used all 32k max_tokens on reasoning, ignoring the 16k budget, and returned no code ($0.37). Per the rules it was not retried at the same settings. The work was split into two smaller tasks with max_tokens 40k.

### Fable regression tests
- tests/tk_review_regressions.rs (Sonnet): Fable F1-F4. All pass.
- tests/tk_review_regressions_2.rs (Sonnet; compile errors fixed by DeepSeek and Sol): F6-F8, F20, pass-3 F5 (passes; nested dialogs were fixed in 0.11), pass-3 F6 (**failed: real bug**).
- **pass-3 F6 fixed** (DeepSeek, f6-ask-directory and listbox-activate, with Sol fixing the new test): `ask_directory` now navigates into a sub-directory or `..` on Enter or double-click, and Enter otherwise accepts. This added `Tk::listbox_bind_activate`, which fires on Enter or double-click but not on selection changes, plus tests/listbox_activate.rs.

### src/dialog.rs lines 646-894
- [high] [bug] `ask_directory`: the ".." branch in `activate` holds a `cur_dir.borrow()` temporary across the whole `if let` body (edition 2021). The `borrow_mut()` inside then panics with `BorrowMutError`, so navigating up crashes (line 803) -> fixed
- [medium] [bug] F7/F8: the save dialog's list was static. Activating a file did not fill the name entry, directories could not be entered, and the path label never updated (lines 678-683) -> fixed. Enter/double-click on a file now fills the entry, a directory or ".." navigates and refreshes the list, and the path label updates. I only used APIs already visible in this chunk, so the fill happens on activation rather than on single-click selection.
- [low] [bug] The save dialog's refresh clears the listbox itself and adds ".." if `populate_file_list` didn't, so the result is the same either way -> fixed
- [low] [bug] F10/F11: the filter syntax and the empty typed-confirm phrase are not in this chunk (646-894), so I could not check or fix them -> not fixed (out of chunk)

### src/dialog.rs lines 345-645
- [medium] [bug] F11: with an empty `phrase`, Confirm started disabled and the entry never fires a change event, so the dialog could only be confirmed with Enter (line 394) -> fixed (Confirm now starts enabled when `phrase` is empty)
- [medium] [bug] F8: filter extensions given as `*.png`, `.png` or `*.*` never matched in `read_dir_entries`, and the combobox label showed `*.*.png` or `*.` (lines 528-530, 555) -> fixed (extensions are normalised once, and the label shows `(*)` for match-all filters)
- [medium] [bug] F7: changing the filter combobox does not refresh the file list. The list is built once from the first filter and only re-filtered on navigation (lines 560-565, 597) -> not fixed (this chunk shows no variable-change or combobox-change hook, and I won't invent API)
- [low] [bug] F10: the save dialog's list does not fill the name entry -> not fixed (`ask_save_filename` is outside this chunk)

### src/dialog.rs lines 1-344
- [medium] [bug] `build_scrim_card` pushed the outer dialog's state onto `modal_stack` and cleared `tk.modal` before creating the scrim and card. If `Frame::new` failed (e.g. arena full), the outer dialog was left on the stack with no active modal, and a half-built scrim could leak. The push now happens only after both frames exist, and a card failure destroys the scrim. (lines 82-123) -> fixed
- [low] [api] Public enum `MessageKind` lacked `#[non_exhaustive]` (line 19) -> fixed
- [low] [bug] `message_box`, `ask_yes_no` and `ask_string` call `.unwrap()` on `Label::new`, `Button::new`, `Entry::new` and `Frame::new`. This can panic and leave the modal half-built if the arena fills mid-dialog. -> not fixed (it needs a larger restructure of each function, and the failure is only reachable at arena exhaustion)
- [low] [api] `message_box` ignores `_kind`, so Info, Warning and Error render identically. -> not fixed (the icon is a feature, not a bug)
- [info] F7/F8/F10/F11 concern the file-dialog filter, filter syntax, save-dialog name fill and typed-confirm phrase. None of that code is in lines 1-344. -> not applicable to this chunk
- The F5 modal stack in `finish_modal`, `close` and `modal_depth` is intact and unchanged.

### src/tk/combobox.rs lines 1-30
- [severity low] [bug] `combobox_set_values` (line 12) neither clamps nor resets the bound selection variable when the new list is shorter. `combobox_get` then returns `(idx, "")` with a stale index. This is safe: `get` returns `None` for out-of-range indices, and a negative `i64` cast to `usize` becomes huge, so it can't panic. Clamping would mean writing to the variable and could surprise users who set the variable first and values second, and the `combo_var` write path isn't visible in this chunk. -> not fixed (intentional; the caller owns the variable)
- [severity low] [api] `combobox_get` (line 21) returns the raw variable index with empty text for out-of-range or negative selections, where tkinter's `current()` gives -1. It also defaults to index 0 when no variable is bound. The result is non-panicking and unambiguous (empty text), and other callers in the crate may rely on the raw index, so I left it unchanged. -> not fixed (behaviour change outside this chunk's scope)
- [severity low] [docs] `combobox_get` doc (line 20) does not state that the text is empty when the index is out of range, nor what `bool` and `Option` mean on the other methods. This is cosmetic, and the lines are otherwise documented. -> not fixed

No edits needed: no panics, UTF-8 slicing, overflow or borrow issues in lines 1-30. The mutable node borrow ends before `self.dirty = true` (NLL), and the casts are safe.

### examples/bench.rs lines 1-166
- [severity low] [docs] The `demo_like` doc comment says the tree is "three side-by-side frames containing a label and entry". The function actually builds radio buttons, a checkbutton, a scale, a spinbox, a progress bar, a listbox with a scrollbar, and a canvas with 60 ovals. Since users copy from this example, the doc could mislead them (line 10) -> fixed
- [severity low] [bug] I found nothing else in this chunk that needs changing. There are no panics on user paths: `?` and `ok_or` are used instead of `unwrap`. `w - (i % 2)` cannot underflow for the sizes used. The integer-literal inference in `f64::from(frames)` compiles, so I left it. -> not fixed (no issue)

### examples/bench_table.rs lines 1-173
- [low] [bug] `build_table` and `build_tree` use `.expect()` on the fallible `Table::new` and `TreeView::new` constructors (lines 24, 53), even though `main` already returns `Result<_, Box<dyn Error>>`. Users copy this example, so it should show error propagation with `?` instead of panicking. I assumed both constructors return `Result<_, RtkError>`, which is consistent with `RtkError::InvalidParent` and `ArenaFull` in lib.rs. -> fixed
- [low] [bug] The `.expect("root")` on `tree_insert` (line 64) and the discarded result of the leaf `tree_insert` (line 66) are not changed. I can't see the return type of `tree_insert`, so I can't convert it safely. -> not fixed (the signature is outside this chunk)
- [low] [docs] The first `Tk` stays alive after it is shadowed at line 156, so two toolkits exist at once. This is harmless in a headless run, and `Tk::new` allocates nothing global as far as the visible API shows. -> not fixed
- [low] [docs] `report` prints FAIL but `main` still exits 0, so the "performance check" cannot gate CI. This is a design choice for an example, so I left it. -> not fixed

### examples/counter.rs lines 1-27
- none

I found no bugs and no API or doc problems in lines 1-27.

I could only see `lib.rs`, not the signatures of `Tk` or `PackOpts`, so I could not check three things. These are whether `Tk::new()` returns a `Result`, whether `mainloop()`'s error type converts to `Box<dyn Error>`, and whether `PackOpts` is `Copy`. The code reuses `options` for two `pack` calls, which only compiles if it is `Copy`.

Within what is visible:
- The `///` doc comment on `main` is present and the module doc matches what the program does.
- Fallible widget construction propagates with `?` instead of `unwrap`.
- `count.get(tk)` is handled as an `Option`.
- `saturating_add` avoids overflow on the counter.

### examples/demo.rs lines 1-139
- [low] [bug] `--png` as the last argument, with no path, was silently ignored. The demo then opened a window instead of writing a PNG. (line 21) -> fixed: it now returns an error.
- [low] [bug] The canvas drag callback adds one oval per Press/Move event with no cap. A long drag grows the item list without bound, but the "Clear Canvas" button already resets it, so this is acceptable for a demo. (line 122) -> not fixed (demo behaviour, user-clearable)
- [low] [api] Several calls cannot be checked against the reference, because only lib.rs is shown. These are `listbox_insert(.., usize::MAX, ..)` as an append, `Var::trace`, `Var::get` returning `Option`, and `Theme: PartialEq`. -> not fixed (no evidence of a bug in the visible code)

### examples/form.rs lines 1-50
- none

I found nothing in lines 1-50 to fix. I could only check this chunk against `lib.rs`, so I could not confirm the signatures of `Tk` methods such as `mainloop`, `grid`, `set_command` and `entry_get`, or the `GridOpts` field types and the `Sticky::EW` variant.

- **Error handling:** every fallible constructor uses `?`, and `main` returns a boxed error, so nothing can panic from a bad parent or arena-full condition. `unwrap_or_default()` on `entry_get` is a safe fallback for a destroyed widget.
- **Grid layout:** the grid options are consistent. The submit button spans both columns with the default column 0, and column 1 is made expandable so the entries stretch.
- **Doc comments:** the module and `main` doc comments match what the code does.

### examples/gallery.rs lines 637-800
- [medium] [bug] `--screenshot-dir` given without a value is silently ignored, so the interactive window opens instead of rendering screenshots (lines 761-765) -> fixed (now returns an error)
- [medium] [bug] Semantic-colour swatches and the label text colour are fixed at build time (lines 644-653). `tokens` is a snapshot, and the label colour is hard-coded `rgb(25,25,25)`. After a theme switch, including the dark-theme screenshots, the swatches still show the Light colours and the labels may be unreadable on a dark canvas. -> not fixed (the canvas items have no re-theme hook visible in this chunk, and the `ColorRole` names for a themed text colour are not shown; this needs a rebuild-on-theme-change callback)
- [low] [bug] The canvas height is hard-coded to 600 (line 646) while the swatch rows are generated from `ColorRole::ALL` at 26px each. If the role count grows past about 22, rows are clipped. -> not fixed (the window is only 760px tall, so the canvas cannot simply be sized to the role count)
- [low] [docs] `iconphoto_rgba` errors are discarded with `let _` (line 778). -> not fixed (the icon is cosmetic and failure on some platforms is acceptable)
- [low] [docs] The `_unused` and `_grid_defaults` placeholder functions exist only to silence warnings (lines 787-799). -> not fixed (they are harmless, documented, and a refactor is out of scope)

### examples/gallery.rs lines 310-636
- [medium] [bug] `tree_insert(...).expect("root"/"child")` panics on a fallible path in an example users copy (lines 481, 483) -> fixed (propagates an error through the function's `Result` return instead of panicking)
- [low] [bug] `chart_add_series(...).unwrap_or(0/1)` silently falls back to a series index that may not exist, so data would go to the wrong series or be dropped (lines 546, 547, 576, 588) -> fixed (propagates an error instead)
- [low] [docs] "Open"/"Quit" menu items show `Ctrl+O`/`Ctrl+Q` accelerators. These are display-only unless a shortcut is registered elsewhere, which this chunk cannot show -> not fixed (cannot verify from this chunk)
- [low] [bug] `tick * 60` in the timer callback could overflow after a `wrapping_add` -> not fixed (unreachable in practice: it would take about 2^64/60 ticks at 500 ms each)

I assumed `tree_insert` and `chart_add_series` return `Option`, based on how the existing code uses them. If either returns `Result`, the `ok_or` edits will not compile.

### examples/gallery.rs lines 1-309
- [medium] [bug] The "row" frame packs Normal/Accent/Disabled with the default side (Top), so the buttons stack vertically instead of sitting in one row (lines 231, 240, 244) -> fixed
- [medium] [bug] The listbox and its scrollbar are packed in `list_row` with the default side, so the vertical scrollbar lands below the listbox instead of beside it (lines 287, 292) -> fixed. This assumes `Fill::Y` exists alongside `Fill::X` and `Fill::Both`.
- [low] [bug] The canvas text uses a hard-coded near-black colour (line 306), which may be unreadable on dark themes -> not fixed. The palette's text-colour field isn't visible in this chunk, so a safe replacement can't be written. It is also a deliberate Okabe-Ito-style fixed colour demo.
- NOTE: edits for examples/gallery.rs:1-309 were REVERTED (checks failed or SEARCH mismatch): '[clippy failed]\nerror: using `clone` on type `PackOpts` which implements the `Copy` trait\n   --> examples/gallery.rs:236:21\n    |\n236 |     tk.pack(normal, row_opts.clone());\n    |                     ^^^^^^^^^^^^^^^^ help: try removing the `clone` call: `row_opts`\n    |\n    = help: for further info'

### examples/gallery_charts_media.rs lines 1-168
- [low] [bug] `Button` and `Image` are imported unconditionally but only used inside the `#[cfg(feature = "svg")]` block, so a build without `svg` emits unused-import warnings (line 12) -> fixed (imports moved under the same cfg).
- [low] [bug] `tick` is advanced with `wrapping_add`, but `tick * 60` is a plain u64 multiplication that would panic on overflow in debug builds. The two are inconsistent, and users copy this pattern (line 107) -> fixed (computed in f64).
- [low] [api] `chart_add_series(...).unwrap_or(0)` and `.unwrap_or(1)` silently fall back to a guessed series index if creation fails, so later appends could go to the wrong series (lines 27, 28, 34, 48) -> not fixed. I can't see the return type, and the fallback can't panic. Propagating the error would be better in a real app.
- [low] [docs] The comment says x is "seconds since 2026-01-01T00:00Z", but the time axis may interpret x as Unix-epoch seconds, so labels could read as 1970 (line 103) -> not fixed. I can't confirm the axis semantics from this chunk.
- [low] [docs] The tray "Show" menu item (index 0) has no handler, and a `Tray::new` error is silently ignored (lines 144-153) -> not fixed. This is acceptable demo behaviour and can't panic.

### examples/gallery_m011.rs lines 1-189
- none

I found nothing to fix in this chunk. I could only check it against `lib.rs`, so the widget and `Tk` method signatures it calls are unverified.

- **Error handling:** The `.expect(...)` calls sit inside the example's builder helpers, which return `WidgetId` rather than `Result`. That is a style choice, not a panic on a path where users supply input, so I left it.
- **Accelerator labels (lines 17, 22):** `with_accel("Ctrl+O")` and `with_accel("Ctrl+Q")` may be display-only labels rather than bound shortcuts. I couldn't confirm this from the reference, so I left it.

### examples/gallery_progress_inputs.rs lines 1-210
- [severity low] [bug] The demo timer rewrites `progress_var` and the ring every 500 ms but never updates the slider. Dragging the slider is overwritten half a second later, and the slider no longer matches the bar and ring, contradicting "wires the sliders to the progress widgets" (lines 190-196) -> fixed (the timer now also sets `slider_var`)
- [severity low] [docs] The page stacks about 1000 px of widgets in a 900 px window with plain `pack`, so the bottom of the page (canvas and its scrollbar) can be clipped on `main` (line 205) -> not fixed (changing the layout or window size is a restyle of working code, and the 1.0 gallery embeds the page in its own container)
- [severity low] [bug] The canvas rectangles capture `accent` once at build time, so they keep the old color when the theme buttons are used (lines 179-183) -> not fixed (canvas items are static draws and there is no theme-change hook in the visible API)

### examples/hello.rs lines 1-21
- none

I found nothing to fix in lines 1-21. I could not check the calls into `Tk` or `PackOpts`, because only `lib.rs` was provided. The file is consistent with the exported API as far as I can see.

- Every public function has a `///` doc comment.
- The `?` operators on `Label::new` and `Button::new` convert `RtkError` into `Box<dyn Error>` correctly, since `RtkError` implements `std::error::Error`.
- The example has no `unwrap` calls and no panics on fallible paths.

Two assumptions I could not verify:
- `PackOpts` must be `Copy`, because `options` is passed by value twice (lines 13-14).
- `mainloop()` must return a `Result` whose error converts into `Box<dyn Error>`.

### examples/modern.rs lines 1-308
- [low] [bug] The worker thread is never told to stop after `mainloop()` returns. `running` is only cleared in screenshot mode, so the thread keeps running and posting to the proxy after the window closes. Users who copy this pattern inherit the leak. (lines 300-306) -> fixed
- [low] [docs] The `val <= 100` loop guard is dead, because `val` wraps at 102 and never exceeds 100. Only the `running` flag ends the loop, and that is now cleared on exit. -> not fixed (harmless, no behaviour change)

I found no other real API misuse or panic paths in this chunk. Every fallible constructor uses `?`, and the `--screenshot` argument parsing is bounds-checked.

