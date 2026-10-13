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

## From the Library work, 2026-10-12 to 2026-10-14 (build plan L7)

| # | Where | Item |
|---|---|---|
| 12 | Vaults made before L7 | `tend_enums.status` was added to the schema in place, so an older vault needs it once: `ALTER TABLE tend_enums ADD COLUMN status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'BLOCKED'));`. Done by hand for the one existing vault. A migration step (the app has a list of them) would do it on open; parked until a second device exists. |
| 13 | Sync | Blocking or deleting a tag, and a recolor against a reorder, are covered by unit tests only. No two-device run yet. |
| 14 | Snaps | Saving a snap into a new stack, and pairing two snaps, are several calls. A failure midway leaves a stray stack (the reload shows it); marked `shortcut:` in `SnapDialogs` and the Snaps page. One Rust command each would make them all-or-nothing. |
| 15 | Words | dictionaryapi.dev timed out on every request on 2026-10-13. There is no fallback: a lookup asks only the source picked in Settings. |
| 16 | Words | freedictionaryapi.com has no recordings, and calls an idiom a verb, so a looked-up idiom no longer switches its kind to *idiom* by itself. |
| 17 | Fonts | `static/fonts/Smile Moon.otf` says "All Rights Reserved" (MJType). Fine in a private repo; check its licence before the app is shared. |
| 18 | Keyboard | No arrow keys in the date picker's month grid; the colour swatches move in order, not as a grid. The stack picker shows eight rows; the rest are reached by typing. |
| 19 | Tests | None for the HTTP fetch (a 404, the size cap) or for Home's add guess at its 40-character edge. Short words with a dot (`Node.js`) are guessed as links; left as it is. |
| 20 | Rust | `notes/`, `tasks/folders.rs` and a few sync helpers still take several positional arguments. |
