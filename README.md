# Tend

A quiet desktop app for notes, tasks and people. Tauri 2 + SvelteKit.

v1 (milestones M1–M10 in `docs/build-plan.md`) is built.

## Run

```sh
bun install
bun run tauri dev      # the app
bun run check          # svelte-check
bun run test           # vitest (src/lib/util, api helpers)
cd src-tauri && cargo test   # Rust: vault, scan, names, folders, statuses, IPC round trips
```

On first launch, choose a vault folder. An empty folder becomes a new vault.

In a plain browser (`bun run dev`, then open http://localhost:1420), the UI runs against fixtures in `src/lib/dev/mock-ipc.ts`. Add `?vault=0` to the URL to see the start page. This mode is for layout checks only and isn't part of production builds.

## What's here

| Path | What |
|---|---|
| `CLAUDE.md` | Instructions Claude Code reads every session: stack, layout, rules, how to verify |
| `docs/build-plan.md` | 10 milestones with scope and acceptance checks |
| `docs/decisions.md` | The spec: every rule we settled during design |
| `docs/schema.sql` | SQLite schema v1 for `db/app.sqlite` |
| `docs/design/` | Mockups (open `screens/*.html` in a browser), screenshots, `tokens.css`, design README |
| `docs/later/` | Calendar / events / reminders: designed, **not** in v1 |
| `docs/open-questions.md` | Choices made where the spec was silent, and the tooling deviations |
| `src-tauri/src/` | Rust: one module per area (`vault`, `db`, `notes`, `tasks`, `people`, `statuses`, …); `commands.rs` is the IPC layer |
| `src/lib/` | `api/` typed command wrappers, `stores/` app state, `components/` shared UI, `util/` pure helpers, `commands.ts` the command registry |

When the spec changes, update `docs/decisions.md` first, then the code.
