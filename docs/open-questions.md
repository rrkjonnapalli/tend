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
| 21 | Mobile | Deleting on a phone: there is no system Trash on iOS or Android (`trash` doesn't build there; the spike makes it delete for good). Delete for good, or move to `.tend/trash/` and empty it after N days? Undecided. |
| 22 | Mobile | Picking the vault folder on a phone. `tauri-plugin-dialog` has no folder picker on iOS or Android; iOS needs a security-scoped bookmark, Android needs *All files access* for a real path. Until decided, the vault is `vaults/<host>/<owner>/<repo>` in Tend's own storage (`docs/mobile/tend-mobile-spec.md` §10). |
| 23 | Mobile | The phone's "Getting your vault ready…" shows an indeterminate bar, not the mockup's steps and counts (MD-Ready): Rust sends no restore progress yet. A restore cut short starts over; it doesn't resume. |
| 24 | Mobile | The Android app still has Tauri's default icon: run `bun tauri icon docs/design/icon/icon-1024.png` and commit `src-tauri/gen/android` icons. |
| 25 | Mobile | On the Android emulator the first launch after `adb install -r` logs "Failed to fetch dynamically imported module" for an old chunk and the process exits; the second launch works. Likely the WebView's cache of the previous build. Check on a real phone and with an APK update before release. |
| 26 | Mobile | Snaps on a phone open the desktop snap editor (a popup over the board: text, pin, delete, paper, stack, tags). MD-Snap-Edit and MD-Snap-New draw a full page with Paper and Tags sections; not built yet. |

## From the Library work, 2026-10-12 to 2026-10-14 (build plan L7)

| # | Where | Item |
|---|---|---|
| 13 | Sync | Blocking or deleting a tag, and a recolor against a reorder, are covered by unit tests only. No two-device run yet. |
| 18 | Keyboard | No arrow keys in the date picker's month grid; the colour swatches move in order, not as a grid. The stack picker shows eight rows; the rest are reached by typing. |
| 19 | Tests | None for the HTTP fetch (a 404, the size cap) or for Home's add guess at its 40-character edge. Short words with a dot (`Node.js`) are guessed as links; left as it is. |
| 20 | Rust | `notes/`, `tasks/folders.rs` and a few sync helpers still take several positional arguments. |

## Resolved since 2026-10-14

| # | Where | What was done |
|---|---|---|
| 12 | Schema | Migration 2 (task order keys) is folded into `docs/schema.sql`, which is the whole of v1 and frozen from the first release; later changes are migration steps. A dev vault at `user_version` 2 opens as incompatible: set `PRAGMA user_version = 1` once (its tables already match v1). |
| 14 | Snaps | Saving into a new stack (`startStack` on `snap_create` / `snap_update`) and pairing two snaps (`stack_pair`) are one transaction each; a failure leaves no stray stack. |
| 15 | Words | dictionaryapi.dev timed out on every request on 2026-10-13. There is no fallback: a lookup asks only the source picked in Settings. Decided: no fallback (`release-plan.md` §1). |
| 16 | Words | freedictionaryapi.com has no recordings, and calls an idiom a verb, so a looked-up idiom no longer switches its kind to *idiom* by itself. Accepted with the same decision. |
| 17 | Fonts | Moved to `release-plan.md` §2.7 (Smile Moon licence). |
