# Tend

A quiet desktop app for notes, tasks and people. Tauri 2 + SvelteKit.

This repo starts as a **build kit**: the design and spec are done, and the code is built milestone by milestone with Claude Code.

## Start

1. Open this folder in a terminal and run `claude`.
2. Say: **"Read CLAUDE.md, then do M1 from docs/build-plan.md."**
3. Check the result against the milestone's *Done when* list, then commit.
4. Repeat for M2 … M10.

## What's here

| Path | What |
|---|---|
| `CLAUDE.md` | Instructions Claude Code reads every session: stack, layout, rules, how to verify |
| `docs/build-plan.md` | 10 milestones with scope and acceptance checks |
| `docs/decisions.md` | The spec: every rule we settled during design |
| `docs/schema.sql` | SQLite schema v1 for `db/app.sqlite` |
| `docs/design/` | Mockups (open `screens/*.html` in a browser), screenshots, `tokens.css`, design README |
| `docs/later/` | Calendar / events / reminders: designed, **not** in v1 |

When the spec changes, update `docs/decisions.md` first, then the code.
