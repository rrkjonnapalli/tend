# Open questions and known gaps

From the check of M1–M10 against the code on 2026-10-09 (`docs/build-plan.md` has the per-milestone
status).

## Resolved on 2026-10-09

| # | Where | What was done |
|---|---|---|
| 1 | M1 | `docs/design/icon/mark.svg` is the favicon (`static/favicon.svg`). |
| 2 | M3 | Watcher tests: an edit to an existing note reaches the index; a write under `.tend/` never triggers a rescan. |
| 3 | M4 | The dev-only `/playground` route is removed. |
| 4 | M5 | A note deleted elsewhere while open keeps its text and asks: **Keep** recreates it as a new note (new id, same folder, name, text, tags); **Discard** goes back to the folder. Sync will use the same path. |
| 5 | M10 | Settings → General → Timers: "Play a sound when a timer ends". |
| 6 | M10 | Quick create remembers a folder path per tree (Notes, Tasks). |
| 7 | M6 | "All" stays top-level only (decided). |
| 8 | M6 | "Upcoming" includes today and Done tasks; only Overdue never shows Done; a "hide done" toggle hides them everywhere (decided). |
| 9 | M6 + sync | Tied `order_key`s: tasks sort by `(order_key, id)` everywhere, and a drop between tied keys lands right after them instead of failing. |

## Still open

| # | Where | Item |
|---|---|---|
| 10 | M10 | A timer firing right after the laptop wakes depends on the interval and focus ticks; only a manual test can confirm it. |
| 11 | M3 | A scan that fixes names renames the files first and logs them in one transaction after; if that transaction fails, the files keep their new names but the `path_log` rows are lost (the next scan indexes them as they are). Accepted: disk is the truth, and the case needs a database failure mid-scan. |
