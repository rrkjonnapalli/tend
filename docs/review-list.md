# Review list — sync S1–S5 (2026-10-09)

Things decided or found while building; each needs a look, a yes, or a no.

## Cut from the sync plan (you said "cut all six")

Each can come back later; none changes the shard layout.

| # | What was cut | Why | Consequence today |
|---|---|---|---|
| C1 | **The 2-way fallback and tombstones** (`deletions` table, `deleted/` shards) | Only needed when two vaults that both have data meet without a shared base. | **Connect needs an empty remote**; a second device restores into an empty folder; a lost base (force-push) stops sync and asks for a restore. *Open:* a cheaper option is a first merge on an empty base (a union of both sides, no tombstones); its cost is that deletions made while disconnected come back once. |
| C2 | **Two-parent merge commits** | Tend writes the whole merged tree, so one commit on the fetched head is enough and the push is a fast-forward. | Linear history. |
| C3 | **The per-write activity log** (`activity` table + triggers) | Nothing needed it except a "recent activity" list. | Settings shows only the last run and its result (`sync_state`). |
| C4 | **The `access` table** | For "recent" lists and ranking, not sync. | No recents. |
| C5 | **The `changes` table (deferred)** | Only speeds things up around 100k rows. | Every run rebuilds all shards; Git decides what changed. Add back if a vault gets slow. |
| C6 | **Restore from Settings** | A restore makes a new vault, which belongs on the Start page. | Restore only on the Start page. |

## Decided for you

1. **`git2` dependency (for S4).** "Do till S4" taken as the OK the plan asked for. Vendored
   libgit2 and OpenSSL: about +4.3 MB to the app, perl at build time. S4 is tested against a
   local bare repo; the Keychain crates came with S5 (#12).
2. **Year from a ULID** is computed with a 10-line calendar function (`sync/shards`), not a date
   crate: it's the only date sync needs.
3. **Shared settings** that sync are `tasks.root_name` and `timers.sound`
   (`settings::SHARED_KEYS`). Every other setting stays on its device.
4. **Folder rows created on two devices with the same path** keep the smaller id; equal label or
   status names get " 2"; clashing note paths get `_1` (spec §9.2, unchanged).

## Found by the simulation (spec updated)

5. **The base moves with the apply**, not only after the push. Otherwise: apply, crash before
   push, delete a row that just came in → the next sync brings it back.
6. **`sync_state.pending_commit`**: the commit being pushed is recorded first; the next run makes
   it the base if the remote has it. Otherwise: push, crash before recording → a row created,
   pushed, then deleted locally comes back.

## Found by the end-to-end tests

7. **New tasks were inserted twice on apply** (tasks are written first, before deletes, so a
   status or folder can be removed; the main pass wrote them again). Fixed, with a test. The
   snapshot-level simulation couldn't see it — it was caught by the two-device test through real
   databases and a real (local) Git remote.

## How S4 behaves (worth a look)

8. **What a commit holds:** only `.gitignore`, `.tend/sync/**`, and the files rows point at (note
   bodies, `.tend/attachments/*`). Other files in the vault (a PDF next to your notes) are never
   committed, so a sync can't spread or delete files Tend doesn't manage. Files over 100 MB are
   left out; their rows still sync.
9. **Connect** refuses a remote that has any branch (`SYNC_REMOTE_NOT_EMPTY`). **Restore** expects
   a new, empty vault. **A rewritten remote** (base gone from its history) stops sync with
   `SYNC_BASE_LOST`.
10. **Network deadlines:** libgit2's connect timeout 15 s and read timeout 60 s, set once per
    process (constants in `sync/git.rs`).
11. **Auth:** HTTPS with a token as the password; username defaults to `x-access-token` (works for
    GitHub). At most 3 credential attempts per operation.

## S5: UI and schedule (worth a look)

12. **Token** in the macOS Keychain only (`keyring-core` + `apple-native-keyring-store`), service
    `app.tend.sync`, one entry per remote URL. Never in the vault, the database, the config file or
    the UI (`sync_status` says only whether one is saved). Other platforms have no store yet: sync
    there reports `SYNC_TOKEN`.
13. **Settings** (URL, branch, username, interval) live in the app config file, keyed by vault
    path. **No device name**, and commits are authored "Tend <tend@localhost>" (no UI for it).
14. **Runner:** one thread per connected vault, its own DB connection (busy timeout 5 s). Runs on
    open (after the scan), every 15 min by default (off / 15 / 60 in Settings), on Sync now, and on
    quit (at most 10 s). Failures back off 1, 2, 4 … 60 min; a bad token, a lost base or a newer
    vault stop automatic runs until Sync now or a new token. One toast on the first failure, then
    only the dock icon.
15. **The watcher is not paused** during an apply: it re-indexes what the apply already wrote, which
    is harmless. A run that changed rows reloads every store.
16. **Disconnect** removes the settings, the token and the base; `.git` and all data stay.
    Connecting again needs a new, empty repo (spec §14).
17. **Left out for now:** snapshot size and the list of files over 100 MB in Settings (spec §16).
18. **Not checked by eye:** I couldn't screenshot or click the app from here (no screen-recording
    or accessibility access). `tauri dev` starts cleanly; please look at Settings → Sync, the
    dock icon and Start → Restore from Git. There are no mockups for them; they reuse the
    Settings and Start page styles.

## Not done

19. An end-to-end simulation through real databases and Git (the snapshot-level one is fast but
    missed #7). Worth adding before release.
