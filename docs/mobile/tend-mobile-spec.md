# Tend mobile: spec and build notes

Status: design settled 2026-11-05. Mockups live on the design canvas, page **Mobile** (rows "B · …" and "Corner menu button · chosen"). Screenshots are in `screens/`, the mockup sources in `design/`. Where a screen and this text disagree, the text wins (the vault folder's *Change*, the font lists, the editor font, the conflict dialog and the tag filter changed after the screens were drawn).

The phone app is the same Tend as the desktop app: same vault, same sync, same data, same tokens (`docs/design/tokens.css`; `design/tend-app.css` is only the mockups' stylesheet). This doc only covers what is different on a phone, plus three changes that apply to **both** apps (marked **Both apps**).

## 1. Scope

- Sections: Notes · Tasks · People · Library (Words, Snaps, References) · Settings.
- Not on mobile v1: command palette, timers (parked; to be discussed), calendar/reminders (later, as on desktop), assigned tasks on the person page, copy (only cut).
- Phone frame for the mockups: 390 × 844. Touch targets 44 px minimum; small controls keep their look and get a 44 px hit area.

## 2. Fonts (Both apps)

Two font settings in Settings › Appearance (mobile) and Settings › General › Fonts (desktop):

| Setting | Applies to | Default |
|---|---|---|
| App font | Everything that isn't an editor or a description: the explorer, lists, page bars, fields, settings | Lato (the app's current UI font) |
| Editor font | Every editor and every description, in view and in edit alike: note view and note editor, task descriptions, word meanings and notes, person notes, comments | Work Sans (today's note view font) |

- **The note view and the note editor use the same font.** The editor is no longer monospace.
- Both pick from the same list as today's reading font (`decisions.md` §2): the app's own font first, then the font families installed on the device. No new bundled fonts; the font options in the mockups are not used.
- The Editor font replaces the desktop's *Reading font* setting. Both settings are kept per device, never synced (`fonts.app`, `fonts.editor`, `fonts.size`).
- Snaps never change: their paper fonts (Gaegu, Caveat, the mono slip) stay as they are.
- Text size: Small · Default · Large (desktop segmented control; mobile slider with 5 steps, Default follows the phone's text size).
- Code, paths and other monospace values keep JetBrains Mono regardless of the editor font.
- Implement as CSS variables (`docs/design/tokens.css`): the App font sets `--font-ui` and `--font-heading`; the Editor font sets `--font-note` (renamed `--font-editor`), used by `--text-editor*`, `.md.note`, `.md.field`, description and comment bodies. Snap styles never read these.

## 3. Due replaces Overdue (Both apps)

- Task views are **My day · All · Due · Upcoming**.
- **Due** = open tasks due on or before a date, overdue included. The date defaults to today. On the phone the calendar icon next to the tabs picks the date. On desktop Due is up to today, and the existing date pill keeps picking a date or range as its own view.
- Desktop tab label changes from "Overdue" to "Due".

## 4. Menu: the corner button

There is no tab bar or dock on the phone. Navigation is one round button in a bottom corner.

- **Closed:** a 56 px round button attached to the screen edge, about a third tucked off-screen. It shows the current section's icon (Notes, Tasks, People, Library, or the gear in Settings) in violet.
- **Open:** the button grows out of the corner into the menu, as one shape. Animate size from the corner (≈300 ms, ease-out); the icon rotates and cross-fades into ×, which stays exactly where the button was. Items fade in just after the shape starts growing. The rest of the screen gets a light veil.
- **Menu contents:** Notes · Tasks · People · Library as text rows with icons (current one highlighted). Bottom row: × at the button's position; Settings and Sync as icons at the other end, with **Sync at the outer end** (the far end from ×). Sync shows the last sync time ("9:42").
- **Closing:** tap ×, any item, or the veil.
- **Side:** Settings › Appearance › Menu button: Left / Right (default Right). The whole thing mirrors.
- **Hidden** while the keyboard is up, while items are selected (the action bar takes the bottom) and while pasting (the paste bar takes the bottom).
- Lists get bottom padding (≈96 px) so the last row can scroll clear of the button.

### Sync state on the button

| State | Closed button | Sync icon in the open menu | Tap Sync |
|---|---|---|---|
| Idle | no dot | grey, with last time | sync now |
| Syncing | violet dot | violet | nothing |
| Failed | red dot | red | opens Settings › Sync with the error |
| Needs a new token | orange dot | orange | opens the Replace token sheet |

## 5. Explorer (Notes and Tasks)

- Top row: the drawer button, the search field (soft gray fill, no border) and **+**.
- **Folders** section: collapsible, shows only the folders inside the current folder. Tapping a folder opens it (it becomes the current folder). Open in Notes, closed in Tasks.
- **The whole tree is in a left drawer** too (decided 2026-11-05): the same folder tree as the desktop explorer, full height, 44 px rows. It opens from the drawer button or a swipe from the left edge, and closes on the veil, Esc/Back, or as soon as a folder or note is picked. Note, task and person pages reach it by the edge swipe.
- Outside the top-level folder: a row with **↑** (up one level) and the path (`notes / work`).
- Below: the items in the current folder.
  - Notes: notes directly in the folder; row = name | last edited.
  - Tasks: tasks in the folder and its subfolders; row = status dot · title | due date. Overdue dates in the overdue colour.
- Tasks only: views row `My day · All · Due · Upcoming · (calendar)` on the left, board toggle on the right. Counts are for the current folder.
- Board: one column at a time with the next one peeking; swipe between columns; pager dots. Cards are soft-gray fills, no borders.
- **+** opens the quick-create sheet: Note · Folder (Notes) or Task · Folder (Tasks), with the current folder filled in. Names follow the name rule `^[a-z0-9]+(_[a-z0-9]+)*$` (`decisions.md`); task titles are free text.

## 6. Pages

- One quiet top row: back, breadcrumb, one or two actions (edit / ⋯ / delete).
- Note: view mode renders markdown; Edit switches to the plain editor. While typing: formatting bar above the keyboard (H, B, I, list, checkbox, link, code, hide keyboard), "Saving…/Saved", Preview and Done; the menu button hides.
- Task page: property rows (Status, Due, My day, Assignee, Tags), Description, Subtasks.
- Person: name, birthday line, First/Last/Nickname/Born, Phone · Email · Address (with call/mail buttons), Notes.
- Word: title, kind and part of speech as plain text (not tags), pronunciation, user tags, Dictionary, What it is for me, Notes.

## 7. Library

- Tabs: Words · Snaps · References (no Home on mobile). No tag filter, as on desktop (`decisions.md` §15); search covers it. Opens on the last used tab.
- Words: a kind dropdown (All kinds by default) that filters the list; rows = word + one-line meaning.
- Snaps: two-column pinboard. Stacks open like folders (↑ `snaps / Recipes`). **+** opens the New snap sheet (paper, colour, text, stack). Tapping a snap opens the snap editor (paper, tags; ⋯ = Cut, Delete).
- References: title, link or author, tags and date. "Why I kept it" is on the reference page.

## 8. Select, cut, paste

- Long-press a note or task row to enter selection (folders are in the drawer, which has its own move and delete). Top: Cancel · "N selected" · Select all. Rows get a circle check.
- Action bar replaces the menu button: **Cut · Rename · Delete** (Rename only with one item). Snaps: **Cut · Tag · Delete**.
- Single item ⋯ menu: Cut · Rename · Delete.
- After Cut: a violet bar at the bottom, "Cut · 2 notes from notes/home", Cancel and **Paste here**. Browse normally; search becomes "Search folders" (or "Search stacks").
- Rules: can't paste where the items already are, or a folder into itself or its own subfolders. Opening an editor clears the clipboard. No copy for now.

## 9. Dialogs

- Delete note: action sheet, "Delete plan_for_the_move?", Delete note (danger) / Cancel. Respects the confirmation settings.
- Delete folder: always asks, and you type the folder name to enable Delete.
- Changed on another device: no dialog. The phone syncs with the desktop's rules (`docs/v1-sync-spec.md`): edits to different fields merge, and when both sides changed the same field the newer edit wins. A note or task deleted elsewhere while open asks to keep it as a new one, as on desktop.

## 10. First run and sync

- Restore: Repo URL, Token, More (branch). Public-repo warning. Restore button. The vault folder isn't asked for (below).
- **Vault folder: deferred** (`open-questions.md` #22). For now the vault is a folder inside Tend's own storage, named after the remote's path: `app_data_dir()/vaults/<host>/<owner>/<repo>` (`https://github.com/rniverse/notes.git` → `vaults/github.com/rniverse/notes`). The host keeps two remotes with the same owner and repo on different hosts apart; the owner is a user or an org; `.git` is dropped; every part is lowercased, since hosts treat these names without case and Android's file system doesn't. The user never sees or browses this folder, so its name only has to be stable and unique per remote. Restoring a remote whose folder already exists asks before replacing it; disconnecting leaves the folder until the user removes it. Picking any folder needs a small plugin, since `tauri-plugin-dialog` has no folder picker on iOS or Android:
  - iOS: the document picker for a folder, plus a security-scoped bookmark so access survives restarts.
  - Android: Rust, Git and SQLite need a real file path, which the system folder picker (a `content://` URI) doesn't give; it needs *All files access* and a path.
  - Either way the vault's `.tend/` folder lives inside the vault, as on desktop.
- Getting your vault ready: progress, steps (Downloading, Applying files, Opening the vault). Resumes if interrupted.
- Settings › Sync: last sync (with error banner when failed), Sync now, Disconnect…, repo and branch, token "saved on this phone" with Replace, schedule ("when you open or leave Tend, and every 15 min").
- Replace token sheet: explanation, new token field, Save.

## 11. Settings (mobile)

- Vault: shows the remote's `<host>/<owner>/<repo>` (§10). **Change vault folder…** waits for the folder picker.
- Sync: repo · branch, last synced, Sync now.
- Appearance: Theme (System / Light / Dark), Menu button (Left / Right).
- Fonts: App font, Editor font, Text size (each opens the Fonts screen).
- Sections: General, Tasks (statuses, labels), Library, Tags, Sync, in the left drawer (its button sits on every section's bar). Settings opens on General, which on a phone starts with the vault, sync, appearance and fonts above Files and Confirmations. No Command palette section on a phone; Feedback isn't there yet.

## 12. Screens

| Screen | File |
|---|---|
| First run · restore / getting ready | `MD-Restore`, `MD-Ready` |
| Menu closed / open / left side | `MV-Corner`, `MV-Corner-Open`, `MV-Corner-Left` (interactive on the canvas) |
| Sync states on the button | `MD-SyncStates` |
| Notes · root / in a folder | `MC-Notes`, `MC-Notes-Open` |
| Note · view / edit | `MD-Note`, `MD-Note-Edit` |
| Quick create · note / task | `MD-Quick`, `MD-Quick-Task` |
| Tasks · root (Due) / in a folder / board | `MC-Tasks`, `MC-Tasks-Open`, `MD-Board` |
| Task page | `MD-Task` |
| People / person | `MD-People`, `MD-Person` |
| Library · words / word / snaps / stack / references | `ML-Words`, `ML-Word`, `ML-Snaps`, `MD-Stack`, `ML-Refs` |
| Snaps · new / edit | `MD-Snap-New`, `MD-Snap-Edit` |
| Item menu / select / paste (notes, snaps) | `MD-Move`, `MD-Select`, `MD-Paste`, `MD-Snap-Select`, `MD-Snap-Move` |
| Delete note / folder | `MD-Delete`, `MD-Delete-Folder` |
| Settings / fonts / sync / token | `MD-Settings`, `MD-Font`, `MD-Sync`, `MD-Token` |

Desktop mockup changes: `Now-Tasks`, `Now-Board` (Due), `Now-Settings-General` (Fonts section).

## 13. Build plan (mobile milestones)

Tauri 2 mobile, same SvelteKit app; phone layout chosen by viewport width. Each milestone: build, test on an iOS simulator and an Android emulator, compare with `screens/`.

| # | Milestone | Done when |
|---|---|---|
| P1 | Fonts and Due (both apps) · done | App/Editor font + text size work on desktop; Due view with date picker replaces Overdue; snaps unaffected |
| P2 | Phone shell + corner menu · done | Phone layout at narrow widths; corner button (left/right), grow animation, veil, hides with keyboard |
| P3 | First run + sync on phone · done (progress without counts, `open-questions.md` #23) | Restore into `vaults/<host>/<owner>/<repo>` (§10), progress screen, sync on open/suspend/resume/interval, dot states, token sheet |
| P4 | Explorer · done (the board too) | Notes and Tasks explorer: search, Folders section, ↑, items, views and counts, quick create |
| P5 | Pages · done (the board was built in P4) | Note view/edit with formatting bar, task page, board, people, person |
| P6 | Library · done (snap editor as a popup, `open-questions.md` #26) | Words with Kinds, word page, snaps board and stacks, new snap and snap editor, references |
| P7 | Select · cut · paste, dialogs · done for notes and tasks (snaps: `open-questions.md` #27) | Long-press selection, action bar, paste bar and rules, delete confirms, changed-elsewhere |
| P8 | Settings · done (fonts as lists, text size in 3 steps; no Feedback row yet) | Vault, sync, appearance, fonts, sub-pages |
