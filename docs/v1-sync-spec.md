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
- **A change made on one side always survives.** Only when both sides changed the same row does the
  newer one win; the older version is discarded and nothing is kept as a conflict copy.
- **A delete always wins** and is never undone by a later edit elsewhere.
- Sync runs are retryable and idempotent.
- **Scale target:** 100k rows per table, built up over many years, with sync cost that grows with
  what changed, not with the size of the vault.

## 2. Non-goals (V1)

SSH remotes · Git LFS · choosing which folders sync · sub-minute sync · field-level merging or
collaborative editing · conflict copies or user-facing conflict resolution · Git history or diffs
inside Tend · mobile (iOS/Android) · journaling or multi-step rollback · archiving rows out of the
database.

---

## 3. Vault layout

The vault is the Git working tree:

```text
<vault>/
  notes/                        note files (body only), named by their titles
  .attachments/                 attachment files, <id>.<ext>
  db/
    app.sqlite                  live database        — never committed
    app.sqlite-wal / -shm       SQLite side files    — never committed
    staging/                    files being applied  — never committed (§10)
    sync/
      manifest.json
      <table>/<YYYY>-<NNN>.jsonl  rows created in year YYYY, in id order, 10,000 per shard (§8)
      <table>/fixed.jsonl       rows with fixed, non-ULID ids (settings keys, 'root', default statuses)
      deleted/<YYYY>.jsonl      deletions, by the year they happened
  .git/
  .gitignore                    db/app.sqlite*, db/staging/, .DS_Store
```

There is no `.conflicts/` folder.

---

## 4. Schema (v1, edited in place)

Nothing has shipped, so `docs/schema.sql` stays **v1** and is edited directly. Migrations start with
the first release.

### 4.1 Identity and change time on every table

Every synced table has:

- `id TEXT PRIMARY KEY` — a ULID generated on the device, never replaced.
- `updated_at INTEGER NOT NULL` — epoch ms, set on every insert and update.

| Table kind | Shape |
|---|---|
| Entity tables (`tasks`, `people`, `tags`, `task_labels`, `attachments`, …) | `id` + `updated_at` |
| Link tables (`task_tags`, `note_tags`, `task_label_links`, `my_day`, `attachment_links`) | `id`; the key pair is `UNIQUE` |
| Name/path-keyed tables (`statuses` by value, `task_folders` / `note_folders` by path, `settings` by key) | `id` is the primary key; the old key stays `UNIQUE` |
| `task_comments` | `updated_at` (no separate `edited_at`) |

Renaming a folder or a status is an update of one row, not a new row.

Link rows get a fresh ULID each time a link is made, never one derived from the pair, so a removed
link can always be added again. Two devices can make the same pair under different ids; reconciling
merges them (§9.2). Ids are fixed only for rows that are never deleted: settings (`id` = key), the
default statuses (`id` = value) and the root folders (`id` = `'root'`).

### 4.2 Notes and attachments

A note is its own record; its body is a file, held by an attachment.

```text
notes        id · attachment_id · updated_at
attachments  id · path · name · mime_type · size_bytes · sha256 · updated_at   (+ local: mtime, missing_at)
```

- `attachments.path` is the file's place in the vault: `notes/<folder>/<name>.md` for a note body,
  `.attachments/<id>.<ext>` for an attached file.
- A note's **title is its file name** without `.md`, so it syncs with `attachments.path`. Tend does
  not read a `# heading` from the file, and new notes start with an empty body.
- Renaming a note **renames the file** (§11.2); only `attachments.path` changes. The note's `id`
  never changes, so tags, labels and links are unaffected by renames and moves.
- Editing a note writes the file and bumps `updated_at` on both rows.
- `attachment_links` always points at an attachment `id`. Linking a note means linking its
  attachment.

### 4.3 Local tables (never synced)

| Table | Purpose |
|---|---|
| `deletions (id, table_name, row_id, deleted_at)` | Every delete, including `ON DELETE CASCADE` rows, by trigger. Mirrored into `db/sync/deleted/` (§8). Kept **forever**. |
| `changes (seq INTEGER PRIMARY KEY AUTOINCREMENT, table_name, row_id, UNIQUE(table_name, row_id))` | Which rows changed locally, by trigger on insert, update and delete of every synced table: `INSERT OR REPLACE`, so each row has one entry carrying its latest `seq`. `seq` only grows and never depends on the clock. Used to find dirty shards (§8.2). |
| `sync_state (id = 1, last_synced_seq, last_synced_commit)` | The markers of the last successful sync (§13). |
| `access (entity_type, entity_id, used_at, UNIQUE(entity_type, entity_id))` | Last read, edit or create of each entity. Drives "recent" lists, ⌘K ranking and what the UI loads first. Never in snapshots, never in `changes`. |
| `activity (id, action, status, entity_type, entity_id, at)` | Every write plus open, close, pull and push; capped to the newest 10,000 rows by a trigger. |

Sync configuration (remote URL, branch, username, author, device name, interval, last result) is
local too and never lives in the vault (§5).

### 4.4 Name rule

Folder names (notes and tasks) and note names:

- Allowed characters: `A–Z a–z 0–9`, space, and `. , _ -`.
- Leading and trailing spaces and dots are **trimmed** automatically; what is left must not be empty.
- At most 200 characters.
- Unique within the folder, **ignoring case** (as macOS does). Folders clash only with folders, notes
  only with notes.
- A clash is **not an error**: the new name gets a suffix, `Plan` → `Plan_1` → `Plan_2`, when
  creating, renaming, moving, and when sync brings in a clash (§9.2).
- Checked in Rust; the UI shows the same rule live. Existing names on disk that break it are listed
  as skipped, never renamed by a scan.

ASCII-only names avoid Unicode normalization differences between devices (macOS can store "é" in a
different byte form than Git) and characters other file systems refuse.

---

## 5. Shared vs local

| Synced | Local to the device |
|---|---|
| tasks, task folders, comments, statuses, labels and their links, my_day, people, contacts, tags and their links, notes, note folders, attachments (rows and files), shared settings (`tasks.root_name`, `timers.sound`) | timers, feedback, `deletions` (beyond what `deleted/` carries), `changes`, `sync_state`, `access`, `activity`, UI settings (nav, explorer, widths, last view, quick-create paths), sync configuration, the token |

The shared/local split of settings is defined in one place (`settings.config.ts` and its Rust twin).
Local columns of synced tables (`attachments.mtime`, `attachments.missing_at`,
`note_folders.missing_at`) are never written to snapshots, and changes to them alone do not make a
row dirty.

---

## 6. Git transport

- **HTTPS + personal access token**, optional username. No SSH.
- **Token:** macOS Keychain via `keyring-core` + `apple-native-keyring-store`. Never written to the
  vault, SQLite or a config file; loaded into memory only for a Git call. Settings shows only
  "Token saved ✓ · Replace · Remove".
- **Engine:** `git2` 0.21 in-process (`https`, `vendored-libgit2`, `vendored-openssl` on macOS —
  ~+4.3 MB, perl at build time). Credentials callback with a bounded number of attempts.

### 6.1 How history is built (no Git merge)

1. **Fetch** the remote branch.
2. **Read trees directly** from Git objects: the base (`last_synced_commit`), the fetched remote
   commit, and the local working state — no `git merge`, no working-tree merge.
3. Reconcile in Tend (§9) and write the result into the working tree (§10).
4. Commit the result. When the remote moved, the commit has **two parents**: the local HEAD and the
   fetched remote commit. Git never resolves conflicts; every device ends with the same tree.
5. Push. If rejected as non-fast-forward (another device pushed meanwhile): fetch, reconcile,
   commit and push again — **once**; a further failure waits for the scheduler.

---

## 7. Change time

`updated_at = now` (epoch ms) on every insert and update.

The clock only matters when **both sides changed the same row** since the base (§9.1); then the
newer `updated_at` wins. Every other decision compares content against the base and ignores time.

**Known risk:** in that both-changed case, a device whose clock is wrong can make an older edit win.
Accepted for V1; a hybrid logical clock is a later decision.

---

## 8. Snapshots

### 8.1 Files

- Rows are grouped by the **year** their ULID was created (UTC, read from the id), sorted by `id`,
  and cut into **shards of 10,000 rows**: `db/sync/<table>/<YYYY>-<NNN>.jsonl`, numbered from `001`.
  A year with 80,000 rows has `001`–`008`; row 80,001 starts `009`. There are no months and no
  stored layout: the shard of a row is its position in the year divided by 10,000, so every device
  builds the same files from the same rows.
- At ~1 KB per row a shard is at most ~10 MB, well under host per-file limits.
- New rows get ids at "now" and land in the year's last shard. A row inserted or deleted earlier
  in a year's id order shifts every later row of that year by one, so the later shards of that year
  are rewritten. This is accepted: it is bounded by one year, and Git stores a one-line shift as a
  tiny delta.
- Rows whose id is not a ULID (settings keys, `'root'`, default status values) go in
  `<table>/fixed.jsonl`.
- Inside a shard: one row per line, fixed column order, stable JSON (serde structs), no values
  generated while writing. Same data → byte-identical file. An edit changes its line in place.
- `deleted/<YYYY>.jsonl`: `{"table":"tasks","id":"01…","deleted_at":…}` per line, sorted by
  `(table, id)`, sharded by the year of `deleted_at`, never truncated (≈8 MB for 100k deletes).
- `manifest.json`: `{ "app_id": "tend", "format_version": 1, "schema_version": 1, "tables": [...] }`.
  A remote with unrelated files and no valid manifest is refused (`SYNC_NOT_A_VAULT`); a manifest
  whose schema is newer than the app is refused (`SNAPSHOT_TOO_NEW`) and nothing is imported.
- Settings shows the snapshot size; warn above 200 MB.

Old years are rarely touched, so Git neither rewrites nor rereads them: they act as an archive
without any archive state in the database.

### 8.2 Dirty shards

A **year** is dirty when a row created in it changed locally since the last sync (rows can move
between a year's shards, so the year is the unit):

```sql
SELECT table_name, row_id FROM changes WHERE seq > :last_synced_seq
```

Each `(table, id)` maps to exactly one year. A dirty year's shards are **rebuilt completely** from
SQLite, never patched line by line, so shards are always a faithful copy of the database. Files whose
bytes did not change are left alone, and clean years are not touched.

Note bodies and attachment files are not in shards; Git sees their changes directly
(`git status` on `notes/` and `.attachments/`), which also catches edits made in Finder.

A **full rebuild** (every shard) runs when there are no markers yet (first sync, restore), after a
schema upgrade, or when the base commit can no longer be read. It uses the same code with "every
row" as the change set.

---

## 9. Reconciliation

### 9.1 Three-way merge

Every row is judged against three versions:

- **base** — the row in `last_synced_commit`, the state both devices last agreed on;
- **local** — the row in this device's database;
- **remote** — the row in the fetched remote commit.

Rows are compared by their snapshot line (synced columns only).

| Base | Local | Remote | Result |
|---|---|---|---|
| A | A | B | remote changed → **B** |
| A | B | A | local changed → **B** |
| A | B | B | same change on both → **B** |
| A | B | C | both changed → **newer `updated_at`** wins; equal times: the line that sorts higher |
| A | A or B | missing | remote deleted → **deleted** |
| A | missing | A or B | local deleted → **deleted** |
| — | new | — | local added → **kept** |
| — | — | new | remote added → **kept** |
| — | X | Y | added on both with the same id (fixed ids only) → newer `updated_at` wins |

Delete beats edit in every case, and a deleted id is never used again.

Only rows in **changed years** are read: local dirty years (§8.2) and years with a shard that differs
between base and remote (`git diff base..remote` over `db/sync/`). Rows are matched by `id` across
all shards of a year. A row outside these years is identical in all three versions and is skipped.
The merged result is written back only into the shards of those years.

**No base** (first connect of a vault that already has data, restore into a non-empty vault, or a
base commit lost to a force-push): fall back to two-way rules —

| Case | Result |
|---|---|
| Row on both sides, different | newer `updated_at` wins |
| Row on one side, with a tombstone on the other (`deleted/`) | deleted |
| Row on one side, no tombstone | kept (absence is not deletion) |

Tombstones exist for this fallback; with a base, deletions are seen by comparing against it.

### 9.2 Unique keys after merging

Two devices can create different rows that collide on a unique key. These are merged, never dropped
with their content:

| Collision | Rule |
|---|---|
| Two link rows for the same pair (`task_tags`, `note_tags`, `task_label_links`, `my_day`, `attachment_links`) | keep the smaller id; the other is deleted |
| Two tags or two labels with the same name | keep the smaller id; every link to the other is **moved** to it; then the other is deleted |
| Two statuses with the same value | keep the smaller id |
| Two folders with the same path (notes or tasks) | keep the smaller id (rows point at the path, so nothing moves) |
| Two notes with the same path | keep both; the note with the larger id gets the next free suffix (`Plan_1`) and its file is renamed |

### 9.3 References after merging

Dependencies are applied parent-first. Rows pointing at something that no longer exists are fixed,
never left dangling:

| Reference | Rule |
|---|---|
| comment, tag/label link, contact, my_day, attachment link → deleted parent | dropped (and recorded in deletions) |
| task → deleted status | status set to the first status |
| task → deleted assignee | unassigned |
| task → deleted folder / parent task | moved to the root / dropped with its parent |
| note → deleted folder | moved to the notes root; a name clash there gets a suffix (§4.4) |

The full per-table contract (identity, unique keys, dependencies) is written before each table is
implemented.

---

## 10. Applying the result

1. Save any note open in the editor (so autosave can't write stale text after the pull).
2. Pause the notes watcher.
3. Write incoming files (note bodies, attachments) into `db/staging/`.
4. Apply rows to SQLite in **one transaction** (a backup copy of `app.sqlite` is taken just
   before); check references; commit. A failed apply rolls back, `db/staging/` is emptied, and the
   sync fails with the vault unchanged.
5. Move staged files into place and remove deleted ones. A **case-only rename** (`plan.md` →
   `Plan.md`) goes through a temporary name, because the disk ignores case. If a move fails, the
   affected files are restored from the last commit's tree.
6. Scan `notes/`, matching files to note rows by `attachments.path` (§11.3). Applied files hash the
   same as their rows, so the scan changes nothing.
7. Resume the watcher; reload stores and reopen the open note if it changed.

Every app command waits for the DB while step 4 runs; it must stay short, and the UI shows
"Syncing…" meanwhile.

---

## 11. Notes

### 11.1 Creating
A name is given (§4.4); the file is `<name>.md` and the body starts empty.

### 11.2 Renaming and moving
Rename → file renamed; move → file moved; a name taken in the target gets a suffix (§4.4).
`attachments.path` and `updated_at` are updated. Links keep working because they point at ids.

### 11.3 Scanning
- A file whose path matches an attachment row → that note; content changed (sha256) since the last
  scan → `updated_at = now`.
- A file no row knows: if a vanished row has the same sha256, it is that note renamed or moved
  outside Tend and keeps its id; otherwise it is a new note.
- A row whose file is gone → marked missing (grace period), then purged.
- An outside rename **combined with an edit** gets a new id — the one case where links break.

---

## 12. Attachments

Files live in `.attachments/<id>.<ext>`. Files over **100 MB** are not committed; their row still
syncs, so other devices show "file not synced". Skipped files are listed in Settings → Sync.

---

## 13. A sync run

```text
lock (one run at a time; triggers during a run coalesce into it)
→ save open note · pause watcher
→ fetch
→ idle check: no dirty years · git status clean · remote head = last_synced_commit  →  stop
→ rebuild dirty years' shards (§8.2)
→ read base, local and remote changed rows · reconcile (§9)
→ apply (§10) · write changed shards · commit (two parents if the remote moved)
→ push if ahead (retry once after a rejection)
→ in one transaction: last_synced_seq = max(changes.seq), last_synced_commit = pushed head
→ record pull/push in activity · resume watcher · update status
```

- The markers move **only after a successful push**. A failed or cut-short run leaves them, so the
  next run sees the same changes again.
- Rows written by the apply step also enter `changes`; recording `last_synced_seq` at the end keeps
  them from counting as local changes next time.
- A crash after the push but before the markers are saved is harmless: the next run reconciles the
  same data, finds nothing new to commit and only moves the markers.
- **On app open** the order is always: open the vault → scan `notes/` → sync. Edits made while Tend
  was closed are indexed (with their `updated_at`) before the merge sees them.

---

## 14. Connecting and restoring

**Connect** (Settings → Sync): URL, branch (default `main`), optional username, token, author
name/email (default "Tend"), device name (default: the Mac's name), interval.
The remote must be empty or hold a valid manifest. Tend clones into a temporary folder, reconciles
with the local vault (two-way, §9.1), moves `.git` into the vault, commits and pushes. If any step
fails, the temporary clone is removed and the vault is untouched.

**Restore** (Start page, or Settings → Sync on an empty vault): choose an empty folder, URL, token
(branch and username under "More") → clone → validate manifest → create the database → **write
files → import rows → scan `notes/`** → set the markers → open. On a vault that already has data,
Settings offers no restore; connecting reconciles instead.

**Disconnect:** clears the remote, token, schedule and markers; `.git` and all data stay.

**Public repos** are allowed. A static warning on the Start-page restore and in Settings → Sync:
"⚠ Beware of public repos: anything in your vault, private content included, is pushed there."

---

## 15. Scheduling, close, failures

- **Triggers:** app open (after the scan, §13) · interval (off / 15 min / 1 h) · app close ·
  **Sync now**.
- **Close:** Tauri close is intercepted; "Syncing…" shows; the app quits when pull (and push, if
  needed) succeed or after **10 s**. A cut-short run isn't treated as successful; the next open
  syncs again.
- **Time limits:** each network step has a deadline from config; the first push of a large vault
  and a restore clone get a longer one, with progress.
- **Failures:** retried by the scheduler with backoff up to 1 h; one toast on the first failure,
  then only the dock icon. Auth failures (bad or expired token) stop automatic sync until fixed.
  A failed run leaves the database and working tree consistent (§10).

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
  git.rs         fetch, read trees and diffs, commit (incl. two-parent), push, clone
  shards.rs      year of a row, dirty years, rebuild a year's shards, read/write deleted/, manifest
  reconcile.rs   §9: three-way and two-way rules, unique keys, reference fixes (pure functions)
  apply.rs       §10: staging, transaction, file moves, scan, reload
  token.rs       Keychain
  schedule.rs    triggers, lock, backoff, close
  state.rs       local config, markers and status
```

Commands: `sync_config_get` · `sync_config_set` · `sync_token_set` · `sync_token_clear` ·
`sync_now` · `sync_status` · `sync_connect` · `sync_restore` · `sync_disconnect` · `activity_recent`.

---

## 19. Determinism, idempotency and testing

The same base, local and remote inputs give the same rows, files, shard bytes and tree on every
device. Running a sync again changes nothing: no duplicate rows, a repeated deletion is harmless,
unchanged shards produce no diff.

`reconcile.rs` is pure, so it is checked by a **simulation test** before any Git code is written:
three simulated devices and a simulated remote, thousands of random operations (create, edit,
delete, rename including case-only, move, link, tag), clocks set wrong on purpose, crashes in the
middle of a run, random sync order. After every round it asserts:

- all devices converge to identical state;
- no reference points at a missing row;
- a deleted row never comes back;
- a change made on only one side is never lost;
- syncing twice in a row produces no new commit.

Failures shrink to the smallest sequence of steps that reproduces them.

## 20. Open items

- **Code to change before sync:** the name rule (§4.4, today's code allows any safe text and refuses
  clashes), the `changes`, `sync_state` and `access` tables, and open → scan → sync ordering.
- **Many files:** at tens of thousands of note files, `git status` and the scan walk every file;
  measure with libgit2 before release.

## 21. Risks

- **Clock skew** decides "newest" wrongly when both sides edited the same row (§7).
- **Outside rename plus edit** gives a note a new id (§11.3).
- **"Newest wins" discards** the other edit with no copy — by decision.
- **Schema skew** between devices: an older app refuses newer snapshots (`SNAPSHOT_TOO_NEW`).
- **Long DB apply** makes app commands wait (§10).
- **Repo size limits** on hosts (GitHub soft 1–5 GB; 100 MB per file).
- **Token expiry** (fine-grained GitHub tokens) pauses sync until replaced.
- **Keychain prompts** may reappear when the app's signature changes (dev build vs `Tend.app`).
- **Host differences:** usernames, default branches, protected branches, login redirects.
- **OpenSSL vendored on macOS** (+size, perl at build); **SecureTransport is deprecated** by Apple.
- **Public repos** publish everything (warning only).
- **The UI loads every row** (task list, notes tree). Fine at 10k, slow near 100k; paginated lists
  and SQLite full-text search are a separate project, guided by `access`.

## 22. Acceptance criteria

**Repository** — connect to an empty remote · connect to an existing Tend remote · unrelated remote
refused · token works and lives only in the Keychain.

**Data** — every synced table has `id` + `updated_at` · notes/attachments model in place · renaming
a note renames the file without breaking links · deletions recorded (incl. cascades) and kept ·
`changes` records every synced write · name rule with `_1` suffixes.

**Sync** — pull always before push · a change on one side always survives · same row edited on both:
newer wins, older gone · delete beats edit · unique-key collisions merged (§9.2) · no Git merge or
conflict ever surfaces · non-fast-forward push retried once · repeated syncs change nothing · an
idle sync writes nothing · only dirty years are rebuilt · failed runs leave a consistent vault and
retry later · simulation test passes.

**Files** — notes and attachments sync as files · WAL/SHM and staging never committed · files over
100 MB skipped and listed · case-only renames applied · note index rebuilt by matching paths.

**Runtime** — one run at a time · open → scan → sync · watcher paused and rescanned · open note
saved before and reloaded after · open / interval / close (10 s) / Sync now all work · status icon
and Settings section show the state · auth failures stop auto-sync.

**Restore** — Start page and Settings (empty vault) restore a vault completely.
