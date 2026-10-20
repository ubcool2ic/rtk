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
