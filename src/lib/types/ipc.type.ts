/** The error shape every Rust command returns (see src-tauri/src/error.rs). */
export type AppError = { code: string; message: string };

export type CallOptions = {
  args?: Record<string, unknown>;
  /**
   * How long to wait before failing with a `timeout` error. `null` waits as long as it takes:
   * only for commands that wait on the user, such as macOS asking for folder access.
   */
  timeoutMs?: number | null;
};
