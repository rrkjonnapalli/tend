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
- `bun run tauri dev` opens the window. Nav switches pages, the active state matches `png/notes.png`, and collapse/expand works.
- The Tend icon shows in the Dock / taskbar.
- `bun run check` and `bun run test` pass (one trivial test is fine).

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

# Library: Words, Snaps, References

Planned on 2026-10-12. Behaviour: `docs/tend-library-plan.md` (the original spec) and `docs/tend-library/designs/` (the mockups; read each `.dc.html` script for exact flows). Where they disagree with `decisions.md` §15–§16, the decisions win. Do one milestone per session, in order; L0 changes existing tables, so it comes first.

**Constants:** no magic strings. Enum kinds, link source and target types, paper types, default enum values and setting keys live in one place per side: `src/lib/enums/*.enum.ts` (TS) and `src-tauri/src/enums/` (Rust, serde `SCREAMING_SNAKE_CASE`). Code imports them; it never types `'TASK'` or `'WORD_KIND'` inline.

---

## L0 · One enum table and one link table  ☑

**Read:** decisions §16 · `docs/schema.sql` §1, §2, §5–§7 · `v1-sync-spec.md` §4.1, §9.2, §9.3

**Build**
- `tend_enums` replaces `statuses`, `task_labels` and `tags`. `tend_links` replaces `note_tags`, `task_tags`, `task_label_links` and `attachment_links`. Shapes and rules: decisions §16.
- `schema.sql` is edited in place (nothing has shipped). Existing vaults are reset: delete `.tend/tend.db` and reopen; note files are untouched.
- Default rows (statuses, word kinds) are seeded with ids derived from kind and value (`TASK_STATUS:DONE`), so every device makes the same rows.
- Clean-up triggers: `AFTER DELETE` on every source table (`tasks`, `notes`, `task_comments`, `people`; later `words`, `snaps`, `refs`) removes its links; on `attachments` and `tend_enums`, removes the links pointing at the row.
- `tasks.status` keeps the status value and loses its foreign key. The app refuses to delete a status in use (not atomic; accepted).
- Rust: `statuses/`, `labels/` and `tags/` read and write `tend_enums`; tags, labels and attachments link through `tend_links`. Code enums for `TEND_SOURCE_TYPE`, `TEND_TARGET_TYPE` and `TEND_TARGET_KIND` (decisions §16).
- Sync: `TABLES` lists the two new tables instead of the seven old ones. `KEEP_SMALLEST` on `tend_enums (kind, value)` and on the `tend_links` key. The " 2" rule for duplicate display names runs per `kind`. Orphans: a link whose source row, attachment or enum is gone is dropped; a task whose status is gone gets `NOT_STARTED`.
- TS: `statusesAPI`, `labelsAPI`, `tagsAPI`, the attachment calls, their stores and Settings → Tasks keep working on the new commands.

**Done when**
- `cargo test`, `bun run check` and `bun run test` pass.
- Statuses, labels, tags and attachments behave exactly as before, including sync.
- Tests: every source type has its clean-up trigger; deleting a folder of tasks leaves no links behind; two devices adding the same tag or status merge into one row; default enum rows land in `fixed.jsonl`.
- By hand: two vaults synced through one remote agree on statuses, labels, tags and attachment links.

**Status (2026-10-12):** built. `statuses`, `labels`, `tags` and `attachments` keep their public functions and commands; only storage moved, so the UI is unchanged except that a label's id is now its value in capitals (`DB_SCRIPT`) and attachment owners are `SOURCE_TYPE` values (`TASK`). `meta` is one column, so sync merges colour and position as one value (a colour change on one device and a reorder on another: the newer row's `meta` wins). Open: the two-vault sync check by hand.

---

## L1 · Library data and Rust core  ☑

**Read:** decisions §15, §16 · `tend-library-plan.md` *Data model*

**Build**
- Tables: `words (title, kind, dictionary, meaning, notes)` with `UNIQUE (title)`; `snap_stacks (parent_id, name)`; `snaps (stack_id, text, paper, color, pinned)`; `refs (type, value, title, why, url_key)` with `UNIQUE (url_key)`. Tags and word kinds go through `tend_enums` and `tend_links`; seed `WORD_KIND` with `WORD`, `IDIOM`, `PHRASE`, `ABBR`, `TERM`, `OTHER`.
- Modules `words/`, `snaps/` (stacks included) and `refs/` with CRUD commands.
- Rules in Rust: a `WORD` title is lowercased on save, other kinds keep the title as typed; a stack can't move into itself or its own sub-stacks; removing a stack moves its snaps and sub-stacks up one level in one transaction; `url_key` is the normalised URL (scheme, trailing slash and case ignored).
- Sync: the new tables in `TABLES` (`snap_stacks` before `snaps`). The new `MERGE_ON_KEY` rule (v1-sync-spec §9.2) for `words` by `title` and `refs` by `url_key`, with `newest_fill_empty`. Orphans: a snap or stack whose stack is gone goes to the board root; a word whose kind is gone becomes `OTHER`. Cycles: when two devices nest two stacks in each other, the stack with the larger id goes to the root.

**Done when**
- `cargo test` covers CRUD, the lowercase rule, the cycle refusal, remove-moves-up, and sync merges: a duplicate word, a duplicate link, an orphan snap, a stack cycle, a deleted kind. Merging gives the same rows on both devices.
- By hand: two vaults synced through one remote agree on words, snaps, stacks, references and their tags, including a word added on both while offline.

**Status (2026-10-12):** built. Commands: `words_list`, `word_kinds_list`, `word_create/update/delete`, `snaps_board`, `snap_create/update/delete`, `stack_create/rename/move/remove`, `refs_list`, `ref_create/update/delete`, and `word|snap|ref_tags_set`. Papers and reference kinds are code enums (`Paper`, `RefKind`); `tags::Owner` was replaced by `SourceType`. `MergeRule` names its rows' link source type instead of link tables, since every link is in `tend_links`. A snap edit, pin or move sets `updated_at`, so it moves to the front of its level (sync needs the new time). Word kinds are list-only in Rust; adding and editing them comes with Settings in L4. Open: the two-vault sync check by hand.

---

## L2 · Library shell  ☑

**Read:** decisions §15 · `designs/library.dc.html`, `library.css`

**Build**
- Fourth bottom-nav tab, Library (the books icon from the Library mockup), route `/library`, ⌘4. Library reopens on the last area (setting `ui.library.area`, per device).
- The bar: a `Segmented` component (Words · Snaps · References; Home arrives in L6), the + icon button, and the scoped search (placeholder per area; switching area, Esc or × clears it).
- Bundle `@fontsource/caveat` (500, 600).
- Settings → General → Confirmations: *Ask before deleting a word / a snap / a reference*, on by default.
- `ui.library.area` and the new `confirm.delete.*` keys stay off the shared-keys list, so they never sync.

**Done when**
- The tab switches areas and reopens on the last one; search clears as described; Library chrome follows decisions §15 (no shadows, no focus rings).

**Status (2026-10-12):** built and checked in the app. Areas are routes (`/library/words`, `/library/snaps`, `/library/references`; `/library` opens the last one). The bar uses a new `Segmented` (radios underneath) and `SearchLine` with a new `clearable` (× and Esc). + bumps `library.adds`; each area opens its add on it from L3 on. Until then each area shows its empty state (`LibraryEmpty`), even if synced data exists.

---

## L3 · References  ☑

**Read:** decisions §15 · `designs/references.dc.html`, `refs.css`

**Build**
- Add `ureq`. Command `ref_title(url)`: async, runs `ureq` on a worker thread, 5 s timeout, reads at most 64 KB, returns the `<title>` text. Unit-test the parser.
- List: Title · Reference · Why I kept it · Tags · Added, newest first. Link references show the URL without `https://` and `www.`, with the open-link icon (opener plugin). Added is relative (`ago`).
- Popup for add and edit: reference input; link · text switch, picked from the input and fixed once changed by hand; title (mirrors text references; for links, fetched about 650 ms after typing stops, with the site name; a typed title is never overwritten); why; tags with up to six "used before" suggestions. Duplicate links show "You saved this link on <date>. Open it" and disable Save. Enter in the reference saves, ⌘↵ from any field, Esc closes; Delete asks first.
- `url_key` normalising exists in TS (live check) and Rust (stored value) with the same test cases.
- Search filters title, reference and why.

**Done when**
- Paste + Enter saves; the title fills in; a duplicate link blocks Save; search filters; inputs never autocorrect (check smart quotes and dashes by hand on macOS).

**Status (2026-10-12):** built and checked in the app (add, title fetch, tags, list). `ref_title` runs `ureq` in an async command (Tauri's worker pool); a failed fetch leaves the title empty. A link saved before its fetch ends gets the title from that same fetch once it returns (no background job). Tags use `TagEditor` as it is (its inline input already suggests from the pool). The open-link icon sits over the Reference column through CSS subgrid. Not done: the row highlight after saving, and the "Added" header is left-aligned (`ColumnHeader` has no per-column alignment).

---

## L4 · Words  ☐

**Read:** decisions §15 · `designs/words.dc.html`, `words.css`

**Build**
- List on the left grouped by kind (always open, in kind order, only kinds with entries), alphabetical within a group; a "new …" row while adding. No random or collapse buttons.
- Entry page: title styled as the page heading (on a saved entry, empty or duplicate on blur reverts); meta row: kind chip with a listbox, IPA + play, part of speech, tags (inline `TagEditor`); Dictionary fold; "What it is for me" fold with Meaning and Notes (markdown as written, click to edit in place, no box).
- Kind guess while typing until picked by hand: uppercase and no spaces → `ABBR`, no spaces → `WORD`, spaces → `PHRASE`; a dictionary idiom switches to `IDIOM`.
- Command `word_lookup(title)` against dictionaryapi.dev (lowercase query) for kinds with `lookup`; map to our JSON (senses with examples, synonyms, IPA, part of speech, audio URL, fetched_at). Fixture test for the mapping. "Not in the dictionary." / "No dictionary for this one." / re-fetch icon.
- Play: CSP gets `media-src https:`.
- Adding: Save / Cancel row (⌘↵ / Esc); "Already saved. Open '<title>'" disables Save. Saved entries save as you go; delete in the top bar asks first. Changing a kind to `WORD` lowercases the title and is refused if that clashes.
- Settings → Library → Word kinds: add, rename, reorder, delete (blocked while in use; `WORD` and `OTHER` can't be deleted). Reuse the labels list editor; extract a shared helper if the code would otherwise be copied.

**Done when**
- Add, look up, rename, change kind, edit Meaning and Notes, tags and delete all persist; word kinds editable in Settings.

---

## L5 · Snaps  ☐

**Read:** decisions §15 · `designs/snaps.dc.html`, `snaps.css`

**Build**
- Board: CSS columns (width 250px, gap 52px), dot grid, tilt by position (±1.8°), pinned first then newest edit, stacks spread through the rest (formula from the mockup), three empty states.
- Papers: scrap, slip, lined, plain, sticky. Sticky tint from the picked colour (hue kept, lightness 87% / band 79%, saturation ≤ 85%) as a tested util; the colour comes from `SwatchPicker`.
- Editor (340 × 300 for every paper): × / Esc, pin to top, delete (asks first), ✓ / ⌘↵; rotate, stack picker (indented by depth, "+ New stack"), colour dot (sticky only), inline tags. Esc closes an open picker first.
- Stacks: open with the path floating at the board's top left; hover tools rename inline ("Untitled stack" when empty) and remove (asks; contents move up).
- Drag with `trackPointer` + `liftGhost`: onto a stack moves in (it opens up), onto a snap makes a new stack named inline, onto a path step moves up; auto-scroll near the edges; a press under 5 px is a click; off while searching. Moves don't ask and show "Moved to …".
- Search: snap text and stack names across all stacks, each result labelled "in <path>".
- Without hover, the snap and stack tools are always visible.

**Done when**
- Every add, edit, pin, move, stack rename, stack removal and delete persists; the board matches the mockup.

---

## L6 · Library Home  ☐

**Read:** decisions §15 · `designs/library.dc.html`

**Build**
- Home first in the switch, reading real data: the counts line, five recent words (title + first line of Meaning), the four latest snaps on the mini board, five newest references. Each item opens in its area; "All →" / "Board →" switch area.
- Search on Home: one list grouped by area with counts; a result opens in its area.
- + on Home: the "what is this?" box. Guess as you type (a URL → References; one line of four words / 40 characters or fewer → Words; anything else → Snaps; util with tests), a Save-to switch overrides it, Continue (Enter) opens that area's add with the text in: a link starts its title fetch, a word its lookup, longer text lands on a snap paper. Shift+Enter adds a line.

**Done when**
- Each hand-off opens the right add with the text in and starts its fetch or lookup; Home and its search show live data.

**Deferred (not in L0–L6):** Undo on deletes and moves, fetching a title in the background after save, a full-text index, the tag filter, phone layouts, the dark board, drag on touch, links between Library items.

---

## After v1

- Polish pass: empty states, error toasts, loading states. (Window state restore is done.)
- Gaps and open questions from the 2026-10-09 check: `docs/open-questions.md`.
- Git sync: `docs/v1-sync-spec.md`.
- Packaging: macOS / Windows / Linux builds, code signing, auto-update.
- v2: calendar, events and reminders, see `docs/later/calendar-design-spec-doc.md` (a later migration step).
