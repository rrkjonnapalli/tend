// Typed bridge to the Rust commands. Every wrapper in api/ goes through `call`.
import { invoke } from '@tauri-apps/api/core';
import { config } from '#lib/config/app.config.ts';
import type { AppError, CallOptions } from '#lib/types/ipc.type.ts';

export const TIMEOUT_CODE = 'timeout';

export async function call<T>(cmd: string, { args, timeoutMs = config.callTimeoutMs }: CallOptions = {}): Promise<T> {
  const reply = invoke<T>(cmd, args);
  if (timeoutMs === null) return reply;

  let timer: ReturnType<typeof setTimeout> | undefined;
  const deadline = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(timeoutError()), timeoutMs);
  });
  try {
    return await Promise.race([reply, deadline]);
  } finally {
    clearTimeout(timer);
  }
}

function timeoutError(): AppError {
  return { code: TIMEOUT_CODE, message: 'Tend took too long to respond. Try again.' };
}

export function isAppError(e: unknown): e is AppError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}

/** A readable message for any thrown value. */
export function messageOf(e: unknown): string {
  if (isAppError(e)) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}
