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
  work/plan.md …                notes and folders: the vault folder is the notes root
  .tend/                        Tend's own folder (hidden)
    attachments/<id>.<ext>      attached files
    tend.db                     live database        — never committed
    tend.db-wal / -shm          SQLite side files    — never committed
    staging/                    files being applied  — never committed (§10)
    sync/
      manifest.json
      <table>/<YYYY>-<NNN>.jsonl  rows created in year YYYY, in id order, 10,000 per shard (§8)
      <table>/fixed.jsonl       rows with natural, non-ULID ids (settings, statuses, tags, labels, 'root')
  .git/
  .gitignore                    .tend/tend.db*, .tend/staging/, .DS_Store
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
merges them (§9.2).

**Natural ids.** Some rows take an id every device derives the same way, so independent creation of
"the same thing" gives one row: settings (`id` = key), statuses (`id` = value), tags (`id` = the
tag, a slug), labels (`id` = slug of the first name, never changed), root folders (`id` =
`'root'`).

### 4.2 Notes and attachments

A note is its own record; its body is a file, held by an attachment.

```text
notes        id · attachment_id · updated_at
attachments  id · path · name · mime_type · size_bytes · sha256 · updated_at   (+ local: mtime, missing_at)
```

- `attachments.path` is the file's place in the vault: `<folder>/<name>.md` for a note body,
  `.tend/attachments/<id>.<ext>` for an attached file.
- A note is **shown by its file name**; the **title is part of the file** (its `# heading`), so it
  syncs with the body. A new note's file is the title's slug and starts with `# <title>` (§4.4).
- Renaming a note **renames the file** (§11.2); only `attachments.path` changes. The note's `id`
  never changes, so tags, labels and links are unaffected by renames and moves.
- Editing a note writes the file and bumps `updated_at` on both rows.
- `attachment_links` always points at an attachment `id`. Linking a note means linking its
  attachment.

### 4.3 Local tables (never synced)

| Table | Purpose |
|---|---|
| `sync_state (id = 1, last_synced_commit, pending_commit, last_run_at, last_result)` | The base of the next merge (the commit both sides last agreed on), the commit being pushed, and how the last run went (§13). |
| `path_log (id, kind, reason, old_path, new_path, at)` | Every rename or move of a folder or note file, by the user, a scan fixing a name, or sync applying a rename, so an earlier layout can be restored. Never trimmed. |

There is no tombstone table and no write log: with a base, a deletion is simply a row the base has
and one side no longer has (§9.1).

Sync configuration (remote URL, branch, username, author, interval) is local too: it lives in the
app config folder, keyed by vault path, never in the vault (§5). The last result is in `sync_state`.

### 4.4 Name rule

Folder names (notes and tasks) and note file names: `^[a-z0-9]+(_[a-z0-9]+)*$`, at most 200
characters, unique in their folder (folders clash only with folders, notes only with notes).

- Typed folder names, note names and new-note titles become their **slug**: accents become plain
  letters, lowercase, each run of other characters one `_`, trimmed. A new-note title with no letter
  or digit gives `note`.
- A clash is **not an error**: the name gets a suffix, `plan` → `plan_1` → `plan_2`, when creating,
  renaming, moving, and when sync brings in a clash (§9.2).
- Names found on disk that break the rule get their slug from the scan once the vault allows it
  (see `decisions.md`); the rename goes to `path_log`.

Lowercase ASCII names avoid every case problem (no case-only renames, no `Plan` next to `plan` on a
disk that ignores case), Unicode normalization differences between devices, and characters other
file systems refuse.

---

## 5. Shared vs local

| Synced | Local to the device |
|---|---|
| tasks, task folders, comments, statuses, labels and their links, my_day, people, contacts, tags and their links, notes, note folders, attachments (rows and files), shared settings (`tasks.root_name`, `timers.sound`) | timers, feedback, `sync_state`, `path_log`, UI settings (nav, explorer, widths, last view, quick-create paths, hide done), sync configuration, the token |

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
4. Commit the result with **one parent: the fetched remote head**. Tend has already merged, so the
   history stays linear and the push is a fast-forward. Git never resolves conflicts; every device
   ends with the same tree.
5. Push. If rejected (another device pushed meanwhile): fetch, reconcile, commit and push again —
   **once**; a further failure waits for the scheduler.

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
  and cut into **shards of 10,000 rows**: `.tend/sync/<table>/<YYYY>-<NNN>.jsonl`, numbered from `001`.
  A year with 80,000 rows has `001`–`008`; row 80,001 starts `009`. There are no months and no
  stored layout: the shard of a row is its position in the year divided by 10,000, so every device
  builds the same files from the same rows.
- At ~1 KB per row a shard is at most ~10 MB, well under host per-file limits.
- New rows get ids at "now" and land in the year's last shard. A row inserted or deleted earlier
  in a year's id order shifts every later row of that year by one, so the later shards of that year
  are rewritten. This is accepted: it is bounded by one year, and Git stores a one-line shift as a
  tiny delta.
- Rows whose id is not a ULID (natural ids, §4.1: settings keys, status values, tags, label ids,
  `'root'`) go in `<table>/fixed.jsonl`, sorted by id.
- Only synced columns are written: local columns (`attachments.mtime`, `attachments.missing_at`,
  `note_folders.missing_at`) are left out, and of `settings` only the shared keys are written.
- Inside a shard: one row per line, fixed column order, stable JSON (serde structs), no values
  generated while writing. Same data → byte-identical file. An edit changes its line in place.
- `manifest.json`: `{ "app_id": "tend", "format_version": 1, "schema_version": 1, "tables": [...] }`.
  A remote with unrelated files and no valid manifest is refused (`SYNC_NOT_A_VAULT`); a manifest
  whose schema is newer than the app is refused (`SNAPSHOT_TOO_NEW`) and nothing is imported.
- Settings shows the snapshot size; warn above 200 MB.

Old years are rarely touched, so Git neither rewrites nor rereads them: they act as an archive
without any archive state in the database.

### 8.2 Writing shards

Every run **rebuilds every shard** from SQLite, never patching line by line, so shards are always a
faithful copy of the database. Only files whose bytes changed are written, and shard files no longer
produced are removed, so Git sees exactly what changed. At today's sizes this takes milliseconds.

Note bodies and attachment files are not in shards; Git sees their changes directly
(`git status` on the vault, leaving out `.tend/` except `.tend/sync/` and `.tend/attachments/`),
which also catches edits made in Finder.

*Later, if a vault grows large enough that the full rebuild is measurably slow:* a `changes` table
(trigger-filled, an ever-growing `seq` per changed row) can limit the rebuild to the years with
changes. The shard layout stays the same.

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

Delete beats edit in every case. A deleted ULID is never used again; a natural id (a tag, say) comes
back only by being created again after the deletion has synced, which the base then shows as an add.

Rows are matched by `id` across all shards of a table.

**There is always a base.** A vault is connected only to an empty remote (its first push is the
base), and every other device starts by restoring into an empty folder (the restored commit is its
base) (§14). If the base commit can no longer be read (the remote was rewritten), sync stops with
`SYNC_BASE_LOST` and asks for a fresh restore; it never guesses.

### 9.2 Unique keys after merging

Two devices can create different rows that collide on a unique key. These are merged, never dropped
with their content:

| Collision | Rule |
|---|---|
| Two link rows for the same pair (`task_tags`, `note_tags`, `task_label_links`, `my_day`, `attachment_links`) | keep the smaller id; the other is deleted |
| Two tags with the same name | cannot happen: the name is the id |
| Two labels or statuses created with the same name or value | cannot happen: the id is derived from it |
| Two labels renamed to the same name on different devices | keep both; the one with the larger id gets ` 2` appended to its name |
| Two folders with the same path (notes or tasks) | keep the smaller id (rows point at the path, so nothing moves) |
| Two notes with the same path | keep both; the note with the larger id gets the next free suffix (`plan_1`) and its file is renamed (logged in `path_log`) |

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
3. Write incoming files (note bodies, attachments) into `.tend/staging/`.
4. Apply rows to SQLite in **one transaction** (a backup copy of `tend.db` is taken just
   before); check references; commit. A failed apply rolls back, `.tend/staging/` is emptied, and the
   sync fails with the vault unchanged.
5. Move staged files into place and remove deleted ones; renames go to `path_log`. If a move fails, the
   affected files are restored from the last commit's tree.
6. Scan the vault, matching files to note rows by `attachments.path` (§11.3). Applied files hash the
   same as their rows, so the scan changes nothing.
   A note or task deleted by this sync while it is open on this device stays on screen and asks
   whether to keep it; **Keep** makes a new one (new id), so the deletion still wins for the old id
   (see `decisions.md`, Notes).
7. Resume the watcher; reload stores and reopen the open note if it changed.

Every app command waits for the DB while step 4 runs; it must stay short, and the UI shows
"Syncing…" meanwhile.

---

## 11. Notes

### 11.1 Creating
A title is given; the file is its slug (`meeting.md` for "Meeting"), starting with `# Meeting`, so
nothing is ever renamed after creation.

### 11.2 Renaming and moving
Rename → file renamed to the slug of what was typed; move → file moved, keeping its name; a name taken in
the target gets a suffix (§4.4). `attachments.path` and `updated_at` are updated and the rename is
logged in `path_log`. Links keep working because they point at ids.

### 11.3 Scanning
- The vault is the notes root. Hidden entries and non-`.md` files are ignored; folders without
  notes are ignored unless Tend made them.
- A file whose path matches an attachment row → that note; content changed (sha256) since the last
  scan → `updated_at = now`.
- A file no row knows: if a vanished row has the same sha256, it is that note renamed or moved
  outside Tend and keeps its id; otherwise it is a new note.
- A row whose file is gone → marked missing (grace period), then purged.
- An outside rename **combined with an edit** gets a new id — the one case where links break.

---

## 12. Attachments

Files live in `.tend/attachments/<id>.<ext>`. Files over **100 MB** are not committed; their row still
syncs, so other devices show "file not synced". Skipped files are listed in Settings → Sync.

---

## 13. A sync run

```text
lock (one run at a time; triggers during a run coalesce into it)
→ save open note · pause watcher
→ fetch
→ rebuild shards (§8.2)
→ idle check: git status clean · remote head = last_synced_commit  →  stop
→ read base, local and remote rows · reconcile (§9)
→ apply (§10), and in the same transaction: last_synced_commit = the fetched remote head
→ rebuild shards · commit on top of the remote head · pending_commit = that commit
→ push if ahead (retry once after a rejection)
→ sync_state: last_synced_commit = the pushed commit, pending_commit cleared, last run and result
→ resume watcher · reload stores · update status
```

- The base moves **with the apply**: once the remote's changes are in the database, the fetched
  head is what this device and the remote agree on, so it becomes the base in the same
  transaction. (Found by the simulation: if the base waited for the push, a crash in between and
  a later delete of a row that just came in would make the next run see it as a remote add and
  bring it back.) After a successful push the base moves again, to the pushed commit. A run cut
  short before the apply changes nothing.
- **A crash after the push** but before the base moves: the next run first checks `pending_commit`.
  If the fetched remote contains it, it becomes the base; otherwise it is dropped. (Also found by
  the simulation: without this, a row created and pushed, then deleted locally, would look like a
  remote add and come back.)
- **On app open** the order is always: open the vault → scan → sync. Edits made while Tend
  was closed are indexed (with their `updated_at`) before the merge sees them.

---

## 14. Connecting and restoring

**Connect** (Settings → Sync): URL and token, with branch (default `main`) and username (default
`x-access-token`) under "More". Commits are authored "Tend <tend@localhost>"; there is no device
name. New connections sync every 15 min. The remote must be **empty**; one that already has commits
is refused (`SYNC_REMOTE_NOT_EMPTY`: "This repo already has a vault. Restore it into an empty folder
instead."). Tend writes the shards, commits, pushes and records that commit as the base. If any step
fails, nothing in the vault changes except `.git`, and the token is removed again.

**Token:** kept only in the system credential store (the macOS Keychain, service `app.tend.sync`,
account = the remote URL). It is never written to the vault, the database or a config file, and
never sent to the UI. The other sync settings live in the app config folder, keyed by vault path.

**Restore** (Start page only): choose an empty folder (`RESTORE_NOT_EMPTY` otherwise), URL, token
(branch and username under "More") → clone → validate manifest → create the database → **write files → import rows → scan**
→ record the cloned commit as the base → open.

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

- **Dock icon** next to the timer chip: ✓ synced · ↻ syncing · ! error (red); hover = last sync
  or the error; click → Settings → Sync. Hidden when the vault isn't connected.
- **Settings → Sync:** not connected: the connect fields and Connect. Connected: URL, branch,
  token status (Replace / Add), interval (off / 15 min / 1 h), last sync and its result, Sync now,
  Disconnect (with a confirmation), the public-repo warning.
- A run that changed rows reloads every store (`sync:applied`); an open note or task that is gone
  afterwards offers to keep it (decisions: keep-deleted).
- The runner works on its own database connection (5 s busy timeout), so commands never wait on
  the network. The file watcher keeps running during an apply; it only re-indexes what the apply
  already wrote.
- Later: snapshot size and the list of files left out (over 100 MB).

---

## 17. Errors (registry)

`SYNC_AUTH` · `SYNC_OFFLINE` · `SYNC_TIMEOUT` · `SYNC_NOT_A_VAULT` · `SYNC_INVALID_MANIFEST` ·
`SNAPSHOT_TOO_NEW` · `SYNC_REMOTE_NOT_EMPTY` · `SYNC_BASE_LOST` · `SYNC_REJECTED` ·
`SYNC_RECONCILE_FAILED` · `SYNC_APPLY_FAILED`.

## 18. Code layout

```text
src-tauri/src/sync/
  shards/        export rows to year shards and fixed.jsonl, read them back, manifest   (S1)
  reconcile/     §9: three-way merge, unique keys, reference fixes; simulation test   (S2)
  apply/         §10: plan and stage files, write rows in one transaction, install    (S3)
  git.rs         the libgit2 boundary: fetch, read trees, build a tree, commit, push  (S4)
  state.rs       the sync_state row: base, pending commit, last result               (S4)
  run.rs         connect, restore, one sync run (retried once on a rejected push)     (S4)
  token.rs       the token in the Keychain                                            (S5)
  runner.rs      one thread per connected vault: triggers, backoff, close, UI events  (S5)
```

Commands (`commands/sync.rs`): `sync_status` · `sync_connect` · `sync_restore` · `sync_now` ·
`sync_set_interval` · `sync_token_set` · `sync_disconnect`.

---

## 18a. Schema changes

Tend will keep growing, so schema changes must not need sync work. Migrations run through
`rusqlite_migration` (`src-tauri/src/db`): `docs/schema.sql` is migration 1, edited in place until
the first release; every later change is a new step, and the version lives in `PRAGMA user_version`.

Rules for every step after release:

1. **Add only.** New tables, and new columns with a constant default (or nullable). No renames or
   drops in place: add the new column, copy into it, switch to it, and drop the old one in a later
   release.
2. **Missing means default.** A snapshot row without a column, or a table with no files, reads as
   the column's default or as empty, so a base written before an upgrade never looks changed.
3. **Deterministic.** No `now()` and no random ULIDs in a migration; rows it creates get ids derived
   from their source, so every device migrating the same data makes the same rows.
4. **An older app never writes a newer snapshot.** A manifest with a higher schema version stops
   sync on that device (`SNAPSHOT_TOO_NEW` → "Update Tend to keep syncing"); the app keeps working
   locally.
5. **Rebuild every shard after a migration** (§8.2), so snapshots carry the new columns.
6. **Fixtures:** each released schema keeps a small snapshot fixture; tests migrate and sync each one
   against the current app.

With these, adding a table or column is a migration step and a default, nothing in sync changes.

## 19. Determinism, idempotency and testing

The same base, local and remote inputs give the same rows, files, shard bytes and tree on every
device. Running a sync again changes nothing: no duplicate rows, a repeated deletion is harmless,
unchanged shards produce no diff.

`reconcile.rs` is pure, so it is checked by a **simulation test** before any Git code is written:
three simulated devices and a simulated remote, thousands of random operations (create, edit,
delete, rename, move, link, tag), clocks set wrong on purpose, crashes in the
middle of a run, random sync order. After every round it asserts:

- all devices converge to identical state;
- no reference points at a missing row;
- a deleted row never comes back;
- a change made on only one side is never lost;
- syncing twice in a row produces no new commit.

Each seed replays exactly. The simulation lives in `sync/reconcile/sim_tests.rs` (150 seeds × 120
steps in the normal test run; 2,000 × 300 also pass). It found the two base-moving rules in §13.

## 20. Open items

- **Many files:** at tens of thousands of note files, `git status` and the scan walk every file;
  measure with libgit2 before release.
- **Large vaults:** the `changes` table (§8.2) if the full shard rebuild gets slow.

## 21. Risks

- **Clock skew** decides "newest" wrongly when both sides edited the same row (§7).
- **Outside rename plus edit** gives a note a new id (§11.3).
- **"Newest wins" discards** the other edit with no copy — by decision. A note or task deleted on
  another device while it is open here can be kept as a new one (`decisions.md`).
- **Schema skew** between devices: an older app refuses newer snapshots (`SNAPSHOT_TOO_NEW`).
- **Long DB apply** makes app commands wait (§10).
- **Repo size limits** on hosts (GitHub soft 1–5 GB; 100 MB per file).
- **Token expiry** (fine-grained GitHub tokens) pauses sync until replaced.
- **Keychain prompts** may reappear when the app's signature changes (dev build vs `Tend.app`).
- **Host differences:** usernames, default branches, protected branches, login redirects.
- **OpenSSL vendored on macOS** (+size, perl at build); **SecureTransport is deprecated** by Apple.
- **Public repos** publish everything (warning only).
- **The UI loads every row** (task list, notes tree). Fine at 10k, slow near 100k; paginated lists
  and SQLite full-text search are a separate project.

## 22. Acceptance criteria

**Repository** — connect to an empty remote · a non-empty remote refused on connect · restore from a
Tend remote into an empty folder · unrelated remote refused · token works and lives only in the
Keychain.

**Data** — every synced table has `id` + `updated_at` · notes/attachments model in place · renaming
a note renames the file without breaking links · name rule with `_1` suffixes.

**Sync** — pull always before push · a change on one side always survives · same row edited on both:
newer wins, older gone · delete beats edit · unique-key collisions merged (§9.2) · no Git merge or
conflict ever surfaces · non-fast-forward push retried once · repeated syncs change nothing · an
idle sync writes nothing · a lost base stops sync and asks for a restore · failed runs leave a consistent vault and
retry later · simulation test passes.

**Files** — notes and attachments sync as files · WAL/SHM and staging never committed · files over
100 MB skipped and listed · renames logged · note index rebuilt by matching paths.

**Runtime** — one run at a time · open → scan → sync · watcher paused and rescanned · open note
saved before and reloaded after · open / interval / close (10 s) / Sync now all work · status icon
and Settings section show the state · auth failures stop auto-sync.

**Restore** — the Start page restores a vault completely into an empty folder.
