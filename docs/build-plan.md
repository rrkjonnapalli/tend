# Tend v1: build plan

Do one milestone per session: *"Do M<n> from docs/build-plan.md"*. M1–M10 were built and checked against the code on 2026-10-09; the *Status* line under each says what changed. The **Build** and **Done when** text is the original plan, kept for reference: where it disagrees with `decisions.md`, the decisions win. Each milestone lists what to read, what to build and how to tell it's done. Tick it off here when you finish, and add a short note if you changed or deferred something.

Spec: `docs/decisions.md` (§ numbers below refer to it) · DB: `docs/schema.sql` · Look: `docs/design/`.

---

## M1 · Project setup and app shell  ☑

**Read:** CLAUDE.md, decisions §1, §4 · `design/tokens.css` · `screens/notes.html`, `png/notes.png`

**Build**
- Tauri 2 + SvelteKit (adapter-static, `ssr = false`), Svelte 5, TS strict, vitest, svelte-check.
- `tokens.css` imported globally. @fontsource fonts bundled. Base reset like the mockups (`body` white, `button,input{font:inherit}`, focus ring).
- Window: min size 960×640. Title "Tend".
- **App icon:** run `npx tauri icon docs/design/icon/icon-1024.png` to generate `src-tauri/icons/`. Use `docs/design/icon/mark.svg` as the favicon.
- Routes `/notes`, `/tasks`, `/people`, `/settings` as empty pages.
- **BottomNav** component, exactly as in the mockups:
  - full-width hairline; Notes · Tasks · People icons centred, active one violet with a dot
  - `⌄` caret on the line hides the nav, and `^` brings it back
  - right side: Settings, Feedback (the timer chip comes in M10)
- **Shortcuts:** ⌘1/⌘2/⌘3 switch sections, ⌘, opens Settings.

**Done when**
- `npm run tauri dev` opens the window. Nav switches pages, the active state matches `png/notes.png`, and collapse/expand works.
- The Tend icon shows in the Dock / taskbar.
- `npm run check` and `npm test` pass (one trivial test is fine).

**Status (2026-10-09):** done. Changed: tooling runs on Bun; the adapter is set in `vite.config.js`; the window is frameless with a 1280×800 default and remembers its size and position (`tauri-plugin-window-state`). Gap: the favicon is the 32px app icon, not `design/icon/mark.svg` (see `open-questions.md`).

---

## M2 · Vault: start page, setup and database  ☑

**Read:** decisions §2, §3 · `schema.sql` (header and `meta`) · `screens/start.html`, `png/start.png`, `png/start-checking.png`

**Build (Rust)**
- `vault_open(path)`:
  - ensures `notes/` exists (uses it as-is if present), creates `.attachments/` and `.tend/` if missing.
  - opens or creates `.tend/tend.db` with `foreign_keys=ON` and WAL.
  - **New DB:** run `schema.sql` as migration 1.
  - **Existing DB:**
    - reads `meta`: `app_id='tend'` and `schema_version` ≤ app version → run pending migrations.
    - Otherwise → error `IncompatibleDb`. Never modify an incompatible file.
  - Returns a step-by-step result, `[{item:'notes/', status:'found'|'created', detail:'42 notes'}, …]`, so the UI can show the checklist.
- Remember the last vault path in the app config dir. On launch, open it directly. If it's gone or broken, show the start page.

**Build (UI)**
- **Start page** as in the mockup:
  - wordmark; "Choose vault folder…" text link (dialog plugin); the three explanation lines
  - after choosing: path + Change, checklist animating line by line, then **Open vault →** or the red error + **Choose another folder**
  - right column: ⌘K examples + shortcuts
- **Settings → General:** vault path + "Change vault folder…" (back to the start page). The rest of Settings comes in M9.

**Done when**
- Empty folder → `notes/`, `.attachments/`, `.tend/tend.db` created; `meta` has `app_id=tend`, `schema_version=1`.
- Existing vault opens without changes. A random `.tend/tend.db` from another app shows the incompatible error and isn't modified.
- Restarting the app reopens the last vault.
- `cargo test` covers: new vault, existing vault, incompatible DB (wrong app_id, higher version, missing meta).

**Status (2026-10-09):** done. Changed: the chosen folder is the notes root (no `notes/`); Tend's files live in `.tend/` (`.tend/attachments/`, `.tend/tend.db`); migrations run through `rusqlite_migration` with the version in `PRAGMA user_version` (`meta` keeps only `app_id`); a non-empty folder that isn't a vault asks first (*Use this folder* / *Choose another folder*). The checklist steps are `notes` (count), `.tend/attachments/`, `.tend/tend.db`.

---

## M3 · Notes index: scan, watcher, sync  ☑

**Read:** decisions §5 (Sync), §5a (Names) · `schema.sql` §2

**Build (Rust)**
- **Scan on open:** walk `notes/` and upsert `note_folders` and `notes` (`folder`, `file_name`, `title` = first `# ` heading or file name, `modified_at` = mtime).
  - Only valid names (`^[a-z0-9_]+$`, `.md` files). Skip everything else and record it in a "skipped" list (shown in Settings → General in M9).
  - Items no longer on disk get `missing_at`; purge after 7 days.
- **File watcher** (`notify`, debounced ~300 ms): apply external creates, renames, moves and deletes the same way. Emit `notes:changed` to the UI.
- **Commands:**
  - `notes_tree()`, `note_read(id)`, `note_write(id, text)`
  - `note_create(folder, name)`, `folder_create(path)`
  - `item_rename(...)`, `item_move(...)`, `item_delete(...)`
- **Order:** disk first, then DB in one transaction. A folder rename updates `note_folders` with a prefix UPDATE (notes follow by FK cascade). Folder delete removes the directory recursively.

**Done when**
- `cargo test` with a temp vault:
  - initial scan; external rename picked up; name validation; rename and move of a folder with children
  - delete; skipped invalid names; DB matches disk after a forced rescan
- Editing a note in another editor updates its title in Tend within ~1 s.

**Status (2026-10-09):** done. Changed: notes are `notes(id, attachment_id, …)` with the body as an `attachments` row (`path` relative to the vault); the name rule is `^[a-z0-9]+(_[a-z0-9]+)*$` and bad names found on disk get their slug only with the vault's consent (`notes.fix_names`), logged in `path_log`; otherwise they are listed as skipped with their fix. There is no indexed title: the title is the `# heading` in the file. The watcher watches the vault root, ignores hidden paths and rescans. Outside renames are matched by sha256. Gap: no watcher test edits an existing note (only a new file).

---

## M4 · Shared components  ☑

**Read:** decisions §4, §5a · `screens/folders.html` + `png/folders-*.png` · task page in `screens/tasks.html` (description, tags, due) · `design/README.md` (logic to reuse)

**Build (Svelte, generic, used by M5–M9)**
- **Explorer:**
  - toolbar (new item, new folder, reload, collapse all), collapsible panel (232px)
  - tree rows with caret / name click semantics
  - `⋯` / right-click menu: Rename (F2) · Move to… (⌘⇧M) · Delete… (⌫)
  - inline rename with live name check
  - **drag rows onto folders:** blue for valid targets, red for invalid, plus a pointer label
  - Takes a data adapter so Notes (folders + notes) and Tasks (folders only) can both use it.
- **MoveToDialog:** path field with folder suggestions, excluding self and subfolders; "will create …".
- **DeleteDialog:** folder version requires typing the folder name; note version is a simple confirm.
- **Popover:** anchored, flips to stay inside the window, closes on Esc and outside click.
- **MarkdownField:** rendered view; pencil or double-click → textarea editor; ✕ / ✓; ⌘↵ saves; Esc cancels. Used for task description, person notes and comments.
- **TagEditor:** chips with red ✕, `+` → input with suggestions from the shared tag pool, ↑↓ Enter, create new.
- **DateField** (popover month grid, plus range mode for the tasks nav) and **TimeField** (h : mm + AM/PM, typed or ↑↓).
- `util/names.ts` (same rule as Rust) and `util/order-key.ts` (`keyBetween`, see `design/README.md`). Unit test both.

**Done when**
- A dev-only `/playground` route shows each component working, matching the mockup states. Remove the route or hide it behind a dev flag afterwards.
- vitest covers `keyBetween` (ordering, many inserts between neighbours) and the name rule.

**Status (2026-10-09):** done. Changed: rename makes slugs and clashes get `_1` (never an error); MarkdownField uses CodeMirror 6 rather than a textarea. Gap: `/playground` still ships (it redirects outside dev); delete it.

---

## M5 · Notes screens  ☑

**Read:** decisions §5 · `screens/notes.html`, `png/notes.png`, `png/notes-list.png`

**Build**
- Explorer wired to the notes index (M3), including rename / move / delete / drag / new note / new folder.
- **Folder list:** breadcrumb, title, counts, `+ new note` row, and rows (name, relative path, last edited) covering subfolders too. Full width.
- **Note page:**
  - sticky top bar: back · breadcrumb · width toggle (760px ↔ full; one setting for all notes) · Read/Edit icon (⌘E)
  - tag row; rendered markdown (Read) or full textarea (Edit)
  - **saving:** autosave to disk, debounced ~500 ms, and on blur. New notes open in Edit.
- Watcher events refresh the explorer and the open note (if it isn't being edited).

**Done when**
- Create, edit, rename, move and delete notes and folders from Tend, and the files on disk match.
- Changes made in Finder or another editor show up.
- Tags persist (`note_tags`). The screens match the PNGs.

**Status (2026-10-09):** done. Changed: notes are shown by file name; PageBar (40px, breadcrumb + count) instead of a big title, a search line and a sticky column header; Read/Edit is ⌘R (⌘E toggles the explorer); autosave also runs on leaving Edit and before navigating. Gap: a note deleted outside Tend while it is open leaves the page on stale text.

---

## M6 · Tasks: list, views, ordering  ☑

**Read:** decisions §6 (Structure, Views) · `schema.sql` §4–5 · `screens/tasks.html`, `png/tasks.png`

**Build**
- Task folders (`task_folders`) in the shared Explorer, folders only. Root display name comes from `settings.tasks.root_name`.
- **List view:**
  - breadcrumb, title, summary
  - nav: My day · All · Overdue · Upcoming · date/range picker, with counts
  - List/Board toggle (the board comes in M7)
  - `+ add task`
- **Rows:** status dot (status color; ring for Not started) · title ("s" badge on subtasks, hover "Subtask") · subtask count · relative path · due (amber if overdue) · `+ my day` / `− my day` on hover.
- **Ordering:** one global `order_key` per task. Dragging rows in All or My day writes only the moved task's key (`keyBetween`). Done tasks sink.
- **My day:** due today = included by default. The `my_day` table stores manual add and remove per day.

**Done when**
- All views filter correctly. Drag order survives a restart and stays consistent between All and My day.
- The range picker filters by `due_date BETWEEN`. The screens match the PNGs.

**Status (2026-10-09):** done. Changed: My day comes first and is the default view; rows have a status dropdown, an inline edit pencil and fixed-width columns (edit · My day · count · due · assignee · path); subtasks are one level deep; the range filter runs in TypeScript (inclusive). Open: "All" shows top-level tasks only and "Upcoming" includes today — confirm both. Risk: equal `order_key`s (possible after a sync merge) make `keyBetween` throw.

---

## M7 · Tasks: task page and board  ☑

**Read:** decisions §6 (Statuses, Task page, Board), §8 · `png/task-page.png`, `png/tasks-board.png`

**Build**
- **Task page:**
  - back · breadcrumb (folders → parent tasks, clickable) · width toggle · title (editable)
  - **Fields:** Status dropdown · Due (DateField + TimeField) · My day toggle · Assignee (person chip / `+ assign` people search) · Tags
  - **Sections** (collapsible, close together):
    - Description (MarkdownField)
    - Subtasks (add, open, drag order)
    - Attachments (drop zone + "link a note"; rows with open / show in folder / remove; files copied to `.attachments/<id>.<ext>` with sha256)
    - Comments (composer, edit inline)
- **Board:**
  - one column per status, full height, columns scroll on their own
  - cards show title, due, subtask count, path and assignee initials
  - pointer drag with a dashed placeholder; drop sets status + order key
  - `+ add task` per column

**Done when**
- Everything on the task page persists. Moving a task moves its subtasks.
- Board drag changes the status and keeps the order. Attachments land in `.attachments/` and open with the OS default app.
- The screens match the PNGs.

**Status (2026-10-09):** done. Changed: Location row and *Move task to* (top-level tasks), Labels row and board label dots, assignee badge instead of initials, a subtask has no Subtasks section; attachments are copied to `.tend/attachments/<id>.<ext>`; board columns share the width up to five, then scroll.

---

## M8 · People  ☑

**Read:** decisions §10 · `screens/people.html`, `png/people.png`, `png/person-page.png`

**Build**
- **List:** search by name, phone or email; `+ add person` (first + last, ↵ opens it); columns Name · Birthday ("in 6 days · turns 31", violet within 14 days) · Phone · Email.
- **Person page:**
  - **Fields:** first, last, nickname, DOB (`yyyy-mm-dd` or `mm-dd`) with next-birthday hint, time of birth
  - **Sections:** Phone / Email / Address (value + label rows, edit / delete, add row; `person_contacts`), Notes (MarkdownField), Assigned tasks
  - `birthday_reminder` is stored but **not shown**.

**Done when**
- CRUD works and persists; invalid DOB or time turns red and doesn't save.
- Assigned tasks link to the task page. The screens match the PNGs.

**Status (2026-10-09):** done. The Birthday column also shows the date ("9 Oct · in 6 days · turns 31").

---

## M9 · Settings and feedback  ☑

**Read:** decisions §6 (Statuses), §13, §4 (Feedback) · `screens/settings.html`, `png/settings.png`, `png/settings-palette.png`

**Build**
- **Sections:** General · Notes (empty) · Tasks · People (empty) · Command palette.
- **General:** vault path + Change vault folder…, plus the skipped-items line from M3 (expands to the list).
- **Tasks → Statuses** (collapsible), with every rule in §6:
  - fixed first and last; drag order; color swatches (hex)
  - rename inline; add with an auto UPPER_SNAKE value
  - reserved values blocked; delete blocked while in use (inline error)
- **Command palette:** read-only command list (generated from the same command registry M10 uses).
- **Feedback popover** (bottom nav): Bug / Idea / Other, text, Send (⌘↵) → `feedback` table.

**Done when**
- Status changes show up immediately in task rows, the dropdown and board columns.
- Rust tests cover the status rules. Feedback rows are saved.

**Status (2026-10-09):** done. Added: Settings → Tasks → Labels (label id = slug of its first name); colors are 20 swatches plus the system picker and a hex field; custom status ids equal their values.

---

## M10 · ⌘K, quick create and timers  ☑

**Read:** decisions §9, §11, §12 · `screens/command-palette.html`, `screens/quick-create.html`, `png/command-palette*.png`, `png/quick-create-*.png`, timer chip in `png/tasks.png`

**Build**
- **Command registry** (`src/lib/commands.ts`): id, group, label, keys, example, run(). Used by the palette, the shortcuts and Settings → Command palette.
- **⌘K palette** (first design): groups, title search across notes and tasks, quick-create phrases (`task:`, `note:`, `person:`, `timer 10m …`), ↑↓ ↵ Esc.
- **Quick create:** ⌘N note · ⌘⇧T task · ⌘⇧N folder (Notes/Tasks switch) · ⌘T timer.
  - Path field pre-filled from `settings` last path, folder suggestions, "will create".
  - After create: note opens in Edit; task opens its page; folder opens its list.
- **Timers:**
  - `timers` table; chip next to Settings (soonest countdown + `+N`), popover with +1m / cancel.
  - Countdown recalculated from `ends_at` every second.
  - At zero: desktop notification + optional sound, `state='rung'`.
  - On launch, overdue running timers show once as "finished while Tend was closed".

**Done when**
- Every shortcut works without triggering webview defaults. The palette runs every command.
- A 10 s timer notifies on time, even after the laptop sleeps mid-countdown (it fires right after wake).

**Status (2026-10-09):** done. Added: ⌘E explorer, ⌘R read/edit or list/board, ⌘L search, ⌘D dock. Gap: the timer sound can't be switched off (`timers.sound` has no control). Note: quick-create folder shares one last path between Notes and Tasks.

---

## After v1

- Polish pass: empty states, error toasts, loading states. (Window state restore is done.)
- Gaps and open questions from the 2026-10-09 check: `docs/open-questions.md`.
- Git sync: `docs/v1-sync-spec.md`.
- Packaging: macOS / Windows / Linux builds, code signing, auto-update.
- v2: calendar, events and reminders, see `docs/later/calendar-design-spec-doc.md` (a later migration step).
