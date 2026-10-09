# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(A macOS desktop app: Tauri 2 webview running a SvelteKit SPA. Designed for a laptop or desktop
window, not phones.)

## Users

One person: the author, using Tend as a daily driver all day for notes, tasks, people and quick
timers. Others may use it later; there is no public audience to design for in v1.

## Product Purpose

Tend keeps notes, tasks and people in one local vault: a folder of plain `.md` files the user owns,
with a SQLite index in `.tend/`. It replaces several separate apps with one keyboard-driven place
to write, plan the day and remember people. Success is that the author opens it first every
morning and never feels slowed down by it.

## Positioning

Notes on disk are the source of truth. The vault folder is the notes root, names follow one slug
rule, and every rename is logged, so the files stay readable and usable without Tend. Optional
Git sync keeps the same vault in step across devices without a server.

## Operating Context

- Used in long sessions on a Mac, switching often between Notes, Tasks and People.
- Most actions start from the keyboard: ⌘K palette, quick create (⌘N note, ⌘T timer), ⌘1–3
  sections, Esc closes, Enter confirms, ↑↓ move through lists.
- Notes are edited as Markdown in a plain monospace editor and read rendered.
- Tasks are seen as views (My day, Upcoming, Overdue, …) in a list or a board.
- Timers count down in the bottom bar; sync status sits next to them.

## Capabilities and Constraints

- Sections: Start (choose or restore a vault), Notes (explorer, list, note page), Tasks (views,
  list/board, task page with subtasks, comments, attachments, labels), People (list, person page
  with birthdays and their tasks), Settings (general, statuses, labels, palette, sync).
- Stack is fixed: Tauri 2, SvelteKit with Svelte 5 runes, plain CSS with `tokens.css` variables in
  scoped component styles. No Tailwind or UI kit. Fonts are bundled offline (no network fonts).
- The UI never touches files or SQL; everything goes through Tauri commands.
- Accessibility basics are required: real buttons, `aria-label` on icon buttons, a visible focus
  ring.

## Brand Commitments

- Name: Tend (wordmark "tend." with the accent dot).
- The author likes the current layout, spacing, sizes and functionality as they are. The
  dissatisfaction is with the look: it reads generic and the screens feel inconsistent with each
  other. The author suspects the fonts are the main cause, especially in the Notes section, but
  isn't sure.
- Keyboard-first must stay, whatever else changes.

## Evidence on Hand

- Original mockups and screenshots: `docs/design/screens/*.html`, `docs/design/screens/png/`.
- Current tokens: `docs/design/tokens.css` (bundled into the app's styles).
- Product rules: `docs/decisions.md`; build history: `docs/build-plan.md`.
- No users, testimonials or metrics beyond the author's own use.

## Product Principles

1. Never slow the author down: keyboard first, no confirmation where undo or a log will do.
2. Files stay the author's: plain Markdown, readable names, nothing hidden that matters.
3. One app, one feel: Notes, Tasks, People and Settings share the same patterns and components.
4. Calm density: lots of information on screen, quiet chrome, no decoration that competes with
   content.
