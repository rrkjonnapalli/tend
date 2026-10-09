// Items a scan left out because their names break the rule.
import type { Skipped } from '#lib/types/note.type.ts';

/** The skipped items Tend can rename to fit (the rest have no free name left). */
export const fixable = (items: Skipped[]) => items.filter((s) => s.fix !== '');
