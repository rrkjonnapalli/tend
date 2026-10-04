import type { ErrorDef } from '#lib/types/error.type.ts';
import { errors } from './registry.ts';

export class AppError extends Error {
  readonly key: string;

  /** `message` replaces the registry's default when this case needs its own words. */
  constructor(def: ErrorDef, message: string = def.message) {
    super(message);
    this.name = 'AppError';
    this.key = def.key;
  }

  /** True when this error came from `def` (a frontend or backend registry key). */
  is(def: ErrorDef | string): boolean {
    const key = typeof def === 'string' ? def : def.key;
    return this.key === key;
  }
}

function isErrorShape(e: unknown): e is ErrorDef {
  return typeof e === 'object' && e !== null && 'key' in e && 'message' in e;
}

/** Anything thrown (a backend `{ key, message }`, an Error, a string) as an AppError. */
export function toAppError(e: unknown): AppError {
  if (e instanceof AppError) return e;
  if (isErrorShape(e)) return new AppError(e);
  if (e instanceof Error) return new AppError(errors.UNKNOWN, e.message);
  if (typeof e === 'string') return new AppError(errors.UNKNOWN, e);
  return new AppError(errors.UNKNOWN);
}

/** The message to show for anything thrown. */
export function messageOf(e: unknown): string {
  return toAppError(e).message;
}
