/** A newer release than the running app (src-tauri/src/update.rs). */
export type AvailableUpdate = {
  version: string;
  /** The release notes, as written on the release. */
  notes: string | null;
};
