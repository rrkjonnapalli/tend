// Typed bridge to the Rust commands. Every wrapper in api/ goes through `call`.
import { invoke } from '@tauri-apps/api/core';

/** The error shape every Rust command returns (see src-tauri/src/error.rs). */
export type AppError = { code: string; message: string };

export function isAppError(e: unknown): e is AppError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}

export function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}

/** A readable message for any thrown value. */
export function messageOf(e: unknown): string {
  if (isAppError(e)) return e.message;
  return e instanceof Error ? e.message : String(e);
}
