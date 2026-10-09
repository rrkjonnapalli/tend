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

## To do when you're back

7. **Your `sample` vault's database** still has the old `deletions` / `activity` tables and
   their triggers, and no `sync_state`: the bundled Tend.app was running, so I didn't touch it.
   With Tend closed, ask me to update it (backup first, as before).
