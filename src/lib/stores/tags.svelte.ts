// The shared tag pool used for suggestions in every TagEditor.
import { tagsAPI } from '#lib/api/tags.ts';

const byName = (a: string, b: string) => a.localeCompare(b, undefined, { sensitivity: 'base' });

class TagPool {
  names = $state<string[]>([]);

  load = async () => {
    this.names = await tagsAPI.all();
  };

  /** Adds names created elsewhere without a round trip. */
  learn = (names: string[]) => {
    const known = new Set(this.names.map((n) => n.toLowerCase()));
    const fresh = names.filter((n) => !known.has(n.toLowerCase()));
    if (!fresh.length) return;
    this.names = [...this.names, ...fresh].sort(byName);
  };
}

export const tagPool = new TagPool();
