# Tend

A quiet desktop app for notes, tasks, people and a Library (words, snaps, references), with simple timers. Notes are plain Markdown files in a folder you pick; Tend keeps its index and settings in `.tend/` inside it. A vault can sync through a Git repository, and the app updates itself from GitHub releases. Tauri 2 + SvelteKit.

## Run

Needs [Bun](https://bun.sh) and the [Rust toolchain](https://rustup.rs) (Tauri's prerequisites for macOS).

```sh
bun install
bun run tauri dev     # run the app in development
bun run app           # build a debug Tend.app and open it
```

## Test

```sh
cd src-tauri && cargo test   # Rust
bun run check                # svelte-check
bun run test                 # vitest
```

## What's here

| Path | What |
|---|---|
| `CLAUDE.md` | Instructions Claude Code reads every session: stack, layout, rules, how to verify |
| `docs/decisions.md` | The spec: every rule the app follows |
| `docs/schema.sql` | The SQLite schema of `.tend/tend.db`, run as migration 1 |
| `docs/build-plan.md` | The milestones (all built) and what was built after them |
| `docs/release-plan.md` | What is left for v1 |
| `docs/v1-sync-spec.md`, `docs/review-list.md` | Git sync: the spec, and the decisions taken while building it |
| `docs/tend-library-plan.md`, `docs/tend-library/` | The Library: plan and mockups |
| `docs/open-questions.md` | Known gaps |
| `docs/design/` | Mockups (open `screens/*.html` in a browser), screenshots, `tokens.css`, design README |
| `docs/later/` | Calendar / events / reminders: designed, **not** in v1 |

When the spec changes, update `docs/decisions.md` first, then the code.
