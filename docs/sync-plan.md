# Git sync — plan

Status: **superseded by `docs/v1-sync-spec.md`** — kept as the record of the discussion.

Original status: design, not built. This file collects what Tend has today, what we want, every
question raised so far, what is decided, what is still open, and the problems we expect.
Decisions are numbered (Q1…) in the order they came up, so the discussion can refer to them.

---

## 1. The idea

Back up (and sync) the whole vault to a git remote over HTTPS, on a schedule, without the user
running git. The user gives a repo URL and a token with write access; Tend commits and pushes the
vault's contents, and always pulls before it pushes. A vault can be restored from that remote on
another machine (or the same one, after a loss).

---

## 2. What Tend has today (facts the design builds on)

**The vault folder** (chosen on the Start page; the last one is remembered in the app config
dir, `~/Library/Application Support/io.github.rniverse.tend/config.json`):

```
<vault>/
  notes/          markdown files and folders — the source of truth for notes
  .attachments/   files copied in by the user, stored as <id>.<ext>
  db/app.sqlite   SQLite (WAL mode: also app.sqlite-wal, app.sqlite-shm while open)
```

**Only notes are files.** Everything else lives in `db/app.sqlite` (schema **v2**:
`docs/schema.sql` is v1, `src-tauri/src/db/migrations/002_task_labels.sql` is v2):

| Table | What | Notes for sync |
|---|---|---|
| `meta` | app id, schema version | per DB |
| `settings` | app prefs (key → JSON) | mixes shared prefs and per-device UI state (see §8) |
| `tags`, `note_tags`, `task_tags` | tag pool and links | shared |
| `note_folders`, `notes` | **index** of `notes/` (ids, titles, missing_at) | rebuilt by scanning disk, but ids are referenced by tags and attachment links (see §8) |
| `people`, `person_contacts` | people | shared |
| `statuses` | task statuses | shared |
| `task_folders`, `tasks`, `task_comments` | tasks | shared |
| `my_day` | per-day manual My day choices | shared (keyed by local date) |
| `task_labels`, `task_label_links` | labels (v2) | shared |
| `attachments`, `attachment_links` | attachment metadata + links (files live in `.attachments/`) | shared |
| `timers` | running timers | per device? (see §8) |
| `feedback` | feedback the user wrote | shared or dropped (see §8) |

**Already in place that sync can reuse:**
- Rust owns all disk and DB work; commands run off the main thread (`#[tauri::command(async)]`).
- Every row id is a ULID, generated on the device — rows from two machines never collide.
- A file watcher re-indexes `notes/` after outside changes; the scan skips dot-folders
  (`.git`, `.conflicts`, `.attachments` are invisible to the notes explorer).
- One error registry (`src-tauri/src/error/registry.rs`) and one UI error handler (toast).
- Schema migrations run on open (`db::migrate`), refusing a DB newer than the app.
- Settings → Tasks already has the list/section patterns; the dock (bottom bar) has a timer chip.

---

## 3. Decisions so far

### Settled

| # | Topic | Decision |
|---|---|---|
| — | Direction | **Always pull before push.** Whether we call it backup or sync doesn't matter; two machines may share one remote. |
| Q1 | DB in git | The live `app.sqlite` (+ `-wal`, `-shm`) is **never** committed. Each sync writes **one** text snapshot, `db/snapshot.sql`, overwritten every time: header with schema version, tables in a fixed order, rows sorted by id, **one row per line** — so a sync changes only the lines of rows that changed, diffs are readable, and history grows by those lines only. *(Exact text format still open — see P8.)* |
| Q1b | Snapshot size | Estimated 3–10 MB for heavy personal use (10k tasks, 1k people, comments); note text and attachments aren't in it. Show its size in Settings → Sync; warn above 50 MB. |
| Q2 | Note conflicts | Conflicts are expected to be rare (pull before push + interval). When git can't merge a note, keep ours in place and write theirs to **`.conflicts/<name>_conflict_<yyyymmdd_hhmm>.md`**. Nothing is lost. |
| Q3 | Auth | **HTTPS + token only** (no SSH). Optional username (GitHub accepts anything; GitLab/Gitea want the real one). The token lives in the **macOS Keychain**, never in the vault, the DB or the config file. The user pastes it once in Settings → Sync and sees only "Token saved ✓ · Replace · Remove"; macOS may ask once to allow Tend to use the keychain. |
| Q4/Q12 | Public repos | **Allowed, no check.** A static warning on the Start-page restore and in Settings → Sync: "⚠ Beware of public repos: anything in your vault, private content included, is pushed there." |
| Q5 | Restore | **In both places.** Start page: "Restore from git" → URL, token, an empty folder → clone, rebuild the DB from the snapshot, open it. Settings: choose/create a new (empty) vault first, configure sync there, then "Restore" pulls it in. *(Non-empty vault: P3.)* |
| Q6 | When it runs | Configured in Settings → Sync: **on app open**, then on an **interval (15 min / 1 h)**, and **on close**. |
| Q6b | Activity log | A new `activity` table — `action, status, entity_type, entity_id, at` — capped at ~2 MB, newest rows kept. **Every write** is recorded, plus **open, close, pull, push**. It stays **on this machine**: never in the snapshot, never pushed. |
| Q7 | What's pushed | The whole vault: `notes/`, `.attachments/`, `db/snapshot.sql` (plus `.gitignore`, `.conflicts/`). Notes and attachments are expected to be small; files over 100 MB are skipped with a warning. |
| Q8 | Commits | Author name/email from Sync settings (default "Tend"). Message: `Tend sync · 2026-10-05 14:30 · <device name>`. |
| Q9 | Git engine | **`git2` 0.21** (libgit2, built into the app) with `https` + `vendored-libgit2`. Only Rust option that does clone, fetch, merge, commit and push; `gix` can't push yet; system `git` isn't on a fresh Mac and pops an install dialog. Token stays in memory (Keychain → credentials callback). Token storage: **`keyring-core`** + `apple-native-keyring-store`. Licence: libgit2 is GPLv2 with a linking exception — OK to ship; credit it in an acknowledgements screen. |
| Q15 | Skipping | **Skip the push when nothing changed; the pull still happens.** "Nothing changed" = no writes in the activity log since the last push *and* git status is clean (git status also catches edits made in Finder). |
| Q16 | Mobile | Deferred. Findings kept for later: git2 and keyring stores work on iOS and Android; iOS builds OpenSSL needlessly (scope `vendored-openssl` to Android only); Android needs the system CA dir set for HTTPS (`/system/etc/security/cacerts`); Android token store is `android-native-keyring-store` 1.0 (young). Nothing in the macOS design blocks mobile. |
| Q17 | On close | Quit waits **until the pull (and push, if anything changed) succeed, or 10 s**, whichever comes first, showing "Syncing…". The next open catches up if it was cut short. |

### Dependency conflicts to accept before installing (Q9)

- **OpenSSL on macOS:** git2 links OpenSSL on macOS only for an init call (TLS itself goes through
  Apple's SecureTransport). Built as-is, the app needs Homebrew OpenSSL on every Mac. Fix:
  `vendored-openssl` → statically linked, **+~4.3 MB** app size in total for git2, and perl is
  needed at build time.
- **SecureTransport is deprecated** by Apple (believed TLS 1.2 max). It still works today.
- New crates: `git2`, `keyring-core`, `apple-native-keyring-store` (all MIT/Apache; libgit2 as above).

---

## 4. Pending questions

Recommendations in *italics*. These need an answer before building.

| # | Question | Recommendation |
|---|---|---|
| P1 (Q10) | **Snapshot clash:** the same row changed on both machines between syncs. | *This machine's row wins; the remote row is written to `.conflicts/snapshot_<yyyymmdd_hhmm>.sql`.* |
| P2 (Q11) | **Where conflicts show** (`.conflicts/` is hidden from the explorer). | *Settings → Sync: "N conflicts" with the file list; a toast when a sync creates one.* |
| P3 (Q13) | **Settings "Restore" on a vault that already has data.** | *Restore only on an empty vault; otherwise the first sync is a normal merge.* |
| P4 (Q14) | **Activity log cap:** newest N rows overall, or only the latest row per item? N? | *Newest 10,000 rows overall (~1 MB), trimmed by a DB trigger.* |
| P5 (Q18) | **Order of a sync run.** | *(1) write snapshot from live DB, commit locally if changed → (2) pull + merge → (3) if the snapshot changed, rebuild the live DB from it in one transaction and refresh the UI → (4) push if there are new commits.* |
| P6 (Q19) | **Where the repo lives.** | *The vault folder is the repo (`.git` at its root); `.gitignore` for the live DB files and `.DS_Store`.* |
| P7 (Q20) | **Connecting to a remote that already has unrelated files.** | *Accept only an empty remote or a Tend vault (has `db/snapshot.sql`); otherwise refuse: "This repo isn't a Tend vault".* |
| P8 | **Snapshot text format.** "One row per line" means multi-line text (descriptions, comments, notes on people) must be escaped. SQL `INSERT` lines need `char(10)` tricks; **JSON Lines** (`{"table":"tasks","id":…,…}` per line, `\n` escaped) is simpler to write, read and diff. | *JSON Lines, named `db/snapshot.jsonl`; header line with app id + schema version.* |
| P9 (Q21) | **Sync settings fields.** | *Remote URL, branch (default `main`), username (optional), token, author name/email, device name (default: the Mac's name), interval (off / 15 min / 1 h), Sync now. Shows: last sync, status, snapshot size, conflicts, recent activity (last ~50).* |
| P10 (Q22) | **Sync state while working.** | *A small dock icon next to the timer chip: ✓ synced / ↻ syncing / ! error (red); hover shows last sync; click opens Settings → Sync.* |
| P11 (Q23) | **Failures** (bad token, offline, host down). | *Each run has a time limit (see I-13 for the first push); retries back off up to 1 h; one toast on the first failure, then only the red dock icon; an auth failure stops automatic sync until fixed in Settings.* |
| P12 | **Which settings are shared vs per device.** UI state (`ui.nav_open`, `ui.explorer_open`, widths, last view, quick-create last paths) is per device; tasks root name, timer sound… could be either. | *Snapshot only shared prefs: `tasks.root_name`, `timers.sound`; the rest stays local. Keep the list in `settings.config.ts`.* |
| P13 | **Timers and feedback in the snapshot?** A timer is "running on this machine"; feedback is a log. | *Leave both out of the snapshot (device-local).* |
| P14 | **Notes index (`notes`, `note_folders`) in the snapshot?** See I-1. | *Leave the index out; rebuild it by scanning after a pull, and make note references (tags, attachment links) point at the note's **path** in the snapshot, mapped back to ids on import.* |
| P15 | **"Sync now" button and sync on/off toggle?** (Q6 named open, interval, close.) | *Yes to both.* |
| P16 | **Disabling sync** later: leave `.git` in the vault, or remove it? | *Leave it (history stays); "Disconnect" clears the remote, token and schedule.* |

---

## 5. How a sync run would work (assuming the recommendations above)

```
on open → pull (+ rebuild DB if snapshot changed) → push if local changes
every 15 min / 1 h → same
on close → same, at most 10 s

run:
  1. lock: one run at a time; pause the notes watcher
  2. if writes since last push or git status dirty:
       write db/snapshot.jsonl from the live DB (shared tables only)
       commit "Tend sync · <time> · <device>"
  3. fetch + merge origin/<branch>
       note clash     → ours stays, theirs → .conflicts/<name>_conflict_<time>.md
       snapshot clash → ours wins, theirs → .conflicts/snapshot_<time>.jsonl   (P1)
  4. if the merge changed the snapshot:
       back up app.sqlite → rebuild shared tables from the snapshot in one transaction
       rescan notes/ → refresh the UI
  5. push if HEAD is ahead of origin
       rejected (someone pushed meanwhile) → back to 3, once
  6. record pull/push in activity; resume the watcher; update the dock icon
```

Restore (empty vault): clone → create `app.sqlite` (migrations) → import the snapshot → scan
`notes/` → open.

---

## 6. What gets built (outline)

**Rust (`src-tauri/src/sync/`)**
- `git.rs` — clone, fetch, merge, commit, push via git2; credentials callback from the token
  (bounded retries so a bad token can't loop); every network call with a deadline from config.
- `snapshot.rs` — write and import the snapshot (fixed table order, rows by id, one per line);
  refuses a snapshot whose schema is newer than the app; migrates older ones on import.
- `conflicts.rs` — conflict copies into `.conflicts/`.
- `token.rs` — Keychain get/set/remove (keyring-core + apple store).
- `schedule.rs` — open / interval / close triggers, one run at a time, backoff.
- `activity/` — the capped `activity` table (schema **v3** migration) and a `record()` used by
  every write.
- Commands: `sync_config_get/set`, `sync_token_set/clear`, `sync_now`, `sync_status`,
  `sync_restore`, `activity_recent`. Errors added to the registry (`SYNC_AUTH`, `SYNC_OFFLINE`,
  `SYNC_NOT_A_VAULT`, `SYNC_REJECTED`, `SNAPSHOT_TOO_NEW`, …).

**UI**
- Settings → **Sync** section (fields from P9, the public-repo warning, activity list, conflicts).
- Start page → **Restore from git**.
- Dock sync icon (P10); "Syncing…" on quit (Q17).

---

## 7. Already addressed along the way

- Notes/attachments are files already; only the DB needed a git-friendly form → text snapshot.
- Binary SQLite in git (corruption mid-write, unmergeable, history bloat) → never committed.
- Token leaking into the vault/repo → Keychain only.
- System git missing / credential side effects → git2 in-process.
- Private-repo detection → dropped by decision; static warning instead.
- Snapshot growing to GBs → no: only structured rows, line-level deltas, size shown, 50 MB warning.
- Mobile feasibility → checked, deferred.

---

## 8. Issues we might face

**Data model**
- **I-1 Notes index ids.** `notes.id` is generated when a device scans a file. If device B scans a
  note that came from A before importing A's snapshot, it gives the note a *different* id, and
  tags/attachment links from A point at nothing. → P14 (reference notes by path in the snapshot)
  or import the snapshot before any scan.
- **I-2 Per-device vs shared settings** in one `settings` table (P12).
- **I-3 Orphans after a line merge.** A task deleted on A while B added a comment to it merges
  cleanly in text but leaves a comment pointing at nothing. Import must check foreign keys and
  drop (or park in `.conflicts/`) orphan rows.
- **I-4 Delete vs edit** of the same row is a git conflict → P1 rule applies.
- **I-5 Schema skew between machines.** Device A on v3 pushes; device B still runs a v2 app.
  The snapshot header carries the schema version; B refuses to import a newer one ("Update Tend
  on this machine") and keeps its own data.
- **I-6 The activity table** must stay out of the snapshot; its writes must not themselves count
  as "changes" for Q15.
- **I-7 `my_day` is keyed by local date** — two machines in different time zones see "today"
  differently. Acceptable; noted.

**Running it**
- **I-8 Rebuilding the live DB while the app is open:** writes during the import, open screens
  showing stale data. → one transaction, a short write lock, then reload stores; keep a backup copy
  of `app.sqlite` from just before the import.
- **I-9 The file watcher** fires for every file the pull writes → pause it during a run, rescan
  once after.
- **I-10 Two machines pushing at once** → push rejected (non-fast-forward) → pull and retry once.
- **I-11 Interrupted runs:** quit after 10 s, sleep, crash. Pushes are atomic per branch; a
  leftover `.git/index.lock` must be cleaned up on the next run.
- **I-12 Case-only renames** (`Notes.md` → `notes.md`) on macOS's case-insensitive disk. The name
  rule (lowercase only) mostly avoids it.
- **I-13 The first push of a large vault** can take longer than a per-run time limit → the first
  push (and restore clone) gets a longer limit or none, with progress.
- **I-14 Large attachments:** GitHub refuses files over 100 MB and warns at 50 MB; repos have soft
  size limits (GitHub ~1–5 GB). Skipped files are listed in Settings → Sync.

**Credentials and hosts**
- **I-15 Tokens expire** (GitHub fine-grained tokens have an expiry). Show "Token expired or
  invalid" and pause sync rather than retrying.
- **I-16 Keychain access prompts** can reappear when the app's signature changes (dev build vs
  the built `Tend.app`).
- **I-17 Host differences:** username requirements, default branch names, protected branches
  rejecting pushes, redirects to login pages on self-hosted servers.
- **I-18 Public repos** publish everything (warning only, by decision).

**Dependencies**
- **I-19 OpenSSL** must be vendored on macOS (+size, perl at build time).
- **I-20 SecureTransport** is deprecated by Apple; a future macOS could drop it.

---

## 9. Out of scope for the first version

- SSH remotes, Git LFS, choosing which folders to sync.
- Showing git history or diffs inside Tend.
- Real-time sync (sub-minute) or merging on every keystroke.
- iOS / Android builds (Q16).
