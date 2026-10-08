# Tend V1 Sync Specification

**Status:** V1 design, agreed in discussion; not built.
**Scope:** Git-backed vault sync, and the v1 schema changes it needs.
**Replaces:** `docs/sync-plan.md` (kept for the history of the discussion).

---

## 1. Goals

Back up and synchronize a Tend vault through a Git remote, automatically.

- Git is only the transport and history layer. Tend decides what the merged state is.
- Every sync pulls before it pushes.
- Tend-to-Tend sync never depends on Git's text merge or on Git conflicts.
- The live SQLite files are never committed; shared data travels as deterministic JSONL files.
- **The newest change wins.** The older version is discarded; nothing is kept as a conflict copy.
- **A delete always wins** and is never undone by a later edit elsewhere.
- Sync runs are retryable and idempotent.

## 2. Non-goals (V1)

SSH remotes · Git LFS · choosing which folders sync · sub-minute sync · field-level merging or
collaborative editing · conflict copies or user-facing conflict resolution · Git history or diffs
inside Tend · mobile (iOS/Android) · journaling or multi-step rollback.

---

## 3. Vault layout

The vault is the Git working tree:

```text
<vault>/
  notes/                      note files (body only), named by their titles
  .attachments/               attachment files, <id>.<ext>
  db/
    app.sqlite                live database        — never committed
    app.sqlite-wal / -shm     SQLite side files    — never committed
    sync/
      manifest.json
      <table>.jsonl           current state of each synced table
      deleted.jsonl           every deletion, all tables
  .git/
  .gitignore                  db/app.sqlite*, .DS_Store
```

There is no `.conflicts/` folder.

---

## 4. Schema (v1, edited in place)

Nothing has shipped, so `docs/schema.sql` stays **v1** and is edited directly; the v2 labels
migration is folded into it and deleted. Migrations start with the first release.

### 4.1 Identity and change time on every table

Every table gets:

- `id TEXT PRIMARY KEY` — a ULID generated on the device, never replaced.
- `updated_at INTEGER NOT NULL` — epoch ms, set on every insert and update.

| Table kind | Change |
|---|---|
| Entity tables (`tasks`, `people`, `tags`, `task_labels`, `attachments`, …) | add/keep `id`, add `updated_at` where missing |
| Link tables (`task_tags`, `note_tags`, `task_label_links`, `my_day`, `attachment_links`) | add `id`; the old key pair becomes a `UNIQUE` constraint |
| Name/path-keyed tables (`statuses` by value, `task_folders` / `note_folders` by path, `settings` by key) | add `id` as the primary key; the old key stays `UNIQUE` |
| `task_comments` | `edited_at` → `updated_at` |

Renaming a folder or a status is therefore an update of one row, not a new row.

Link rows get a fresh ULID each time a link is made, never one derived from the pair: deletions are
kept forever, so a derived id would stop a removed link from ever being added again. Two devices can
therefore make the same pair under different ids; reconciling keeps the newer row for each pair
(§9) and records the other as deleted. Ids are fixed only for rows that are never deleted: settings
(`id` = key), the default statuses (`id` = value) and the root folders (`id` = `'root'`).

### 4.2 Notes and attachments

A note is its own record; its body is a file, held by an attachment.

```text
notes        id · attachment_id · updated_at
attachments  id · path · name · mime_type · size_bytes · sha256 · updated_at
```

- `attachments.path` is the file's place in the vault: `notes/<folder>/<name>.md` for a note body,
  `.attachments/<id>.<ext>` for an attached file.
- A note's **title is its file name** without `.md`, so it syncs with `attachments.path`. Tend does
  not read a `# heading` from the file, and new notes start with an empty body.
- Renaming a note **renames the file** (§11.2); only `attachments.path` changes. The note's `id`
  never changes, so tags, labels and links are unaffected by renames and moves.
- Editing a note writes the file and bumps `updated_at` on both rows.
- `attachment_links` always points at an attachment `id`. Linking a note means linking its
  attachment; the separate `kind = 'note'` / `note_id` columns go.

### 4.3 New local tables

- `deletions (id, table_name, row_id, deleted_at)` — written for every delete, including rows
  removed by `ON DELETE CASCADE` (comments, links, contacts). Kept **forever**.
- `activity (id, action, status, entity_type, entity_id, at)` — every write plus open, close,
  pull and push; capped to the newest rows by a trigger (size: open item §20). Local only.
- Sync configuration and state (remote URL, branch, username, author, device name, interval,
  last result) — local only, never in the vault (§5).

### 4.4 Name rule

File and folder names are free text, safe as one name on any OS: no `/ \ :` or control
characters, no leading `.`, no leading or trailing space, at most 200 characters, unique in the
folder ignoring ASCII case. Checked in Rust; the UI shows the same message live. Existing names that
break it are listed as skipped, never renamed by Tend.

---

## 5. Shared vs local

| Synced | Local to the device |
|---|---|
| tasks, task folders, comments, statuses, labels and their links, my_day, people, contacts, tags and their links, notes, note folders, attachments (rows and files), shared settings (`tasks.root_name`, `timers.sound`) | timers, feedback, activity, deletions bookkeeping beyond what `deleted.jsonl` carries, UI settings (nav, explorer, widths, last view, quick-create paths), sync configuration, the token |

The shared/local split of settings is defined in one place (`settings.config.ts` and its Rust twin).

---

## 6. Git transport

- **HTTPS + personal access token**, optional username. No SSH.
- **Token:** macOS Keychain via `keyring-core` + `apple-native-keyring-store`. Never written to the
  vault, SQLite or a config file; loaded into memory only for a Git call. Settings shows only
  "Token saved ✓ · Replace · Remove".
- **Engine:** `git2` 0.21 in-process (`https`, `vendored-libgit2`, `vendored-openssl` on macOS —
  ~+4.3 MB, perl at build time). Credentials callback with a bounded number of attempts.

### 6.1 How history is built (no Git merge)

1. Commit local changes (snapshots, files) if any.
2. **Fetch** the remote branch.
3. **Read the remote tree directly** from the fetched commit (snapshots, deletions, files) — no
   `git merge`, no working-tree merge.
4. Reconcile in Tend (§9) and write the result into the working tree.
5. Commit the result as a **merge commit with two parents**: local HEAD and the fetched remote
   commit. Git never resolves conflicts; every device ends with the same tree.
6. Push. If rejected as non-fast-forward (another device pushed meanwhile): fetch, reconcile,
   commit and push again — **once**; a further failure waits for the scheduler.

---

## 7. Change time

`updated_at = now` (epoch ms) on every insert and update. Comparisons trust each device's clock.

**Known risk:** a device whose clock is wrong can make an older edit win (or lose). Accepted for V1;
mitigation (e.g. `max(now, previous + 1)` or a hybrid logical clock) is a later decision.

No incremental cursors or "last synced" markers: each sync compares complete snapshots, which are
small (a few MB), and Git itself records what was merged. (The earlier `(updated_at, id)` marker
idea is dropped — it added state without enabling anything.)

---

## 8. Snapshots

- One JSONL file per synced table, `db/sync/<table>.jsonl`: the **current state**, one row per
  line, rows **sorted by `id`** (an edit changes its line in place — small, readable diffs).
- Fixed column order, stable JSON (serde structs), no values generated while writing. Same data →
  byte-identical file.
- `deleted.jsonl`: `{"table":"tasks","id":"01…","deleted_at":…}` per line, sorted by
  `(table, id)`, never truncated.
- `manifest.json`: `{ "app_id": "tend", "format_version": 1, "schema_version": 1, "tables": [...] }`.
  A remote with unrelated files and no valid manifest is refused (`SYNC_NOT_A_VAULT`); a manifest
  whose schema is newer than the app is refused (`SNAPSHOT_TOO_NEW`) and nothing is imported.
- Estimated size: 3–10 MB for heavy personal use. Settings shows it; warn above 50 MB.

---

## 9. Reconciliation

Inputs: local rows + local deletions, remote rows + remote deletions. Output: one row set and one
deletion set, identical on every device given the same inputs.

| Case | Result |
|---|---|
| Row only on one side | kept |
| Row on both sides | **newer `updated_at` wins**; the other version is discarded. Equal times: the row whose JSON line sorts higher wins (deterministic). |
| Row deleted on either side (any time) | **deleted** — a deletion is never undone by an edit |
| Row absent from one side, no deletion record | kept (absence is not deletion) |
| Deletion on both sides | one record (earliest `deleted_at`) |
| Two link rows with the same pair (or two rows with the same unique key) | the newer `updated_at` wins; the other id is recorded as deleted |

Files follow their rows: a note's or attachment's file comes from the side whose attachment row won;
a deleted attachment's file is removed.

### 9.1 References after reconciling

Dependencies are applied parent-first. Rows pointing at something that no longer exists are fixed,
never left dangling:

| Reference | Rule |
|---|---|
| comment, tag/label link, contact, my_day, attachment link → deleted parent | dropped (and recorded in deletions) |
| task → deleted status | status set to the first status |
| task → deleted assignee | unassigned |
| task → deleted folder / parent task | per folder rules: moved to the root / dropped with its parent |
| note → deleted folder | moved to the notes root |

The full per-table contract (identity, unique keys, dependencies) is written before each table is
implemented.

---

## 10. Applying the result

1. Save any note open in the editor (so autosave can't write stale text after the pull).
2. Pause the notes watcher.
3. Write files (note bodies, attachments) and remove deleted ones.
4. Apply rows to SQLite in **one transaction** (a backup copy of `app.sqlite` is taken just
   before); check references; commit. A failed apply rolls back and the sync fails.
5. Scan `notes/`, matching files to note rows by `attachments.path` (§11.3).
6. Resume the watcher; reload stores and reopen the open note if it changed.

Every app command waits for the DB while step 4 runs; it must stay short, and the UI shows
"Syncing…" meanwhile.

---

## 11. Notes

### 11.1 Creating
A name is given (§4.4); the file is `<name>.md` and the body starts empty.

### 11.2 Renaming and moving
Rename → file renamed; move → file moved (a name taken in the target is refused); `attachments.path` and
`updated_at` updated. Links keep working because they point at ids.

### 11.3 Scanning
- A file whose path matches an attachment row → that note; content changed since the last scan →
  `updated_at = now`.
- A file no row knows → new note.
- A row whose file is gone → marked missing (existing grace period), unless a deletion says it's
  deleted.
- Renames done outside Tend (Finder) are matched by today's modification-time-and-content heuristic;
  if that fails the note gets a new id — the one case where links can break.

---

## 12. Attachments

Files live in `.attachments/<id>.<ext>`. Files over **100 MB** are not committed; their row still
syncs, so other devices show "file not synced". Skipped files are listed in Settings → Sync.

---

## 13. A sync run

```text
lock (one run at a time; triggers during a run coalesce into it)
→ save open note · pause watcher
→ write snapshots + deleted.jsonl · commit if Git sees changes
→ fetch
→ read remote tree · reconcile (§9)
→ apply (§10) · write snapshots · merge commit (§6.1)
→ push if ahead (retry once after a rejection)
→ record pull/push in activity · resume watcher · update status
```

"Anything to push?" is answered by Git status after writing the snapshots — deterministic files make
this exact, and it also catches edits made in Finder. The activity log never makes a run dirty.
The pull always happens; the push is skipped when nothing changed.

---

## 14. Connecting and restoring

**Connect** (Settings → Sync): URL, branch (default `main`), optional username, token, author
name/email (default "Tend"), device name (default: the Mac's name), interval.
The remote must be empty or hold a valid manifest. Tend clones into a temporary folder, reconciles
with the local vault (§9), moves `.git` into the vault, commits and pushes. If any step fails, the
temporary clone is removed and the vault is untouched.

**Restore** (Start page, or Settings → Sync on an empty vault): clone → validate manifest → create
the database → **write files → import rows → scan `notes/`** → open. On a vault that already has
data, Settings offers no restore; the first sync reconciles instead.

**Disconnect:** clears the remote, token and schedule; `.git` and all data stay.

**Public repos** are allowed. A static warning on the Start-page restore and in Settings → Sync:
"⚠ Beware of public repos: anything in your vault, private content included, is pushed there."

---

## 15. Scheduling, close, failures

- **Triggers:** app open · interval (off / 15 min / 1 h) · app close · **Sync now**.
- **Close:** Tauri close is intercepted; "Syncing…" shows; the app quits when pull (and push, if
  needed) succeed or after **10 s**. A cut-short run isn't treated as successful; the next open
  syncs again.
- **Time limits:** each network step has a deadline from config; the first push of a large vault
  and a restore clone get a longer one, with progress.
- **Failures:** retried by the scheduler with backoff up to 1 h; one toast on the first failure,
  then only the dock icon. Auth failures (bad or expired token) stop automatic sync until fixed.
  A failed run leaves the database and working tree consistent (§10.4).

## 16. Status UI

- **Dock icon** next to the timer chip: ✓ synced · ↻ syncing · ! error (red); hover = last sync;
  click → Settings → Sync.
- **Settings → Sync:** the connection fields, token status, interval, last sync, status, snapshot
  size, skipped files, recent activity (last ~50), Sync now, Disconnect, the public-repo warning.

## 17. Errors (registry)

`SYNC_AUTH` · `SYNC_OFFLINE` · `SYNC_TIMEOUT` · `SYNC_NOT_A_VAULT` · `SYNC_INVALID_MANIFEST` ·
`SNAPSHOT_TOO_NEW` · `SYNC_REJECTED` · `SYNC_RECONCILE_FAILED` · `SYNC_APPLY_FAILED`.

## 18. Code layout

```text
src-tauri/src/sync/
  git.rs         fetch, read tree, commit (incl. two-parent), push, clone
  snapshot.rs    write/read <table>.jsonl, deleted.jsonl, manifest
  reconcile.rs   §9 rules, per-table contracts, reference fixes
  apply.rs       §10: files, transaction, scan, reload
  token.rs       Keychain
  schedule.rs    triggers, lock, backoff, close
  state.rs       local config and status
```

Commands: `sync_config_get` · `sync_config_set` · `sync_token_set` · `sync_token_clear` ·
`sync_now` · `sync_status` · `sync_connect` · `sync_restore` · `sync_disconnect` · `activity_recent`.

---

## 19. Determinism and idempotency

The same local + remote inputs give the same rows, deletions, files, JSONL bytes and tree on every
device. Running a sync again changes nothing: no duplicate rows, a repeated deletion is harmless,
unchanged snapshots produce no diff.

## 20. Open items

- **Existing test vault:** exported on 2026-10-07 to `~/Documents/tend-v2-export/` (one JSONL per
  table, a `manifest.json` with row counts, and `app.sqlite.bak`). After the schema rework, a
  one-off import loads it into the new v1 layout; then that folder is deleted.
- **When:** make the schema changes now, or as part of the sync build.
- **Activity log size:** newest 10,000 rows (~1 MB) proposed.

## 21. Risks

- **Clock skew** decides "newest" wrongly (§7).
- **Outside renames** that the heuristic misses give a note a new id (§11.3).
- **"Newest wins" discards** the other edit with no copy — by decision.
- **Schema skew** between devices: an older app refuses newer snapshots (`SNAPSHOT_TOO_NEW`).
- **Long DB apply** makes app commands wait (§10).
- **Repo size limits** on hosts (GitHub soft 1–5 GB; 100 MB per file).
- **Token expiry** (fine-grained GitHub tokens) pauses sync until replaced.
- **Keychain prompts** may reappear when the app's signature changes (dev build vs `Tend.app`).
- **Host differences:** usernames, default branches, protected branches, login redirects.
- **OpenSSL vendored on macOS** (+size, perl at build); **SecureTransport is deprecated** by Apple.
- **Public repos** publish everything (warning only).

## 22. Acceptance criteria

**Repository** — connect to an empty remote · connect to an existing Tend remote · unrelated remote
refused · token works and lives only in the Keychain.

**Data** — every synced table has `id` + `updated_at` · notes/attachments model in place · title
change renames the file without breaking links · deletions recorded (incl. cascades) and kept.

**Sync** — pull always before push · independent changes converge on two devices · same row edited
on both: newer wins, older gone · delete beats edit · no Git merge or conflict ever surfaces ·
non-fast-forward push retried once · repeated syncs change nothing · failed runs leave a consistent
vault and retry later.

**Files** — notes and attachments sync as files · WAL/SHM never committed · files over 100 MB
skipped and listed · note index rebuilt by matching paths.

**Runtime** — one run at a time · watcher paused and rescanned · open note saved before and
reloaded after · open / interval / close (10 s) / Sync now all work · status icon and Settings
section show the state · auth failures stop auto-sync.

**Restore** — Start page and Settings (empty vault) restore a vault completely.
