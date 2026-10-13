// The shared tag pool: the names every TagEditor suggests, and every tag with its status and
// uses for Settings → Tags.
import { tagsAPI } from '#lib/api/tags.ts';
import { ENUM_STATUS } from '#lib/enums/status.enum.ts';
import type { TagRow } from '#lib/types/tag.type.ts';

const byName = (a: string, b: string) => a.localeCompare(b, undefined, { sensitivity: 'base' });

class TagPool {
  /** The active tags, which is all a picker offers. */
  names = $state<string[]>([]);
  /** Every tag, blocked ones too. */
  rows = $state<TagRow[]>([]);

  load = async () => {
    const [names, rows] = await Promise.all([tagsAPI.all(), tagsAPI.overview()]);
    this.names = names;
    this.rows = rows;
  };

  /** Adds names created elsewhere without a round trip. */
  learn = (names: string[]) => {
    const known = new Set(this.names.map((n) => n.toLowerCase()));
    const fresh = names.filter((n) => !known.has(n.toLowerCase()));
    if (!fresh.length) return;
    this.names = [...this.names, ...fresh].sort(byName);
  };

  /** A blocked tag can't be added to anything (the backend refuses it too, with TAG_BLOCKED). */
  isBlocked = (name: string) => this.rows.some((r) => r.name === name && r.status === ENUM_STATUS.blocked);
}

export const tagPool = new TagPool();
