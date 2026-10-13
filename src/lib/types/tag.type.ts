import type { EnumStatus } from '#lib/enums/status.enum.ts';

/** One tag of the pool, as `tags_overview` returns it. */
export type TagRow = {
  name: string;
  status: EnumStatus;
  /** How many notes, tasks, words, snaps and references carry it. */
  uses: number;
};
