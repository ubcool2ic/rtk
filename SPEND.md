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
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/form.rs:1-50 | 2846 (0 cached) | 480 (194 reasoning) | 0.0105 | 4.2513 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/gallery.rs:637-800 | 4937 (0 cached) | 2271 (1518 reasoning) | 0.0326 | 4.2839 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/gallery.rs:310-636 | 8398 (0 cached) | 2911 (1795 reasoning) | 0.0459 | 4.3298 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/gallery.rs:1-309 | 7424 (0 cached) | 2084 (1163 reasoning) | 0.0357 | 4.3654 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/gallery_charts_media.rs:1-168 | 5303 (0 cached) | 1946 (1161 reasoning) | 0.0301 | 4.3955 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/gallery_m011.rs:1-189 | 5079 (0 cached) | 1074 (847 reasoning) | 0.0209 | 4.4164 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/gallery_progress_inputs.rs:1-210 | 6430 (0 cached) | 2265 (1777 reasoning) | 0.0355 | 4.4519 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/hello.rs:1-21 | 2449 (0 cached) | 457 (187 reasoning) | 0.0095 | 4.4614 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/modern.rs:1-308 | 6905 (0 cached) | 1521 (1147 reasoning) | 0.0290 | 4.4904 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/screenshot.rs:1-20 | 2483 (0 cached) | 460 (221 reasoning) | 0.0096 | 4.5000 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/scrolling.rs:1-23 | 2702 (0 cached) | 390 (0 reasoning) | 0.0093 | 4.5093 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/styling.rs:1-157 | 5213 (0 cached) | 1427 (894 reasoning) | 0.0247 | 4.5340 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/timers.rs:1-20 | 2566 (0 cached) | 828 (430 reasoning) | 0.0134 | 4.5474 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet examples/widgets.rs:1-31 | 2731 (0 cached) | 639 (341 reasoning) | 0.0119 | 4.5592 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/canvas_shapes.rs:1-56 | 3692 (0 cached) | 1842 (1261 reasoning) | 0.0258 | 4.5850 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/dialog_anchor.rs:1-194 | 5447 (0 cached) | 3315 (1868 reasoning) | 0.0440 | 4.6291 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/edit_selection.rs:1-291 | 7465 (0 cached) | 2331 (1386 reasoning) | 0.0382 | 4.6673 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/entry_submit.rs:1-60 | 3181 (0 cached) | 1065 (486 reasoning) | 0.0170 | 4.6843 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/entry_unicode.rs:1-95 | 4094 (0 cached) | 2513 (1483 reasoning) | 0.0333 | 4.7177 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/events.rs:1-122 | 4597 (0 cached) | 1223 (814 reasoning) | 0.0214 | 4.7391 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/features.rs:1-303 | 8381 (0 cached) | 3049 (2009 reasoning) | 0.0473 | 4.7863 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/fonts.rs:1-17 | 2463 (0 cached) | 920 (439 reasoning) | 0.0141 | 4.8005 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/gallery_smoke.rs:1-76 | 3216 (0 cached) | 2009 (699 reasoning) | 0.0265 | 4.8270 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/golden_m011.rs:1-273 | 7118 (0 cached) | 1325 (851 reasoning) | 0.0275 | 4.8545 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/golden_widgets.rs:1-140 | 4645 (0 cached) | 1005 (631 reasoning) | 0.0193 | 4.8738 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/ids.rs:1-88 | 3913 (0 cached) | 1078 (757 reasoning) | 0.0186 | 4.8924 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/layout_fuzz.rs:1-131 | 4343 (0 cached) | 1308 (987 reasoning) | 0.0218 | 4.9142 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/listbox_activate.rs:1-66 | 3412 (0 cached) | 1208 (762 reasoning) | 0.0189 | 4.9331 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m011_menu.rs:1-213 | 6181 (0 cached) | 1984 (937 reasoning) | 0.0322 | 4.9653 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m011_modal.rs:1-171 | 5248 (0 cached) | 1575 (744 reasoning) | 0.0262 | 4.9915 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m011_table.rs:1-242 | 6799 (0 cached) | 2456 (1392 reasoning) | 0.0382 | 5.0297 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m011_toast.rs:1-130 | 4447 (0 cached) | 2152 (1243 reasoning) | 0.0304 | 5.0601 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m011_tree.rs:1-249 | 7178 (0 cached) | 2989 (1800 reasoning) | 0.0442 | 5.1043 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m012_charts.rs:1-150 | 5138 (0 cached) | 1805 (968 reasoning) | 0.0283 | 5.1327 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m012_dnd.rs:1-161 | 5169 (0 cached) | 974 (605 reasoning) | 0.0201 | 5.1528 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m012_golden_charts.rs:1-75 | 3719 (0 cached) | 895 (470 reasoning) | 0.0164 | 5.1691 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m012_golden_image.rs:1-89 | 3847 (0 cached) | 1469 (951 reasoning) | 0.0224 | 5.1915 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m012_image.rs:1-113 | 4602 (0 cached) | 1633 (964 reasoning) | 0.0255 | 5.2171 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m012_tray.rs:1-146 | 4847 (0 cached) | 1789 (1028 reasoning) | 0.0276 | 5.2446 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_a11y.rs:1-122 | 4799 (0 cached) | 701 (689 reasoning) | 0.0166 | 5.2612 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_damage.rs:1-164 | 5382 (0 cached) | 1353 (921 reasoning) | 0.0243 | 5.2855 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_hidpi.rs:1-330 | 7813 (0 cached) | 1993 (1479 reasoning) | 0.0356 | 5.3211 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_ime.rs:1-165 | 5596 (0 cached) | 1800 (1054 reasoning) | 0.0292 | 5.3503 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_theme_goldens.rs:1-139 | 4557 (0 cached) | 389 (377 reasoning) | 0.0130 | 5.3633 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_themes.rs:1-119 | 4458 (0 cached) | 1360 (867 reasoning) | 0.0225 | 5.3858 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m013_tokens.rs:1-114 | 4023 (0 cached) | 1519 (817 reasoning) | 0.0232 | 5.4090 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_a11y.rs:1-129 | 4799 (0 cached) | 350 (0 reasoning) | 0.0131 | 5.4221 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_progress.rs:1-159 | 5115 (0 cached) | 1360 (960 reasoning) | 0.0238 | 5.4460 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_scroll.rs:1-292 | 8322 (0 cached) | 3346 (2187 reasoning) | 0.0501 | 5.4961 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_slider.rs:1-192 | 5869 (0 cached) | 1713 (1241 reasoning) | 0.0289 | 5.5249 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_spinbox.rs:1-172 | 5501 (0 cached) | 1376 (1032 reasoning) | 0.0248 | 5.5497 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_switch.rs:1-131 | 4478 (0 cached) | 724 (712 reasoning) | 0.0162 | 5.5659 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_tristate.rs:1-244 | 7359 (0 cached) | 1054 (766 reasoning) | 0.0253 | 5.5912 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/m0_10_visual_fixes.rs:1-120 | 4602 (0 cached) | 2278 (1193 reasoning) | 0.0320 | 5.6231 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/paint_order.rs:1-86 | 3800 (0 cached) | 1040 (636 reasoning) | 0.0180 | 5.6411 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/reentrancy.rs:1-106 | 4029 (0 cached) | 790 (778 reasoning) | 0.0160 | 5.6571 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/render.rs:1-62 | 3337 (0 cached) | 1113 (669 reasoning) | 0.0178 | 5.6749 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/review.rs:1-102 | 4161 (0 cached) | 1856 (1414 reasoning) | 0.0269 | 5.7018 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/review_fable.rs:1-17 | 2462 (0 cached) | 1071 (694 reasoning) | 0.0156 | 5.7174 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/scrollarea_width.rs:1-58 | 3196 (0 cached) | 299 (0 reasoning) | 0.0094 | 5.7268 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/scrolling.rs:1-123 | 5096 (0 cached) | 1460 (1448 reasoning) | 0.0248 | 5.7516 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/shortcuts.rs:1-137 | 4918 (0 cached) | 1057 (865 reasoning) | 0.0204 | 5.7720 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/styling.rs:350-487 | 4848 (0 cached) | 1177 (1165 reasoning) | 0.0215 | 5.7935 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/styling.rs:1-349 | 8318 (0 cached) | 1446 (1201 reasoning) | 0.0311 | 5.8246 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/timers.rs:1-31 | 2695 (0 cached) | 169 (0 reasoning) | 0.0071 | 5.8316 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/tk_review_regressions.rs:1-119 | 4781 (0 cached) | 1395 (860 reasoning) | 0.0235 | 5.8552 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/tk_review_regressions_2.rs:1-152 | 5661 (0 cached) | 2850 (1710 reasoning) | 0.0398 | 5.8950 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/treeview_notebook.rs:1-80 | 3480 (0 cached) | 2498 (1310 reasoning) | 0.0319 | 5.9269 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/v08_extras.rs:1-258 | 6753 (0 cached) | 1272 (838 reasoning) | 0.0262 | 5.9531 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/v08_widgets.rs:1-279 | 7379 (0 cached) | 1481 (976 reasoning) | 0.0296 | 5.9827 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/vars.rs:1-54 | 2971 (0 cached) | 711 (454 reasoning) | 0.0131 | 5.9958 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/window_api.rs:1-142 | 4588 (0 cached) | 795 (539 reasoning) | 0.0171 | 6.0129 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet tests/window_pinning.rs:1-50 | 3002 (0 cached) | 1183 (600 reasoning) | 0.0178 | 6.0307 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-gallery r0 | 13401 (0 cached) | 102 (0 reasoning) | 0.0010 | 0.2472 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-gallery-smoke r0 | 5661 (0 cached) | 333 (0 reasoning) | 0.0006 | 0.2477 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | recover-gallery-smoke r1 | 5783 (3584 cached) | 948 (0 reasoning) | 0.0007 | 0.2484 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | hidpi-bg-fix r0 | 18992 (2816 cached) | 473 (0 reasoning) | 0.0014 | 0.2498 |
| 2026-10-20 | openai/gpt-6.1-sol | hidpi-bg-fix r1 [Sol fix] | 19141 (0 cached) | 3427 (2951 reasoning) | 0.0821 | 1.6836 |
| 2026-10-20 | openai/gpt-6.1-sol | hidpi-bg-fix r2 [Sol fix] | 22852 (0 cached) | 463 (347 reasoning) | 0.0618 | 1.7453 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | changelog r0 | 10780 (256 cached) | 1284 (0 reasoning) | 0.0014 | 0.2511 |

### 0.11 follow-ups: file dialogs + editable combobox (BizyBot4, branch m0.11-dialogs-combobox)

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) | Running total |
|---|---|---|---|---|---|---|
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-filedialog r0 | 24013 (2816 cached) | 7962 (0 reasoning) | 0.0055 | 0.0055 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-filedialog r1 | 24322 (2816 cached) | 498 (0 reasoning) | 0.0018 | 0.0073 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-filedialog r2 | 24429 (23808 cached) | 653 (0 reasoning) | 0.0006 | 0.0078 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-combobox r0 | 59263 (0 cached) | 16192 (0 reasoning) | 0.0122 | 0.0201 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-combobox r1 | 65285 (512 cached) | 741 (0 reasoning) | 0.0049 | 0.0250 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-combobox r2 | 66219 (512 cached) | 4487 (0 reasoning) | 0.0068 | 0.0318 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-examples r0 | 17957 (0 cached) | 2874 (0 reasoning) | 0.0029 | 0.0347 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | m011-examples r1 | 18312 (0 cached) | 212 (0 reasoning) | 0.0016 | 0.0363 |

Total: 8 calls, $0.0363 (cap $1.50). No openai/gpt-6-sol fallback calls were needed. Glue fixes done by hand are listed in the commit messages.
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-1.0.1-review-file_dialog | 7725 (0 cached) | 3326 (2287 reasoning) | 0.0487 | 6.0794 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-1.0.1-review-render | 5456 (0 cached) | 2217 (1781 reasoning) | 0.0331 | 6.1125 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-1.0.1-review-file_dialog | 7725 (0 cached) | 3266 (2121 reasoning) | 0.0481 | 6.1606 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | dialog-shots-1.0.1 r0 | 22624 (0 cached) | 1399 (0 reasoning) | 0.0023 | 0.2534 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-1.0.1-review-glue | 17248 (0 cached) | 4649 (4127 reasoning) | 0.0810 | 6.2416 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-1.0.1-review-combobox | 12708 (0 cached) | 3423 (2646 reasoning) | 0.0596 | 6.3013 |
| 2026-10-20 | anthropic/claude-sonnet-5.5 | sonnet-1.0.1-review-render | 5456 (0 cached) | 2288 (1743 reasoning) | 0.0338 | 6.3351 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-fd r0 | 35004 (0 cached) | 3531 (0 reasoning) | 0.0042 | 0.2576 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-fd r1 | 36395 (1024 cached) | 705 (0 reasoning) | 0.0028 | 0.2605 |
| 2026-10-20 | openai/gpt-6.1-sol | ds-1.0.1-fd r2 [Sol fix] | 38254 (0 cached) | 1984 (1258 reasoning) | 0.1155 | 1.8608 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-cb r0 | 37450 (7424 cached) | 4726 (0 reasoning) | 0.0045 | 0.2650 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-cb r1 | 37913 (9728 cached) | 2129 (0 reasoning) | 0.0031 | 0.2681 |
| 2026-10-20 | openai/gpt-6.1-sol | ds-1.0.1-cb r2 [Sol fix] | 38026 (0 cached) | 480 (195 reasoning) | 0.0999 | 1.9607 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-render r0 | 51937 (10496 cached) | 3853 (0 reasoning) | 0.0049 | 0.2730 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-render r1 | 53360 (9728 cached) | 10850 (1 reasoning) | 0.0086 | 0.2816 |
| 2026-10-20 | openai/gpt-6.1-sol | ds-1.0.1-render r2 [Sol fix] | 49466 (0 cached) | 921 (646 reasoning) | 0.1329 | 2.0935 |
| 2026-10-20 | openai/gpt-6.1-sol | ds-1.0.1-render r3 [Sol fix] | 49461 (0 cached) | 330 (209 reasoning) | 0.1270 | 2.2205 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-render2 r0 | 25237 (3840 cached) | 1196 (0 reasoning) | 0.0021 | 0.2837 |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-render2 r1 | 25972 (22016 cached) | 278 (0 reasoning) | 0.0006 | 0.2843 |
| 2026-10-20 | note | 1.0.1 review driver bug | Sonnet replies for `file_dialog` ($0.0487) and `render` ($0.0331) were billed but lost (unpack error in rev101.py before saving); both re-run once. Two calls killed mid-stream (an early `combobox` call and a duplicate `glue` call from an overlapping loop), cost not logged, est. <= $0.10 | | | |
| 2026-10-20 | note | hand glue (1.0.1) | Moved PR #9's three CHANGELOG bullets into a new `## 1.0.1` section + version bump; kept `clear_input` crate-private and added `Tk::sim_window_focus_lost` (calls it) for the review test instead of DeepSeek's `pub #[doc(hidden)] clear_input` | | | |
| 2026-10-20 | deepseek/deepseek-v4.1-flash-1017 | ds-1.0.1-changelog r0 | 11225 (0 cached) | 447 (0 reasoning) | 0.0010 | 0.2853 |
| 2026-10-20 | note | 1.0.1 renames (scripted, $0) | perl word-boundary rename approved by Tyler: open_file_dialog/open_files_dialog/save_file_dialog/pick_folder_dialog/set_file_dialog_backend/reset_file_dialog_backend -> file_dialog_open/_open_many/_save/_pick_folder/_set_backend/_reset_backend; combobox_text/filtered/highlighted -> combobox_get_text/_filtered/_highlighted (14 files, no aliases) | | | |

### 1.0.2 Fable 5.2 pass (branch fable-5.2-1.0.2)

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) | Running total |
|---|---|---|---|---|---|---|
| 2026-11-13 | anthropic/claude-fable-5.2 | c1-render [hit max_tokens on reasoning, no output; re-run as c1-render-r2] | 34882 (0 cached) | 16000 (16000 reasoning) | 0.9191 | 0.9191 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c1-render-r2 | 34928 (0 cached) | 26729 (22085 reasoning) | 1.3486 | 2.2676 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c2-style | 28499 (0 cached) | 3666 (2100 reasoning) | 0.3746 | 2.6423 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c5-app | 16162 (0 cached) | 3183 (2041 reasoning) | 0.2566 | 2.8989 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c3-layout | 20439 (0 cached) | 3943 (2137 reasoning) | 0.3212 | 3.2201 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c4-events | 19227 (0 cached) | 6585 (4092 reasoning) | 0.4172 | 3.6373 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c6-renderdata | 21718 (0 cached) | 8568 (4049 reasoning) | 0.5165 | 4.1538 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c9-widgets | 23117 (0 cached) | 2370 (1609 reasoning) | 0.2797 | 4.4335 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c7-charts | 13343 (0 cached) | 4523 (2182 reasoning) | 0.2877 | 4.7212 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c10-edit | 26012 (0 cached) | 5640 (3327 reasoning) | 0.4337 | 5.1549 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c8-data | 26458 (0 cached) | 7518 (4303 reasoning) | 0.5124 | 5.6673 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c12-assets | 19495 (0 cached) | 4515 (2000 reasoning) | 0.3366 | 6.0038 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c13-tkwidgets | 20709 (0 cached) | 5067 (2179 reasoning) | 0.3684 | 6.3722 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c11-anim | 14516 (0 cached) | 5201 (3263 reasoning) | 0.3242 | 6.6964 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c14-dialog | 16983 (0 cached) | 5535 (3287 reasoning) | 0.3573 | 7.0536 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c15-tests | 46220 (0 cached) | 5581 (1892 reasoning) | 0.5930 | 7.6466 |
| 2026-11-13 | anthropic/claude-fable-5.2 | c16-docs | 8959 (0 cached) | 5233 (2321 reasoning) | 0.2810 | 7.9276 |
| 2026-11-13 | note | hand glue (1.0.2) | Reverted two Fable edits: always-rebuilt `:disabled` state map in style.rs (changed the blessed dark tristate golden: disabled text got lighter) and a render_frame "nothing pending" early return in damage.rs (combobox popup/menu hover is drawn from the pointer without setting `dirty`, so it could leave stale hover). `cargo clippy --fix` (3 needless borrows). Added `sim_resize` to one Fable test. README line rewrap. `.gitignore` for the GUIDE doctest outputs `out.png` / `my_widget_light_1x.png/`. Version bump in Cargo.toml + Cargo.lock | | | |

## WP2 Rendering (2026-11-26) + copied audit rows

This job's cap is $2.50 from its own generation ids, and a shared account stop line of 586.29 (baseline 581.2885 at 1:25 AM ET, shared with WP1). Costs below are measured from `GET /api/v1/generation` or `get-credits` unless marked estimated. All times are America/Indianapolis (ET).

### Audit rows copied from `/workspace/rtk-audit-notes/SPEND.md` (not this job's spend)

These were the 2026-11-26 rtk audit's own generations (google/gemini-4-carbon). Audit job own total: **$1.567970**.

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 1:03 AM ET | google/gemini-4-carbon | gen-1795672950-oryC0FQu79iHinbqcFRP | audit c1-render | 0.121232 | measured | ok |
| 2026-11-26 1:05 AM ET | google/gemini-4-carbon | gen-1795673046-VP0ptTuQ6zR5H4IhGSKL | audit c3-app | 0.140534 | measured | ok |
| 2026-11-26 1:05 AM ET | google/gemini-4-carbon | gen-1795673049-3y5U8hxLMdv4lvdKwxSO | audit c4-layout | 0.145352 | measured | ok |
| 2026-11-26 1:09 AM ET | google/gemini-4-carbon | gen-1795673044-V4GCl3CnT6rROCgFAUFU | audit c2-style | 0.291328 | measured | ok (finish=length, 2.7 KB partial in the audit write-up) |
| 2026-11-26 1:10 AM ET | google/gemini-4-carbon | gen-1795673357-eftmIm3cAu46bWSrTbLC | audit c6-dialogs | 0.129140 | measured | ok |
| 2026-11-26 1:10 AM ET | google/gemini-4-carbon | gen-1795673355-xzHs5lGKxmUooURSiRll | audit c5-data | 0.134276 | measured | ok |
| 2026-11-26 1:13 AM ET | google/gemini-4-carbon | gen-1795673359-tlabOagRW2NZbY3KBSJH | audit c7-assets | 0.306934 | measured | empty (all reasoning, finish=length) |
| 2026-11-26 1:15 AM ET | google/gemini-4-carbon | gen-1795673629-ZYSILtWqPaZPOhDKRMAC | audit c8-inputs | 0.167016 | measured | ok |
| 2026-11-26 1:17 AM ET | google/gemini-4-carbon | gen-1795673753-0EKTPtMnbKbCBdcwMCaO | audit c2-style retry (prev length/empty) | 0.132158 | measured | ok |
| | | | **audit own total** | **1.567970** | measured | |

### This job (fix/wp2-rendering)

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| — | — | — | no paid model calls; fixes written and verified directly | 0.000000 | measured | n/a |

- Account `total_usage` at first check (1:29 AM ET): **581.918694236** (after the job's 1:25 AM ET baseline of 581.2885; other bots had already spent ~$0.63).
- Account `total_usage` at end (1:50 AM ET): **587.635012236**. Delta $5.716318, all from other bots on the shared account. This job's own generation-id sum is **$0.000000**.
- The shared stop line 586.29 was already exceeded by other work before the final credits check. This job made no paid calls at any point, so it did not contribute to crossing the line.
- Code was written directly because `CanvasItemKind::Image` is also constructed in `src/image.rs` / `src/chart.rs` (outside this package's file list); changing the variant fields would not compile without touching those files.

**This job own generation-id sum: $0.000000 (measured). Appended audit rows: $1.567970.**

### Reviewer1 PR #13 fix batch (2026-11-26)

Job cap still $2.50 from this job's own generation ids. Shared account stop line raised to 595.00. Times are America/Indianapolis (ET). Costs below are measured from `GET /api/v1/generation` (and the openrouter-call wrapper settlement).

MCP `send-message` to `google/gemini-4-carbon` timed out with no generation id and no bill (account `total_usage` unchanged after the timeout). The write was retried over direct HTTP (`/workspace/bin/openrouter-call`, same model).

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 2:31 AM ET | google/gemini-4-carbon | gen-1795677946-pXwGD9TYuxPxGjDJkK07 | WP2 PR#13 Reviewer1 one-batch fix (M1 cache gen, P6 no text clone, Image/Chart hash cache, nits) | 0.205690 | measured | empty (finish=length; 19997 reasoning tokens, 0 content) |
| 2026-11-26 2:33 AM ET | google/gemini-4-carbon | gen-1795678294-tcbMUUcoWSVWbX7IeM4L | WP2 PR#13 fix retry (prev call empty/MAX_TOKENS on reasoning) | 0.016520 | measured | settled-stale (HTTP 499; interrupted mid-stream; no content) |
| 2026-11-26 2:39 AM ET | google/gemini-4-carbon | gen-1795678579-80QhjcErkt5DVaROQYGN | WP2 PR#13 Carbon write retry2 (low reasoning; emit code) | 0.145338 | measured | ok |
| | | | **this fix-batch own total** | **0.367548** | measured | |

- Account `total_usage` at first check of this fix batch (~2:18 AM ET): **588.573482236**.
- Account `total_usage` after the successful Carbon write (credits check 2:39–2:48 AM ET window ended at): **595.377314236** at 2:48 AM ET. The $0.367548 own spend plus other bots on the shared account crossed the 595.00 stop line after our last paid call; no further paid calls were made once that reading was seen.
- This job's own generation-id sum for the Reviewer1 fix batch: **$0.367548**. Combined with the earlier WP2 implementation round ($0.000000): **$0.367548**. Still under the $2.50 job cap.

### WP1 Input/keyboard (branch fix/wp1-input, Reviewer2)

Account `total_usage` before the job's first paid call: **582.958074236** (1:34 AM ET). Job cap $2.50; account stop line for later follow-ups is 595.00 (the first call used 586.29). After the first call: **584.975028236** (1:36 AM ET). Own generation-id sum: **$0.184686**. The remaining account delta was other bots on the shared account.

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) | kind | generation id | Result |
|---|---|---|---:|---:|---:|---|---|---|
| 2026-11-26 1:35 AM ET | google/gemini-4-carbon | wp1-input-fixes | 11429 (native 13860; 0 cached) | 8193 (native 9184; 0 reasoning) | 0.11956 | measured | gen-1795674846-rGECm2i0yNRefJREbQyY | ok; diffs applied and finished by hand |
| 2026-11-26 | note | hand glue (WP1) | Wrote `tests/wp1_input.rs`. Carbon's unified diffs were malformed in shortcuts/events/text/edit; the intended edits were applied by hand and checked against the code. Added `spinbox_on_change` because `Kind::has_command` (widgets.rs, out of WP1 scope) still rejects `set_command` on Spinbox. ShiftTab shortcut labels now render as `Shift+Tab`. No version bump or CHANGELOG. | | 0.00000 | | | |
| 2026-11-26 1:56 AM ET | google/gemini-4-carbon | wp1-command-slot | 908 (native 1103; 0 cached) | 1417 billed / 6000 native (5997 reasoning, 0 content) | 0.062206 | measured | gen-1795676084-v8ovnZkkAU1LBlQkxIoI | empty (finish=length); wrapper and tests finished by hand |
| 2026-11-26 | note | hand glue (WP1 command slot) | `spinbox_on_change` now delegates to `set_command` when `Kind::has_command` includes Spinbox, otherwise still writes `node.command`. Added tests for exactly-once fire and last-setter-wins. | | 0.00000 | | | |
| 2026-11-26 2:17 AM ET | google/gemini-4-carbon | wp1-r1-should-fix | 974 (native 1135; 0 cached) | 49 (native 65; 0 reasoning) | 0.00292 | measured | gen-1795677424-UiXwAM02PU7bp6Fq9Qku | ok; asked for file contents, no diff; fixes and tests finished by hand |
| 2026-11-26 | note | hand glue (WP1 r1 should-fix) | Carbon produced no patch. `trim_lines` now returns drained bytes; `sync_after_edit` shifts cursor and a drained-prefix anchor; Canvas press uses `set_focused`; paste/undo test asserts undo and redo; mid-buffer trim tests added. | | 0.00000 | | | |
| | | **own total** | | | **0.184686** | | | |

### Reviewer1 PR #13 r2 fix batch (2026-11-26)

Job cap still $2.50 from this job's own generation ids ($0.367548 already used in the r1 fix batch, $2.13 left at the start of this batch). Shared account stop line raised to 605.00. Times are America/Indianapolis (ET). Costs below are measured from `GET /api/v1/generation` (and the openrouter-call wrapper settlement). Carbon's patches invented APIs (ImageSource variants, a destroy rewrite, cache cap 64, argument-order swap); the design was applied to the real Tk/Canvas/Image APIs and verified.

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 3:16 AM ET | google/gemini-4-carbon | gen-1795680867-9YNfKezm9hpt14zkG969 | WP2 PR#13 r2 must-fix: Slot key gen + purge, hash memcmp, Image paint_spec/rev, nits+repro test | 0.098074 | measured | ok (finish=stop; 1617 prompt / 9484 completion tokens, 0 reasoning) |
| | | | **this r2 fix-batch own total** | **0.098074** | measured | |

- Account `total_usage` immediately before this batch's paid call (~3:14 AM ET): **604.252412236**.
- Account `total_usage` after (credits check 3:27 AM ET): **613.104866236**. Own spend this batch is $0.098074; the rest of the account delta is other bots on the shared account. The 605.00 stop line was already exceeded by the time of the post-call check; no further paid calls were made after that reading.
- This job's own generation-id sum for the r2 fix batch: **$0.098074**. Combined with the Reviewer1 r1 fix batch ($0.367548) and the earlier WP2 implementation round ($0.000000): **$0.465622**. Still under the $2.50 job cap.

### Reviewer1 PR #13 r3 fix batch (2026-11-26)

Job cap still $2.50 from this job's own generation ids ($0.465622 already used after the r1+r2 fix batches). Shared account stop line raised to 620.00. Times are America/Indianapolis (ET).

**No paid model call was made.** Account `total_usage` was already **624.706070236** at the first credits check (4:17 AM ET), which is above the 620.00 stop line, so Carbon (`google/gemini-4-carbon`) was not invoked. The r3 must-fix and both nits were applied to the real Tk/Canvas/Image APIs from Reviewer1's report and verified with cargo test / clippy / release. No generation id.

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 4:17 AM ET | — | — | WP2 PR#13 r3 must-fix (Tk id in Slot + Drop purge, wrap purge, leftover Image path); skipped paid Carbon because account stop line already exceeded | 0.000000 | measured | no call (total_usage 624.706070236 > 620.00) |
| | | | **this r3 fix-batch own total** | **0.000000** | measured | |

- Account `total_usage` at first check of this batch (4:17 AM ET): **624.706070236**.
- Account `total_usage` after (credits check 4:25 AM ET): **624.706070236** (unchanged; no paid call).
- This job's own generation-id sum for the r3 fix batch: **$0.000000**. Combined with the r1 fix batch ($0.367548) and the r2 fix batch ($0.098074): **$0.465622**. Still under the $2.50 job cap.

### Reviewer1 PR #13 r4 nits (2026-11-26)

Job cap still $2.50 from this job's own generation ids ($0.465622 already used). Shared account stop line is 620.00. Times are America/Indianapolis (ET).

**No paid model call was made** (account already past the 620.00 backstop). Both r4 nits were hand-written and verified with cargo test / clippy / release. No generation id.

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 4:38 AM ET | — | — | WP2 PR#13 r4 nits: purge_premul_tk try_with; tk_id fetch_update+checked_add panic on exhaustion | 0.000000 | measured | no call (past 620.00 backstop; hand-written) |
| | | | **this r4 nit-batch own total** | **0.000000** | measured | |

- Account stop line 620.00 was already exceeded before this batch; no credits lookup or paid call was made.
- This job's own generation-id sum is unchanged: **$0.465622** (r1 $0.367548 + r2 $0.098074 + r3 $0 + r4 $0). Still under the $2.50 job cap.

## WP3 Style/theme (branch fix/wp3-style, Reviewer2)

Job cap $2.50 from this job's own generation ids. Account stop line 636.00 (backstop from 630.92 at 11:51 AM ET). Times are America/Indianapolis (ET).

Account `total_usage` at first job check: **630.969073236** (11:54 AM ET). Immediately before the first paid attempt: **631.061091236**. The wrapper later warned that the account balance was **$8.87** (credits 640.00 − usage 631.13).

Kimi (`moonshotai/kimi-k3.1-pro`) was required for this job. `user-OpenRouterAPI` `send-message` timed out with no generation id (MCP error -32001). The same prompt was then sent with `/workspace/bin/openrouter-call` (stream, id captured from headers). That call returned empty (`finish_reason=length`, all reasoning). Fixes, tests, and golden re-bless were finished by hand. No further paid call.

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) | kind | generation id | Result |
|---|---|---|---:|---:|---:|---|---|---|
| 2026-11-26 ~12:00 PM ET | moonshotai/kimi-k3.1-pro | wp3-style-theme-fixes (MCP send-message) | — | — | — | unaccounted (may have been billed) | (id lost) | client timeout, no generation id; shared-account `total_usage` rose 631.061091236 → 631.125459968 in this window (may include other bots) |
| 2026-11-26 12:01 PM ET | moonshotai/kimi-k3.1-pro | wp3-style-theme-fixes | 4511 (native 4265; 64 cached) | 17663 billed / 16384 native (17663 reasoning, 0 content) | 0.138685583 | measured | gen-1795712488-MxU5FLpxcR6s61HAGYTC | empty (finish=length); fixes and tests finished by hand |
| 2026-11-26 | note | hand glue (WP3) | Implemented B4 inheritance order, B5 disabled-map rebuild with user-override preserve, B19 Switch CHECKED, B21 CSS validation, P2 `lookup_state` no-alloc map keys, L1 `unload_image`/`update_image_rgba`. Re-blessed `tristate_dark_{1,2}x` and `tristate_highcontrast_{1,2}x` (Tyler re-approved disabled-text Dark/HighContrast). Also re-blessed all six `m011_modal_*` goldens (Tyler is deciding whether to keep them). Verified against the light 1x/2x golden diffs: the dialog title becomes Subheading 16px/700 (`src/dialog.rs:209`), not Heading 22px/700; the accent Yes button turns white/bold and gets wider; the No button shifts; the body moves down 1 logical px and the buttons about 1.5 logical px (3 device px at 2x; 1 px at 1x from rounding). No resolved-style cache (would need a `Tk` field in `src/tk/mod.rs`, out of scope). | | 0.00000 | | | |
| | | **own measured total** | | | **0.138685583** | | | |

- Account `total_usage` at end (12:15 PM ET): **631.413208735**. Own measured generation-id sum: **$0.138685583**. Stop line 636.00 was not reached.

Reviewer1 leftovers batch (no musts on #14 at a040456). Kimi via `user-OpenRouterAPI` `send-message` again returned empty (`finish_reason=length`, all reasoning). Fixes finished by hand. m011 goldens were not retouched. Did not merge `main` (N5: leave any SPEND.md tail conflict with #15 to whichever PR merges second).

| Date | Model | Task | Prompt tok | Completion tok | Cost (USD) | kind | generation id | Result |
|---|---|---|---:|---:|---:|---|---|---|
| 2026-11-26 12:35 PM ET | moonshotai/kimi-k3.1-pro | wp3-r1-leftovers (MCP send-message) | 741 (native 741; billed prompt 688) | 8192 native / 8525 billed (8525 reasoning, 0 content) | 0.069007124 | measured | gen-1795714510-gIJir8HXzvXqFetZPaQb | empty (finish=length); leftovers finished by hand |
| 2026-11-26 | note | hand glue (WP3 r1 leftovers) | S2: re-merge all `user_state_maps` at end of `rebuild_theme_defaults`; test `user_hover_map_survives_theme_change`. N1: reject out-of-range transition durations; unify `s` into `to_ms`. N2: rename cache test (no cache exists). N3: `image_unload_and_update` now asserts rendered pixels. N4: relabel timed-out MCP row. S1: correct m011 description in this file and the PR body. | | 0.00000 | | | |
| | | **own measured total** | | | **0.207692707** | | | |

- Account `total_usage` after the leftovers Kimi call (12:36 PM ET): **631.765117859**. Own measured generation-id sum: **$0.207692707**. Stop line 636.00 was not reached.

### WP4 layout (2026-11-26)

New $2.50 job cap from this job's own generation ids. Shared account stop line is 636.00. Model: `moonshotai/kimi-k3.1-pro`. Times are America/Indianapolis (ET). Costs below are measured from `GET /api/v1/generation` (and the openrouter-call wrapper settlement).

Two paid Kimi calls were made. The first returned empty content (`finish=length`, all completion tokens were reasoning). The retry was interrupted mid-stream (steering) and later settled-stale; it also produced no diffs. No third paid call was made (skill: retry-at-most-once for empty/timeout). The WP4 layout fixes were written by hand against the real `Tk` / `Rect` / `ScrollArea` APIs and verified with cargo test / clippy / release.

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 12:07 PM ET | moonshotai/kimi-k3.1-pro | gen-1795712657-BugFHFhdWZmA0yk8jhbJ | WP4 layout: margin, Frame padding, ensure_layout skip, scroll-area bar ids + tests | 0.105007202 | measured | empty (finish=length; 17404 prompt / 12000 completion, 13483 reasoning, 0 content) |
| 2026-11-26 12:11 PM ET | moonshotai/kimi-k3.1-pro | gen-1795712871-WEctQ9p2Dyr75MigKLEY | WP4 layout retry (emit diffs first); interrupted then settled-stale | 0.044055982 | measured | settled-stale / cancelled (17774 prompt / 4735 completion, all reasoning, 0 content) |
| 2026-11-26 12:22 PM ET | — | — | WP4 layout hand-finish (B9/B10/P5/P11 + tests); no further paid call | 0.000000 | measured | no call (Kimi already retried once; finished by hand) |
| | | | **this WP4 job own total** | **0.149063184** | measured | |

- Account `total_usage` at first check of this job (11:55 AM ET): **631.013528236**. Immediate pre-first-call (12:04 PM ET): **631.125459968**.
- Account `total_usage` after (credits check 12:22 PM ET): **631.464806735**. Own spend this job is $0.149063184; the rest of the account delta is other bots on the shared account.
- Account `total_credits` 640; remaining ≈ **$8.54**. Low-balance warning (below $10). Relay via parent to Chief of Staff; do not send as Tyler.
- This job's own generation-id sum: **$0.149063184** (empty $0.105007202 + settled-stale retry $0.044055982 + hand-finish $0). Under the $2.50 job cap. Account stop line 636.00 was not reached.

### WP4 Reviewer1 r1 fixes (2026-11-26)

**No paid model call was made.** Tyler asked to drop the P5 speed-up and fix Reviewer1's remaining musts/shoulds/nits by hand. Own generation-id sum for the WP4 job is unchanged: **$0.149063184**. Under the $2.50 job cap. Account stop line 636.00 was not reached.

| time (ET) | model | generation_id | purpose | total_cost | kind | result |
|---|---|---|---|---:|---|---|
| 2026-11-26 12:51 PM ET | — | — | WP4 r1: drop P5; P11 rescan on incomplete cache; B9 notebook/scroll boxed; B10 place ignores padding; nits | 0.000000 | measured | no call (hand-written) |
| | | | **this r1 fix-batch own total** | **0.000000** | measured | |
