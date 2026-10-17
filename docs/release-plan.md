# Release plan: what's left before v1

Everything Tend needs before its first release, decided on 2026-10-15. The goal is to ship v1
complete, with nothing knowingly left to patch later. Three things are hard or impossible to
change after release, so they come first: the **updater**, the **app identifier**, and the
**schema and sync format** (both freeze at release).

Status: **todo**, **doing**, **done**, **decided** (no code needed), **user** (only you can do it).

---

## 1. Decisions taken

| Topic | Decision |
|---|---|
| App identifier | `io.github.rniverse.tend` (you own the `rniverse` domain). Replaces `com.sage.tend`. |
| Update host | GitHub Releases (`latest.json` uploaded with each release by a GitHub Action). |
| Platforms | v1 is macOS, iOS and Android. No Windows or Linux. |
| Apple account | None yet. iOS needs one ($99/year) before it can ship, even on TestFlight; with it, the macOS app gets notarized too. |
| Dictionary | No fallback. The source picked in Settings is always the one asked (#15, #16 closed). |
| Date input | No typed date field. `DateField` stays a button plus month popover. |
| Fonts | Smile Moon stays for now. Its licence is noted in the About page's credits. |
| Snap stacks | Keep "+ New stack" in the snap editor (it is the keyboard path); make it atomic. |
| Sync, remote files | Compare remote files by Git's blob id instead of SHA-256. |
| Attachments | The existing size limit (Settings → General, `files.max_mb`, 1–90 MB) is the rule. |
| Quit | Quitting waits for the last sync's result, success or failure, with no time cap. |

---

## 2. Must be in v1 (hard to fix after release)

### 2.1 Auto-update — todo
- Add `tauri-plugin-updater`. Its signing key is Tauri's own, separate from Apple, and free:
  `bun tauri signer generate`. The app checks every update against the public key, so nobody
  else can push one.
- **Back up the private key.** If it is lost, installed copies can never be updated again.
- A GitHub Action builds each release and uploads the bundles and `latest.json`.
- UI: "Update" in Settings → General, and in the Settings → Sync banner shown on
  `SNAPSHOT_TOO_NEW` ("This vault was synced by a newer Tend…").
- Desktop only: on mobile, updates come from the stores (iOS) or a new APK (Android).

### 2.2 Signing (macOS) — decided
- No Apple account yet: the app gets an ad-hoc signature (Apple Silicon requires at least that).
- The first install shows "Apple could not verify…"; the user opens it once through
  System Settings → Privacy & Security → Open Anyway.
- Updates installed by the updater should not show that warning (its downloads are not
  quarantined). **Confirm this on the first real release.**
- Get an Apple Developer account ($99/year) later for a clean first install, and for iOS.

### 2.3 App identifier — todo
- Change `com.sage.tend` to `io.github.rniverse.tend` in `tauri.conf.json` and anywhere else it
  appears (Keychain service name, config paths in docs).
- The app config dir moves with it. Move the existing sync config across (or reconnect once).
- Never change it after release: a new identifier is a new app (config, Keychain token and
  update channel all lost).

### 2.4 Platforms — decided
- v1 ships on **macOS** (Apple Silicon and Intel), **iOS** and **Android**.
- Replace the bundle target `"all"` with the macOS targets; add the iOS and Android projects
  (`tauri ios init`, `tauri android init`).
- The mobile work is in §5.

### 2.5 Sync, before the format freezes — todo
- **Lost base.** Sync remembers the last commit both sides agreed on (the base). A force-push or
  a deleted branch erases it. Today sync stops with `SYNC_BASE_LOST` and asks for a restore, which
  only works into an empty vault, so unsynced edits are stranded. Fix: merge with an empty base,
  keeping rows from both sides.
- **C1, two vaults that both hold data.** Connecting a vault with data to a remote that already
  holds a vault. Same fix as the lost base: a first merge on an empty base.
- **Case pairs.** A remote can hold the user's own files, such as `README.md` and `readme.md`.
  On a Mac those are one file, so one overwrites the other. Tend's own notes are lowercase and
  never hit this. Fix: detect the pair, keep one, show a notice.
- **Blob ids.** Hash remote files by Git's blob id instead of reading and hashing them with
  SHA-256 (speed; see §3.6).

### 2.6 Two-device sync test end to end — todo
- Run on two devices (laptop and phone once the mobile build exists; two laptops or two vaults
  before that). Covers open question #13 (tag block or delete, recolor against reorder).

### 2.7 Font licence — decided
- Smile Moon (`static/fonts/Smile Moon.otf`, MJType) says "All Rights Reserved". Credit is not
  permission: it can't ship in a public app even with credit. Fine while the repo is private.
  Buy a licence or swap the font before the app is shared publicly.

---

## 3. In v1, no decisions needed

### 3.1 Snap stacks are atomic — todo (#14)
- "+ New stack" in the snap editor and dropping a snap on a snap each become one Rust command
  (make the stack, move the snap(s), all or nothing). Removes both `shortcut:` notes
  (`SnapDialogs.svelte`, `routes/library/snaps/+page.svelte`).

### 3.2 Tests — todo (#19, audit #10)
- HTTP fetch: a 404, and a body over the size cap.
- Home's add guess at its 40-character edge.
- The note page's conflict logic (`NOTE_CHANGED`).

### 3.3 Review and tidy — todo (audit #14, #20)
- Run a ponytail review and a codecd review on the whole codebase. Fix what both agree on; list
  where they disagree for a decision.
- Known items:
  - several positional arguments in Rust (`notes/`, `tasks/folders.rs`, sync helpers)
  - private types in `util/*.ts`
  - the inline `'notes:changed'` string, which moves to a constant
  - the swallowed `allowNotifications` error
  - a stale comment in `commands/mod.rs`
  - the outline in `SwatchPicker.svelte:141`

### 3.4 Sync config keyed by vault path — todo (audit #12)

### 3.5 About page with credits — todo
- Settings → About: version, licence notices for the libraries and fonts (MIT and OFL require
  their notices to ship), and the Smile Moon note.

### 3.6 Big syncs — todo
Covers the first sync, a sync after a long time, and syncs with many changes. A loading screen
before the database is ready is fine.

1. **Progress by phase.** "Downloading 12 / 40 MB", "Applying 300 / 2,000 files", "Uploading".
   `git2` already reports transfer progress for fetch and push.
2. **"Getting your vault ready…"** full-screen for a restore or a large pull; the app opens when
   the apply is done. Small syncs stay in the background.
3. **Work grows with the change, not the vault.** Blob ids on the remote side (§2.5) and a size
   and mtime cache on the local side, so unchanged files aren't read again. First measure how
   much the current code rehashes on each run.
4. **Big files — done.** The size limit (`files.max_mb`, 1–90 MB, default 30) already stops a file
   that big from being attached, indexed or committed by sync, so GitHub's 100 MB per-file limit
   is never hit. Check: a file over the limit that the user drops into the vault folder by hand is
   left out of sync and the user can see that it was.
5. **Quit waits for the result.** Today quitting waits for the last sync at most 10 s
   (`CLOSE_SYNC_LIMIT` in `lib.rs`), then exits. Change: wait until the run ends, success or
   failure (each Git call has its own deadline, so it always ends), and show "Finishing sync…"
   meanwhile.
6. **Resume — done.** A run stopped mid-push or mid-apply picks up where it left off (pending
   commit, staged files). The apply stays all-or-nothing.

### 3.7 Snapshot size in Settings — todo
- Settings → Sync shows the snapshot size; warn above 200 MB (spec §8).

---

## 4. Manual checks (user)

- [x] Reconnect sync on `sample` (done, on a new branch).
- [ ] The note conflict dialog.
- [ ] Delete goes to the Trash.
- [ ] Disconnect moves `.git` to the Trash.
- [ ] A second launch focuses the running app.
- [ ] A `.command` attachment is refused.
- [ ] The CSP in a built app (`bun run app`).
- [ ] A timer firing after the laptop wakes (#10).

---

## 5. Mobile (iOS and Android) — in v1

Decided: both ship in v1. Tauri 2 builds iOS and Android apps, but mobile is a project of its own:

- **UI.** The mockups are desktop only (sidebar, popovers, keyboard). Phones need designed screens.
- **Storage.** iOS can't use an arbitrary folder as the vault; Android only awkwardly. On mobile
  the vault lives in the app's own storage and is filled by sync. Tend on a phone is a sync device.
- **Rust crates.** `git2` (libgit2 plus TLS) must cross-compile; `keyring`, `trash` and `notify`
  have no mobile path or need replacing.
- **Distribution.** iOS needs the Apple account even for TestFlight. Android can install an APK
  from GitHub Releases.
- **Updates.** The desktop updater doesn't run on mobile.

Nothing in the schema or the sync format needs to change for mobile.

Spike done 2026-10-15 on branch `spike/mobile`: the Rust core builds and links for Android; iOS
needs Xcode first. Results, findings and your actions: `docs/mobile-spike.md` on that branch.

Order:
1. **Spike.** Build the Rust core for Android and the iOS simulator; list what breaks (`git2`
   and TLS, `keyring`, `trash`, `notify`, the folder picker) and pick a replacement for each.
2. **Platform layer.** Put each desktop-only crate behind a small module with a mobile path:
   vault in app storage, token store per platform, delete without a Trash, no watcher.
3. **Mobile screens.** Design the phone layouts (none exist yet), then build them.
4. **First-run on mobile** is a restore from the remote (§3.6, "Getting your vault ready…").
5. **Distribution.** Android: an APK on GitHub Releases (Play Store later, $25 once). iOS:
   TestFlight or the App Store, which needs the Apple account.
6. **Two-device test** laptop ↔ phone (§2.6).

---

## 6. After v1, by design

- Calendar, events and reminders (`docs/later/`), as a schema migration.
- #11: a name-fix log lost when its transaction fails. Accepted; the disk is the truth.
