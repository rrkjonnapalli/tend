// Typed bridge to the Rust commands. Every wrapper in api/ goes through `call`, and every
// failure leaves it as an AppError with a registry key (see lib/errors).
import { invoke } from '@tauri-apps/api/core';
import { config } from '#lib/config/app.config.ts';
import { AppError, toAppError } from '#lib/errors/app-error.ts';
import { errors } from '#lib/errors/registry.ts';
import type { CallOptions } from '#lib/types/ipc.type.ts';

export async function call<T>(cmd: string, { args, timeoutMs = config.callTimeoutMs }: CallOptions = {}): Promise<T> {
  try {
    return await withDeadline(invoke<T>(cmd, args), timeoutMs);
  } catch (e) {
    throw toAppError(e);
  }
}

/** `reply`, or a TIMEOUT error once `timeoutMs` passes. `null` waits as long as it takes. */
async function withDeadline<T>(reply: Promise<T>, timeoutMs: number | null): Promise<T> {
  if (timeoutMs === null) return reply;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const deadline = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new AppError(errors.TIMEOUT)), timeoutMs);
  });
  try {
    return await Promise.race([reply, deadline]);
  } finally {
    clearTimeout(timer);
  }
}
