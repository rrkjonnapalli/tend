# Tend: design decisions

Reference for building Tend (Tauri 2 + SvelteKit). The mockups in `docs/design/screens/` show the screens; this file has the rules behind them. Schema: `docs/schema.sql`. Times are INTEGER milliseconds (UTC); local dates and times stay `YYYY-MM-DD` / `HH:MM`.

---

## 1. Product

- **Tend**: notes, tasks and people in one desktop app, plus simple timers.
- **v1 scope:** Notes · Tasks · People · Timers. Calendar, events and reminders come later (§14, and `docs/later/calendar-design-spec-doc.md`).
- Light mode only for v1. Minimal, editor-like (Sublime / VS Code / JetBrains feel).
- No floating or detached panels, no shadows. Panels sit flush, separated by hairlines.
- Design system: **Clean Colors**.
  - Fonts: Montserrat (page and section titles), Lato (all UI text), JetBrains Mono (note editor, values, paths, keys, code). Tokens: `docs/design/tokens.css`.
  - Key colors:

    | Role | Hex |
    |---|---|
    | text | `#303841` |
    | muted | `#5f6b76` / `#7d8995` |
    | faint | `#b8c4ce` |
    | border | `#dce3e8` |
    | page | `#ffffff` (hover `#f7fbfc`, selected `#eef3f6`, chip `#f2f5f7`) |
    | primary | `#1259d3` (hover `#113eb0`, soft `#dbfaff`) |
    | accent | `#6f5ee4` (soft `#ede9ff`) |
    | success | `#1e7a55` |
    | warning | `#a84f2c` |
    | danger | `#da0037` (soft `#ffebeb`) |
- **App icon:** a lowercase t whose crossbar grows into a leaf, white on a violet (`#6f5ee4`) rounded square. Files in `docs/design/icon/`: `icon.svg` (source), `icon-1024.png` (input for `npx tauri icon`), `mark.svg` (one-colour glyph, `currentColor`, for favicon and in-app use).

## 2. Vault

The user picks one **vault folder**. Tend works only inside it:

```
<vault>/
  notes/          root of all notes (used if present, created if missing)
  .attachments/   attachment files (created if missing)
  .tend/tend.db   everything else (created if missing; hidden, like .git)
```

- **Opening a vault:** check or create the three items, in that order. Two folders ask first and write nothing until the user agrees:
  - **Not a vault and not empty** (no `.tend/tend.db`, no `notes/`, other visible items): "This folder isn’t a Tend vault and already has N items…" → *Use this folder* / *Choose another folder*.
  - **A vault whose `notes/` is gone** while the database lists notes: "`notes/` is missing… Was it moved?" → *Choose another folder* / *Start without them*. On launch this shows as the reopen error.
  - A folder with `notes/` but no database is opened directly and indexed. What sits above the chosen folder is never checked.
  - **`.tend/tend.db` exists:** read `meta`.
    - `app_id = 'tend'` and `schema_version` ≤ app → migrate up if lower, then open.
    - Anything else → **"An incompatible database exists in .tend/tend.db"**. The user picks another folder.
- **Existing notes** are never moved or renamed.
- **Changing vault:** Settings → General → *Change vault folder…*. This reopens the start-up flow.

## 3. Start-up page

- Shown when no vault is set.
- **Left column:**
  - The `tend.` wordmark.
  - *Start* → **Choose vault folder…** as a text link.
  - Three lines explaining `notes/`, `.attachments/` and `.tend/tend.db`.
- **After choosing:**
  - The path is shown with a *Change* link.
  - A checklist runs line by line (✓ green, ✕ red).
  - It ends in either **Open vault →** or the incompatible-db error with *Choose another folder*.
- **Right column:** ⌘K examples and a few shortcuts.
- No bottom nav, no recent vaults, no "create vault" (choosing an empty folder creates one).

## 4. App shell

- **Explorer** (left, 232px, collapsible with a small `<` / `>` caret):
  - Shown in Notes and Tasks only.
  - Toolbar icons: new item · new folder · reload · collapse all.
  - Hovering or selecting highlights the full row; the selection is subtle.
- **Bottom nav:**
  - A full-width hairline, with icons centred below it and no container.
  - The active section is **violet** with a small dot under it.
  - One caret sits on the line: `⌄` hides the nav, and it becomes `^` at the bottom edge to show it again.
  - **Sections, centred:** Notes (doc) · Tasks (checkbox) · People (person).
  - **Right side, in order:** *Timer chip* (only while timers run) · Settings (gear) · Feedback.
- **Timer chip:**
  - A violet pill showing the soonest countdown, e.g. `⏱ 4:32`, plus `+1` when more timers run.
  - Hidden when nothing is running.
  - Click → popover listing every timer: title · live countdown · **+1m** · cancel ✕ (red).
- **No bell in v1.** It comes back with reminders.
- **Feedback:**
  - A popover with Bug / Idea / Other, a text area and *Send* (⌘/Ctrl+Enter also sends).
  - Stored in the `feedback` table. Confirmation: "Thanks, saved."
- **Page bar:** every page starts with one 40px row on the same line as the explorer's icon toolbar: back (detail pages) · breadcrumb · a short count ("12 tasks", "5 notes · 3 folders") · actions on the right. List pages have no big title.
- **List pages** (notes, tasks, people): under the bar, a search line with the field at the right end (filters names, titles and paths as you type); tasks add the views line under it; then the add field, which takes focus when the page opens, then a column header (Title · My day · Subs · Due · Who · Folder; Name · Modified · Folder; Name · Birthday · Phone · Email). Everything down to the header stays put; only the rows scroll. The board has no column header; up to 5 status columns share the full width, more scroll sideways at a fifth of the width each (never narrower than 200px). Drag order is off while a task search is typed. A person just added opens with First name focused.
- **Subtasks in lists and on the board** carry a tiny green "s" badge before the title; hovering it reads "Subtask".
- **List rows** (tasks, notes): a left part (status dot · type badge · title) and a right part of fixed-width, left-aligned columns (`--col-*` in tokens.css, 6px apart): edit 24 · My day 64 · subtask count 36 · due 64 · assignee 32 · path 80 (notes: edit · modified · path). The path is relative to the open folder, cut off with … and shows a tooltip only when cut.
- **Edit in place:** task rows, note rows and subtask rows show a pencil on hover, first in the right-hand part of the row. It turns the title into a field: Enter or leaving saves, Esc cancels. Every in-place edit, explorer rename included, uses the same soft-fill field (no border).
- **No autocorrect** (nor spellcheck, auto-capitalise, autofill) in any input or textarea; the markdown editors have it off too.
- **Assignee in task rows:** the person's initials, or a gray "U" (hover: Unassigned); click opens a people search to assign, with Unassign at the bottom. Subtask rows on a task page show it too, after My day.
- **Labels** (schema v2, `src-tauri/src/db/migrations/002_task_labels.sql`): a fixed list in Settings → Tasks → Labels (name + color, drag to order, rename, recolor; deleting one in use asks, then takes it off those tasks). A task (or subtask) carries any number, picked on the task page from that list only. Board cards show them as colored dots (up to 4, then +N; hover names them); list rows don't. Tags stay free-typed and shared with notes.
- **Colors:** 20 swatches (tokens.css `--sw-*`, `util/swatches.ts`) shared by statuses and labels, plus any color: the system color picker or a typed #rrggbb (Enter applies).
- **Status dot** in task list rows and subtask rows opens the status list to change it in place (trial; may go).
- **Width toggle:**
  - On note and task pages, the top section (breadcrumb, tags) is full width.
  - Content is centred at **760px**, with an icon to switch to full width.
  - One setting for notes and one for tasks.
  - Lists and the board are always full width.
- **Popovers** stay inside the visible area: they flip up near the bottom and never grow the page.

## 5. Notes

- **The files in `notes/` are the truth.** A note is a record (`notes`: id) whose body is a file held by an attachment (`attachments.path` = `notes/<folder>/<name>.md`). `note_folders` indexes the folders by path.
- **A note's title is its file name** without `.md` ("Q4 plan – draft.md" shows as "Q4 plan – draft"). Renaming a note renames the file; renaming the file outside Tend renames the note. The body never sets the title.
- **Sync:**
  1. **Vault open:** walk `notes/` once. Insert or update every folder in `note_folders` and every `.md` file in `notes`. Anything gone from disk gets `missing_at` and is cleaned up after a grace period.
  2. **While running:** a file watcher applies changes made outside Tend (Finder, other editors) the same way.
  3. **Inside Tend:** create, rename, move and delete write to disk first, then to the database.
  4. **If they ever disagree,** the next scan makes the database match the disk.
- **Rename a folder:** rename it on disk, then one update on `note_folders`; the notes follow automatically. Note ids never change, so tags and attachments follow too.
- The explorer, folder lists and search read **only from the database**. Empty folders show like any other folder.
- **Explorer:**
  - Folders and notes.
  - Caret = expand/collapse; folder name = open folder list; note = open note.
  - Collapsing never changes the current selection.
- **Folder list:**
  - Shows every note under that folder path (recursive).
  - Rows show name, relative path and last edited.
- **Note page:**
  - Sticky top bar: back · breadcrumb · width toggle · Read/Edit icon toggle (⌘R). ⌘E toggles the explorer. On the task list ⌘R switches list / board. ⌘L focuses the search on the notes, tasks and people lists.
  - Tag row.
  - Rendered markdown in Read mode; a full markdown editor in Edit mode.
  - New notes open in Edit mode.

## 5a. Folders (notes and tasks)

- **Separate trees:** notes folders and tasks folders are unrelated, even when they share a name.
- **Names** of every folder (notes and tasks) and every note:
  - Letters `A–Z a–z`, digits, spaces and `. , _ -` only; at most 200 characters.
  - Leading and trailing spaces and dots are trimmed when saved.
  - Unique within the folder, ignoring case (as macOS does). Folders clash only with folders, notes only with notes. A clash is not an error: the name gets a suffix, `Plan` → `Plan_1` → `Plan_2`, on create, rename and move.
  - A typed folder path takes the spelling of folders that already exist (`work/x` lands in `Work/x`).
  - Task titles are free text with no rule.
  - **Names found on disk that break the rule** (folders and `.md` files under `notes/`) are renamed to the closest valid name: accents become plain letters (`Café` → `Cafe`), other characters are dropped, spaces collapse, the result is trimmed, and a clash gets a suffix. A name nothing valid is left of (`日本.md`) is skipped and not indexed. Hidden (`.`) and non-`.md` files are never touched.
  - Renaming needs the vault's consent once (setting `notes.fix_names`). A new, empty vault has it from the start. Opening a folder that already has such names asks: *Rename and open →* or *Open without renaming*. Until then they are skipped and listed in Settings → General, each with the name it would get and a *Rename N items to fit* link.
  - Every rename is written to the activity log (`old → new`).
- **Explorer actions:** hover a row → `⋯`, or right-click, opens a menu: **Rename** (F2) · **Move to…** (⌘⇧M) · **Delete…** (⌫). The root has no menu.
  - **Rename:** edited in place in the row. Invalid or duplicate names turn red with a hint underneath. Enter saves, Esc cancels.
  - **New note / new folder** (toolbar): an inline row in the selected folder, checked against the same rules.
- **Move:** both ways are supported.
  - **Drag** any row onto a folder. Valid targets highlight blue; invalid ones (into itself, same folder, name clash) turn red. A small label follows the pointer.
  - **Move to…** opens a popup like quick create. The path field suggests folders and leaves out the item's own folder and its subfolders. A new path shows "will create …" and is created.
  - Moving a folder moves everything inside it. Moving a task moves its subtasks.
- **Delete:**
  - **Folder:** a dialog shows "It contains N notes and M folders. Everything inside is deleted from disk." You type the **folder name** to enable the red *Delete folder* button.
  - **Note:** a simple confirmation, with no typing.
  - **Tasks folders:** delete works the same way, with task counts.

## 6. Tasks

### Structure
- Task folders form one tree under a single renameable root that can't be deleted.
- Tasks live in folders and have a `path` (e.g. `work/tend_v0_1`). Renaming a folder rewrites the paths underneath it.
- Folders are sorted alphabetically, with no manual order.
- Subtasks are tasks with a `parent_id`. You reach them from the parent task, not the explorer.
- A task can be assigned to a person (`assignee_id`).
- Due-date range queries use `due_date BETWEEN ? AND ?` on an indexed `YYYY-MM-DD` column.
- A folder list shows all top-level tasks under that path (recursive), with subtask counts.

### Statuses (app-wide, Settings → Tasks)
- Each status has a fixed **value** (`NOT_STARTED`) and an editable **label** ("Not started", renameable e.g. to "Ready").
- Defaults:
  - `NOT_STARTED`: always **first**; new tasks start here.
  - `IN_PROGRESS`: can be placed anywhere between first and last.
  - `DONE`: always **last**; counts as finished (done tasks sink, Overdue skips them, subtask "2/5" counts).
- **Defaults:** can be renamed, never deleted.
- **Custom statuses:**
  - Label plus a value that is auto-filled from the label in UPPER_SNAKE_CASE and editable before saving.
  - Fixed once created.
  - Inserted before Done.
- **Blocked:** values `NOT_STARTED` / `IN_PROGRESS` / `DONE` (any letter case) and duplicate labels.
- **Deleting:** if any task uses the status → inline red error; otherwise it's deleted.
- **Color:** stored as a hex code (`#rrggbb`). The picker offers 10 palette swatches. The color is used for the status dot everywhere; Not started is drawn as a ring.
- **Settings row:**
  - drag handle (not on first or last) · color dot · label · value (grey) · delete · edit
  - In edit mode: delete · save ✓ · cancel ✕
  - The Statuses section is collapsible.

### Views
- **Nav:** My day · All · Overdue · Upcoming · [date or range picker], each with a count. With no view in the URL the list opens My day on the first visit of a day, then the view picked last that day.
- **List / Board** toggle at the right end of the nav.
- **My day:** tasks due today are included by default. +/− my day (blue/red) stores manual overrides per day. My day has an "add task" field like All; a task added there is added to today's My day.
- **Subtasks are one level deep:** a subtask's page has no Subtasks section, and Rust refuses a subtask under a subtask (SUBTASK_DEPTH).
- **Task page:** top-level tasks show a **Location** row (the folder path, e.g. `tasks / work / tend_v0_1`) that opens Move to…; subtasks don't, they live with their parent.
- **Ordering:**
  - One global **fractional order key** per task (`keyBetween`, base-62).
  - Drag-reordering in any view writes only the moved task's key.
  - Done tasks sink to the bottom.
- **Board:**
  - One column per status, in order, filling the full height.
  - Each column scrolls on its own.
  - Cards show title · due (amber if overdue) · subtask count · relative path.
  - Dragging a card moves it with the pointer and shows a dashed placeholder.
  - Dropping sets the status from the column and the order key from the position.
  - **+ add task** at the bottom of each column creates a task with that status in the current folder.

### Task page
- Back · clickable breadcrumb (folders → parent tasks) · width toggle · title.
- **Fields:**
  - **Status:** dropdown of the configured statuses, current one ticked.
  - **Assignee:** a person chip (initials + name, red ✕ to unassign) or *+ assign*, which opens a people search (↑↓, Enter). Board cards show the assignee's initials.
  - **Due:** date popover plus an inline h:mm AM/PM field.
  - **My day:** toggle.
  - **Tags:** inline editor with suggestions; red ✕ per tag; long tags cut off at about 120px.
- **Sections** (collapsible):
  - **Description:** rendered markdown; pencil → editor with ✕ (red) / ✓ (green).
  - **Subtasks.**
  - **Attachments.**
  - **Comments:** composer; each comment shows a pencil on hover and edits inline.

## 7. Tags
- One shared pool for notes and tasks (`tags`, `note_tags`, `task_tags`); names are case-insensitive.

## 8. Attachments

- **Upload:** copy to `.attachments/<id>.<ext>` and insert a row in `attachments` (original name, mime type, size, sha256).
- **Linking:** link to a task, note, comment or person through `attachment_links`. A note can also be linked to a task.
- The UI always refers to attachments by id, never by path.
- **Task page → Attachments:**
  - A dashed drop zone: "Drop files here or click to choose" (several files at once), plus *or link a note*, which opens a note search.
  - **Rows:** type icon (image / file / note in violet) · name · `PNG · 240 KB · 2 Oct` or `note · work/tend_v0_1`.
  - **On hover:** open · show in folder (files only) · remove (red ✕, removes the link only).

## 9. Timers

- **Start:** ⌘T or ⌘K `timer 10m …`. The popup takes `mm:ss` (minutes any length, seconds 0–59, total > 0; a bare number means minutes) and optional text.
- **Shown:** the timer chip in the bottom bar (§4) and its popover.
- **No background process.** The database only stores `{ title, ends_at, state }`, and the window does the work:
  1. **Countdown:** `ends_at − now` is recalculated every second, never counted down, so it stays correct after sleep or throttling.
  2. **Ends:** when `now ≥ ends_at`, show a desktop notification (Tauri notification plugin, called from the window), play a sound if enabled, and set `state = 'rung'`.
  3. **+1m / snooze:** update `ends_at`. **Cancel:** `state = 'cancelled'`.
  4. **App quit:** nothing fires while Tend is closed. On the next launch, running timers already past `ends_at` show once as "finished while Tend was closed". Minimised or tray is fine, since the window keeps running.
- Timers never appear in tasks or notes.

## 10. People

- Its own section in the bottom nav.
- **List:**
  - Search by name, phone or email.
  - `+ add person` (first and last name, ↵), which opens the new person.
  - Columns: Name · Birthday (date · "in 6 days" · "turns 31"; violet within 14 days) · Phone · Email.
- **Person page:**
  - Back · breadcrumb · delete.
  - Fields:
    - First name, Last name, Nickname (shown instead of the name).
    - Date of birth: `yyyy-mm-dd`, or `mm-dd` when the year is unknown. The hint shows the next birthday and age.
    - Time of birth (optional).
    - Birthday reminder: stored in the database (`birthday_reminder`) but **hidden in v1**. It's used when reminders arrive.
  - **Phone / Email / Address:** collapsible sections that work like Settings → Statuses.
    - Rows show value + label, with delete · edit; in edit mode, ✓ / ✕.
    - An add row (value + label) sits at the bottom.
  - **Notes** (markdown, with the same editor as a task description) and **Assigned tasks** (from `tasks.assignee_id`).
  - Sections sit close together like on the task page, with no counts in their headers.

## 11. Command palette (⌘K)

- Use the first design (grouped results), not the minimal variant.
- **Contents:** groups for Go to · Create · Commands, plus title search across notes and tasks.
- **Quick-create phrases:**
  - `task: …`
  - `note: …`
  - `timer 10m …`
  - `person: …`
- **Keys:** ↑↓ to move · ↵ to run · Esc to close.
- **Settings → Command palette:** a read-only list of every command with its example and shortcut.

## 12. Quick create shortcuts

| Shortcut | Popup | After Enter |
|---|---|---|
| ⌘N | New note: path + name | opens the note in Edit mode |
| ⌘⇧T | New task: path + name | opens the task page |
| ⌘⇧N | New folder: Notes/Tasks switch (defaults to the current section) + path + name | opens the folder list |
| ⌘T | New timer: `mm:ss` (default `01:00`, selected) + text | toast only |

- **Path field:**
  - Pre-filled with the last path used for that kind.
  - Suggests folders as you type; Tab or Enter accepts.
  - An unknown path shows "will create …" and is created.
- **Timer:**
  - Minutes ≥ 0 with no upper limit; seconds 0–59; total > 0.
  - A bare number means minutes.
- **Shortcuts:** ⌘1 Notes · ⌘2 Tasks · ⌘3 People · ⌘, Settings · ⌘K palette · ⌘D toggle the dock (bottom bar).

## 13. Settings

- Sections on the left: General · Notes · Tasks · People · Command palette.
- **General:** vault path as plain text, with a *Change vault folder…* link on the right. Below it, a quiet "N items skipped" line (files/folders whose names break the rule) that expands to the list.
- **Tasks:** statuses (see §6).
- Notes and People settings come later; their sections are empty in v1.

## 14. Out of scope for v1

- **Calendar, events and reminders**, including repeat rules, alerts, the notification engine, the bell, birthday and due-date alerts. The design is in `docs/later/calendar-design-spec-doc.md`, and mockups are in `docs/later/screens/`. They'll arrive as a schema migration (`schema_version` 2).

- Gantt / timeline view.
- Dark mode.
- Per-folder status lists (statuses are app-wide).
- Recent vaults list.
