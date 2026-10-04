export type CallOptions = {
  args?: Record<string, unknown>;
  /**
   * How long to wait before failing with a TIMEOUT error. `null` waits as long as it takes:
   * only for commands that wait on the user, such as macOS asking for folder access.
   */
  timeoutMs?: number | null;
};
