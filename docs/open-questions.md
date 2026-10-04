# Open questions and deviations

Choices made where the spec was unclear or the stack had moved on. Each one is the simplest option that matches the mockups.

## Tooling

- **SvelteKit 3** is the latest release. It no longer reads `svelte.config.js`, so the adapter is passed to `sveltekit()` in `vite.config.js`. `$lib` was removed in favour of the `#lib` subpath import (`package.json` → `imports`). Imports use explicit extensions (`#lib/util/keys.ts`, `#lib/components/Icon.svelte`).
- **TypeScript 6.0.3**, not 7. SvelteKit 3 and svelte-check need the TypeScript JS API, which TypeScript 7 (the native port) does not ship.
- **Bun** runs the scripts (`bun run check`, `bun run test`, `bun run tauri dev`). The `npm run …` commands in CLAUDE.md work the same way.
- **shadcn-svelte and Tailwind were removed.** Their CSS variables (`--primary`, `--border`, `--accent`) clash with `tokens.css`, and CLAUDE.md asks for plain CSS.
- **Note editor:** CodeMirror 6 (markdown mode) instead of a plain `<textarea>`, at the owner's request. It keeps the monospace look from the mockups.

## Tasks

- **Deleting and moving a task.** The spec has no UI for either. The task page top bar now has *Move to…* (folder icon, ⌘⇧M; it reuses the explorer's Move dialog) and *Delete task* (trash icon, with a confirmation). Subtasks move and get deleted with their parent. Subtasks themselves can't be moved on their own.
- **New tasks** go to the top of the global order. The board's `+ add task` puts the task at the bottom of its column.
- **Comments** show "You · 2h", because v1 has a single user.
- **Board scope.** The board shows the tasks of the current view (All, Overdue, …), grouped into one column per status.

## ⌘K, quick create and timers

- **Note names in quick create and `note: …`** are typed as free text and turned into a valid file name ("Meeting notes" → `meeting_notes.md`). The note starts with a matching `# Heading`.
- **`task: …` in ⌘K** adds the task to the last task path used, shows a toast, and stays where you are. `note: …` and `person: …` open what they create.
- **Page commands** in the registry (toggle read/edit, add tag, width, set status, set due date, add to My day) only show in ⌘K while a page that supports them is open.
- **Timer sound** is a short Web Audio chime, so no sound file needs bundling. It is on by default and stored in `settings.timers.sound`, which has no UI in v1.
- **After sleep**, the countdown is recomputed when the window regains focus or visibility, as well as every second, so a timer that ended during sleep rings right after wake.
