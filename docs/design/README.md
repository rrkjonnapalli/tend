# Design reference

Interactive mockups of every v1 screen, exported from the design canvas. Open any `screens/*.html` in a browser. They run on their own through `dc-runtime.js`, which is a tiny template runner for the mockups only and **not** part of the app. The data in them is sample data.

Fonts load from Google Fonts in the mockups. The app bundles them with @fontsource (see `tokens.css`).

## Screens

| Mockup | Screenshots | Shows |
|---|---|---|
| `start.html` | `png/start.png`, `png/start-checking.png` | Start page; choosing a vault; checklist; incompatible DB (switch with the "Demo folder" links bottom-left) |
| `notes.html` | `png/notes.png`, `png/notes-list.png` | Explorer, note page (read/edit, tags, width toggle), folder list, bottom nav, timer chip, feedback |
| `folders.html` | `png/folders.png`, `png/folders-menu.png`, `png/folders-move.png`, `png/folders-delete.png` | Explorer actions: ⋯ / right-click menu, inline rename with name rule, drag to move, Move to… dialog, delete with typed folder name |
| `tasks.html` | `png/tasks.png`, `png/tasks-board.png`, `png/task-page.png` | Task list + views + date range picker, drag order, board, task page (status, due, My day, assignee, tags, description, subtasks, attachments, comments) |
| `people.html` | `png/people.png`, `png/person-page.png` | People list, person page with contact sections and markdown notes |
| `settings.html` | `png/settings.png`, `png/settings-palette.png` | Settings: General (vault), Tasks → Statuses (drag, color, rename, add, delete rules), Command palette list |
| `command-palette.html` | `png/command-palette.png`, `png/command-palette-typed.png` | ⌘K palette: groups, search, quick-create phrases |
| `quick-create.html` | `png/quick-create-note.png`, `png/quick-create-timer.png` | ⌘N / ⌘⇧T / ⌘⇧N / ⌘T popups (use the buttons at the bottom; ⌘ keys are usually taken by the browser) |

Ignore anything in a mockup that isn't in `decisions.md`; it's a leftover.

## Logic worth reusing

Each mockup keeps its logic in the `<script type="text/x-dc">` block, `renderVals()`. These parts are already worked out; port them into `src/lib/util` or Rust, with tests:

| What | Where | Notes |
|---|---|---|
| Fractional order keys `keyBetween(a, b)` | `tasks.html`, search `const keyBetween` | Base-62 keys; one global key per task |
| List drag-to-reorder | `tasks.html`, `const dndFor` | Pointer events, row-height maths, done tasks stay below open ones |
| Board drag with ghost + placeholder | `tasks.html`, `const grabCard` | Column hit-testing, insert index ignores the placeholder, rAF throttled |
| Markdown subset render | `tasks.html`, `const md` / `const inline` | Only to see what's supported; use a real parser (marked + DOMPurify) in the app |
| Date / range picker | `tasks.html`, `vals.cal`, `shiftMonth` | Presets, two-click range, hover preview |
| Name rule, rename, move, delete | `folders.html`, `validate`, `moveFolder`, `canDropFor`, `doDelete` | Prefix rewrite of child paths; can't drop into self or a descendant; name clash check |
| Timer input parse | `quick-create.html`, `const pd` | `mm:ss`, minutes unlimited, seconds 0–59, bare number = minutes |
| Time parse | `quick-create.html`, `const pt` | `5pm`, `5:30 pm`, `17:30` |
| DOB + next birthday | `people.html`, `parseDob`, `nextBd` | `yyyy-mm-dd` or `mm-dd`, age only when the year is known |
| Status rules | `settings.html`, `slug`, `labelTaken`, `grab` | UPPER_SNAKE value from label, reserved values, first/last pinned |
| Palette quick-create parsing | `command-palette.html`, `task:` / `note:` / `person:` / `timer` regexes | |

## Tokens

`tokens.css` has the Clean Colors palette, semantic roles, status swatches, type styles, spacing, radius and layout sizes. The mockups use the same values, hard-coded.
