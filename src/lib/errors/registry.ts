// Errors the frontend raises itself. Backend errors arrive with their own key and message
// (src-tauri/src/error/registry.rs); `toAppError` gives everything the same shape.
import type { ErrorDef } from '#lib/types/error.type.ts';

export const errors = {
  /** Mirrors the backend's NOT_FOUND, for call sites that recover from it. */
  NOT_FOUND: { key: 'NOT_FOUND', message: 'Not found' },
  TIMEOUT: { key: 'TIMEOUT', message: 'Tend took too long to respond. Try again.' },
  UNKNOWN: { key: 'UNKNOWN', message: 'Something went wrong.' },
} as const satisfies Record<string, ErrorDef>;
