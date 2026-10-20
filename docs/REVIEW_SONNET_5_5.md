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

