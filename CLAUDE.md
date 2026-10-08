# Tend: instructions for Claude Code

Tend is a desktop app for **notes, tasks and people** (plus simple timers), built with **Tauri 2 + SvelteKit**. The design is done. Your job is to build it milestone by milestone.

## Read first (every session)

1. `docs/build-plan.md`: the milestone you were asked to do, and its *Done when* checks.
2. `docs/decisions.md`: the rules. This is the spec. If code and spec disagree, the spec wins.
3. `docs/schema.sql`: the database (`.tend/tend.db`, schema v1). Nothing has shipped, so it is edited in place until the first release; migrations start after that.
4. `docs/design/`: what it should look like.
   - `screens/*.html`: interactive mockups. Open them in a browser; they run on their own. Their `<script type="text/x-dc">` blocks hold working reference logic.
   - `screens/png/*.png`: screenshots of the mockups and key states. Match these.
   - `tokens.css`: colors, type, spacing and radius as CSS variables. Use these; don't hard-code hex values in components.
   - `README.md`: an index of the screens, plus the logic in the mockups worth reusing.

`docs/later/` is **out of scope** (calendar, events, reminders). Don't build any of it, and don't add tables for it.

## Stack

- **Tauri 2** (Rust). **SvelteKit** with `adapter-static`, SPA mode (`ssr = false`), **Svelte 5** (runes), **TypeScript** (strict).
- **Database:** SQLite through **Rust** (`rusqlite` with the `bundled` feature), behind Tauri commands. The frontend never touches SQL or the file system directly.
- **Rust owns all disk and DB work:** vault setup, scan, file watcher (`notify` crate, events sent to the UI), note read/write, folder rename/move/delete, attachments, migrations. Disk first, then DB, as one unit. Roll back the DB step if the disk step fails.
- **IDs:** ULID strings (`ulid` crate), generated in Rust. **Times:** INTEGER ms UTC. Local dates and times are `YYYY-MM-DD` / `HH:MM` text (see the conventions at the top of `schema.sql`).
- **Plugins:** `tauri-plugin-dialog` (folder picker), `tauri-plugin-notification` (timers), `tauri-plugin-store` or the `settings` table for app prefs. The last vault path lives in the app config dir, not inside a vault.
- **Styling:** plain CSS plus `tokens.css`, scoped `<style>` in components. No Tailwind or UI kit; the look is custom and minimal.
- **Fonts:** `@fontsource/montserrat` (500, 600), `@fontsource/lato` (400, 700), `@fontsource/jetbrains-mono` (400, 500), bundled offline.
- **Markdown:** `marked` (or `markdown-it`) + `DOMPurify` for rendering. The editor is a plain `<textarea>` with monospace font, as in the mockups.

## Project layout (create as you go)

```
src-tauri/src/
  main.rs, lib.rs
  vault.rs        open / create / check / migrate
  db.rs           connection, migrations (embed docs/schema.sql as v1)
  notes.rs        scan, watcher, note + folder commands
  tasks.rs, people.rs, statuses.rs, attachments.rs, timers.rs, feedback.rs, settings.rs
  names.rs        the name rule, clash suffixes and path helpers
src/lib/
  api/            typed wrappers around invoke() (one file per area)
  stores/         Svelte 5 state (vault, notes, tasks, people, timers, ui)
  components/     BottomNav, Explorer, Popover, MarkdownField, TagEditor, DateField, TimeField, …
  util/           order-key.ts (keyBetween), names.ts, dates.ts, timer-parse.ts
src/routes/       start, notes, tasks, people, settings
```

## Rules

- **Build only what the spec says.** If something is unclear or missing, pick the simplest option that matches the mockups. Note it in `docs/open-questions.md` (create it if needed) and keep going; don't stop to ask.
- **Match the mockups:** spacing, sizes, colors, hover and selected states, icons (inline SVG paths are in the mockup HTML; reuse them). No shadows on panels. Popovers may have a border only.
- **The name rule** (letters, digits, spaces and `. , _ -`; leading and trailing spaces and dots trimmed; at most 200 characters; unique in its folder ignoring case, a clash gets `_1`, `_2`) applies to note folders, task folders and note names. A note's name is its file name and its title. Check it in Rust; the UI shows the same message live.
- **Notes on disk are the source of truth.** The DB is an index. A scan renames a folder or `.md` file only to fix a name that breaks the rule, and only once the vault allows it (new empty vaults do; otherwise the user agrees once). It never moves files, never touches hidden or non-`.md` files, and skips names nothing valid can be made from.
- **Keyboard:** every popover closes on Esc. Enter confirms. ↑↓ move through lists. Shortcuts are in `decisions.md` §12; override ⌘R/⌘N/⌘T so the webview doesn't reload or open windows.
- **Accessibility basics:** real `<button>`s, `aria-label` on icon buttons, a visible focus ring (`:focus-visible`, 2px `--focus-ring`).
- Keep functions small. No dead code, no TODO stubs in finished milestones.

## Verify before saying a milestone is done

1. `cargo test` (in `src-tauri/`). Add tests for anything in Rust with rules: vault check, migrations, scan, name validation, folder rename/move/delete, status rules.
2. `npm run check` (svelte-check, no errors) and `npm test` (vitest, for `src/lib/util`).
3. `npm run tauri dev` starts. Walk through the milestone's *Done when* list by hand.
4. Compare the screens with `docs/design/screens/png/` and fix visible differences.
5. Update `docs/build-plan.md`: tick the milestone and list anything you changed or deferred.

Commit once per milestone: `M<n>: <title>`.
