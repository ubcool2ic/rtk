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
