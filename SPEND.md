# LLM spend log (OpenRouter)

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) |
|---|---|---|---|---|---|
| 2026-10-17 | google/gemini-3.8-flash | src/timer.rs (after/every/cancel + unit tests) | 861 | 11102 | 0.0423 |
| 2026-10-18 | google/gemini-3.8-flash | v0.8 widgets (Text/Combobox/Notebook/tooltips/canvas image/colormap) | 54649 | 51593 | 0.2345 |
| 2026-10-18 | google/gemini-3.8-flash | v0.8 proxy/dialogs/modern theming/icons | 71646 | 59902 | 0.2784 |
| 2026-10-18 | anthropic/claude-fable-5.1 | v0.8.1 review pass 1 (tk/widgets) | $2.9074 |
| 2026-10-18 | anthropic/claude-fable-5.1 | review pass 2 (render/style/layout, empty, estimated) | $3.1000 |
| 2026-10-18 | anthropic/claude-fable-5.1 | review pass 3 (lib/var/timer/app/dialog/icons/colormap) | $2.8394 |
| | | **Fable review total (separate $12 cap)** | **$8.8468** |

## rtk 1.0 build (running total per model)

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) | Model running total |
|---|---|---|---|---|---|---|
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-0-clippy r0 | 64007 (0 cached) | 7163 (0 reasoning) | 0.0081 | 0.0081 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-0-clippy r1 | 64719 (2304 cached) | 600 (0 reasoning) | 0.0047 | 0.0127 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-0-clippy r0 [Sol fix] | 14821 (0 cached) | 1156 (901 reasoning) | 0.0486 | 0.0486 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-0-clippy r1 [Sol fix] | 7243 (0 cached) | 1808 (98 reasoning) | 0.0362 | 0.0848 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-0-clippy-examples r0 | 3058 (512 cached) | 235 (0 reasoning) | 0.0003 | 0.0130 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-0-clippy: 3 aborted calls (runaway reasoning, killed before usage was reported; reasoning now disabled for build calls) | — | — | ~0.30 (est.) | — |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-1-split-plan a0 | 39485 (0 cached) | 603 (0 reasoning) | 0.0031 | 0.0161 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-1-split-plan a1 | 40568 (39680 cached) | 607 (0 reasoning) | 0.0007 | 0.0168 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-1-split-plan a2 | 41538 (40704 cached) | 613 (0 reasoning) | 0.0007 | 0.0175 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-1-split-plan a0 | 39485 (39168 cached) | 551 (0 reasoning) | 0.0006 | 0.0181 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-1-split-plan a1 | 40201 (39680 cached) | 569 (0 reasoning) | 0.0006 | 0.0187 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-1-split-plan a2 | 40964 (40448 cached) | 579 (0 reasoning) | 0.0006 | 0.0194 |
| 2026-10-20 | (hand glue, no model) | 09-1 split: applied DeepSeek's JSON line-range plan mechanically; renamed child module `layout`→`geometry` (name clash), re-exported 6 free helpers from `tk/mod.rs`, made `NodeSnapshot` fields `pub(crate)`, reattached doc comments to their items | — | — | 0 | — |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-2-golden r0 | 29308 (256 cached) | 4258 (0 reasoning) | 0.0042 | 0.0236 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-2-golden r1 | 31027 (0 cached) | 1688 (0 reasoning) | 0.0030 | 0.0266 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-2-golden r2 [Sol fix] | 5156 (0 cached) | 660 (215 reasoning) | 0.0195 | 0.1043 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-2-golden r3 [Sol fix] | 3539 (0 cached) | 979 (698 reasoning) | 0.0186 | 0.1229 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-3-window-ci r0 | 10365 (256 cached) | 5807 (0 reasoning) | 0.0036 | 0.0302 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-3-window-ci r1 | 12135 (1280 cached) | 1067 (0 reasoning) | 0.0013 | 0.0315 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-3-window-ci r2 [Sol fix] | 6200 (0 cached) | 281 (192 reasoning) | 0.0183 | 0.1412 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-3-window-ci r3 [Sol fix] | 5588 (0 cached) | 221 (0 reasoning) | 0.0162 | 0.1574 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-3b-window-fix r0 | 7606 (256 cached) | 3527 (0 reasoning) | 0.0023 | 0.0338 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-3b-window-fix r1 | 9049 (1536 cached) | 726 (0 reasoning) | 0.0009 | 0.0347 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-4-shortcuts r0 | 20759 (0 cached) | 8539 (0 reasoning) | 0.0057 | 0.0404 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-4-shortcuts r1 | 22594 (0 cached) | 5625 (1 reasoning) | 0.0044 | 0.0448 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-4-shortcuts r2 [Sol fix] | 20394 (0 cached) | 2071 (1963 reasoning) | 0.0717 | 0.2291 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-4-shortcuts r3 [Sol fix] | 20899 (0 cached) | 3855 (1811 reasoning) | 0.0908 | 0.3199 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-4b-clippy r0 | 6832 (768 cached) | 388 (0 reasoning) | 0.0006 | 0.0454 |
| 2026-10-20 | (hand glue, no model) | 09-4 shortcuts: after GPT-6.1 Sol's 2nd fix dropped `ShortcutId::from_raw`, changed one call site to the tuple constructor (`ShortcutId(id)`); Sol stopped on this item per rule, the rest was green | — | — | 0 | — |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-5-edit r0 | 53985 (0 cached) | 60000 (0 reasoning) | 0.0338 | 0.0792 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-5-edit r1 | 57952 (256 cached) | 793 (0 reasoning) | 0.0044 | 0.0836 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-5-edit r2 [Sol fix] | 4884 (0 cached) | 255 (0 reasoning) | 0.0148 | 0.3347 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-5-edit r3 [Sol fix] | 4862 (0 cached) | 935 (812 reasoning) | 0.0215 | 0.3562 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-5b-edit-tests r0 | 38996 (0 cached) | 2996 (0 reasoning) | 0.0042 | 0.0879 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 09-5b-edit-tests r1 | 40231 (35840 cached) | 3008 (0 reasoning) | 0.0021 | 0.0899 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-5b-edit-tests r2 [Sol fix] | 6530 (0 cached) | 1117 (714 reasoning) | 0.0275 | 0.3837 |
| 2026-10-20 | openai/gpt-6.1-sol | 09-5b-edit-tests r3 [Sol fix] | 3768 (0 cached) | 422 (349 reasoning) | 0.0136 | 0.3973 |
| 2026-10-20 | (no model) | 09-5: GPT-6.1 Sol stopped after 2 tries on the first test file (truncated DeepSeek output); DeepSeek rewrote the tests as a new scoped item. The last failure was a stale golden PNG written during an earlier round, so it was re-blessed and checked by eye (no code change) | — | — | 0 | — |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | [BizyBot6, PR #2] 13-1-tokens r0 + r1 self-fix | 64314 | 5989 | 0.0075 | (BizyBot6 DeepSeek total 0.0075; Sol 0.00) |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 10-docs-a r0 | 36574 (0 cached) | 9703 (0 reasoning) | 0.0074 | 0.0974 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | [BizyBot5, PR #4] 0.12 charts, image/SVG, drag-and-drop, tray: all calls (itemized in PR #4) | — | — | 0.0448 | (BizyBot5 DeepSeek total 0.0448; Sol 0.00) |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet smoke | 17 (0 cached) | 4 (0 reasoning) | 0.0001 | 0.0001 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | [BizyBot6, PR #5] 0.13 themes, a11y, IME, HiDPI, damage redraw: 25 calls (itemized in PR #5) | — | — | 0.0943 | (BizyBot6 DeepSeek total 0.1018 incl. PR #2; Sol 0.00) |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 10-docs-b r0 | 18251 (0 cached) | 6535 (0 reasoning) | 0.0045 | 0.1019 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | 10-docs-b r1 | 19604 (0 cached) | 28558 (1 reasoning) | 0.0157 | 0.1176 |
| 2026-10-20 | openai/gpt-6.1-sol | 10-docs-b r2 [Sol fix] | 32683 (0 cached) | 785 (695 reasoning) | 0.0896 | 0.4868 |
| 2026-10-20 | openai/gpt-6.1-sol | 10-docs-b r3 [Sol fix] | 32683 (0 cached) | 895 (459 reasoning) | 0.0907 | 0.5775 |
| 2026-10-20 | (no model) | 10-docs-b: DeepSeek round 1 rewrote src/render.rs with a FILE block even though the file wasn't in its context, and the two Sol rounds couldn't repair it. I reverted render.rs by hand (git checkout); the style.rs docs-only additions were kept. Fixed the driver so it now rejects FILE blocks for existing files outside the context | — | — | 0 | — |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | fix010-visual r0 | 36060 (256 cached) | 6401 (0 reasoning) | 0.0057 | 0.1233 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | fix010-visual r1 | 37663 (0 cached) | 1651 (0 reasoning) | 0.0035 | 0.1267 |
| 2026-10-20 | openai/gpt-6.1-sol | fix010-visual r2 [Sol fix] | 12586 (0 cached) | 1470 (1421 reasoning) | 0.0462 | 0.6237 |
| 2026-10-20 | openai/gpt-6.1-sol | fix010-visual r3 [Sol fix] | 12538 (0 cached) | 1052 (985 reasoning) | 0.0419 | 0.6655 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | fix010-scrollarea r0 | 31625 (3072 cached) | 1022 (0 reasoning) | 0.0025 | 0.1293 |
| 2026-10-20 | openai/gpt-6.1-sol | fix010-scrollarea r1 [Sol fix] | 33056 (0 cached) | 4595 (3624 reasoning) | 0.1286 | 0.7941 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | [BizyBot3, PR #6] 0.10 progress and inputs: 23+ calls (itemized in PR #6) | — | — | 0.174 | (BizyBot3 DeepSeek 0.174) |
| 2026-10-20 | openai/gpt-6.1-sol | [BizyBot3, PR #6] Sol fixes (itemized in PR #6) | — | — | 0.563 | (BizyBot3 Sol fixes 0.563) |
| 2026-10-20 | (no model) | fix010-visual: the driver wrongly refused DeepSeek's round-1 rewrite of its own new test file (overwrite guard bug), so the two Sol rounds lacked context. I applied the refused DeepSeek output from the log and fixed the guard. ScrollArea got one DeepSeek self-fix, then Sol fixed it | — | — | 0 | — |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | [BizyBot4, PR #7] 0.11 table, tree, menus, toasts, modal stack (itemized in PR #7) | — | — | 0.1011 | (BizyBot4 DeepSeek 0.1011; Sol 0.00) |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docs-c r0 | 38532 (0 cached) | 1244 (0 reasoning) | 0.0033 | 0.1326 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery r0 | 89053 (256 cached) | 60000 (0 reasoning) | 0.0362 | 0.1688 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery2 r0 | 23380 (3072 cached) | 7856 (0 reasoning) | 0.0054 | 0.1742 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery2 r1 | 23864 (23040 cached) | 168 (0 reasoning) | 0.0003 | 0.1745 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery-fix r0 | 25666 (2560 cached) | 8738 (0 reasoning) | 0.0060 | 0.1805 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery-fix r1 | 27147 (256 cached) | 474 (0 reasoning) | 0.0021 | 0.1826 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery-fix2 r0 | 37048 (256 cached) | 5799 (0 reasoning) | 0.0055 | 0.1881 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | gallery-fix2 r1 | 37716 (3328 cached) | 110 (0 reasoning) | 0.0025 | 0.1906 |
| 2026-10-20 | openai/gpt-6.1-sol | gallery-fix2 r2 [Sol fix] | 35119 (0 cached) | 1164 (908 reasoning) | 0.0994 | 0.8936 |
| 2026-10-20 | openai/gpt-6.1-sol | gallery-fix2 r3 [Sol fix] | 34986 (0 cached) | 418 (200 reasoning) | 0.0916 | 0.9852 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | guide r0 | 30536 (256 cached) | 5493 (0 reasoning) | 0.0049 | 0.1955 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | guide-doctest r0 | 29074 (256 cached) | 7774 (0 reasoning) | 0.0059 | 0.2014 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | guide-doctest2 r0 | 44323 (256 cached) | 748 (0 reasoning) | 0.0035 | 0.2048 |
| hand glue | lead | docs/GUIDE.md: 2 doctest one-liners (drop nonexistent `Opt::Variable` line; mark Window API block `no_run` since it feeds empty PNG bytes) + `GuideDoctests` include in src/lib.rs | $0 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | readme r0 | 8406 (7680 cached) | 958 (0 reasoning) | 0.0006 | 0.2054 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docfill-fns-0 | 7205 (0 cached) | 1239 (0 reasoning) | 0.0012 | 0.2066 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docfill-fns-1 | 6991 (0 cached) | 1177 (0 reasoning) | 0.0011 | 0.2077 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docfill-fns-2 | 7643 (0 cached) | 1415 (0 reasoning) | 0.0012 | 0.2089 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docfill-fns-3 | 8312 (0 cached) | 1250 (0 reasoning) | 0.0012 | 0.2101 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docfill-fns-4 | 6256 (0 cached) | 1133 (0 reasoning) | 0.0011 | 0.2112 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | docfill-headers | 5853 (0 cached) | 551 (0 reasoning) | 0.0007 | 0.2119 |
| 2026-10-20 07:10 | (cap) | **Tyler 7:05 AM: hard cap $20 total for the remaining rtk 1.0 work, counted from here** (baseline: lead ledger total above + helper rows ≈ $2.48 incl. ~$0.30 estimated aborted calls). Driver enforces the cap (stops at $19 over the baseline, counting lead ledger plus helper rows added after this line). $1.50 balance floor still applies. | — | — | — | — |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:1352-1397 | 2913 (0 cached) | 754 (269 reasoning) | 0.0134 | 0.0134 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:1002-1351 | 10100 (0 cached) | 3491 (2467 reasoning) | 0.0551 | 0.0685 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:652-1001 | 9973 (0 cached) | 2897 (1600 reasoning) | 0.0489 | 0.1175 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:302-651 | 9259 (0 cached) | 7698 (5179 reasoning) | 0.0955 | 0.2130 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:1-301 | 7632 (0 cached) | 2310 (1704 reasoning) | 0.0384 | 0.2513 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:1003-1352 | 10117 (0 cached) | 2953 (2131 reasoning) | 0.0498 | 0.3011 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render.rs:303-652 | 9295 (0 cached) | 4506 (3431 reasoning) | 0.0636 | 0.3647 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/layout.rs:694-807 | 4401 (0 cached) | 999 (670 reasoning) | 0.0188 | 0.3835 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/layout.rs:351-693 | 7384 (0 cached) | 3074 (2517 reasoning) | 0.0455 | 0.4290 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/layout.rs:1-350 | 7214 (0 cached) | 2659 (1271 reasoning) | 0.0410 | 0.4701 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:1640-1766 | 4811 (0 cached) | 1473 (818 reasoning) | 0.0244 | 0.4944 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:1300-1639 | 7953 (0 cached) | 6280 (3290 reasoning) | 0.0787 | 0.5731 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:956-1299 | 8204 (0 cached) | 2862 (1614 reasoning) | 0.0450 | 0.6181 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:781-955 | 5111 (0 cached) | 367 (355 reasoning) | 0.0139 | 0.6320 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:606-780 | 5319 (0 cached) | 870 (448 reasoning) | 0.0193 | 0.6514 |
| 2026-10-20 07:50 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:1655 accidental restart, killed within seconds of sending (possible tiny charge, not in ledger) | — | — | ~0.00 | — |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:339-605 | 6751 (0 cached) | 2520 (1208 reasoning) | 0.0387 | 0.6901 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/style.rs:1-338 | 7809 (0 cached) | 700 (688 reasoning) | 0.0226 | 0.7127 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render/data.rs:533-670 | 4330 (0 cached) | 2272 (1536 reasoning) | 0.0314 | 0.7441 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render/data.rs:287-532 | 6710 (0 cached) | 1766 (1012 reasoning) | 0.0311 | 0.7752 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render/data.rs:1-286 | 7447 (0 cached) | 1831 (1153 reasoning) | 0.0332 | 0.8084 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render/inputs.rs:338-567 | 6485 (0 cached) | 4315 (2526 reasoning) | 0.0561 | 0.8645 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/render/inputs.rs:1-337 | 8202 (0 cached) | 6464 (3789 reasoning) | 0.0810 | 0.9455 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-style-1300 r0 | 23766 (0 cached) | 433 (0 reasoning) | 0.0019 | 0.2138 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-data r0 | 12207 (0 cached) | 627 (0 reasoning) | 0.0012 | 0.2149 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-data r1 | 12782 (0 cached) | 1398 (0 reasoning) | 0.0016 | 0.2165 |
| 2026-10-20 | openai/gpt-6.1-sol | recover-data r2 [Sol fix] | 13992 (0 cached) | 1347 (1262 reasoning) | 0.0484 | 1.0336 |
| 2026-10-20 | openai/gpt-6.1-sol | recover-data r3 [Sol fix] | 14029 (0 cached) | 1975 (1873 reasoning) | 0.0548 | 1.0885 |
| hand glue | lead | Sonnet recovery of src/render/data.rs: removed dead TreeSnapshot.top field + initializer (clippy dead_code); re-blessed m011_menu 2x goldens after visual check (old goldens had a clipped 2x menu) | $0 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/app.rs:651-925 | 7107 (0 cached) | 3110 (2281 reasoning) | 0.0453 | 0.9908 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/app.rs:308-650 | 8580 (0 cached) | 10883 (8430 reasoning) | 0.1260 | 1.1168 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/app.rs:1-307 | 7251 (0 cached) | 3932 (2722 reasoning) | 0.0538 | 1.1706 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/widgets.rs:675-888 | 5301 (0 cached) | 929 (446 reasoning) | 0.0199 | 1.1905 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/widgets.rs:340-674 | 7034 (0 cached) | 1726 (831 reasoning) | 0.0313 | 1.2219 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/widgets.rs:1-339 | 8005 (0 cached) | 1879 (1035 reasoning) | 0.0348 | 1.2567 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/events.rs:523-817 | 6909 (0 cached) | 2748 (2068 reasoning) | 0.0413 | 1.2980 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/events.rs:188-522 | 7956 (0 cached) | 3021 (2175 reasoning) | 0.0461 | 1.3441 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/events.rs:1-187 | 5559 (0 cached) | 2101 (1545 reasoning) | 0.0321 | 1.3762 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/menu.rs:693-698 | 2178 (0 cached) | 347 (140 reasoning) | 0.0078 | 1.3840 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/menu.rs:345-692 | 7648 (0 cached) | 5165 (3672 reasoning) | 0.0669 | 1.4510 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/menu.rs:1-344 | 7807 (0 cached) | 2656 (2066 reasoning) | 0.0422 | 1.4932 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/treeview.rs:350-595 | 5899 (0 cached) | 4035 (2889 reasoning) | 0.0521 | 1.5453 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/treeview.rs:1-349 | 7753 (0 cached) | 2402 (1683 reasoning) | 0.0395 | 1.5848 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/mod.rs:306-577 | 6488 (0 cached) | 3479 (1327 reasoning) | 0.0478 | 1.6326 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/mod.rs:1-305 | 7858 (0 cached) | 1610 (975 reasoning) | 0.0318 | 1.6644 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/theme.rs:283-507 | 5396 (0 cached) | 1393 (1010 reasoning) | 0.0247 | 1.6891 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/theme.rs:1-282 | 6266 (0 cached) | 595 (583 reasoning) | 0.0185 | 1.7076 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/chart.rs:269-494 | 5774 (0 cached) | 2201 (1630 reasoning) | 0.0336 | 1.7412 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/chart.rs:1-268 | 6297 (0 cached) | 4393 (2910 reasoning) | 0.0565 | 1.7977 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/geometry.rs:325-483 | 5058 (0 cached) | 2502 (1808 reasoning) | 0.0351 | 1.8328 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/geometry.rs:1-324 | 8543 (0 cached) | 2361 (1875 reasoning) | 0.0407 | 1.8735 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/table.rs:339-445 | 3872 (0 cached) | 2213 (1308 reasoning) | 0.0299 | 1.9034 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/table.rs:1-338 | 7557 (0 cached) | 2632 (1759 reasoning) | 0.0414 | 1.9448 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/style.rs:314-432 | 4130 (0 cached) | 1010 (598 reasoning) | 0.0184 | 1.9632 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/style.rs:1-313 | 7699 (0 cached) | 3262 (2036 reasoning) | 0.0480 | 2.0112 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/icons.rs:351-417 | 3731 (0 cached) | 1470 (586 reasoning) | 0.0222 | 2.0334 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/icons.rs:1-350 | 10367 (0 cached) | 1819 (989 reasoning) | 0.0389 | 2.0723 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/toast.rs:1-337 | 7828 (0 cached) | 3809 (2723 reasoning) | 0.0537 | 2.1261 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/scroll.rs:1-333 | 9555 (0 cached) | 4996 (2916 reasoning) | 0.0691 | 2.1951 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/timer.rs:1-318 | 6996 (0 cached) | 1964 (1692 reasoning) | 0.0336 | 2.2288 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tray.rs:1-317 | 7323 (0 cached) | 3331 (1783 reasoning) | 0.0480 | 2.2767 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/edit.rs:1-310 | 7268 (0 cached) | 3921 (2203 reasoning) | 0.0537 | 2.3305 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/a11y.rs:1-297 | 6835 (0 cached) | 2511 (1790 reasoning) | 0.0388 | 2.3692 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/tree.rs:1-296 | 6721 (0 cached) | 2489 (1943 reasoning) | 0.0383 | 2.4076 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/double_spinbox.rs:1-287 | 7609 (0 cached) | 4194 (2641 reasoning) | 0.0572 | 2.4647 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/shortcut.rs:1-277 | 6054 (0 cached) | 1810 (1288 reasoning) | 0.0302 | 2.4949 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/text.rs:1-275 | 6442 (0 cached) | 4262 (2418 reasoning) | 0.0555 | 2.5504 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/canvas.rs:1-255 | 7834 (0 cached) | 3109 (1988 reasoning) | 0.0468 | 2.5972 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/damage.rs:1-251 | 6456 (0 cached) | 3121 (1918 reasoning) | 0.0441 | 2.6413 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/scroll_area.rs:1-240 | 6563 (0 cached) | 2177 (1637 reasoning) | 0.0349 | 2.6762 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/testing.rs:1-232 | 5900 (0 cached) | 998 (593 reasoning) | 0.0218 | 2.6980 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/slider.rs:1-228 | 6421 (0 cached) | 4395 (2785 reasoning) | 0.0568 | 2.7548 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/colormap.rs:1-218 | 5660 (0 cached) | 1337 (811 reasoning) | 0.0247 | 2.7795 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/a11y_accesskit.rs:1-207 | 5793 (0 cached) | 1935 (879 reasoning) | 0.0309 | 2.8104 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/progress.rs:1-203 | 5730 (0 cached) | 2206 (1597 reasoning) | 0.0335 | 2.8439 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/dnd.rs:1-202 | 5471 (0 cached) | 5404 (2582 reasoning) | 0.0650 | 2.9089 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/window.rs:1-200 | 5242 (0 cached) | 1739 (1034 reasoning) | 0.0279 | 2.9368 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/var.rs:1-199 | 5078 (0 cached) | 2623 (1972 reasoning) | 0.0364 | 2.9732 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/fonts.rs:1-187 | 5939 (0 cached) | 958 (676 reasoning) | 0.0215 | 2.9946 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/image.rs:1-181 | 5200 (0 cached) | 1452 (800 reasoning) | 0.0249 | 3.0196 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/entry.rs:1-162 | 4454 (0 cached) | 1677 (1121 reasoning) | 0.0257 | 3.0452 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | checkbox-tokens r0 | 22020 (768 cached) | 1981 (0 reasoning) | 0.0025 | 0.2190 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | checkbox-border r0 | 21005 (2816 cached) | 156 (0 reasoning) | 0.0014 | 0.2204 |
| 2026-10-20 09:07 | deepseek/deepseek-v4.1-flash-1017 | [BizyBot3, PR #8] tri-state Checkbutton, 3 calls (itemized in PR #8) | — | — | 0.0125 | (BizyBot3 PR #8 DeepSeek 0.0125; Sol 0.00) |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/chart.rs:1-155 | 4610 (0 cached) | 1042 (633 reasoning) | 0.0196 | 3.0649 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/ime.rs:1-151 | 4615 (0 cached) | 2818 (1646 reasoning) | 0.0374 | 3.1023 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/image.rs:1-147 | 4496 (0 cached) | 1733 (1069 reasoning) | 0.0263 | 3.1286 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/shortcuts.rs:1-146 | 4460 (0 cached) | 2044 (1254 reasoning) | 0.0294 | 3.1580 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/ring.rs:1-129 | 4473 (0 cached) | 1949 (937 reasoning) | 0.0284 | 3.1864 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/listbox.rs:1-128 | 3992 (0 cached) | 1394 (982 reasoning) | 0.0219 | 3.2083 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/theme.rs:1-126 | 4194 (0 cached) | 1230 (964 reasoning) | 0.0207 | 3.2290 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/switch.rs:1-106 | 3869 (0 cached) | 1118 (805 reasoning) | 0.0189 | 3.2479 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/a11y.rs:1-97 | 3352 (0 cached) | 277 (0 reasoning) | 0.0095 | 3.2574 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/notebook.rs:1-92 | 3597 (0 cached) | 1404 (963 reasoning) | 0.0212 | 3.2786 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/inputs_state.rs:1-116 | 4220 (0 cached) | 670 (386 reasoning) | 0.0151 | 3.2938 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/lib.rs:1-88 | 3659 (0 cached) | 771 (341 reasoning) | 0.0150 | 3.3088 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/clipboard.rs:1-69 | 3076 (0 cached) | 1002 (688 reasoning) | 0.0162 | 3.3250 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/dnd.rs:1-43 | 2839 (0 cached) | 200 (0 reasoning) | 0.0077 | 3.3327 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/tooltip.rs:1-37 | 2559 (0 cached) | 1440 (671 reasoning) | 0.0195 | 3.3522 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-widgets r0 | 21291 (640 cached) | 118 (0 reasoning) | 0.0017 | 0.2221 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-dnd r0 | 7699 (3072 cached) | 520 (0 reasoning) | 0.0007 | 0.2228 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-damage r0 | 7167 (3072 cached) | 302 (0 reasoning) | 0.0005 | 0.2233 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-damage r1 | 7310 (7040 cached) | 299 (0 reasoning) | 0.0002 | 0.2235 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/tristate.rs:1-172 | 4934 (0 cached) | 1321 (916 reasoning) | 0.0231 | 3.3753 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | dialog-anchor r0 | 22286 (0 cached) | 1579 (0 reasoning) | 0.0026 | 0.2261 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | dialog-anchor-fix r0 | 23581 (3200 cached) | 2194 (0 reasoning) | 0.0028 | 0.2288 |
| 2026-10-20 | openai/gpt-6.1-sol | dialog-anchor-fix r1 [Sol fix] | 23597 (0 cached) | 91 (0 reasoning) | 0.0599 | 1.1484 |
| 2026-10-20 | openai/gpt-6.1-sol | dialog-anchor-fix r2 [Sol fix] | 25194 (0 cached) | 1664 (1552 reasoning) | 0.0796 | 1.2280 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | paint-order r0 | 20044 (3072 cached) | 1820 (0 reasoning) | 0.0023 | 0.2311 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | paint-order-fix r0 | 21174 (7808 cached) | 347 (0 reasoning) | 0.0013 | 0.2324 |
| 2026-10-20 | openai/gpt-6.1-sol | paint-order-fix r1 [Sol fix] | 24871 (0 cached) | 498 (167 reasoning) | 0.0672 | 1.2951 |
| 2026-10-20 | openai/gpt-6.1-sol | paint-order-fix r2 [Sol fix] | 22467 (0 cached) | 1451 (1383 reasoning) | 0.0707 | 1.3658 |
| hand glue | lead | Dialog anchor + paint-order fix: reverted a Sol edit to tests/golden_m011.rs (it tried to dodge the expected golden change); tests/paint_order.rs scene: added `sim_resize`, place `width/height: None` instead of `Some(0.0)`; marked the hover hit-test `#[ignore]` as an open decision (hit-testing skips non-interactive frames). Re-blessed m011_modal_* after a visual check. | $0 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-regressions r0 | 25874 (0 cached) | 32000 (32000 reasoning) | 0.3717 | 3.7470 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-regressions-a r0 | 25727 (0 cached) | 5618 (3172 reasoning) | 0.1076 | 3.8546 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-regressions-b r0 | 25849 (0 cached) | 7747 (4278 reasoning) | 0.1292 | 3.9838 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | regressions-b-fix r0 | 16569 (0 cached) | 193 (0 reasoning) | 0.0014 | 0.2338 |
| 2026-10-20 | openai/gpt-6.1-sol | regressions-b-fix r1 [Sol fix] | 22744 (0 cached) | 93 (0 reasoning) | 0.0578 | 1.4236 |
| 2026-10-20 | openai/gpt-6.1-sol | regressions-b-fix r2 [Sol fix] | 23772 (0 cached) | 981 (684 reasoning) | 0.0692 | 1.4929 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | f6-ask-directory r0 | 24415 (0 cached) | 1067 (0 reasoning) | 0.0025 | 0.2363 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | f6-ask-directory r1 | 25057 (256 cached) | 165 (0 reasoning) | 0.0018 | 0.2381 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | listbox-activate r0 | 42477 (0 cached) | 5266 (0 reasoning) | 0.0056 | 0.2438 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | listbox-activate r1 | 43562 (11008 cached) | 81 (0 reasoning) | 0.0024 | 0.2462 |
| 2026-10-20 | openai/gpt-6.1-sol | listbox-activate r2 [Sol fix] | 41654 (0 cached) | 445 (164 reasoning) | 0.1086 | 1.6014 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/dialog.rs:646-894 | 6272 (0 cached) | 4204 (2637 reasoning) | 0.0546 | 4.0384 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/dialog.rs:345-645 | 7135 (0 cached) | 3035 (1852 reasoning) | 0.0446 | 4.0830 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/dialog.rs:1-344 | 7535 (0 cached) | 2869 (1609 reasoning) | 0.0438 | 4.1268 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet src/tk/combobox.rs:1-30 | 2514 (0 cached) | 1185 (677 reasoning) | 0.0169 | 4.1436 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/bench.rs:1-166 | 5723 (0 cached) | 1447 (1026 reasoning) | 0.0259 | 4.1696 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/bench_table.rs:1-173 | 4972 (0 cached) | 2733 (1487 reasoning) | 0.0373 | 4.2068 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/counter.rs:1-27 | 2556 (0 cached) | 705 (429 reasoning) | 0.0122 | 4.2190 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/demo.rs:1-139 | 4778 (0 cached) | 1222 (746 reasoning) | 0.0218 | 4.2408 |
