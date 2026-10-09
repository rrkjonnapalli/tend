# Review list — sync S1–S4 (built while you were away, 2026-10-09)

Things decided or found while building; each needs a look, a yes, or a no.

## Decided for you

1. **`git2` dependency (for S4).** "Do till S4" taken as the OK the plan asked for. Vendored
   libgit2 and OpenSSL: about +4.3 MB to the app, perl at build time. The Keychain crate is *not*
   added yet; S4 is tested against a local bare repo, and the token is only needed for S5's UI.
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

## Not done (S5 and later)

12. Tauri commands, Settings → Sync, Start-page restore, the dock icon, the schedule (open,
    interval, close with 10 s), Keychain for the token, pausing the watcher during an apply and
    reloading stores afterwards. Until then sync runs only in tests.
13. An end-to-end simulation through real databases and Git (the snapshot-level one is fast but
    missed #7). Worth adding before release.

## To do when you're back

14. **Your `sample` vault's database** still has the old `deletions` / `activity` tables and
    their triggers, and no `sync_state` / `path_log` reason `sync`: the bundled Tend.app was
    running, so I didn't touch it. With Tend closed, ask me to update it (backup first, as before).
