# Open questions and deviations

Choices made where the spec was unclear or the stack had moved on. Each one is the simplest option that matches the mockups.

## Tooling

- **SvelteKit 3** is the latest release. It no longer reads `svelte.config.js`, so the adapter is passed to `sveltekit()` in `vite.config.js`. `$lib` was removed in favour of the `#lib` subpath import (`package.json` → `imports`). Imports use explicit extensions (`#lib/util/keys.ts`, `#lib/components/Icon.svelte`).
- **TypeScript 6.0.3**, not 7. SvelteKit 3 and svelte-check need the TypeScript JS API, which TypeScript 7 (the native port) does not ship.
- **Bun** runs the scripts (`bun run check`, `bun run test`, `bun run tauri dev`). The `npm run …` commands in CLAUDE.md work the same way.
- **shadcn-svelte and Tailwind were removed.** Their CSS variables (`--primary`, `--border`, `--accent`) clash with `tokens.css`, and CLAUDE.md asks for plain CSS.
- **Note editor:** CodeMirror 6 (markdown mode) instead of a plain `<textarea>`, at the owner's request. It keeps the monospace look from the mockups.
