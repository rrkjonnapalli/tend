# Tend: design decisions

Reference for building Tend (Tauri 2 + SvelteKit). The mockups in `docs/design/screens/` show the screens; this file has the rules behind them. Schema: `docs/schema.sql`. Times are INTEGER milliseconds (UTC); local dates and times stay `YYYY-MM-DD` / `HH:MM`.

---

## 1. Product

- **Tend**: notes, tasks, people and a Library (words, snaps, references) in one desktop app, plus simple timers.
- **v1 scope:** Notes · Tasks · People · Library (§15) · Timers. Calendar, events and reminders come later (§14, and `docs/later/calendar-design-spec-doc.md`).
- Light and dark themes (§2 *Theme*). Minimal, editor-like (Sublime / VS Code / JetBrains feel).
- No floating or detached panels, no shadows. Panels sit flush, separated by hairlines.
- Design system: **Clean Colors**.
  - Fonts: Montserrat (page and section titles), Lato (all UI text), JetBrains Mono (note editor, values, paths, keys, code, snap text). The Library's hands: Gaegu (sticky notes), Smile Moon (stack names; a file in `static/fonts`, personal-use licence to check before the app is shared), Caveat (the line on an empty area). Tokens: `docs/design/tokens.css`.
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

The user picks one **vault folder**. It is the **notes root**: its `.md` files and folders are the notes. Tend's own files live in a hidden `.tend/` folder:

```
<vault>/
  work/plan.md          notes and folders (the folder itself is the notes root)
  .tend/attachments/    attached files (created if missing)
  .tend/tend.db         everything else (created if missing)
```

- Hidden entries (`.tend/`, `.git/`, `.obsidian/`, `.DS_Store`) are never read, renamed or watched. Files other than `.md` are ignored, and so are folders with no notes in them (unless made in Tend).
- **Opening a vault:** count the notes, then check or create `.tend/attachments/` and `.tend/tend.db`.
  - **Empty folder** → a new vault.
  - **Not a vault and not empty** (no `.tend/tend.db`, other visible items) → ask first, writing nothing: "This folder isn’t a Tend vault yet. It has N items, with M notes inside. Tend will use it as your notes folder… and rename notes and folders that hold notes to lowercase names…" → *Use this folder* / *Choose another folder*. Agreeing also lets scans fix names (§5a).
  - **`.tend/tend.db` exists:** read `meta`.
    - `app_id = 'tend'` and `schema_version` ≤ app → migrate up if lower, then open.
    - Anything else → **"An incompatible database exists in .tend/tend.db"**. The user picks another folder.
  - What sits above the chosen folder is never checked.
- **Changing vault:** Settings → General → *Change vault folder…*. This reopens the start-up flow.
- **File size limit:** 30 MB by default, set in Settings → General → Files (1–90 MB; 0 isn't allowed, it would drop every note). Shared through sync (setting `files.max_mb`), so every device agrees; changing it rescans. A `.md` that size or larger is never indexed; a file that size can't be attached; sync never commits one. A note already indexed that grows to the limit keeps its row unchanged (never marked missing), so nothing syncs as a deletion.
- **Theme:** Settings → General → Appearance: *System* (default, follows macOS and changes with it), *Light* or *Dark*. Kept per device (setting `ui.theme`, never synced). The dark theme remaps the same token roles (`tokens.css`). ⌘K has *Toggle light / dark theme*, the first line when it opens: it switches to the one not showing, which turns *System* into a fixed choice.
- **Delete confirmations:** Settings → General → Confirmations: *Ask before deleting a note / a task / a person / a word / a snap / a reference*, each on by default. Turned off, that delete (page trash button, explorer menu or ⌫) goes at once. Kept per device (settings `confirm.delete.note|task|person|word|snap|reference`, never synced). Folders, stacks, statuses, labels and word kinds always ask.

## 3. Start-up page

- Shown when no vault is set.
- **Left column:**
  - The `tend.` wordmark.
  - *Start* → **Choose vault folder…** as a text link.
  - Three lines explaining the folder (your notes), `.tend/attachments/` and `.tend/tend.db`.
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
  - **Sections, centred:** Notes (doc) · Tasks (checkbox) · People (person) · Library (books, §15).
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
- **List pages** (notes, tasks, people): under the bar, a search line with the field at the right end (filters names, titles and paths as you type); tasks add the views line under it; then the add field, which takes focus when the page opens (not when a view switch brings it back), then a column header (Title · My day · Subs · Due · Who · Folder; Name · Modified · Folder; Name · Birthday · Phone · Email). Everything down to the header stays put; only the rows scroll. The board has no column header; up to 5 status columns share the full width, more scroll sideways at a fifth of the width each (never narrower than 200px). Drag order is off while a task search is typed. A person just added opens with First name focused.
- **Subtasks in lists and on the board** carry a tiny green "s" badge before the title; hovering it reads "Subtask".
- **List rows** (tasks, notes): a left part (status dot · type badge · title) and a right part of fixed-width, left-aligned columns (`--col-*` in tokens.css, 6px apart): edit 24 · My day 64 · subtask count 36 · due 64 · assignee 32 · path 80 (notes: edit · modified · path). The path is relative to the open folder, cut off with … and shows a tooltip only when cut.
- **Edit in place:** task rows, note rows and subtask rows show a pencil on hover, first in the right-hand part of the row. It turns the title into a field: Enter or leaving saves, Esc cancels. Every in-place edit, explorer rename included, uses the same soft-fill field (no border).
- **No autocorrect** (nor spellcheck, auto-capitalise, autofill) in any input or textarea; the markdown editors have it off too.
- **Assignee in task rows:** the person's initials, or a gray "U" (hover: Unassigned); click opens a people search to assign, with Unassign at the bottom. Subtask rows on a task page show it too, after My day.
- **Labels:** a fixed list in Settings → Tasks → Labels (name + color, drag to order, rename, recolor; a label has a value, like a status: typed when it is added (it starts as the name in capitals, `DB Script` → `DB_SCRIPT`, and can be edited then); a value another label has is refused with a request to change it; it is shown and stored in capitals (a `TASK_LABEL` row in `tend_enums`, §16) and never changes; deleting one always asks; one in use says it comes off those tasks). A task (or subtask) carries any number, picked on the task page from that list only. Board cards show them as colored dots (up to 4, then +N; hover names them); list rows don't. Tags stay free-typed and shared with notes.
- **Colors:** 20 swatches (tokens.css `--sw-*`, `util/swatches.ts`) shared by statuses and labels, plus any color: the system color picker or a typed #rrggbb (Enter applies).
- **Status dot** in task list rows and subtask rows opens the status list to change it in place (trial; may go).
- **Width toggle:**
  - On note and task pages, the top section (breadcrumb, tags) is full width.
  - Content is centred at **760px**, with an icon to switch to full width.
  - One setting for notes and one for tasks.
  - Lists and the board are always full width.
- **Popovers** stay inside the visible area: they flip up near the bottom and never grow the page.

## 5. Notes

- **The files in the vault are the truth.** A note is a record (`notes`: id) whose body is a file held by an attachment (`attachments.path` = `<folder>/<name>.md`, relative to the vault). `note_folders` indexes the folders by path.
- **A note is shown by its file name** (without `.md`) in the explorer, lists, ⌘K and breadcrumbs. **The title is part of the file:** its first `# heading`; the database keeps no copy.
- **Creating** "Meeting" makes `meeting.md`, whose first line is `# Meeting`; nothing is renamed afterwards because the name was right from the start. **Renaming** a note renames its file to the slug of what was typed (a clash gets `_1`); the body, heading included, is untouched.
- **Sync:**
  1. **Vault open:** walk the vault once. Insert or update every folder in `note_folders` and every `.md` file in `notes`. Anything gone from disk gets `missing_at` and is cleaned up after a grace period.
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
- **An open note or task deleted elsewhere** is not dropped while it is on screen. Nothing polls for this: whenever data reloads (a sync, or for notes the file watcher), the open page checks that its item came back. If it didn't, the page keeps showing it and asks *"This note/task was deleted. Keep it as a new one?"*
  - **Keep** recreates it with a new id. A note keeps its folder, name, text (unsaved edits included) and tags. A task keeps its folder, parent (if still there), title, status, due date and time, assignee (if still there), description, tags and labels; its subtasks, comments and attachment links went with the old one.
  - **Discard** goes back to the folder. The deletion itself stands, so sync's delete-wins rule holds for the old id.
- **Note page:**
  - Sticky top bar: back · breadcrumb · width toggle · Read/Edit icon toggle (⌘R). ⌘E toggles the explorer. On the task list ⌘R switches list / board. ⌘L focuses the search on the notes, tasks and people lists.
  - Tag row.
  - Rendered markdown in Read mode; a full markdown editor in Edit mode.
  - New notes open in Edit mode.

## 5a. Folders (notes and tasks)

- **Separate trees:** notes folders and tasks folders are unrelated, even when they share a name.
- **Names** of every folder (notes and tasks) and every note file: `^[a-z0-9]+(_[a-z0-9]+)*$`, at most 200 characters.
  - Anything typed becomes its **slug**: accents become plain letters, everything lowercase, each run of other characters one `_`, trimmed ("Café notes (Q4)" → `cafe_notes_q4`). A note title with no letter or digit becomes `note`; a folder name with none is refused ("Use at least one letter or digit").
  - Unique within the folder. Folders clash only with folders, notes only with notes. A clash is not an error: the name gets a suffix, `plan` → `plan_1` → `plan_2`, on create, rename and move.
  - Task titles and note titles (the heading inside a note) are free text.
  - **Names found on disk that break the rule** (folders holding notes, and `.md` files) get their slug, with a suffix when taken. The old path is kept in `path_log`.
  - Fixing needs the vault's consent once (setting `notes.fix_names`): a new empty vault has it, confirming a non-vault folder gives it, and an existing vault asks on the Start page (*Rename and open →* / *Open without renaming*). Until then such items are skipped and listed in Settings → General with the name each would get and a *Rename N items to fit* link.
  - **Every rename or move** of a folder or note file, by the user or by a fix, is written to `path_log` (old path, new path, why, when), so an earlier layout can be restored later.
- **Explorer actions:** hover a row → `⋯`, or right-click, opens a menu: **Rename** (F2) · **Move to…** (⌘⇧M) · **Delete…** (⌫), on the row with the keyboard cursor or else the open item. The root has no menu.
  - **Rename:** edited in place in the row. Invalid or duplicate names turn red with a hint underneath. Enter saves, Esc cancels.
  - **New note / new folder** (toolbar): an inline row in the selected folder, checked against the same rules.
- **Move:** both ways are supported.
  - **Drag** any row onto a folder. Valid targets highlight blue; invalid ones (into itself, same folder, name clash) turn red. A small label follows the pointer.
  - **Move to…** opens a popup like quick create. The path field suggests folders and leaves out the item's own folder and its subfolders. A new path shows "will create …" and is created.
  - Moving a folder moves everything inside it. Moving a task moves its subtasks.
- **Delete:**
  - **Folder:** a dialog shows "It contains N notes and M folders. Everything inside is deleted from disk." You type the **folder name** to enable the red *Delete folder* button.
  - **Note:** a simple confirmation, with no typing (can be turned off, see *Delete confirmations*).
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
- Statuses are `TASK_STATUS` rows in `tend_enums` (§16). A task can only be given a status that is in the list and not blocked; anything else is refused (`NOT_FOUND`, `ENUM_BLOCKED`).
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
- **Deleting:** always asks first; then, if any task uses the status → inline red error; otherwise it's deleted.
- **Color:** stored as a hex code (`#rrggbb`). The picker offers 10 palette swatches. The color is used for the status dot everywhere; Not started is drawn as a ring.
- **Settings row:**
  - drag handle (not on first or last) · color dot · label · value (grey) · delete · edit
  - In edit mode: delete · save ✓ · cancel ✕
  - The Statuses section is collapsible.

### Views
- **Nav:** My day · All · Overdue · Upcoming · [date or range picker], each with a count; at the right end, [hide done] then the List/Board switch. With no view in the URL the list opens My day on the first visit of a day, then the view picked last that day.
- **Views** (all limited to the open folder and below; done tasks sink):
  - **My day:** added by hand, or due today and not removed by hand.
  - **All:** top-level tasks (subtasks show on their parent).
  - **Overdue:** not done, due before today. Never shows Done tasks.
  - **Upcoming:** due today or later.
  - **Date range:** due within the range, both ends included.
  - Tasks without a due date show only in All (or in My day when added by hand).
- **Hide done** (icon at the right end of the views line, before List/Board; `tasks.hide_done`): leaves Done tasks out of every view, the counts and the board.
- **List / Board** toggle at the right end of the nav.
- **My day:** tasks due today are included by default. +/− my day (blue/red) stores manual overrides per day. My day has an "add task" field like All; a task added there is due today and added to today's My day.
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
- One shared pool for notes, tasks, words, snaps and references: `TAG` rows in `tend_enums`, linked through `tend_links` (§16).
- **A tag is its name**, under the name rule (`^[a-z0-9]+(_[a-z0-9]+)*$`). What is typed becomes its slug ("UI Design" → `ui_design`, "C++" → `c`), the same on every device. The slug is the tag's label; its value is the slug in capitals (`UI_DESIGN`). Tags are never renamed.
- **One editor everywhere** (`TagEditor`): chips with a red ✕ and an inline field whose list suggests the pool's active tags and offers *Create “x”* for a new one (↑↓ move, Enter, comma or Tab add, Esc closes). The list floats above everything, so a dialog or the snap editor can't cut it off. Notes, tasks and saved words save each change at once and show the tags the save answered with; a new word, the reference popup and the snap editor keep them until Save.
- **Blocked tags:** a tag is active or blocked. Blocking one (Settings → Tags, §13) takes it off everything that carries it, and it can't be added again until it is unblocked: it is never suggested, and typed by hand the list says it is blocked and the chips stay as they were. The backend refuses it too (`TAG_BLOCKED`, for a tag blocked on another device since the pool was loaded): nothing is added, the message is shown, and the pool is loaded again. When that happens on Save, the word or reference is saved without the new tags. Unblocking only makes the tag usable again; it doesn't put it back on anything.
- **Deleting a tag** (Settings → Tags, §13; command `tag_delete`) removes its `tend_enums` row and, by trigger, every link to it. Nothing blocks it from being typed again, which makes a new tag.

## 8. Attachments

- **Upload:** copy to `.tend/attachments/<id>.<ext>` and insert a row in `attachments` (original name, mime type, size, sha256).
- **Linking:** link to a task, note, comment or person through `tend_links` (§16, target type `ATTACHMENT`). A note can also be linked to a task.
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
    - **Born:** the date and the time of birth side by side, the same date and time fields as a task's due date. The date shows as *12 May 1990*, or *12 May* when its year was left unassigned. The hint shows the next birthday and age. *Clear*, centred under the days of a single-day picker, removes the date (a due date too).
    - **The date picker** (due date, date of birth, the tasks range) has three rows: the year between two arrows, the month between two arrows, then that month's days. The year is also a dropdown (type to find one, or scroll): the years back to 1900 for a date of birth, last year to ten years on for the others. For a date of birth the year starts *Unassigned*: the days are then plain numbers with no weekdays, and a pick keeps only the day and month.
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
| ⌘S | New snap: the snap editor, the same one the Snaps board opens, over any page. Pressed again while it is open, it turns to the next paper (as its rotate button does) and keeps everything written | toast only; the snap goes in the stack the board shows, else loose on the board |

- **Path field:**
  - Pre-filled with the last path used for that kind.
  - Suggests folders as you type; Tab or Enter accepts.
  - An unknown path shows "will create …" and is created.
- **Timer:**
  - Minutes ≥ 0 with no upper limit; seconds 0–59; total > 0.
  - A bare number means minutes.
- **Shortcuts:** ⌘1 Notes · ⌘2 Tasks · ⌘3 People · ⌘4 Library · ⌘, Settings · ⌘K palette · ⌘D toggle the dock (bottom bar).
- **Dropdowns:** every list of choices in a popover takes ↑↓ (and Home / End), ↵ picks (Space too, where it ticks), Esc closes; ↓ from the button that opened it enters the list. The choice with the cursor has the cursor fill, like a list row. When a dropdown closes with the focus inside, the focus returns to what opened it. A picker with a search field (people, a note to link, a word's kind) filters as you type, and only a listed item can be picked.
- **Keyboard focus** is never an outline or ring. A focused button or link gets the hover fill, a list row the cursor fill, a field its border colour, a filled button its hover colour.
- **Arrow keys across a page (zones):** a list page is a stack of zones: page bar · (task views) · search · add field · rows (or the board). ↑ / ↓ leave a zone for the one above or below (a list is left with ↑ only from its first row; empty zones are skipped). ← / → move along a horizontal zone (page bar, task views); moving onto a task view switches to it, like the Settings sections. Fields keep ←→ for typing. **⌘← / ⌘→** jump between the side pane (the explorer, opened if hidden, or the Settings sections) and the page, returning to where you were. On a row, ← / → step through its own controls (edit, My day, assignee…), Enter opens, F2 renames. Task and person pages are zones too: page bar · title · fields (↑↓ between lines, ←→ along a line) · each section's head (←→ to its edit pencil) · its body (subtasks, contacts, attachments and comments as rows; the add fields and an open editor as fields). A multi-line field is left with ↑ / ↓ only from its first or last character; the Markdown editor keeps all its arrows. Space presses whatever has focus, like Enter (rows, links, the dock, board cards). **⌘⇧↓** syncs now (when the vault is connected; not inside a field, where it selects). **⌘↓** goes to the dock (opening it if hidden) on the current section; ← / → move along it, Enter opens, ↑ / Esc / ⌘↑ return to the page. **⌘[ / ⌘]** go back and forward (Back falls back to the parent page when there is no history).
- **Lists and the explorer:** ↓ from the search or add field enters the list · ↑↓ Home End move the cursor (a slightly darker row) · ↵ opens · F2 renames · → / ← open and close a folder in the explorer. The explorer's F2, ⌘⇧M and ⌫ act on the row with the cursor, else on the open item. On the task board: ↑↓ within a column, ←→ across columns (skipping empty ones), ⇧← ⇧→ move the card to the previous or next status (to the end of that column), ↵ opens. The Settings sections, a task's subtasks and a person's tasks take ↑↓ too.

## 13. Settings

- Sections on the left: General · Tasks · Library · Tags · Sync · Command palette.
- **Library:** the dictionary to look words up in (a field-sized dropdown) and word kinds (§15).
- **Tags:** a search line at the top right, like the list pages, filters both columns by name. Every tag in two columns, **Active** and **Blocked**, by name; each row shows the tag and how many things use it. An active row has *Block*, a blocked row *Unblock*. Block asks first (*Block “name”?* — it comes off N things and can't be added again until unblocked); Unblock doesn't ask. Each row also has a delete icon: it always asks (*Delete “name”?* — it comes off N things), then removes the tag from the pool and from everything that carries it; unlike a blocked tag it can be typed again later. Each of the three reloads the tag pool and every list that shows tags (§7).
- **General:** vault path as plain text, with a *Change vault folder…* link on the right. Below it, a quiet "N items skipped" line (files/folders whose names break the rule) that expands to the list.
- **Tasks:** statuses (see §6).
- Notes and People settings come later; they have no section yet.

## 14. Out of scope for v1

- **Calendar, events and reminders**, including repeat rules, alerts, the notification engine, the bell, birthday and due-date alerts. The design is in `docs/later/calendar-design-spec-doc.md`, and mockups are in `docs/later/screens/`. They'll arrive as a schema migration (`schema_version` 2).

- Gantt / timeline view.
- Per-folder status lists (statuses are app-wide).
- Recent vaults list.

## 15. Library (Words, Snaps, References)

The full behaviour is in `docs/tend-library-plan.md` and the mockups in `docs/tend-library/designs/`. This section records where Tend differs from them; where they disagree, this section wins.

- **Tab:** the fourth bottom-nav section, Library (⌘4). A bar on top switches Home · Words · Snaps · References, holds the + button and a search scoped to the area. Library reopens on the last area used (setting `ui.library.area`, per device).
- **Inputs:** every Library input keeps the text exactly as typed (no autocorrect, spellcheck, auto-capitalise or smart punctuation), except a `WORD` title, which is saved in lowercase.
- **Word kinds:** `WORD_KIND` rows in `tend_enums`, edited in Settings → Library → Word kinds (add, rename, reorder, delete). Defaults, in order: word, idiom, abbr, concept, phrase, term, other. `WORD`, `IDIOM` and `PHRASE` look things up in the dictionary; custom kinds don't. `WORD` and `OTHER` can't be deleted; a kind in use can't be deleted.
- **Word titles** are unique by exact match across all kinds. A `WORD` title is lowercased on save; other kinds keep their case, so `it` (word) and `IT` (abbr) are different entries. Changing a saved entry's kind to `WORD` lowercases its title and is refused if that title already exists.
- **Kind guess** while adding, until a kind is picked by hand: all capitals (two characters or more) and no spaces guesses abbr (`NASA`, but not `Petrichor`), no spaces guesses word, spaces guess phrase; a dictionary hit marked idiom switches to idiom.
- **Words list:** grouped by kind in kind order, always open; no random or collapse buttons. ↑↓ move through it and the entry on the right follows the cursor at once (no Enter, no new step in history); a click opens a word as its own step. While a new word is being typed the arrows don't switch words.
- **Meaning and Notes** render the markdown as written; Tend adds no link syntax of its own. Each opens for writing with its pencil or a double-click, empty or not, like every markdown field (a task's description, a person's notes, a comment, a note's body); a single click never opens one.
- **A word's kind** is picked like a person: the kind chip opens a search over the kinds (type to filter, ↑↓, ↵), with the current one ticked. The chip and the Settings dictionary dropdown are one component (`ChoicePicker`; the Settings one is field-sized, with no search).
- **Dictionary sources:** two, picked in Settings → Library → Dictionary (a plain dropdown, no search; setting `words.dictionary_source`, per device; default freedictionaryapi.com): **freedictionaryapi.com** (current Wiktionary: idioms, more synonyms and antonyms, no recordings; CC BY-SA 4.0) and **dictionaryapi.dev** (has recordings). Each has its own file under `src-tauri/src/words/dictionary/` with its URL, reply shape and mapper to Tend's `Dictionary`; adding a source is one file and one enum value. A stored entry records its `source` and, when given, its `sourceUrl` (the entry's own page, opened from the Dictionary section: the credit Wiktionary's licence asks for) and its antonyms. A fetched word keeps its entry when the setting changes; *Fetch again* asks the source now picked. An entry stored before sources were recorded counts as dictionaryapi.dev.
- **Pronunciation:** the dictionary's audio URL is stored with the entry and played from it (CSP `media-src https:`).
- **Tags:** the app's shared tag pool and name rule (§7), not the spec's 24-character rule. The inline chip input from the mockups is `TagEditor`'s inline style.
- **Tag filter:** none. The mockups only show it on standalone pages, which Tend doesn't have; search covers it.
- **Deletes ask** (§2 *Delete confirmations*); removing a stack always asks. Moves (drag or the stack picker) don't ask and show "Moved to …". Undo comes later.
- **The snap editor is app-wide** (`SnapDialogs`, shown by the root layout): ⌘S opens a blank paper on any page, and the board opens the same one for a new or an existing snap. A snap written elsewhere lands on the board: in the stack the board shows, else loose; a new stack made that way stays untitled until it is named on the board. ⌘S again while it is open turns to the next paper. The stack picker is a search over the stacks by path (type to filter, ↑↓, ↵; "Snaps (loose)" and "+ New stack" first); text that matches no stack picks nothing.
- **Dark theme on the board:** the board and the scrap, slip, lined and plain papers use the app's dark greys with light ink. A sticky stays a bright sheet with dark ink, a little dimmer, in both themes, and so does everything on it (the editor's buttons, the board's hover tools): it is marked `data-surface="light"`, which gives it the light colour roles (`tokens.css`).
- **Look:** the board, papers, stacks and the drag ghost keep their shadows. Popovers, dialogs, the segmented switch and toasts follow the app: border only, no shadow. Fields show focus with their border, never a ring (§12).
- **Stacks** have their own table (`snap_stacks`: parent id + free-text name), not the path-based folder model. A stack can't go inside itself or its sub-stacks; removing one moves its contents up one level.
- **Title fetch and dictionary lookup** run in Rust (`ureq` on a worker thread) only while the popup or entry page asks for them. Nothing fetches in the background.
- **Search** is substring matching over the loaded lists; there is no text index.

## 16. Enums and links

- **`tend_enums`** holds every list the user can change: `TASK_STATUS`, `TASK_LABEL`, `WORD_KIND`, `TAG`. Columns: `id` (`<KIND>:<VALUE>`, the same on every device, so a tag made on two devices syncs as one row), `kind`, `value`, `label`, `is_default`, `status` (`ACTIVE` or `BLOCKED`), `meta` (JSON: colour, position, lookup, …; sync merges it key by key, so a recolor and a reorder on two devices both survive), `created_at`, `updated_at`; `UNIQUE (kind, value)`.
  - Kinds and values are SCREAMING_SNAKE_CASE (the name rule in capitals). A value never changes; the label is what the user sees and renames.
  - `meta` is checked in the app, not in SQL. `is_default` rows can't be deleted.
  - Order within a kind is `meta.position`, then value; a row without a position sorts last.
  - **Blocked rows** (every kind): a `BLOCKED` row stays in its list but can't be put on anything new. Linking it is refused (`ENUM_BLOCKED`; for a tag `TAG_BLOCKED`), and so is giving a task a blocked status or a word a blocked kind. Blocking a row removes every link to it in the same transaction; unblocking restores none. Only tags can be blocked from the UI so far (§7): `tags_all` lists the active tags, `tags_overview` every tag with its status and how many records carry it, `tag_set_status` blocks or unblocks one.
- **`tend_links`** joins any record to an enum or an attachment: `id`, `source_type`, `source_id`, `target_type`, `target_kind`, `target_value`, `created_at`, `updated_at`; `UNIQUE (source_type, source_id, target_type, target_kind, target_value)`.
  - `target_type` `ENUM`: `target_kind` is the enum kind, `target_value` the enum value.
  - `target_type` `ATTACHMENT`: `target_kind` is `''` (not NULL, so the unique key holds), `target_value` the attachment id.
  - Sources: `TASK`, `NOTE`, `COMMENT`, `PERSON`, `WORD`, `SNAP`, `REF`.
- **Table names** live in one place too (`src-tauri/src/db/tables.rs`), with the column names that are passed around as data (`db/columns.rs`): sync rules, snapshot keys and SQL built at run time take them from there.
- **Code enums, not rows:** `TEND_SOURCE_TYPE`, `TEND_TARGET_TYPE` and `TEND_TARGET_KIND` change only with a release, so they live in code: `src-tauri/src/enums/` and `src/lib/enums/*.enum.ts`, same values on both sides. No enum kind, value or type is typed inline anywhere else.
- **A row that can't be read** (its `meta` isn't a JSON object, which only a sync can bring) is left out of its list and logged; the other rows still load.
- **Integrity in the app:** an enum in use can't be deleted where the rules say so (statuses, word kinds); the check is not atomic. Triggers remove links when their source row, attachment or enum row is deleted. Deleting a label or tag removes it from everything that carries it.
- **Sync:** duplicates by `(kind, value)` and by the link key keep the smaller id (v1-sync-spec §9.2). `status` syncs like any other column; after a merge, links whose enum row is blocked are dropped (a tag blocked on one device while another device put it on something).
