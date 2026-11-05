# Tend: instructions for Claude Code

Tend is a desktop app for **notes, tasks, people and a Library** (words, snaps, references), plus simple timers, with **Git sync** and auto-update, built with **Tauri 2 + SvelteKit**. The milestones in `docs/build-plan.md` are all built; work now is fixes, features and the v1 release (`docs/release-plan.md`).

## Read first (every session)

1. `docs/decisions.md`: the rules. This is the spec. If code and spec disagree, the spec wins.
2. `docs/schema.sql`: the database (`.tend/tend.db`). `src-tauri/src/db/mod.rs` embeds it as migration 1 (`rusqlite_migration`; the version is `PRAGMA user_version`). Nothing has shipped, so it is edited in place until the first release; after that, changes are new migration steps.
3. `docs/release-plan.md`: what is left for v1, and the decisions taken for it.
4. `docs/design/`: what it should look like.
   - `screens/*.html`: interactive mockups. Open them in a browser; they run on their own. Their `<script type="text/x-dc">` blocks hold working reference logic.
   - `screens/png/*.png`: screenshots of the mockups and key states. Match these.
   - `tokens.css`: colors, type, spacing and radius as CSS variables. Use these; don't hard-code hex values in components.
   - `README.md`: an index of the screens, plus the logic in the mockups worth reusing.

The Library (Words, Snaps, References) has its own plan, `docs/tend-library-plan.md`, and mockups in `docs/tend-library/`; `decisions.md` §15 says where Tend differs from them, and wins. Sync is specified in `docs/v1-sync-spec.md`, with the decisions taken while building it in `docs/review-list.md` (`docs/sync-plan.md` is superseded history). Known gaps are in `docs/open-questions.md`; `docs/build-plan.md` records the milestones and what was built after them.

`docs/later/` is **out of scope** (calendar, events, reminders). Don't build any of it, and don't add tables for it.

## Stack

- **Tauri 2** (Rust). **SvelteKit** with `adapter-static`, SPA mode (`ssr = false`), **Svelte 5** (runes), **TypeScript** (strict).
- **Database:** SQLite through **Rust** (`rusqlite` with the `bundled` feature, `rusqlite_migration`), behind Tauri commands. The frontend never touches SQL or the file system directly.
- **Rust owns all disk and DB work:** vault setup, scan, file watcher (`notify` + `notify-debouncer-full`, events sent to the UI), note read/write, folder rename/move/delete, attachments, migrations, sync. Disk first, then DB, as one unit. Roll back the DB step if the disk step fails. Deletes go to the system Trash (`trash`).
- **IDs:** ULID strings (`ulid` crate), generated in Rust. **Times:** INTEGER ms UTC. Local dates and times are `YYYY-MM-DD` / `HH:MM` text (see the conventions at the top of `schema.sql`).
- **Sync:** `git2` (vendored libgit2 and OpenSSL); the token lives in the macOS Keychain only (`keyring-core` + `apple-native-keyring-store`).
- **Other Rust crates:** `ureq` (dictionary lookups and HTTP), `fontdb` (installed font families for the reading font).
- **Plugins:** `tauri-plugin-dialog` (folder picker), `tauri-plugin-notification` (timers), `tauri-plugin-opener`, `tauri-plugin-single-instance`, `tauri-plugin-window-state`, `tauri-plugin-updater` (auto-update from GitHub releases). App prefs live in the `settings` table; the last vault path and sync settings live in the app config dir (`src-tauri/src/config/`), not inside a vault.
- **Styling:** plain CSS plus `tokens.css`, scoped `<style>` in components. No Tailwind or UI kit; the look is custom and minimal.
- **Fonts:** `@fontsource/montserrat` (500, 600), `@fontsource/lato` (400, 700), `@fontsource/jetbrains-mono` (400, 500), `@fontsource/work-sans` (400, 700; the default Editor font), bundled offline. The Library's handwriting: `@fontsource/caveat` (500, 600) and `@fontsource/gaegu` (400, sticky notes), plus `static/fonts/Smile Moon.otf` (stack names; not on Fontsource).
- **Markdown:** `marked` + `DOMPurify` for rendering. The editor is CodeMirror 6 (`@codemirror/*`, `MarkdownEditor.svelte`) in the Editor font (`--font-editor`, Work Sans by default), the same font as the note view.
- **Order keys:** `fractional-indexing` (`src/lib/util`, `keyBetween`).

## Project layout

```
src-tauri/src/
  main.rs, lib.rs   app setup, plugins, the command list
  commands/         the Tauri commands, one file per area
  vault/            open / create / check / migrate
  db/               connection, migrations (docs/schema.sql as v1); tables.rs, columns.rs hold
                    table and column names (never typed as strings elsewhere); update.rs
  notes/            scan, watcher, note + folder edits
  tasks/, people/, statuses/, labels/, attachments/, comments/, timers/, feedback/, settings/
  enums/, links/, tags/   the shared enum table (statuses, labels, word kinds, tags) and its links
  words/, snaps/, refs/   the Library; words/dictionary/ has one file per dictionary source
  sync/             git, shards, reconcile, apply, run + runner, token, upgrade
  names/            the name rule ^[a-z0-9]+(_[a-z0-9]+)*$, slugs, clash suffixes, path helpers
  config/           the app config file (last vault, sync settings)
  error/            typed errors with stable codes
  fonts/            installed font families
  macos/            macOS-only pieces
  android.rs        Android-only pieces (CA certificates for libgit2)
  update.rs, http.rs, files.rs, dates.rs, colors.rs, state.rs, util.rs
src/lib/
  api/              typed wrappers around invoke() (one file per area)
  stores/           Svelte 5 state, one *.svelte.ts per area (vault, notes, tasks, people, library,
                    words, snaps, refs, timers, sync, settings, tags, update, ui, toast, …)
  components/       BottomNav, Explorer, Popover, ChoicePicker, MarkdownField, TagEditor, DateField, …
    phone/          the phone layout (docs/mobile/): CornerMenu, Sheet, ActionSheet, PhoneStart, …
  actions/          Svelte actions (focus, keyboard navigation, portal, …)
  config/, enums/, errors/, hooks/, types/   app constants, enums, error codes, shared types
  styles/           app.css (base styles, :focus-visible), fonts.css
  util/             order keys, names, dates, timer parsing, markdown, fonts, … (vitest)
  commands.ts, icons.ts   the command palette's commands, the inline SVG icons
src/routes/         start, notes, tasks, people, library, settings
```

## Rules

- **Build only what the spec says.** If something is unclear or missing, pick the simplest option that matches the mockups. Note it in `docs/open-questions.md` and keep going; don't stop to ask.
- **Spec first:** when a rule changes, update `docs/decisions.md` (and `docs/schema.sql` if the data changes) with the code.
- **Match the mockups:** spacing, sizes, colors, hover and selected states, icons (inline SVG paths are in the mockup HTML; reuse them). No shadows on panels. Popovers may have a border only.
- **The name rule** (`^[a-z0-9]+(_[a-z0-9]+)*$`, unique in its folder; a clash gets `_1`, `_2`) applies to note folders, task folders and note file names. Typed folder names and note titles become slugs. A note is shown by its file name; its title is the `# heading` inside the file. Check it in Rust; the UI shows the same rule live.
- **Notes on disk are the source of truth.** The vault folder is the notes root; Tend's files live in `.tend/`. The DB is an index. A scan renames a folder or `.md` file only to give a bad name its slug, only with the vault's consent, and logs it in `path_log`. It never moves files and never touches hidden or non-`.md` files.
- **Keyboard:** every popover closes on Esc. Enter confirms. ↑↓ move through lists. Shortcuts are in `decisions.md` §12; override ⌘R/⌘N/⌘T so the webview doesn't reload or open windows.
- **Accessibility basics:** real `<button>`s, `aria-label` on icon buttons, and visible keyboard focus *without outlines or rings*: a focused control gets the hover fill, a list row the cursor fill, a field its focus border, a filled button its hover colour (see `:focus-visible` in `src/lib/styles/app.css`). Never draw an outline.
- Keep functions small. No dead code, no TODO stubs.

## Verify before saying a change is done

1. `cargo test` (in `src-tauri/`). Add tests for anything in Rust with rules: vault check, migrations, scan, name validation, folder rename/move/delete, status rules, sync merges.
2. `bun run check` (svelte-check, no errors) and `bun run test` (vitest, for `src/lib/util`; plain `bun test` runs Bun's own runner, not vitest).
3. `bun run tauri dev` starts. Try the change by hand. Phone screens: `bun run android` builds and installs on a running emulator.
4. Compare the screens with `docs/design/screens/png/` and fix visible differences.
5. Update the docs the change touches (`decisions.md`, `open-questions.md`, `release-plan.md`).

Commits use Conventional Commits (`feat(area): …`, `fix: …`, `docs: …`).
