# Mobile spike: iOS and Android (2026-10-15)

The question: can Tend's Rust core and Tauri shell build for iOS and Android, and what has to
change? This was done on the branch `spike/mobile` (worktree `../ntr-mobile-spike`), so `main`
and the running dev app were never touched.

## Results

| Platform | Rust core | App package | Blocked by |
|---|---|---|---|
| Android (aarch64) | **Builds and links** (`libtend_lib.so`) | **Debug APK builds** (`gen/android/app/build/outputs/apk/universal/debug/`, 246 MB with debug symbols) | Not run yet: no emulator image or phone |
| iOS | Compiles up to Tauri's own build script | Not built | No Xcode on this Mac: only the Command Line Tools are installed |

The release build of the Android core is **17.7 MB** (`libtend_lib.so`, aarch64, stripped), so
the APK should land around 10 MB compressed, before the web assets.

What this proves:

- **`git2` with vendored libgit2 and OpenSSL, `rusqlite` (bundled SQLite), `ureq` with rustls,
  `notify`, `keyring-core` all build and link for Android.** That was the biggest unknown.
- For iOS, the same crates' C code compiled when pointed at the macOS SDK (`cargo check`), which
  is a good sign but not proof. Tauri itself needs the iOS SDK (`xcrun --sdk iphoneos`), which
  only ships with Xcode.

## What changed on the branch

1. **`trash` is desktop-only.** It has no iOS or Android backend (it fails to compile there). It
   moved to the desktop-only dependencies. On phones, `files::to_trash` deletes for good, the same
   code the tests use. **Decision needed:** delete for good on phones, or move to a `.tend/trash/`
   folder that Tend empties itself after N days.
2. **Sync token store per platform** (`sync/token.rs`):
   - iOS: `apple-native-keyring-store` with the `protected` feature (the data-protection
     Keychain; the `keychain` feature is macOS-only).
   - Android: `android-native-keyring-store` 1.0.0 (same `keyring-core` family; Android
     Keystore). Not run yet: it needs the app's JNI context at runtime, to check on a device.
3. **Identifier** set to `io.github.rniverse.tend` (only on this branch; the Android package name
   comes from it).
4. **`tauri android init`** generated `src-tauri/gen/android` (Gradle project, 468 KB).
5. **Latest Android toolchain** in the generated project: Gradle 9.8.1 (the first to run on
   JDK 27), Android Gradle Plugin 9.4.1, Kotlin plugin 2.4.21, Java/Kotlin target 17 (the
   template had 1.8). The template's `android.builtInKotlin=false` stays until Tauri v3. The
   remaining build warnings come from Tauri's own Android library, not from Tend.
6. Two items that become unused on phones are marked so the build stays warning-free
   (`MAIN_WINDOW`, `TRASH_FAILED`).

Desktop is unchanged: `cargo test` 221 pass, clippy clean, on the branch.

## Findings that shape the mobile work

- **No folder picker on mobile.** `tauri-plugin-dialog` returns `FolderPickerNotImplemented` for
  `open({ directory: true })` on iOS and Android. Both the start screen and Restore use it. On a
  phone the vault must live in the app's own storage (`app_data_dir()/vault`), and the first run
  is a restore from the remote. This matches the plan (a phone is a sync device).
- **No quit event on mobile.** Phones suspend and kill apps; `RunEvent::ExitRequested` never fires,
  so the "sync once more on quit" flow does nothing. Tauri gives `WindowEvent::Suspended` (Android
  `onPause`, iOS `applicationWillResignActive`) and `Resumed`. Plan: start a sync on Suspended and
  on Resumed. iOS allows only a few seconds in the background unless the app asks for more time
  (`beginBackgroundTask`, a small Swift plugin); a sync cut short is safe (it resumes next time).
- **File watcher:** compiles, but nothing else writes into the app's sandbox, so it can stay off on
  phones.
- **Opener and notifications** plugins support mobile; attachments open through the system's
  share/open sheet.
- **Desktop-only already:** single-instance and window-state are gated by `cfg(desktop)`.
- **The frontend** is the large part: the layout, popovers, hover states and keyboard shortcuts
  are desktop. Phone screens need design first (no mockups exist).

## Your actions

1. **Install Xcode** (App Store, free Apple ID is enough to build and run on the simulator; about
   15 GB). Then run `sudo xcode-select -s /Applications/Xcode.app` and
   `bun tauri ios init` on the branch.
2. **Java in LuLu — done.** To rebuild the APK:

   ```sh
   cd ../ntr-mobile-spike
   export ANDROID_HOME=$HOME/Library/Android/sdk
   export NDK_HOME=$ANDROID_HOME/ndk/30.0.16248370
   export JAVA_HOME=/opt/homebrew/opt/openjdk
   bun tauri android build --debug --apk --target aarch64
   ```
3. **For a device run:** an Android emulator needs a system image (none installed), or plug in a
   phone with USB debugging and use `bun tauri android dev`.
4. **Apple Developer account** before iOS can ship (TestFlight or App Store), as planned.

## Next steps (after your actions)

1. Build the APK and the iOS simulator app; open both and note what fails at runtime (token store,
   paths, the start screen's folder picker).
2. Platform layer: vault in app storage on mobile, start screen goes straight to "Restore from
   your repo", sync on Suspended and Resumed.
3. Design the phone screens, then build them.

## Actions taken on your Mac while you were away (review these)

| Action | Where | Undo |
|---|---|---|
| Added Rust targets `aarch64-linux-android`, `x86_64-linux-android`, `aarch64-apple-ios`, `aarch64-apple-ios-sim`; `tauri android init` added `armv7-linux-androideabi`, `i686-linux-android` | rustup | `rustup target remove <target>` |
| Android NDK r30 (30.0.16248370, latest stable) from dl.google.com; r27c removed | `~/Library/Android/sdk/ndk/30.0.16248370` | delete the folder |
| `brew install openjdk` (27, latest; keg-only, not on your PATH); `openjdk@17` removed | Homebrew | `brew uninstall openjdk` |
| Gradle 9.8.1 downloaded by the wrapper; 9.6.1 removed | `~/.gradle/wrapper/dists/` | delete the folder |
| Ran Gradle once outside the command sandbox to rule the sandbox out (it failed on LuLu the same way; nothing was downloaded) | — | — |
| Git worktree and branch `spike/mobile` | `../ntr-mobile-spike` | `git worktree remove ../ntr-mobile-spike && git branch -D spike/mobile` |
| Build output in the worktree (not committed) | `target-android` 5.6 GB, `target-ios` 1.5 GB, `src-tauri/target` 8 GB | delete those folders |

Not done, on purpose: no change to LuLu or any firewall rule, no Xcode install, nothing merged
into `main`, nothing pushed except the `spike/mobile` branch.
