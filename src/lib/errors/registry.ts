// Errors the frontend raises itself. Backend errors arrive with their own key and message
// (src-tauri/src/error/registry.rs); `toAppError` gives everything the same shape.
import type { ErrorDef } from '#lib/types/error.type.ts';

export const errors = {
  /** Mirrors the backend's NOT_FOUND, for call sites that recover from it. */
  NOT_FOUND: { key: 'NOT_FOUND', message: 'Not found' },
  /** Mirrors the backend's NO_PERMISSION: the vault screen adds the System Settings hint. */
  NO_PERMISSION: { key: 'NO_PERMISSION', message: 'Tend doesn’t have permission to access this folder' },
  /** Mirrors the backend's TAG_BLOCKED: a tag editor says it beside the field. */
  TAG_BLOCKED: { key: 'TAG_BLOCKED', message: 'This tag is blocked. Unblock it in Settings → Tags.' },
  TIMEOUT: { key: 'TIMEOUT', message: 'Tend took too long to respond. Try again.' },
  UNKNOWN: { key: 'UNKNOWN', message: 'Something went wrong.' },
} as const satisfies Record<string, ErrorDef>;
