// Errors the frontend raises itself. Backend errors arrive with their own key and message
// (src-tauri/src/error/registry.rs); `toAppError` gives everything the same shape.
import type { ErrorDef } from '#lib/types/error.type.ts';

export const errors = {
  TIMEOUT: { key: 'TIMEOUT', message: 'Tend took too long to respond. Try again.' },
  UNKNOWN: { key: 'UNKNOWN', message: 'Something went wrong.' },
} as const satisfies Record<string, ErrorDef>;
