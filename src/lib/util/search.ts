// Plain substring search for the list pages' search lines.

/** The query as it is compared: trimmed and lower-case. */
export const searchNeedle = (query: string) => query.trim().toLowerCase();

/** True when any field contains `needle` (from `searchNeedle`); an empty needle matches all. */
export function textMatches(fields: string[], needle: string): boolean {
  if (!needle) return true;
  return fields.some((field) => field.toLowerCase().includes(needle));
}
