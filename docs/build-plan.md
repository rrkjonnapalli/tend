# Tend v1: build plan

Do one milestone per session: *"Do M<n> from docs/build-plan.md"*. Each milestone lists what to read, what to build and how to tell it's done. Tick it off here when you finish, and add a short note if you changed or deferred something.

Spec: `docs/decisions.md` (§ numbers below refer to it) · DB: `docs/schema.sql` · Look: `docs/design/`.

---

## M1 · Project setup and app shell  ☑

> Done. SvelteKit 3 / TypeScript 6 notes are in `docs/open-questions.md`. Window title "Tend", min 960×640. The Feedback button is in place; its popover arrives in M9.

**Read:** CLAUDE.md, decisions §1, §4 · `design/tokens.css` · `screens/notes.html`, `png/notes.png`

**Build**
- Tauri 2 + SvelteKit (adapter-static, `ssr = false`), Svelte 5, TS strict, vitest, svelte-check.
- `tokens.css` imported globally. @fontsource fonts bundled. Base reset like the mockups (`body` white, `button,input{font:inherit}`, focus ring).
- Window: min size 960×640. Title "Tend".
- Routes `/notes`, `/tasks`, `/people`, `/settings` as empty pages.
- **BottomNav** component, exactly as in the mockups:
  - full-width hairline; Notes · Tasks · People icons centred, active one violet with a dot
  - `⌄` caret on the line hides the nav, and `^` brings it back
  - right side: Settings, Feedback (the timer chip comes in M10)
- **Shortcuts:** ⌘1/⌘2/⌘3 switch sections, ⌘, opens Settings.

**Done when**
- `npm run tauri dev` opens the window. Nav switches pages, the active state matches `png/notes.png`, and collapse/expand works.
- `npm run check` and `npm test` pass (one trivial test is fine).

---

## M2 · Vault: start page, setup and database  ☑

> Done. An existing `db/app.sqlite` is checked read-only before it is opened read-write, so an incompatible file is never touched. A non-SQLite file also counts as incompatible. The last vault path is in `<app config dir>/config.json`. In dev, `src/lib/dev/mock-ipc.ts` lets the UI run in a plain browser for layout checks.

**Read:** decisions §2, §3 · `schema.sql` (header and `meta`) · `screens/start.html`, `png/start.png`, `png/start-checking.png`

**Build (Rust)**
- `vault_open(path)`:
  - ensures `notes/` exists (uses it as-is if present), creates `.attachments/` and `db/` if missing.
  - opens or creates `db/app.sqlite` with `foreign_keys=ON` and WAL.
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
- Empty folder → `notes/`, `.attachments/`, `db/app.sqlite` created; `meta` has `app_id=tend`, `schema_version=1`.
- Existing vault opens without changes. A random `db/app.sqlite` from another app shows the incompatible error and isn't modified.
- Restarting the app reopens the last vault.
- `cargo test` covers: new vault, existing vault, incompatible DB (wrong app_id, higher version, missing meta).

---

## M3 · Notes index: scan, watcher, sync  ☑

> Done. A file renamed or moved outside Tend keeps its id when its mtime and title still match a note that vanished in the same scan. Folder prefix updates use `substr` instead of `LIKE`, because `_` is a `LIKE` wildcard and folder names may contain it. Non-`.md` files and invalid names are skipped; dotfiles are ignored silently. The watcher rescans the whole tree after each debounced batch.

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

---

## M4 · Shared components  ☑

> Done. Also built: `Popover` / `Modal` share one Esc stack (`util/layers.ts`), `Panel` is the header/rows/footer shell for Move to…, quick create and ⌘K, and `Section` + `EditControls` sit around `MarkdownField`. The markdown editor is CodeMirror 6 (`MarkdownEditor`), used by every markdown field. `/playground` exists only in dev builds (`import.meta.env.DEV`). The explorer selection colour follows the spec (`#eef3f6`), not the darker grey in `notes.png`.

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

---

## M5 · Notes screens  ☑

> Done. Routes: `/notes?f=<folder>` (folder list) and `/notes/<id>` (note page; `?edit=1` opens in Edit). Autosave runs 500 ms after typing stops, on blur, on ⌘E and before navigating away. `src-tauri/src/ipc_tests.rs` drives the real commands through Tauri's mock runtime with the same JSON arguments the UI sends. The folder-list summary also counts subfolders ("3 notes · 1 folder"), as in `folders.png`.

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

---

## M6 · Tasks: list, views, ordering  ☑

> Done. View rules live in `util/task-views.ts` (unit-tested): All shows top-level tasks; Overdue, My day, Upcoming and the range view include subtasks, with a parent hint. All and My day are drag-sortable; the other views sort by due date. A drag can't move a task across the open/done boundary. New tasks go to the top of the global order. The Rust `order.rs` uses the same key scheme as `order-key.ts` (parity is tested). The task root can be renamed from its explorer menu (`settings.tasks.root_name`). The explorer's "new task" button opens the folder list with the add field focused. The board arrives with M7.

**Read:** decisions §6 (Structure, Views) · `schema.sql` §4–5 · `screens/tasks.html`, `png/tasks.png`

**Build**
- Task folders (`task_folders`) in the shared Explorer, folders only. Root display name comes from `settings.tasks.root_name`.
- **List view:**
  - breadcrumb, title, summary
  - nav: All · Overdue · My day · Upcoming · date/range picker, with counts
  - List/Board toggle (the board comes in M7)
  - `+ add task`
- **Rows:** status dot (status color; ring for Not started) · title (parent hint outside All) · subtask count · relative path · due (amber if overdue) · `+ my day` / `− my day` on hover.
- **Ordering:** one global `order_key` per task. Dragging rows in All or My day writes only the moved task's key (`keyBetween`). Done tasks sink.
- **My day:** due today = included by default. The `my_day` table stores manual add and remove per day.

**Done when**
- All views filter correctly. Drag order survives a restart and stays consistent between All and My day.
- The range picker filters by `due_date BETWEEN`. The screens match the PNGs.

---

## M7 · Tasks: task page and board  ☑

> Done. Files are added through the dialog or dropped from the OS. Dropped files arrive as Tauri drag-drop events, which are hit-tested against the drop zone. Open and Show in folder go through Rust (`attachment_open` / `attachment_reveal`), so the UI only ever passes attachment ids. Also added: a task page *Move to…* and *Delete task* (see `docs/open-questions.md`). `ipc_tests.rs` covers the task, comment and attachment commands.

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

---

## M8 · People  ☑

> Done. Date and time of birth are checked in the UI, which turns the field red and doesn't save, and again in Rust, so an invalid value never reaches the database. Fields save on blur or Enter. "Add person" splits on the first space (first name, then the rest as the last name). Deleting a person unassigns their tasks (`ON DELETE SET NULL`). `birthday_reminder` is stored, not shown.

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

---

## M9 · Settings and feedback  ☑

> Done. Rust enforces the status rules (first and last pinned, reserved values in any case, duplicate labels, value format, deletes blocked while in use or for defaults), and the UI shows Rust's message inline. A new status gets the first unused swatch. Feedback stores the app version from `tauri.conf.json`. The Settings → Command palette list is generated from `src/lib/commands.ts`.

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

---

## M10 · ⌘K, quick create and timers  ☐

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

---

## After v1

- Polish pass: empty states, error toasts, loading states, window state restore.
- Packaging: macOS / Windows / Linux builds, code signing, auto-update.
- v2: calendar, events and reminders, see `docs/later/calendar-design-spec-doc.md` (adds `schema_version` 2).
