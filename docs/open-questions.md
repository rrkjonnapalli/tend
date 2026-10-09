# Open questions and known gaps

From the check of M1–M10 against the code on 2026-10-09 (`docs/build-plan.md` has the per-milestone
status). Each item is small; none blocks using the app.

## Gaps

| # | Where | What | Suggested fix |
|---|---|---|---|
| 1 | M1 | The favicon is the 32px app icon (`static/favicon.png`), not `docs/design/icon/mark.svg`. | Use `mark.svg` as the favicon. |
| 2 | M3 | No watcher test edits an existing note; `watcher_tests.rs` only checks that a new file is indexed. | Add a test that edits a file and waits for `updated_at` to move. |
| 3 | M4 | `/playground` still ships (it redirects to Notes outside dev). | Delete the route. |
| 4 | M5 | A note deleted outside Tend while it is open stays on screen; the re-read after a watcher event has no error path. | On a failed re-read, go back to the note's folder with a toast. |
| 5 | M10 | Timer sound can't be turned off: `timers.sound` (default on) has no control anywhere. | Add a switch (Settings → General, or the timer popover). |
| 6 | M10 | Quick create's folder kind keeps one last path for both Notes and Tasks. | Keep one per tree. |

## Questions

| # | Where | Question |
|---|---|---|
| 7 | M6 | "All" lists top-level tasks only, while My day, Overdue and Upcoming include subtasks. Intended? |
| 8 | M6 | "Upcoming" includes tasks due today. Intended, or should it start tomorrow? |

## Risks

| # | Where | Risk |
|---|---|---|
| 9 | M6 + sync | `keyBetween` throws when two neighbours have the same `order_key`. A sync merge can produce equal keys (two devices inserting at the top). Reordering must handle ties, e.g. by re-spacing keys or comparing `(order_key, id)`. |
| 10 | M10 | A timer firing right after wake depends on the interval and focus ticks; only a manual test can confirm it. |
