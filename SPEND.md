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
