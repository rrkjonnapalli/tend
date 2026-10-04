// The shared tag pool used for suggestions in every TagEditor.
import { tagsApi } from '#lib/api/tags.ts';

class TagPool {
  names = $state<string[]>([]);

  load = async () => {
    this.names = await tagsApi.all();
  };

  /** Adds names created elsewhere without a round trip. */
  learn = (names: string[]) => {
    const known = new Set(this.names.map((n) => n.toLowerCase()));
    const fresh = names.filter((n) => !known.has(n.toLowerCase()));
    if (fresh.length) this.names = [...this.names, ...fresh].sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'base' }));
  };
}

export const tagPool = new TagPool();
