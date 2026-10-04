// App-wide settings, kept in one place.
export const config = {
  /** How long the UI waits for a Rust command before showing an error. */
  callTimeoutMs: 15_000,
  /** For commands that copy or scan many files (rescan, adding attachments). */
  slowCallTimeoutMs: 120_000,
};
